use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use actix3::{
    dev::{ServiceRequest, ServiceResponse},
    web::BytesMut,
    Error, HttpMessage,
};
use actix_http2::h1::Payload;
use actix_service1::{Service, Transform};
use futures::{
    future::{ok, Future, Ready},
    stream::StreamExt,
};
use http::{header::CONTENT_LENGTH, HeaderValue};
use log::error;

use crate::generic_http::{BodyCapture, GenericRequest};
use crate::middleware::{RequestId, MAX_SIZE};
use crate::{path_hint, MiddlewareController, MiddlewareMessageSender};

use super::MiddlewareMessage;

#[derive(Clone)]
pub struct SpeakeasySdk {
    sender: MiddlewareMessageSender,
}

impl SpeakeasySdk {
    pub(crate) fn new(sender: MiddlewareMessageSender) -> Self {
        Self { sender }
    }
}

impl<S: 'static, B> Transform<S> for SpeakeasySdk
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SpeakeasySdkMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SpeakeasySdkMiddleware {
            service: Rc::new(RefCell::new(service)),
            sender: self.sender.clone(),
        })
    }
}

pub struct SpeakeasySdkMiddleware<S> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<RefCell<S>>,
    sender: MiddlewareMessageSender,
}

impl<S, B> Service for SpeakeasySdkMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let request_id = RequestId::from(uuid::Uuid::new_v4().to_string());
        let mut svc = self.service.clone();
        let mut sender = self.sender.clone();

        Box::pin(async move {
            let mut max_reached = false;
            let mut captured_body = BytesMut::new();

            let mut body = BodyCapture::Empty;

            let headers = req.headers();

            let path_hint = req
                .match_pattern()
                .map(|path_hint| path_hint::normalize(&path_hint));

            // attempt to content length from headers
            let content_length = headers
                .get(CONTENT_LENGTH)
                .and_then(|value| value.to_str().unwrap().parse::<usize>().ok())
                .unwrap_or_default();

            // if content_length is smaller than the max size attempt to capture the body
            if content_length <= MAX_SIZE {
                if content_length > 0 {
                    captured_body.reserve(content_length);
                }

                // take the payload stream out of the request to work with it
                let mut payload_stream = req.take_payload();

                // create new empty payload, we will fill put the original payload back into this
                // and put back into the request after we have captured the body
                let (mut payload_sender, mut payload) = Payload::create(true);

                while let Some(chunk) = payload_stream.next().await {
                    captured_body.extend_from_slice(&chunk?);

                    // content_length might have not been accurate so we need to check the size
                    if captured_body.len() >= MAX_SIZE {
                        max_reached = true;
                        break;
                    }
                }

                // put read data into the new payload
                payload.unread_data(captured_body.clone().freeze());

                if max_reached {
                    // if max size is reached, send the rest of the data straight into the new payload
                    // without reading it to memory
                    while let Some(chunk) = payload_stream.next().await {
                        payload_sender.feed_data(chunk?);
                    }

                    // if max was reached then body was dropped (not included in HAR)
                    body = BodyCapture::Dropped;
                } else if !captured_body.is_empty() {
                    body = BodyCapture::Captured(captured_body.into_iter().collect());
                }

                // put the payload back into the ServiceRequest
                req.set_payload(payload.into());
            } else {
                // if content_length is larger than the max size, drop the body
                body = BodyCapture::Dropped;
            }

            // create a new GenericRequest from the ServiceRequest
            let generic_request = GenericRequest::new(&req, path_hint, body);

            if let Err(error) = sender
                .send(MiddlewareMessage::Request {
                    request_id: request_id.clone(),
                    request: generic_request,
                })
                .await
            {
                error!(
                    "Failed to send request to channel: {}, id {}",
                    error, &request_id
                );
            }

            req.extensions_mut().insert(MiddlewareController::new(
                request_id.clone(),
                sender.clone(),
            ));

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}

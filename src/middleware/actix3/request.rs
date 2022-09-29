use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use actix3::{
    dev::{ServiceRequest, ServiceResponse},
    web::BytesMut,
    Error, HttpMessage,
};
use actix_http2::h1::Payload;
use actix_service1::{Service, Transform};
use chrono::Utc;
use futures::{
    future::{ok, Future, Ready},
    stream::StreamExt,
};
use http::header::CONTENT_LENGTH;

use crate::controller::Controller;
use crate::generic_http::{BodyCapture, GenericRequest};
use crate::transport::Transport;
use crate::{path_hint, sdk};
#[derive(Clone)]
pub struct SpeakeasySdk<T: Transport> {
    sdk: sdk::SpeakeasySdk<T>,
}

impl<T> SpeakeasySdk<T>
where
    T: Transport + Send + Clone + 'static,
{
    pub(crate) fn new(sdk: sdk::SpeakeasySdk<T>) -> Self {
        Self { sdk }
    }
}

impl<S: 'static, B, T> Transform<S> for SpeakeasySdk<T>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    T: Transport + Send + Clone + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SpeakeasySdkMiddleware<S, T>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SpeakeasySdkMiddleware {
            service: Rc::new(RefCell::new(service)),
            sdk: self.sdk.clone(),
        })
    }
}

pub struct SpeakeasySdkMiddleware<S, T> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<RefCell<S>>,
    sdk: crate::sdk::SpeakeasySdk<T>,
}

impl<S, B, T> Service for SpeakeasySdkMiddleware<S, T>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    T: Transport + Send + Clone + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let start_time = Utc::now();
        let mut svc = self.service.clone();
        let mut controller = Controller::new(&self.sdk);

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
            if content_length <= controller.max_capture_size {
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
                    if captured_body.len() >= controller.max_capture_size {
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
            let generic_request = GenericRequest::new(&req, start_time, path_hint, body);
            controller.set_request(generic_request);

            req.extensions_mut()
                .insert(Arc::new(RwLock::new(controller)));

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}

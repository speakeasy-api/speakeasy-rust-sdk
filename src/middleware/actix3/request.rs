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
use actix_http::{h1::Payload, http::HeaderName};
use actix_service::{Service, Transform};
use futures::{
    future::{ok, Future, Ready},
    stream::StreamExt,
};
use http::{header::CONTENT_LENGTH, HeaderValue};

use crate::generic_http::BodyCapture;

#[derive(Clone)]
pub struct SpeakeasySdk {
    sdk: Arc<RwLock<crate::SpeakeasySdk>>,
}

impl SpeakeasySdk {
    pub fn new(sdk: Arc<RwLock<crate::SpeakeasySdk>>) -> Self {
        Self { sdk }
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
            sdk: self.sdk.clone(),
        })
    }
}

pub struct SpeakeasySdkMiddleware<S> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<RefCell<S>>,
    sdk: Arc<RwLock<crate::SpeakeasySdk>>,
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
        let mut svc = self.service.clone();

        Box::pin(async move {
            let max_size = 1 * 1024 * 1024;
            let mut max_reached = true;
            let mut captured_body = BytesMut::new();

            let mut body = BodyCapture::Empty;

            let headers = req.headers();

            // attempt to content length from headers
            let content_length = headers
                .get(CONTENT_LENGTH)
                .and_then(|value| value.to_str().unwrap().parse::<usize>().ok())
                .unwrap_or_default();

            // if content_length is smaller than the max size attempt to capture the body
            if content_length <= max_size {
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
                    if captured_body.len() > max_size {
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
                } else {
                    body = BodyCapture::Captured(captured_body.into_iter().collect());
                }

                // put the payload back into the ServiceRequest
                req.set_payload(payload.into());
            };

            let header_name = HeaderName::from_static("speakeasy-request-id");

            let mut res = svc.call(req).await?;
            res.headers_mut()
                .insert(header_name, HeaderValue::from_str("123").unwrap());

            Ok(res)
        })
    }
}

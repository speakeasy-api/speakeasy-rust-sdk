use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use axum::extract::MatchedPath;
use http::header::CONTENT_LENGTH;
use bytes::BytesMut;
use chrono::Utc;
use futures::{future::{BoxFuture}, stream::StreamExt};

use axum::{
    response::Response,
    body::Body,
    http::Request,
};
use tower::{Service, Layer};

use crate::controller::Controller;
use crate::generic_http::{BodyCapture, GenericRequest};
use crate::transport::Transport;
use crate::{path_hint, GenericSpeakeasySdk};

#[derive(Clone)]
pub struct SpeakeasySdk<T>
where
    T: Transport + Send + Clone + 'static,
 {
    
    sdk: GenericSpeakeasySdk<T>,
}

impl<T> SpeakeasySdk<T>
where
    T: Transport + Send + Clone + 'static,
{
    pub(crate) fn new(sdk: GenericSpeakeasySdk<T>) -> Self {
        Self { sdk }
    }
}

impl<S,T: Transport> Layer<S> for SpeakeasySdk<T> 
where T: Transport + Send + Clone + 'static {

    type Service = SpeakeasySdkMiddleware<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        SpeakeasySdkMiddleware { sdk: self.sdk.clone(), inner }
    }
}

pub struct SpeakeasySdkMiddleware<S, T> {
    // This is special: We need this to avoid lifetime issues.
    inner: S,
    sdk: GenericSpeakeasySdk<T>,
}

impl<S,T> Service<Request<Body>> for SpeakeasySdkMiddleware<S,T>
where
    S: Service<Request<Body>, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
    T: Transport + Send  + Sync  + Clone + 'static ,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let start_time = Utc::now();
        let mut svc = self.inner.clone();
        let mut controller = Controller::new(&self.sdk);

        Box::pin(async move {
            let mut max_reached = false;
            let mut captured_body = BytesMut::new();

            let mut body = BodyCapture::Empty;

            let headers = request.headers();

            let path_hint = request.extensions().get::<MatchedPath>() 
                .map(|path_hint| path_hint::normalize(path_hint.as_str()));

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
                let mut payload_stream = request.body_mut();


                // create new empty payload, we will fill put the original payload back into this
                // and put back into the request after we have captured the body
                let (mut payload_sender, mut payload) = Body::channel();

                while let Some(chunk) = payload_stream.next().await {
                    captured_body.extend_from_slice(&chunk.unwrap());

                    // content_length might have not been accurate so we need to check the size
                    if captured_body.len() >= controller.max_capture_size {
                        max_reached = true;
                        break;
                    }
                }

                // put read data into the new payload
                payload_sender.send_data(captured_body.clone().freeze()).await.unwrap();

                if max_reached {
                    // if max size is reached, send the rest of the data straight into the new payload
                    // without reading it to memory
                    while let Some(chunk) = payload_stream.next().await {
                        payload_sender.send_data(chunk.unwrap()).await.unwrap();
                    }

                    // if max was reached then body was dropped (not included in HAR)
                    body = BodyCapture::Dropped;
                } else if !captured_body.is_empty() {
                    body = BodyCapture::Captured(captured_body.into_iter().collect());
                }

                // put the payload back into the ServiceRequest
                let request_body = request.body_mut();
                *request_body = payload;
            } else {
                // if content_length is larger than the max size, drop the body
                body = BodyCapture::Dropped;
            }

            // create a new GenericRequest from the ServiceRequest
            let generic_request = GenericRequest::new(&request, start_time, path_hint, body);
            controller.set_request(generic_request);

            request.extensions_mut()
                .insert(Arc::new(RwLock::new(controller)));

            let response = svc.call(request).await?;

            Ok(response)
        })
    }
}

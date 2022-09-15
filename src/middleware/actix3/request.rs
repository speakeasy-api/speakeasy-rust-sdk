use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use actix3::web::BytesMut;
use actix3::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage};
use actix_service::{Service, Transform};
use futures::future::{ok, Future, Ready};
use futures::stream::StreamExt;

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
            let mut body = BytesMut::new();
            let mut stream = req.take_payload();
            while let Some(chunk) = stream.next().await {
                body.extend_from_slice(&chunk?);
            }

            println!("request body: {:?}", body);
            let res = svc.call(req).await?;

            println!("response: {:?}", res.headers());
            Ok(res)
        })
    }
}

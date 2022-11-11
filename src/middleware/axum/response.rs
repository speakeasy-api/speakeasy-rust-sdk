use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use actix3::body::{BodySize, MessageBody, ResponseBody};
use actix3::web::{Bytes, BytesMut};
use actix3::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_service1::{Service, Transform};
use futures::future::{ok, Ready};
use log::error;

use crate::async_runtime;
use crate::controller::{Controller, MAX_SIZE};
use crate::generic_http::{BodyCapture, GenericResponse};
use crate::transport::Transport;

#[derive(Clone)]
pub struct SpeakeasySdk<T: Transport + Send + Clone + 'static> {
    _t: PhantomData<T>,
}

impl<T> SpeakeasySdk<T>
where
    T: Transport + Send + Clone + 'static,
{
    pub(crate) fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<S: 'static, B, T> Transform<S> for SpeakeasySdk<T>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody + 'static,
    T: Transport + Send + Clone + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<ResponseWithBodySender<B, T>>;
    type Error = Error;
    type InitError = ();
    type Transform = SpeakeasySdkMiddleware<S, T>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SpeakeasySdkMiddleware {
            _t: PhantomData,
            service,
        })
    }
}

pub struct SpeakeasySdkMiddleware<S, T: Transport + Send + Clone + 'static> {
    _t: PhantomData<T>,
    service: S,
}

impl<S, B, T> Service for SpeakeasySdkMiddleware<S, T>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
    T: Transport + Send + Clone + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<ResponseWithBodySender<B, T>>;
    type Error = Error;
    type Future = WrapperStream<S, B, T>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        WrapperStream {
            fut: self.service.call(req),
            _t: PhantomData,
        }
    }
}

#[pin_project::pin_project]
pub struct WrapperStream<S, B, T>
where
    B: MessageBody,
    S: Service,
{
    #[pin]
    fut: S::Future,
    _t: PhantomData<(B, T)>,
}

impl<S, B, T> Future for WrapperStream<S, B, T>
where
    B: MessageBody,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    T: Transport + Send + Clone + 'static,
{
    type Output = Result<ServiceResponse<ResponseWithBodySender<B, T>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = futures::ready!(self.project().fut.poll(cx));

        Poll::Ready(res.map(|res| {
            let ext = res.request().head().extensions();
            let controller = ext.get::<Arc<RwLock<Controller<T>>>>().cloned();
            drop(ext);

            let generic_response = GenericResponse::new(&res);

            res.map_body(move |_head, body| {
                ResponseBody::Body(ResponseWithBodySender {
                    body,
                    generic_response,
                    controller,
                    body_dropped: false,
                    body_accum: BytesMut::new(),
                })
            })
        }))
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct ResponseWithBodySender<B, T>
where
    T: Transport + Send + Clone + 'static,
{
    #[pin]
    body: ResponseBody<B>,
    generic_response: GenericResponse,
    controller: Option<Arc<RwLock<Controller<T>>>>,
    body_accum: BytesMut,
    body_dropped: bool,
}

#[pin_project::pinned_drop]
impl<B, T> PinnedDrop for ResponseWithBodySender<B, T>
where
    T: Transport + Send + Clone + 'static,
{
    fn drop(self: Pin<&mut Self>) {
        if let Some(controller) = self.controller.as_ref() {
            let mut response = self.generic_response.clone();

            // set body field, initialized as empty
            if self.body_dropped {
                response.body = BodyCapture::Dropped
            } else if !self.body_accum.is_empty() {
                response.body = BodyCapture::Captured(self.body_accum.to_vec().into())
            }

            let controller: Controller<T> = controller.read().unwrap().clone();

            async_runtime::spawn_task(async move {
                if let Err(error) = controller.build_and_send_har(response) {
                    error!("Error building and sending HAR: {}", error)
                }
            });
        }
    }
}

impl<B: MessageBody, T> MessageBody for ResponseWithBodySender<B, T>
where
    T: Transport + Send + Clone + 'static,
{
    fn size(&self) -> BodySize {
        self.body.size()
    }

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        let max_size = if let Some(controller) = self.controller.as_ref() {
            controller.read().unwrap().max_capture_size
        } else {
            MAX_SIZE
        };

        let this = self.project();

        match this.body.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                if !*this.body_dropped {
                    this.body_accum.extend_from_slice(&chunk);

                    if this.body_accum.len() > max_size {
                        *this.body_dropped = true;
                        this.body_accum.clear();
                    }
                }

                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

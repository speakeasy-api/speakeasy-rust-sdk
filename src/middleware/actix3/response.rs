use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix3::body::{BodySize, MessageBody, ResponseBody};
use actix3::web::{Bytes, BytesMut};
use actix3::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_service::{Service, Transform};
use futures::future::{ok, Ready};

use crate::generic_http::{BodyCapture, GenericResponse};
use crate::middleware::MAX_SIZE;
use crate::MiddlewareController;

#[derive(Clone)]
pub struct SpeakeasySdk {}

impl SpeakeasySdk {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl<S: 'static, B> Transform<S> for SpeakeasySdk
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<ResponseWithBodySender<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = SpeakeasySdkMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SpeakeasySdkMiddleware { service })
    }
}

pub struct SpeakeasySdkMiddleware<S> {
    service: S,
}

impl<S, B> Service for SpeakeasySdkMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<ResponseWithBodySender<B>>;
    type Error = Error;
    type Future = WrapperStream<S, B>;

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
pub struct WrapperStream<S, B>
where
    B: MessageBody,
    S: Service,
{
    #[pin]
    fut: S::Future,
    _t: PhantomData<(B,)>,
}

impl<S, B> Future for WrapperStream<S, B>
where
    B: MessageBody,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<ResponseWithBodySender<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = futures::ready!(self.project().fut.poll(cx));

        Poll::Ready(res.map(|res| {
            let generic_response = GenericResponse::new(&res);

            res.map_body(move |head, body| {
                let ext = head.extensions();
                let controller = ext.get::<MiddlewareController>().cloned();

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
pub struct ResponseWithBodySender<B> {
    #[pin]
    body: ResponseBody<B>,
    generic_response: GenericResponse,
    controller: Option<MiddlewareController>,
    body_accum: BytesMut,
    body_dropped: bool,
}

#[pin_project::pinned_drop]
impl<B> PinnedDrop for ResponseWithBodySender<B> {
    fn drop(self: Pin<&mut Self>) {
        if let Some(controller) = &self.controller {
            let mut sender = controller.sender().clone();
            let request_id = controller.request_id().clone();
            let mut response = self.generic_response.clone();

            // set body field, initialized as empty
            if self.body_dropped {
                response.body = BodyCapture::Dropped
            } else if !self.body_accum.is_empty() {
                response.body = BodyCapture::Captured(self.body_accum.to_vec().into())
            }

            tokio02::task::spawn(async move {
                sender
                    .send(super::MiddlewareMessage::Response {
                        request_id,
                        response,
                    })
                    .await
            });
        }
    }
}

impl<B: MessageBody> MessageBody for ResponseWithBodySender<B> {
    fn size(&self) -> BodySize {
        self.body.size()
    }

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        let this = self.project();

        match this.body.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                if !*this.body_dropped {
                    this.body_accum.extend_from_slice(&chunk);

                    if this.body_accum.len() > MAX_SIZE {
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

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use actix3::body::{BodySize, MessageBody, ResponseBody};
use actix3::web::{Bytes, BytesMut};
use actix3::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_service::{Service, Transform};
use futures::future::{ok, Ready};
use tokio02::sync::mpsc::Sender;

use crate::middleware::MAX_SIZE;

use super::Message;

#[derive(Clone)]
pub struct SpeakeasySdk {
    sdk: Arc<RwLock<crate::SpeakeasySdk>>,
    sender: Sender<Message>,
}

impl SpeakeasySdk {
    pub(crate) fn new(sdk: Arc<RwLock<crate::SpeakeasySdk>>, sender: Sender<Message>) -> Self {
        Self { sdk, sender }
    }
}

impl<S: 'static, B> Transform<S> for SpeakeasySdk
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<BodyLogger<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = SpeakeasySdkMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SpeakeasySdkMiddleware {
            service,
            sdk: self.sdk.clone(),
            sender: self.sender.clone(),
        })
    }
}

pub struct SpeakeasySdkMiddleware<S> {
    service: S,
    sdk: Arc<RwLock<crate::SpeakeasySdk>>,
    sender: Sender<Message>,
}

impl<S, B> Service for SpeakeasySdkMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<BodyLogger<B>>;
    type Error = Error;
    type Future = WrapperStream<S, B>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        WrapperStream {
            sender: self.sender.clone(),
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
    sender: Sender<Message>,
    #[pin]
    fut: S::Future,
    _t: PhantomData<(B,)>,
}

impl<S, B> Future for WrapperStream<S, B>
where
    B: MessageBody,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<BodyLogger<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut sender = self.sender.clone();
        let res = futures::ready!(self.project().fut.poll(cx));

        Poll::Ready(res.map(|res| {
            res.map_body(move |head, body| {
                ResponseBody::Body(BodyLogger {
                    body,
                    request_id: head
                        .headers()
                        .get("speakeasy-request-id")
                        .and_then(|request_id| request_id.to_str().ok())
                        .map(ToString::to_string),
                    sender,
                    body_dropped: false,
                    body_accum: BytesMut::new(),
                })
            })
        }))
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct BodyLogger<B> {
    #[pin]
    body: ResponseBody<B>,
    request_id: Option<String>,
    sender: Sender<Message>,
    body_accum: BytesMut,
    body_dropped: bool,
}

#[pin_project::pinned_drop]
impl<B> PinnedDrop for BodyLogger<B> {
    fn drop(self: Pin<&mut Self>) {
        if let Some(request_id) = &self.request_id {
            let mut sender = self.sender.clone();
            let request_id = request_id.clone();

            tokio02::task::spawn(async move {
                sender
                    .send(super::Message::Response {
                        request_id: request_id.clone(),
                    })
                    .await
            });
        }
    }
}

impl<B: MessageBody> MessageBody for BodyLogger<B> {
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

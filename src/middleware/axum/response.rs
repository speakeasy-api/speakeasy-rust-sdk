use bytes::{Buf, Bytes, BytesMut};
use futures::ready;
use http::{Request, Response};

use http_body::Body;
use pin_project::pin_project;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

use crate::async_runtime;
use crate::controller::{Controller, MAX_SIZE};
use crate::generic_http::{BodyCapture, GenericResponse};
use crate::transport::Transport;

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

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

impl<S, T> Layer<S> for SpeakeasySdk<T>
where
    T: Transport + Send + Clone + 'static,
{
    type Service = SpeakeasySdkMiddleware<S, T>;

    fn layer(&self, service: S) -> Self::Service {
        SpeakeasySdkMiddleware::new(service)
    }
}

#[derive(Debug, Clone)]
pub struct SpeakeasySdkMiddleware<S, T: Transport + Send + Clone + 'static> {
    _t: PhantomData<T>,
    inner: S,
}

impl<S, T> SpeakeasySdkMiddleware<S, T>
where
    T: Transport + Send + Clone + 'static,
{
    fn new(inner: S) -> Self {
        Self {
            inner,
            _t: PhantomData,
        }
    }
}

impl<ReqBody, ResBody, S, T> Service<Request<ReqBody>> for SpeakeasySdkMiddleware<S, T>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Body,
    T: Transport + Send + Sync + Clone + 'static,
{
    type Response = Response<ResponseWithBodySender<ResBody, T>>;
    type Error = S::Error;
    type Future = WrapperStream<S::Future, T>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        WrapperStream {
            response_future: self.inner.call(req),
            _t: PhantomData,
        }
    }
}

impl<B, T> Body for ResponseWithBodySender<B, T>
where
    B: Body,
    B::Error: Into<BoxError>,
    T: Transport + Send + Clone + 'static,
{
    type Data = Bytes;
    type Error = BoxError;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let max_size = if let Some(controller) = self.controller.as_ref() {
            controller.read().unwrap().max_capture_size
        } else {
            MAX_SIZE
        };

        let this = self.project();

        match this.body.poll_data(cx) {
            Poll::Ready(Some(Ok(mut chunk))) => {
                if !*this.body_dropped {
                    this.body_accum.extend_from_slice(chunk.chunk());

                    if this.body_accum.len() > max_size {
                        *this.body_dropped = true;
                        this.body_accum.clear();
                    }
                }

                let bytes = chunk.copy_to_bytes(chunk.remaining());
                Poll::Ready(Some(Ok(bytes)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        self.project().body.poll_trailers(cx).map_err(Into::into)
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct ResponseWithBodySender<B, T>
where
    T: Transport + Send + Clone + 'static,
{
    #[pin]
    body: B,
    generic_response: GenericResponse,
    controller: Option<Arc<RwLock<Controller<T>>>>,
    _t: PhantomData<T>,
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
                    log::error!("Error building and sending HAR: {}", error)
                }
            });
        }
    }
}

#[pin_project]
pub struct WrapperStream<F, T> {
    #[pin]
    response_future: F,
    _t: PhantomData<T>,
}

impl<F, B, E, T> Future for WrapperStream<F, T>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Body,
    T: Transport + Send + Sync + Clone + 'static,
{
    type Output = Result<Response<ResponseWithBodySender<B, T>>, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = ready!(self.as_mut().project().response_future.poll(cx)?);
        let ext = res.extensions();

        let controller = ext.get::<Arc<RwLock<Controller<T>>>>().cloned();

        if controller.is_none() {
            log::error!("No controller found in extensions, please add request layer to your axum service");
        }

        let generic_response = GenericResponse::new(&res);
        let (parts, body) = res.into_parts();

        let body_with_sender = ResponseWithBodySender {
            body,
            generic_response,
            controller,
            _t: PhantomData,
            body_accum: BytesMut::new(),
            body_dropped: false,
        };

        let res = Response::from_parts(parts, body_with_sender);
        Poll::Ready(Ok(res))
    }
}

//! Middleware for Axum

mod request_response_impl;

pub(crate) mod request;
pub(crate) mod response;

use crate::{transport::Transport, GenericSpeakeasySdk};

/// Container struct the contains the middleware's for capturing request and response
pub struct Middleware<T: Transport + Send + Clone + 'static> {
    pub(crate) request_capture: request::SpeakeasySdk<T>,
    // TODO: switch back
    pub(crate) response_capture: response::SpeakeasySdk<T>,
}

impl<T> Middleware<T>
where
    T: Transport + Send + Clone + 'static,
{
    /// Create new middleware
    pub fn new(sdk: GenericSpeakeasySdk<T>) -> Self {
        Self {
            request_capture: request::SpeakeasySdk::new(sdk),
            response_capture: response::SpeakeasySdk::new(),
        }
    }

    /// Get request and response capture middleware
    ///
    /// ```ignore
    /// // initialize SDK
    /// let middleware = Middleware::new(sdk);
    /// let (request_capture_middleware, response_capture_middleware) = middleware.into();
    /// ```
    pub fn into(self) -> (request::SpeakeasySdk<T>, response::SpeakeasySdk<T>) {
        (self.request_capture, self.response_capture)
    }
}

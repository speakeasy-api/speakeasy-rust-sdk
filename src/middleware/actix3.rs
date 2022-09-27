mod request_response_impl;

pub mod request;
pub mod response;

use crate::{transport::Transport, GenericSpeakeasySdk};

pub struct Middleware<T: Transport + Send + Clone + 'static> {
    pub request_capture: request::SpeakeasySdk<T>,
    pub response_capture: response::SpeakeasySdk<T>,
}

impl<T> Middleware<T>
where
    T: Transport + Send + Clone + 'static,
{
    pub fn new(sdk: GenericSpeakeasySdk<T>) -> Self {
        Self {
            request_capture: request::SpeakeasySdk::new(sdk),
            response_capture: response::SpeakeasySdk::new(),
        }
    }

    pub fn into(self) -> (request::SpeakeasySdk<T>, response::SpeakeasySdk<T>) {
        (self.request_capture, self.response_capture)
    }
}

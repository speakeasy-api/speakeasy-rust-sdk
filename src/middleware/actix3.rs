mod request_response_impl;

pub mod request;
pub mod response;

use crate::{sdk::SpeakeasySdk, transport::Transport};

pub struct Middleware<T: Transport + Send + Clone + 'static> {
    pub request_capture: request::SpeakeasySdk<T>,
    pub response_capture: response::SpeakeasySdk<T>,
}

impl<T> Middleware<T>
where
    T: Transport + Send + Clone + 'static,
{
    pub fn new(sdk: SpeakeasySdk<T>) -> Self {
        Self {
            request_capture: request::SpeakeasySdk::new(sdk),
            response_capture: response::SpeakeasySdk::new(),
        }
    }

    pub fn init(self) -> (request::SpeakeasySdk<T>, response::SpeakeasySdk<T>) {
        (self.request_capture, self.response_capture)
    }
}

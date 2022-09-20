mod request_response_impl;

pub mod request;
pub mod response;

use tokio02::sync::mpsc::Receiver;

use super::{messages::MiddlewareMessage, State};
use crate::SpeakeasySdk;

pub struct Middleware {
    state: State,
    receiver: Receiver<MiddlewareMessage>,
    pub request_capture: request::SpeakeasySdk,
    pub response_capture: response::SpeakeasySdk,
}

impl Middleware {
    pub fn new(sdk: SpeakeasySdk) -> Self {
        let (sender, receiver) = tokio02::sync::mpsc::channel(100);

        Self {
            state: State::new(sdk),
            receiver,
            request_capture: request::SpeakeasySdk::new(sender.clone()),
            response_capture: response::SpeakeasySdk::new(sender),
        }
    }

    pub fn init(self) -> (request::SpeakeasySdk, response::SpeakeasySdk) {
        let mut receiver = self.receiver;
        let mut state = self.state;

        tokio02::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                state.handle_middleware_message(msg);
            }
        });

        (self.request_capture, self.response_capture)
    }
}

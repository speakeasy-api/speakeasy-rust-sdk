mod request_response_impl;

pub mod request;
pub mod response;

use super::{messages::MiddlewareMessage, State};
use crate::{
    async_runtime::{self, Receiver},
    sdk::SpeakeasySdk,
    transport::Transport,
};

pub struct Middleware<T: Transport> {
    state: State<T>,
    receiver: Receiver<MiddlewareMessage>,
    pub request_capture: request::SpeakeasySdk,
    pub response_capture: response::SpeakeasySdk,
}

impl<T> Middleware<T>
where
    T: Transport + Clone + Send + 'static,
{
    pub fn new(sdk: SpeakeasySdk<T>) -> Self {
        let (sender, receiver) = async_runtime::channel(100);

        Self {
            state: State::new(sdk),
            receiver,
            request_capture: request::SpeakeasySdk::new(sender),
            response_capture: response::SpeakeasySdk::new(),
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

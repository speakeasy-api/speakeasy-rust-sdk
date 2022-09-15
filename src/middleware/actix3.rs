use std::sync::{Arc, RwLock};

use crate::SpeakeasySdk;

pub mod request;
pub mod response;

pub struct Middleware {
    pub request_capture: request::SpeakeasySdk,
    pub response_capture: request::SpeakeasySdk,
}

impl Middleware {
    pub fn new(sdk: SpeakeasySdk) -> Self {
        let global = Arc::new(RwLock::new(sdk));

        Self {
            request_capture: request::SpeakeasySdk::new(global.clone()),
            response_capture: request::SpeakeasySdk::new(global.clone()),
        }
    }
}

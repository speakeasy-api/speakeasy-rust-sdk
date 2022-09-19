mod request_response_impl;

pub mod request;
pub mod response;

use actix_http::http::HeaderName;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio02::sync::mpsc::Receiver;

use crate::{generic_http::GenericRequest, SpeakeasySdk};

pub(crate) fn speakeasy_header_name() -> HeaderName {
    HeaderName::from_static("speakeasy-request-id")
}

#[derive(Debug)]
pub(crate) enum Message {
    Request {
        request_id: String,
        request: GenericRequest,
    },
    Response {
        request_id: String,
    },
}

pub struct Middleware {
    requests: HashMap<String, GenericRequest>,
    receiver: Receiver<Message>,

    pub request_capture: request::SpeakeasySdk,
    pub response_capture: response::SpeakeasySdk,
}

impl Middleware {
    pub fn new(sdk: SpeakeasySdk) -> Self {
        let global = Arc::new(RwLock::new(sdk));

        let (sender, receiver) = tokio02::sync::mpsc::channel(100);

        Self {
            requests: HashMap::new(),
            receiver,
            request_capture: request::SpeakeasySdk::new(global.clone(), sender.clone()),
            response_capture: response::SpeakeasySdk::new(global, sender),
        }
    }

    pub fn start(self) -> (request::SpeakeasySdk, response::SpeakeasySdk) {
        let mut receiver = self.receiver;

        tokio02::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    Message::Request {
                        request_id,
                        request,
                    } => {
                        println!("request: {:#?}", request)
                    }
                    Message::Response { request_id } => {
                        println!("Response: {}", request_id)
                    }
                }
            }
        });

        (self.request_capture, self.response_capture)
    }
}

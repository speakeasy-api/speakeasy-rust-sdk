mod request_response_impl;

pub mod request;
pub mod response;

use actix_http::http::HeaderName;
use std::collections::HashMap;
use tokio02::sync::mpsc::Receiver;

use crate::{
    generic_http::{GenericRequest, GenericResponse},
    har_builder::HarBuilder,
    SpeakeasySdk,
};

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
        response: GenericResponse,
    },
}

pub struct Middleware {
    sdk: SpeakeasySdk,
    requests: HashMap<String, GenericRequest>,
    receiver: Receiver<Message>,

    pub request_capture: request::SpeakeasySdk,
    pub response_capture: response::SpeakeasySdk,
}

impl Middleware {
    pub fn new(sdk: SpeakeasySdk) -> Self {
        let (sender, receiver) = tokio02::sync::mpsc::channel(100);

        Self {
            sdk,
            requests: HashMap::new(),
            receiver,
            request_capture: request::SpeakeasySdk::new(sender.clone()),
            response_capture: response::SpeakeasySdk::new(sender),
        }
    }

    pub fn start(self) -> (request::SpeakeasySdk, response::SpeakeasySdk) {
        let mut requests = self.requests;
        let mut receiver = self.receiver;
        let masking = self.sdk.masking.clone();

        tokio02::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    Message::Request {
                        request_id,
                        request,
                    } => {
                        log::debug!(
                            "request received id: {}, request: {:?}",
                            &request_id,
                            &request
                        );
                        requests.insert(request_id.clone(), request);
                    }
                    Message::Response {
                        request_id,
                        response,
                    } => {
                        if let Some(request) = requests.remove(&request_id) {
                            log::debug!(
                                "response received, request_id: {}, request: {:?}, response: {:?}",
                                &request_id,
                                &request,
                                &response
                            );

                            let har = HarBuilder::new(request, response).build(&masking);
                            println!("HAR BUILT: {:#?}", har);
                        }
                    }
                }
            }
        });

        (self.request_capture, self.response_capture)
    }
}

pub(crate) mod controller;
mod request_id;

use std::collections::HashMap;

use actix_http::http::HeaderName;

use crate::{
    generic_http::{GenericRequest, GenericResponse},
    har_builder::HarBuilder,
    SpeakeasySdk,
};

// 1MB
pub(crate) const MAX_SIZE: usize = 1024 * 1024;

pub(crate) fn speakeasy_header_name() -> HeaderName {
    HeaderName::from_static("speakeasy-request-id")
}

pub(crate) type RequestId = request_id::RequestId;

#[derive(Debug)]
pub(crate) struct State {
    sdk: SpeakeasySdk,
    requests: HashMap<RequestId, GenericRequest>,
    responses: HashMap<RequestId, GenericResponse>,
    controller_state: controller::State,
}

impl State {
    pub(crate) fn new(sdk: SpeakeasySdk) -> Self {
        Self {
            sdk,
            requests: HashMap::new(),
            responses: HashMap::new(),
            controller_state: controller::State::new(),
        }
    }

    pub(crate) fn handle_middleware_message(&mut self, msg: MiddlewareMessage) {
        match msg {
            MiddlewareMessage::Request {
                request_id,
                request,
            } => {
                log::debug!(
                    "request received id: {:?}, request: {:?}",
                    &request_id,
                    &request
                );
                self.requests.insert(request_id, request);
            }
            MiddlewareMessage::Response {
                request_id,
                response,
            } => {
                if let Some(request) = self.requests.remove(&request_id) {
                    log::debug!(
                        "response received, request_id: {:?}, request: {:?}, response: {:?}",
                        &request_id,
                        &request,
                        &response
                    );

                    let request_specific_masking = self.controller_state.get_masking(&request_id);

                    // if mask is found for request, use it, otherwise use global mask
                    let masking = if let Some(masking) = request_specific_masking.as_ref() {
                        masking
                    } else {
                        &self.sdk.masking
                    };

                    let har = HarBuilder::new(request, response).build(masking);
                    println!("HAR BUILT: {:#?}", har);
                }
            }
            MiddlewareMessage::ControllerMessage(msg) => {
                self.controller_state.handle_message(msg);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum MiddlewareMessage {
    Request {
        request_id: RequestId,
        request: GenericRequest,
    },
    Response {
        request_id: RequestId,
        response: GenericResponse,
    },
    ControllerMessage(controller::Message),
}
// framework specific middleware
#[cfg(feature = "actix3")]
pub mod actix3;

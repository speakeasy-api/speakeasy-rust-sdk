pub(crate) mod messages;

mod request_id;

use crate::{controller, generic_http::GenericRequest, har_builder::HarBuilder, SpeakeasySdk};
use actix_http::http::HeaderName;
use log::debug;
use std::collections::HashMap;

use self::messages::MiddlewareMessage;

// 1MB
pub(crate) const MAX_SIZE: usize = 1024 * 1024;

#[doc(hidden)]
pub type RequestId = request_id::RequestId;

#[derive(Debug)]
pub(crate) struct State {
    sdk: SpeakeasySdk,
    requests: HashMap<RequestId, GenericRequest>,
    controller: controller::State,
}

impl State {
    pub(crate) fn new(sdk: SpeakeasySdk) -> Self {
        Self {
            sdk,
            requests: HashMap::new(),
            controller: controller::State::new(),
        }
    }

    pub(crate) fn handle_middleware_message(&mut self, msg: MiddlewareMessage) {
        match msg {
            MiddlewareMessage::Request {
                request_id,
                request,
            } => {
                debug!(
                    "request received id: {:?}, request: {:?}",
                    &request_id, &request
                );
                self.requests.insert(request_id, request);
            }
            MiddlewareMessage::Response {
                request_id,
                response,
            } => {
                if let Some(request) = self.requests.remove(&request_id) {
                    debug!(
                        "response received, request_id: {:?}, request: {:?}, response: {:?}",
                        &request_id, &request, &response
                    );

                    let request_specific_masking = self.controller.get_masking(&request_id);

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
                self.controller.handle_message(msg);
            }
        }
    }
}

// PUBLIC
// framework specific middleware
#[cfg(feature = "actix3")]
pub mod actix3;

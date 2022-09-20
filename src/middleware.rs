pub(crate) mod messages;

mod request_id;

use crate::{
    controller,
    generic_http::{GenericRequest, GenericResponse},
    har_builder::HarBuilder,
    SpeakeasySdk,
};
use log::{debug, error};
use speakeasy_protos::ingest::IngestRequest;
use std::collections::HashMap;
use thiserror::Error;

use self::messages::MiddlewareMessage;

// 1MB
pub(crate) const MAX_SIZE: usize = 1024 * 1024;

#[derive(Debug, Error)]
pub enum Error {
    #[error("error while serializing HAR: {0}")]
    HarSerializeError(#[from] serde_json::Error),
}

#[doc(hidden)]
pub type RequestId = request_id::RequestId;

#[derive(Debug)]
pub(crate) struct State {
    sdk: SpeakeasySdk,
    requests: HashMap<RequestId, GenericRequest>,
    controller_state: controller::ControllerState,
}

impl State {
    pub(crate) fn new(sdk: SpeakeasySdk) -> Self {
        Self {
            sdk,
            requests: HashMap::new(),
            controller_state: controller::ControllerState::new(),
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

                    if let Err(error) = self.build_and_send_har(request_id, request, response) {
                        error!("Failed to send HAR to Speakeasy: {:#?}", error);
                    }
                }
            }
            MiddlewareMessage::ControllerMessage(msg) => {
                self.controller_state.handle_message(msg);
            }
        }
    }

    fn build_and_send_har(
        &mut self,
        request_id: RequestId,
        request: GenericRequest,
        response: GenericResponse,
    ) -> Result<(), Error> {
        // look for path hint for request, if not look in the request
        let path_hint = self
            .controller_state
            .get_path_hint(&request_id)
            .or_else(|| request.path_hint.as_ref().map(ToString::to_string))
            .unwrap_or_else(|| "".to_string());

        // look for a request specific mask if not use global one
        let masking = self
            .controller_state
            .get_masking(&request_id)
            .unwrap_or_else(|| self.sdk.masking.clone());

        let config = self.sdk.config.clone();
        let har = HarBuilder::new(request, response).build(&masking);
        let har_json = serde_json::to_string(&har)?;

        let customer_id = self
            .controller_state
            .get_customer_id(&request_id)
            .unwrap_or_default();

        let ingest = IngestRequest {
            har: har_json,
            path_hint,
            api_id: config.api_id,
            version_id: config.version_id,
            customer_id,
            masking_metadata: None,
        };

        Ok(())
    }
}

// PUBLIC
// framework specific middleware
#[cfg(feature = "actix3")]
pub mod actix3;

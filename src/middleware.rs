pub(crate) mod messages;

mod request_id;

use crate::{
    async_runtime, controller,
    generic_http::{GenericRequest, GenericResponse},
    har_builder::HarBuilder,
    sdk::SpeakeasySdk,
    transport::Transport,
};
use log::{debug, error};
use speakeasy_protos::ingest::IngestRequest;
use std::collections::HashMap;
use thiserror::Error;

use tonic03::transport::Error as TonicError;

use self::messages::MiddlewareMessage;

// 1MB
pub(crate) const MAX_SIZE: usize = 1024 * 1024;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid server address {0}")]
    InvalidServerError(String),
    #[error("unable to connect to server {0}")]
    ConnectionError(TonicError),
    #[error("invalid tls {0}")]
    InvalidTls(TonicError),
}

#[doc(hidden)]
pub type RequestId = request_id::RequestId;

#[derive(Debug)]
pub(crate) struct State<T: Transport> {
    sdk: SpeakeasySdk<T>,
    requests: HashMap<RequestId, GenericRequest>,
    controller_state: controller::ControllerState,
}

impl<T> State<T>
where
    T: Transport + Clone + Send + 'static,
{
    pub(crate) fn new(sdk: SpeakeasySdk<T>) -> Self {
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
                debug!("request received id: {:?}", &request_id);
                self.requests.insert(request_id, request);
            }
            MiddlewareMessage::Response {
                request_id,
                response,
            } => {
                if let Some(request) = self.requests.remove(&request_id) {
                    debug!("response received, request_id: {:?}", &request_id);

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
        let customer_id = self
            .controller_state
            .get_customer_id(&request_id)
            .unwrap_or_default();

        let transport = self.sdk.transport.clone();

        async_runtime::spawn_task(async move {
            let har = HarBuilder::new(request, response).build(&masking);
            let har_json = serde_json::to_string(&har).expect("har will serialize to json");

            let masking_metadata = if masking.is_empty() {
                None
            } else {
                Some(masking.into())
            };

            let ingest = IngestRequest {
                har: har_json,
                path_hint,
                api_id: config.api_id,
                version_id: config.version_id,
                customer_id,
                masking_metadata,
            };

            transport.send(ingest)
        });

        Ok(())
    }
}

// PUBLIC
// framework specific middleware
#[cfg(feature = "actix3")]
pub mod actix3;

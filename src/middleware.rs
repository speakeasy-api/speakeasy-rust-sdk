pub(crate) mod messages;

mod request_id;

use crate::{
    controller,
    generic_http::{GenericRequest, GenericResponse},
    har_builder::HarBuilder,
    SpeakeasySdk,
};
use http::uri::InvalidUri;
use log::{debug, error};
use once_cell::sync::Lazy;
use speakeasy_protos::ingest::{ingest_service_client::IngestServiceClient, IngestRequest};
use std::{collections::HashMap, time::Duration};
use thiserror::Error;

use tonic03::{
    metadata::{errors::InvalidMetadataValue, MetadataValue},
    transport::{Channel, ClientTlsConfig, Endpoint, Error as TonicError},
    Request,
};

use self::messages::MiddlewareMessage;

static SPEAKEASY_SERVER_SECURE: Lazy<bool> = Lazy::new(|| {
    !matches!(
        std::env::var("SPEAKEASY_SERVER_SECURE").as_deref(),
        Ok("false")
    )
});

static SPEAKEASY_SERVER_URL: Lazy<String> = Lazy::new(|| {
    let domain = std::env::var("SPEAKEASY_SERVER_URL")
        .unwrap_or_else(|_| "grpc.prod.speakeasyapi.dev:443".to_string());

    if !domain.starts_with("http") {
        if *SPEAKEASY_SERVER_SECURE {
            format!("https://{}", domain)
        } else {
            format!("http://{}", domain)
        }
    } else {
        domain
    }
});

// 1MB
pub(crate) const MAX_SIZE: usize = 1024 * 1024;

#[derive(Debug, Error)]
pub enum Error {
    #[error("error while serializing HAR: {0}")]
    HarSerializeError(#[from] serde_json::Error),
    #[error("invalid server address {0}")]
    InvalidServerError(InvalidUri),
    #[error("unable to connect to server {0}")]
    ConnectionError(TonicError),
    #[error("invalid api key {0}")]
    InvalidApiKey(InvalidMetadataValue),
    #[error("invalid tls {0}")]
    InvalidTls(TonicError),
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
        let har = HarBuilder::new(request, response).build(&masking);
        let har_json = serde_json::to_string(&har)?;

        let customer_id = self
            .controller_state
            .get_customer_id(&request_id)
            .unwrap_or_default();

        tokio02::task::spawn(async move {
            let ingest = IngestRequest {
                har: har_json,
                path_hint,
                api_id: config.api_id,
                version_id: config.version_id,
                customer_id,
                masking_metadata: None,
            };

            if let Err(error) = send(ingest, config.api_key).await {
                error!("Failed to send HAR to Speakeasy: {:#?}", error);
            }
        });

        Ok(())
    }
}

async fn send(request: IngestRequest, api_key: String) -> Result<(), Error> {
    let endpoint: Endpoint =
        Channel::from_shared(&**SPEAKEASY_SERVER_URL).map_err(Error::InvalidServerError)?;

    let endpoint = if *SPEAKEASY_SERVER_SECURE {
        let tls = ClientTlsConfig::new().domain_name(SPEAKEASY_SERVER_URL.as_str());
        endpoint
            .tls_config(tls)
            .map_err(Error::InvalidTls)?
            .tcp_keepalive(Some(Duration::from_secs(5)))
    } else {
        endpoint
    };

    let channel = endpoint.connect().await.map_err(Error::ConnectionError)?;

    let token = MetadataValue::from_str(&api_key).map_err(Error::InvalidApiKey)?;

    let mut client = IngestServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("x-api-key", token.clone());
        Ok(req)
    });

    let request = Request::new(request);

    if let Err(error) = client.ingest(request).await {
        error!("Failed to send HAR to Speakeasy: {:?}", error);
    }

    Ok(())
}

// PUBLIC
// framework specific middleware
#[cfg(feature = "actix3")]
pub mod actix3;

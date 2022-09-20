pub(crate) mod messages;

mod request_id;

use crate::{
    controller,
    generic_http::{GenericRequest, GenericResponse},
    har_builder::HarBuilder,
    SpeakeasySdk,
};
use http::{header::InvalidHeaderValue, uri::InvalidUri, HeaderValue, Uri};
use log::{debug, error};
use once_cell::sync::Lazy;
use speakeasy_protos::ingest::{ingest_service_client::IngestServiceClient, IngestRequest};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

use tonic03::{
    metadata::{errors::InvalidMetadataValue, MetadataValue},
    transport::Error as TonicError,
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
    InvalidApiKey(InvalidHeaderValue),
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
    let https_connector = hyper_openssl::HttpsConnector::new().unwrap();

    let client = hyper::Client::builder()
        .http2_only(true)
        .build(https_connector);

    let uri = hyper::Uri::from_str(&SPEAKEASY_SERVER_URL).unwrap();
    let token = HeaderValue::from_str(&api_key).map_err(Error::InvalidApiKey)?;

    // Hyper's client requires that requests contain full Uris include a scheme and
    // an authority. Tonic's transport will handle this for you but when using the client
    // manually you need ensure the uri's are set correctly.
    let add_origin = tower::service_fn(|mut req: hyper::Request<tonic03::body::BoxBody>| {
        let uri = Uri::builder()
            .scheme(uri.scheme().unwrap().clone())
            .authority(uri.authority().unwrap().clone())
            .path_and_query(req.uri().path_and_query().unwrap().clone())
            .build()
            .unwrap();

        *req.uri_mut() = uri;
        req.headers_mut().insert("x-api-key", token.clone());

        client.request(req)
    });

    let mut client = IngestServiceClient::new(add_origin);

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

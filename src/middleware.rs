pub(crate) mod messages;

mod request_id;

use crate::{
    controller,
    generic_http::{GenericRequest, GenericResponse},
    har_builder::HarBuilder,
    SpeakeasySdk,
};
use http::{header::InvalidHeaderValue, HeaderValue, Uri};
use log::{debug, error};
use once_cell::sync::Lazy;
use speakeasy_protos::ingest::{ingest_service_client::IngestServiceClient, IngestRequest};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

use tonic03::{transport::Error as TonicError, Request};

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
    #[error("invalid server address {0}")]
    InvalidServerError(String),
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
        let customer_id = self
            .controller_state
            .get_customer_id(&request_id)
            .unwrap_or_default();

        tokio02::task::spawn(async move {
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

            if let Err(error) = send(ingest, config.api_key).await {
                error!("Failed to send HAR to Speakeasy: {:#?}", error);
            }
        });

        Ok(())
    }
}

async fn send(request: IngestRequest, api_key: String) -> Result<(), Error> {
    // NOTE: Using hyper directly as there seems to be a bug with tonic v0.3 throwing
    // an error from rustls. When making the middleware for actix4 we can hopefully
    // avoid doing this and just use the client directly from tonic.
    let insecure_client = hyper::Client::builder().http2_only(true).build_http();

    let client = hyper::Client::builder()
        .http2_only(true)
        .build(hyper_openssl::HttpsConnector::new().expect("Need OpenSSL"));

    let uri = hyper::Uri::from_str(&SPEAKEASY_SERVER_URL).unwrap();
    let token = HeaderValue::from_str(&api_key).map_err(Error::InvalidApiKey)?;

    let authority = uri
        .authority()
        .ok_or_else(|| Error::InvalidServerError("authority".to_string()))?;

    let add_origin = tower::service_fn(|mut req: hyper::Request<tonic03::body::BoxBody>| {
        let uri = Uri::builder()
            .scheme(uri.scheme().unwrap().clone())
            .authority(authority.clone())
            .path_and_query(
                req.uri()
                    .path_and_query()
                    .expect("path and query always present")
                    .clone(),
            )
            .build()
            .unwrap();

        *req.uri_mut() = uri;
        req.headers_mut().insert("x-api-key", token.clone());

        if *SPEAKEASY_SERVER_SECURE {
            client.request(req)
        } else {
            insecure_client.request(req)
        }
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

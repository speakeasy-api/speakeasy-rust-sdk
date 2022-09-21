mod generic_http;
mod har_builder;
mod masking;
mod path_hint;
mod util;

pub(crate) mod async_runtime;
pub(crate) mod controller;
pub(crate) mod sdk;
pub(crate) mod transport;

pub mod middleware;

use http::header::InvalidHeaderValue;
use thiserror::Error;
use transport::GrpcClient;

pub type Masking = masking::Masking;
pub type StringMaskingOption = masking::StringMaskingOption;
pub type NumberMaskingOption = masking::NumberMaskingOption;
pub type MiddlewareController = controller::Controller;

pub(crate) type MiddlewareMessageSender =
    async_runtime::Sender<middleware::messages::MiddlewareMessage>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid api key {0}")]
    InvalidApiKey(InvalidHeaderValue),
}

/// Configuration struct for configuring the global speakeasy SDK instance
#[derive(Debug, Clone)]
pub struct Config {
    /// Retrieve from Speakeasy API dashboard.
    pub api_key: String,
    /// Name that you'd like to associate captured requests with.
    ///
    /// This name will show up in the Speakeasy dashboard. e.g. "PetStore" might be a good ApiID for a Pet Store's API.
    /// No spaces allowed.
    pub api_id: String,
    /// Version that you would like to associate captured requests with.
    ///
    /// The combination of ApiID (name) and VersionID will uniquely identify your requests in the Speakeasy Dashboard.
    /// e.g. "v1.0.0". You can have multiple versions for the same ApiID (if running multiple versions of your API)
    pub version_id: String,
}

/// Speakeasy SDK instance
pub type SpeakeasySdk = sdk::SpeakeasySdk<GrpcClient>;

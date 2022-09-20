mod generic_http;
mod har_builder;
mod masking;
mod path_hint;
mod util;

pub mod middleware;

pub type Masking = masking::Masking;
pub type StringMaskingOption = masking::StringMaskingOption;
pub type NumberMaskingOption = masking::NumberMaskingOption;
pub type MiddlewareController = middleware::controller::Controller;

/// Configuration struct for configuring the global speakeasy SDK instance
#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub api_id: String,
    pub version_id: String,
    pub port: i32,
}

/// Speakeasy SDK instance
#[derive(Debug, Clone)]
pub struct SpeakeasySdk {
    config: Config,
    pub masking: Masking,
}

impl SpeakeasySdk {
    /// Create a new Speakeasy SDK instance
    ///
    /// # Examples:
    /// ```rust
    /// use speakeasy_rust_sdk::{SpeakeasySdk, Config, StringMaskingOption};
    ///
    /// let config = Config{
    ///     api_key: "YOUR API KEY HERE".to_string(),       // retrieve from Speakeasy API dashboard.
    ///     api_id: "YOUR API ID HERE".to_string(),         // enter a name that you'd like to associate captured requests with.
    ///     // This name will show up in the Speakeasy dashboard. e.g. "PetStore" might be a good ApiID for a Pet Store's API.
    ///     // No spaces allowed.
    ///     version_id: "YOUR VERSION ID HERE".to_string(), // enter a version that you would like to associate captured requests with.
    ///     // The combination of ApiID (name) and VersionID will uniquely identify your requests in the Speakeasy Dashboard.
    ///     // e.g. "v1.0.0". You can have multiple versions for the same ApiID (if running multiple versions of your API)
    ///     port: 3000,                        // The port number your express app is listening on (required to build full URLs on non-standard ports)
    /// };
    ///
    /// // Create a new Speakeasy SDK instance
    /// let mut sdk = SpeakeasySdk::new(config);
    ///
    /// // Configure masking for query
    /// // see [Masking::with_query_string_mask] for more options
    /// sdk.masking.with_query_string_mask("secret", "********");
    /// sdk.masking.with_query_string_mask("password", StringMaskingOption::default());
    ///
    /// // Configure other masks
    /// // see [Masking] for more options
    /// ```
    pub fn new(config: Config) -> Self {
        Self {
            config,
            masking: Default::default(),
        }
    }
}

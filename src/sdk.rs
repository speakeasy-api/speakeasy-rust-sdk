use crate::{
    transport::{GrpcClient, Transport},
    Config, Error, Masking, RequestConfig,
};

/// Speakeasy SDK instance
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct GenericSpeakeasySdk<GrpcClient> {
    pub masking: Masking,

    pub(crate) config: RequestConfig,
    pub(crate) transport: GrpcClient,
}

impl<T: Transport + Send + Clone + 'static> GenericSpeakeasySdk<T> {
    pub fn new_with_transport(config: Config, transport: T) -> Self {
        let config = RequestConfig::from(config);
        let masking = Masking::default();

        Self {
            masking,
            config,
            transport,
        }
    }
}

impl GenericSpeakeasySdk<GrpcClient> {
    /// Create a new Speakeasy SDK instance
    ///
    /// # Examples:
    /// ```rust
    /// use speakeasy_rust_sdk::{SpeakeasySdk, Config, masking::StringMaskingOption};
    ///
    /// let config = Config{
    ///     api_key: "YOUR API KEY HERE".to_string(),       // retrieve from Speakeasy API dashboard.
    ///     api_id: "YOUR API ID HERE".to_string(),         // enter a name that you'd like to associate captured requests with.
    ///     // This name will show up in the Speakeasy dashboard. e.g. "PetStore" might be a good ApiID for a Pet Store's API.
    ///     // No spaces allowed.
    ///     version_id: "YOUR VERSION ID HERE".to_string(), // enter a version that you would like to associate captured requests with.
    ///     // The combination of ApiID (name) and VersionID will uniquely identify your requests in the Speakeasy Dashboard.
    ///     // e.g. "v1.0.0". You can have multiple versions for the same ApiID (if running multiple versions of your API)
    /// };
    ///
    /// // Create a new Speakeasy SDK instance
    /// let mut sdk = SpeakeasySdk::try_new(config).expect("valid API key");
    ///
    /// // Configure masking for query
    /// // see [Masking::with_query_string_mask] for more options
    /// sdk.masking.with_query_string_mask("secret", "********");
    /// sdk.masking.with_query_string_mask("password", StringMaskingOption::default());
    ///
    /// // Configure other masks
    /// // see [Masking] for more options
    /// ```
    pub fn try_new(config: Config) -> Result<Self, Error> {
        Ok(Self {
            transport: GrpcClient::new(config.api_key.clone())?,
            config: config.into(),
            masking: Default::default(),
        })
    }
}

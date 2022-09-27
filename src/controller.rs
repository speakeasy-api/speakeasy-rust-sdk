//! Control [masking](Controller::set_masking()), [path_hint](Controller::set_path_hint()) and [customer_id](Controller::set_customer_id()) on a per request basis

// 1MB
pub(crate) const MAX_SIZE: usize = 1024 * 1024;

use crate::speakeasy_protos::ingest::IngestRequest;

use crate::{
    async_runtime,
    generic_http::{GenericRequest, GenericResponse},
    har_builder::HarBuilder,
    path_hint,
    transport::Transport,
    Error, GenericSpeakeasySdk, Masking, RequestConfig,
};

/// Control [masking](Controller::set_masking()), [path_hint](Controller::set_path_hint()) and [customer_id](Controller::set_customer_id()) on a per request basis
#[derive(Debug, Clone)]
pub struct Controller<T: Transport> {
    transport: T,
    config: RequestConfig,

    request: Option<GenericRequest>,

    masking: Masking,
    path_hint: Option<String>,
    customer_id: Option<String>,

    pub(crate) max_capture_size: usize,
}

// Public
impl<T> Controller<T>
where
    T: Transport + Send + Clone + 'static,
{
    #[doc(hidden)]
    pub fn new(sdk: &GenericSpeakeasySdk<T>) -> Self {
        Self {
            transport: sdk.transport.clone(),
            config: sdk.config.clone(),
            request: None,
            masking: sdk.masking.clone(),
            path_hint: None,
            customer_id: None,
            max_capture_size: MAX_SIZE,
        }
    }

    /// Set the path_hint request
    pub fn set_path_hint(&mut self, path_hint: &str) {
        let path_hint = path_hint::normalize(path_hint);
        self.path_hint = Some(path_hint)
    }

    /// Set new masking for the request, see [Masking](crate::masking::Masking) for more
    pub fn set_masking(&mut self, masking: Masking) {
        self.masking = masking
    }

    /// Set new customer_id for the request
    pub fn set_customer_id(&mut self, customer_id: String) {
        self.customer_id = Some(customer_id)
    }

    /// Set new max_capture_size for the request, if the request or response bodies are above this, it will be dropped
    pub fn set_max_capture_size(&mut self, max_capture_size: usize) {
        self.max_capture_size = max_capture_size
    }
}

// Crate use only
impl<T> Controller<T>
where
    T: Transport + Send + Clone + 'static,
{
    pub(crate) fn set_request(&mut self, request: GenericRequest) {
        self.request = Some(request)
    }

    pub(crate) fn build_and_send_har(self, response: GenericResponse) -> Result<(), Error> {
        let request = self.request.clone().ok_or(Error::RequestNotSaved)?;

        // look for path hint for request, if not look in the request
        let path_hint = self
            .path_hint
            .as_ref()
            .or(request.path_hint.as_ref())
            .map(ToString::to_string)
            .unwrap_or_else(|| "".to_string());

        let masking = self.masking.clone();

        let customer_id = self.customer_id.clone().unwrap_or_default();

        let max_capture_size = self.max_capture_size;

        let config = self.config.clone();
        let transport = self.transport;

        async_runtime::spawn_task(async move {
            let har = HarBuilder::new(request, response, max_capture_size).build(&masking);
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

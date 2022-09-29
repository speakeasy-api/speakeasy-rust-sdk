/// IngestRequest is the request message for the ingest rpc.
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct IngestRequest {
    /// har is string containing a HTTP Archive 1.2 formatted file contents.
    #[prost(string, tag = "1")]
    pub har: std::string::String,
    /// path_hint is a hint to the ingest service about the structure of the request path.
    #[prost(string, tag = "2")]
    pub path_hint: std::string::String,
    /// api_id is used to associate requests with a particular Api in the Speakeasy platform.
    #[prost(string, tag = "3")]
    pub api_id: std::string::String,
    /// version_id is used to associate requests with a particular version of an Api in the Speakeasy platform.
    #[prost(string, tag = "4")]
    pub version_id: std::string::String,
    /// customer_id is the id of the customer who is making the request.
    #[prost(string, tag = "5")]
    pub customer_id: std::string::String,
    /// masking_metadata contains information about any masking added to the har.
    #[prost(message, optional, tag = "6")]
    pub masking_metadata: ::std::option::Option<ingest_request::MaskingMetadata>,
}
pub mod ingest_request {
    /// MaskingMetadata contains information about any masking added to the har.
    #[derive(Clone, PartialEq, Eq, ::prost::Message)]
    pub struct MaskingMetadata {
        /// request_header_masks contains a map of header keys to the masks applied to them.
        #[prost(map = "string, string", tag = "1")]
        pub request_header_masks:
            ::std::collections::HashMap<std::string::String, std::string::String>,
        /// request_cookie_masks contains a map of cookie keys to the masks applied to them.
        #[prost(map = "string, string", tag = "2")]
        pub request_cookie_masks:
            ::std::collections::HashMap<std::string::String, std::string::String>,
        /// request_field_masks_string contains a map of string body fields to the masks applied to them.
        #[prost(map = "string, string", tag = "3")]
        pub request_field_masks_string:
            ::std::collections::HashMap<std::string::String, std::string::String>,
        /// request_field_masks_number contains a map of number body fields to the masks applied to them.
        #[prost(map = "string, string", tag = "4")]
        pub request_field_masks_number:
            ::std::collections::HashMap<std::string::String, std::string::String>,
        /// response_header_masks contains a map of header keys to the masks applied to them.
        #[prost(map = "string, string", tag = "5")]
        pub response_header_masks:
            ::std::collections::HashMap<std::string::String, std::string::String>,
        /// response_cookie_masks contains a map of cookie keys to the masks applied to them.
        #[prost(map = "string, string", tag = "6")]
        pub response_cookie_masks:
            ::std::collections::HashMap<std::string::String, std::string::String>,
        /// response_field_masks_string contains a map of string body fields to the masks applied to them.
        #[prost(map = "string, string", tag = "7")]
        pub response_field_masks_string:
            ::std::collections::HashMap<std::string::String, std::string::String>,
        /// response_field_masks_number contains a map of number body fields to the masks applied to them.
        #[prost(map = "string, string", tag = "8")]
        pub response_field_masks_number:
            ::std::collections::HashMap<std::string::String, std::string::String>,
        /// query_string_masks contains a map of query string keys to the masks applied to them.
        #[prost(map = "string, string", tag = "9")]
        pub query_string_masks:
            ::std::collections::HashMap<std::string::String, std::string::String>,
    }
}
/// IngestResponse is the response message for the ingest rpc.
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct IngestResponse {}
#[doc = r" Generated client implementations."]
pub mod ingest_service_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic03::codegen::*;
    #[doc = " IngestService is the service definition for the registry ingest endpoint."]
    pub struct IngestServiceClient<T> {
        inner: tonic03::client::Grpc<T>,
    }
    impl IngestServiceClient<tonic03::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic03::transport::Error>
        where
            D: std::convert::TryInto<tonic03::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic03::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> IngestServiceClient<T>
    where
        T: tonic03::client::GrpcService<tonic03::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic03::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic03::Interceptor>) -> Self {
            let inner = tonic03::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        #[doc = "  Ingest is the rpc handling ingest from the SDK."]
        pub async fn ingest(
            &mut self,
            request: impl tonic03::IntoRequest<super::IngestRequest>,
        ) -> Result<tonic03::Response<super::IngestResponse>, tonic03::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic03::Status::new(
                    tonic03::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic03::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/ingest.IngestService/Ingest");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for IngestServiceClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for IngestServiceClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "IngestServiceClient {{ ... }}")
        }
    }
}

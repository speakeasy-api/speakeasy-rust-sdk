#![doc = r" Generated client implementations."]
#![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
use speakeasy_protos_tokio_02::ingest::{IngestRequest, IngestResponse};
use tonic03::codegen::*;
#[doc = " IngestService is the service definition for the registry ingest endpoint."]
#[derive(Debug, Clone)]
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
        request: impl tonic03::IntoRequest<IngestRequest>,
    ) -> Result<tonic03::Response<IngestResponse>, tonic03::Status> {
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

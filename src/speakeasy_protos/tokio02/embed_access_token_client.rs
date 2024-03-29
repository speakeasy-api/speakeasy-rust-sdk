use speakeasy_protos_tokio_02::embedaccesstoken::{
    EmbedAccessTokenRequest, EmbedAccessTokenResponse,
};

use tonic03::codegen::*;
#[doc = " EmbedAccessTokenService is the service definition for the registry embed-access-token endpoint."]
pub struct EmbedAccessTokenServiceClient<T> {
    inner: tonic03::client::Grpc<T>,
}
impl EmbedAccessTokenServiceClient<tonic03::transport::Channel> {
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
impl<T> EmbedAccessTokenServiceClient<T>
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
    #[doc = "  Get is the rpc handling access token retrieval from the SDK"]
    pub async fn get(
        &mut self,
        request: impl tonic03::IntoRequest<EmbedAccessTokenRequest>,
    ) -> Result<tonic03::Response<EmbedAccessTokenResponse>, tonic03::Status> {
        self.inner.ready().await.map_err(|e| {
            tonic03::Status::new(
                tonic03::Code::Unknown,
                format!("Service was not ready: {}", e.into()),
            )
        })?;
        let codec = tonic03::codec::ProstCodec::default();
        let path =
            http::uri::PathAndQuery::from_static("/embedaccesstoken.EmbedAccessTokenService/Get");
        self.inner.unary(request.into_request(), path, codec).await
    }
}
impl<T: Clone> Clone for EmbedAccessTokenServiceClient<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<T> std::fmt::Debug for EmbedAccessTokenServiceClient<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EmbedAccessTokenServiceClient {{ ... }}")
    }
}

use crate::{async_runtime, Error};

use crate::speakeasy_protos::embedaccesstoken::embed_access_token_service_client::EmbedAccessTokenServiceClient;
use crate::speakeasy_protos::embedaccesstoken::{
    EmbedAccessTokenRequest, EmbedAccessTokenResponse,
};
use crate::speakeasy_protos::ingest::{ingest_service_client::IngestServiceClient, IngestRequest};
use http::HeaderValue;
use once_cell::sync::Lazy;
use std::{str::FromStr, sync::Arc};

#[cfg(feature = "tokio02")]

mod tokio02 {
    pub use hyper13::Client as HyperClient;
    pub use hyper13::Request as HyperRequest;
    pub use hyper13::Uri;
    pub use hyper_openssl08::HttpsConnector;
    pub use tonic03::body::BoxBody;
    pub use tonic03::Request as TonicRequest;
    pub use tower03::service_fn;
}

#[cfg(feature = "tokio02")]
use self::tokio02::*;

#[cfg(feature = "tokio")]
mod tokio {
    pub use hyper::Client as HyperClient;
    pub use hyper::Request as HyperRequest;
    pub use hyper::Uri;
    pub use hyper_openssl::HttpsConnector;
    pub use tonic::body::BoxBody;
    pub use tonic::Request as TonicRequest;
    pub use tower::service_fn;
}

#[cfg(feature = "tokio")]
use self::tokio::*;

pub(crate) static SPEAKEASY_SERVER_SECURE: Lazy<bool> = Lazy::new(|| {
    !matches!(
        std::env::var("SPEAKEASY_SERVER_SECURE").as_deref(),
        Ok("false")
    )
});

pub(crate) static SPEAKEASY_SERVER_URL: Lazy<String> = Lazy::new(|| {
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

pub trait Transport {
    type Output: Send + 'static;
    type Error: Send + 'static;

    fn send(&self, request: IngestRequest) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct GrpcClient {
    token: Arc<HeaderValue>,
}

impl GrpcClient {
    pub(crate) fn new(token: impl AsRef<str>) -> Result<Self, crate::Error> {
        let token = HeaderValue::from_str(token.as_ref()).map_err(crate::Error::InvalidApiKey)?;

        Ok(Self {
            token: Arc::new(token),
        })
    }

    pub async fn get_embedded_access_token(
        &self,
        request: EmbedAccessTokenRequest,
    ) -> Result<EmbedAccessTokenResponse, Error> {
        // NOTE: Using hyper directly as there seems to be a bug with tonic v0.3 throwing
        // an error from rustls. When making the middleware for actix4 we can hopefully
        // avoid doing this and just use the client directly from tonic.
        let uri = Uri::from_str(&SPEAKEASY_SERVER_URL).unwrap();
        let authority = uri
            .authority()
            .ok_or_else(|| Error::InvalidServerError("authority".to_string()))?
            .clone();

        let token = self.token.clone();

        let add_origin = service_fn(move |mut req: HyperRequest<BoxBody>| {
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
            req.headers_mut()
                .insert("x-api-key", token.as_ref().clone());

            if *SPEAKEASY_SERVER_SECURE {
                let client = HyperClient::builder()
                    .http2_only(true)
                    .build(HttpsConnector::new().expect("Need OpenSSL"));
                client.request(req)
            } else {
                let insecure_client = HyperClient::builder().http2_only(true).build_http();
                insecure_client.request(req)
            }
        });

        let mut client = EmbedAccessTokenServiceClient::new(add_origin);
        let request = TonicRequest::new(request);

        let response = client
            .get(request)
            .await
            .map_err(Error::UnableToGetEmbeddedAccessToken)?
            .into_inner();

        Ok(response)
    }
}

impl Transport for GrpcClient {
    type Output = ();
    type Error = crate::Error;

    fn send(&self, request: IngestRequest) -> Result<Self::Output, Self::Error> {
        // NOTE: Using hyper directly as there seems to be a bug with tonic v0.3 throwing
        // an error from rustls. When making the middleware for actix4 we can hopefully
        // avoid doing this and just use the client directly from tonic.

        let uri = Uri::from_str(&SPEAKEASY_SERVER_URL).unwrap();
        let authority = uri
            .authority()
            .ok_or_else(|| Self::Error::InvalidServerError("authority".to_string()))?
            .clone();

        let token = self.token.clone();

        let add_origin = service_fn(move |mut req: HyperRequest<BoxBody>| {
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
            req.headers_mut()
                .insert("x-api-key", token.as_ref().clone());

            if *SPEAKEASY_SERVER_SECURE {
                let client = HyperClient::builder()
                    .http2_only(true)
                    .build(HttpsConnector::new().expect("Need OpenSSL"));
                client.request(req)
            } else {
                let insecure_client = HyperClient::builder().http2_only(true).build_http();
                insecure_client.request(req)
            }
        });

        let mut client = IngestServiceClient::new(add_origin);
        let request = TonicRequest::new(request);

        async_runtime::spawn_task(async move {
            let response = client.ingest(request).await;

            if let Err(e) = response {
                log::error!("Error sending request: {}", e);
            }
        });

        Ok(())
    }
}

#[cfg(feature = "mock")]
pub(crate) mod mock {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct GrpcMock {}

    impl GrpcMock {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Default for GrpcMock {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Transport for Arc<GrpcMock> {
        type Output = ();
        type Error = ();

        fn send(&self, _request: IngestRequest) -> Result<Self::Output, Self::Error> {
            Ok(())
        }
    }
}

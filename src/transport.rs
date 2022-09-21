use crate::{async_runtime, middleware::Error};

use http::{HeaderValue, Uri};
use once_cell::sync::Lazy;
use speakeasy_protos::ingest::{ingest_service_client::IngestServiceClient, IngestRequest};
use std::{str::FromStr, sync::Arc};
use tonic03::Request;

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
    fn send(&self, request: IngestRequest) -> Result<(), Error>;
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
}

impl Transport for GrpcClient {
    fn send(&self, request: IngestRequest) -> Result<(), Error> {
        // NOTE: Using hyper directly as there seems to be a bug with tonic v0.3 throwing
        // an error from rustls. When making the middleware for actix4 we can hopefully
        // avoid doing this and just use the client directly from tonic.
        let insecure_client = hyper::Client::builder().http2_only(true).build_http();

        let client = hyper::Client::builder()
            .http2_only(true)
            .build(hyper_openssl::HttpsConnector::new().expect("Need OpenSSL"));

        let uri = hyper::Uri::from_str(&SPEAKEASY_SERVER_URL).unwrap();

        let authority = uri
            .authority()
            .ok_or_else(|| Error::InvalidServerError("authority".to_string()))?
            .clone();

        let token = self.token.clone();

        let add_origin =
            tower::service_fn(move |mut req: hyper::Request<tonic03::body::BoxBody>| {
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
                    client.request(req)
                } else {
                    insecure_client.request(req)
                }
            });

        let mut client = IngestServiceClient::new(add_origin);
        let request = Request::new(request);

        async_runtime::spawn_task(async move {
            let response = client.ingest(request).await;

            if let Err(e) = response {
                log::error!("Error sending request: {}", e);
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {}

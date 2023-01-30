/*!
![180100416-b66263e6-1607-4465-b45d-0e298a67c397](https://user-images.githubusercontent.com/68016351/181640742-31ab234a-3b39-432e-b899-21037596b360.png)

Speakeasy is your API Platform team as a service. Use our drop in SDK to manage all your API Operations including embeds for request logs and usage dashboards, test case generation from traffic, and understanding API drift.

The Speakeasy Rust SDK for evaluating API requests/responses. Currently compatible with axum and actix.

## Requirements

Supported Frameworks:

- Axum
- Actix 4
- Actix 3

## Usage

Available on crates: [crates.io/crates/speakeasy-rust-sdk](https://crates.io/crates/speakeasy-rust-sdk)

Documentation available at: [docs.rs/speakeasy-rust-sdk](<(https://docs.rs/speakeasy-rust-sdk)>)

Run:

```
cargo add speakeasy-rust-sdk --features actix4
```

Or add it directly to your `Cargo.toml`

```toml
speakeasy-rust-sdk = {version = "0.2.0", features = ["actix4"]}
```
### Minimum configuration

[Sign up for free on our platform](https://www.speakeasyapi.dev/). After you've created a workspace and generated an API key enable Speakeasy in your API as follows:

Configure Speakeasy at the start of your `main()` function:

```ignore
use actix_web::{
    get, post,
    web::{self, ReqData},
    App, HttpResponse, HttpServer, Responder,
};
use speakeasy_rust_sdk::{middleware::actix3::Middleware, Config, SpeakeasySdk};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
   HttpServer::new(|| {
        let config = Config {
            // retrieve from Speakeasy API dashboard.
            api_key: "YOUR API KEY HERE".to_string(),
            // enter a name that you'd like to associate captured requests with.
            // This name will show up in the Speakeasy dashboard. e.g. "PetStore" might be a good ApiID for a Pet Store's API.
            // No spaces allowed.
            api_id: "YOUR API ID HERE".to_string(),
            // enter a version that you would like to associate captured requests with.
            // The combination of ApiID (name) and VersionID will uniquely identify your requests in the Speakeasy Dashboard.
            // e.g. "v1.0.0". You can have multiple versions for the same ApiID (if running multiple versions of your API)
            version_id: "YOUR VERSION ID HERE".to_string(),
        };

        // Create a new Speakeasy SDK instance
        let mut sdk = SpeakeasySdk::try_new(config).expect("API key is valid");

        // create middleware
        let speakeasy_middleware = Middleware::new(sdk);
        let (request_capture, response_capture) = speakeasy_middleware.into();

        App::new()
            .wrap(request_capture)
            .wrap(response_capture)
            ...
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

Build and deploy your app and that's it. Your API is being tracked in the Speakeasy workspace you just created
and will be visible on the dashboard next time you log in. Visit our [docs site](https://docs.speakeasyapi.dev/) to
learn more.

### On-Premise Configuration

The SDK provides a way to redirect the requests it captures to an on-premise deployment of the Speakeasy Platform. This is done through the use of environment variables listed below. These are to be set in the environment of your services that have integrated the SDK:

- `SPEAKEASY_SERVER_URL` - The url of the on-premise Speakeasy Platform's GRPC Endpoint. By default this is `grpc.prod.speakeasyapi.dev:443`.
- `SPEAKEASY_SERVER_SECURE` - Whether or not to use TLS for the on-premise Speakeasy Platform. By default this is `true` set to `SPEAKEASY_SERVER_SECURE="false"` if you are using an insecure connection.

## Request Matching

The Speakeasy SDK out of the box will do its best to match requests to your provided OpenAPI Schema. It does this by extracting the path template used by one of the supported routers or frameworks above for each request captured and attempting to match it to the paths defined in the OpenAPI Schema, for example:

```ignore
use actix_web::{post, Responder};

// The path template "/v1/users/{id}" is captured automatically by the SDK
#[get("v1/users/{id}")]
async fn handler_function(id: web::Path<String>) -> impl Responder {
    // handler function code
}
```

This isn't always successful or even possible, meaning requests received by Speakeasy will be marked as `unmatched`, and potentially not associated with your Api, Version or ApiEndpoints in the Speakeasy Dashboard.

To help the SDK in these situations you can provide path hints per request handler that match the paths in your OpenAPI Schema:

```ignore
use std::sync::{Arc, RwLock};
use actix3::web::ReqData;

#[post("/special_route")]
async fn special_route(controller: ReqData<Arc<RwLock<MiddlewareController>>>) -> HttpResponse {
    // Provide a path hint for the request using the OpenAPI Path templating format:
    //  https://swagger.io/specification/#path-templating-matching
    controller
        .write()
        .unwrap()
        .set_path_hint("/special_route/{wildcard}");

    // the rest of your handlers code
}
```

## Capturing Customer IDs

To help associate requests with customers/users of your APIs you can provide a customer ID per request handler:

```ignore
use std::sync::{Arc, RwLock};
use actix3::web::ReqData;

#[post("/index")]
async fn index(controller: ReqData<Arc<RwLock<MiddlewareController>>>) -> HttpResponse {
    controller
        .write()
        .unwrap()
        .set_customer_id("123customer_id".to_string());

    // rest of the handlers code
}

```

Note: This is not required, but is highly recommended. By setting a customer ID you can easily associate requests with your customers/users in the Speakeasy Dashboard, powering filters in the [Request Viewer](https://docs.speakeasyapi.dev/speakeasy-user-guide/request-viewer).

## Masking sensitive data

Speakeasy can mask sensitive data in the query string parameters, headers, cookies and request/response bodies captured by the SDK. This is useful for maintaining sensitive data isolation, and retaining control over the data that is captured.

You can set masking options globally, this options will be applied to all requests and response.

```ignore
use speakeasy_rust_sdk::{
    middleware::actix3::Middleware, Config, Masking, MiddlewareController, SpeakeasySdk,
    StringMaskingOption,
};
use std::sync::{Arc, RwLock};
use actix3::web::ReqData;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
   HttpServer::new(|| {
        let config = Config {
            // retrieve from Speakeasy API dashboard.
            api_key: "YOUR API KEY HERE".to_string(),
            // enter a name that you'd like to associate captured requests with.
            // This name will show up in the Speakeasy dashboard. e.g. "PetStore" might be a good ApiID for a Pet Store's API.
            // No spaces allowed.
            api_id: "YOUR API ID HERE".to_string(),
            // enter a version that you would like to associate captured requests with.
            // The combination of ApiID (name) and VersionID will uniquely identify your requests in the Speakeasy Dashboard.
            // e.g. "v1.0.0". You can have multiple versions for the same ApiID (if running multiple versions of your API)
            version_id: "YOUR VERSION ID HERE".to_string(),
        };

        // Create a new Speakeasy SDK instance
        let mut sdk = SpeakeasySdk::try_new(config).expect("API key is valid");

        // Configure masking for query
        sdk.masking.with_query_string_mask("secret", "********");
        sdk.masking
            .with_query_string_mask("password", StringMaskingOption::default());

        // Configure masking for request
        sdk.masking
            .with_request_field_mask_string("password", StringMaskingOption::default());

        // Configure masking for response
        sdk.masking
            .with_response_field_mask_string("secret", StringMaskingOption::default());

        // create middleware
        let speakeasy_middleware = Middleware::new(sdk);
        let (request_capture, response_capture) = speakeasy_middleware.into();

        App::new()
            .wrap(request_capture)
            .wrap(response_capture)
            // rest of the handlers
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

But if you would like to be more selective you can mask certain sensitive data using our middleware controller allowing you to mask fields as needed in different handlers:

```ignore
use speakeasy_rust_sdk::{Masking, MiddlewareController, SpeakeasySdk, StringMaskingOption};

#[post("/index")]
async fn index(controller: ReqData<Arc<RwLock<MiddlewareController>>>) -> HttpResponse {
    // create a specific masking for this request/response
    let mut masking = Masking::default();
    masking.with_request_field_mask_string("password", StringMaskingOption::default());

    // set new masking for this request/response
    controller.write().unwrap().set_masking(masking);

    // rest of the handlers code
}
```

The [Masking](crate::masking::Masking) struct can be set with a number of different options to mask sensitive data in the request:

- `masking.with_query_string_mask` - **with_query_string_mask** will mask the specified query strings with an optional mask string.
- `masking.with_request_header_mask` - **with_request_header_mask** will mask the specified request headers with an optional mask string.
- `masking.with_response_header_mask` - **with_response_header_mask** will mask the specified response headers with an optional mask string.
- `masking.with_request_cookie_mask` - **with_request_cookie_mask** will mask the specified request cookies with an optional mask string.
- `masking.with_response_cookie_mask` - **with_response_cookie_mask** will mask the specified response cookies with an optional mask string.
- `masking.with_request_field_mask_string` - **with_request_field_mask_string** will mask the specified request body fields with an optional mask. Supports string fields only. Matches using regex.
- `masking.with_request_field_mask_number` - **with_request_field_mask_number** will mask the specified request body fields with an optional mask. Supports number fields only. Matches using regex.
- `masking.with_response_field_mask_string` - **with_response_field_mask_string** will mask the specified response body fields with an optional mask. Supports string fields only. Matches using regex.
- `masking.with_response_field_mask_number` - **with_response_field_mask_number** will mask the specified response body fields with an optional mask. Supports number fields only. Matches using regex.


### Embedded Request Viewer Access Tokens

The Speakeasy SDK can generate access tokens for the [Embedded Request Viewer](https://docs.speakeasyapi.dev/docs/using-speakeasy/build-dev-portals/intro/index.html) that can be used to view requests captured by the SDK.

For documentation on how to configure filters, find that [HERE](https://docs.speakeasyapi.dev/docs/using-speakeasy/build-dev-portals/intro/index.html).

Below are some examples on how to generate access tokens:

```ignore
use speakeasy_rust_sdk::speakeasy_protos::embedaccesstoken::{
    embed_access_token_request::Filter, EmbedAccessTokenRequest,
};

let request = EmbedAccessTokenRequest {
    filters: vec![Filter {
        key: "customer_id".to_string(),
        operator: "=".to_string(),
        value: "a_customer_id".to_string(),
    }],
    ..Default::default()
};

let token_response = app_state
    .speakeasy_sdk
    .get_embedded_access_token(request)
    .await
    .unwrap();
```

*/

mod generic_http;
mod har_builder;
mod path_hint;
mod util;

pub(crate) mod async_runtime;

#[doc(hidden)]
pub mod sdk;
pub mod speakeasy_protos;
#[doc(hidden)]
pub mod transport;

pub mod controller;
pub mod masking;
pub mod middleware;

use crate::speakeasy_protos::embedaccesstoken::{
    EmbedAccessTokenRequest, EmbedAccessTokenResponse,
};
use http::header::InvalidHeaderValue;
use thiserror::Error;
use transport::GrpcClient;

/// All masking options, see functions for more details on setting them
pub type Masking = masking::Masking;

/// Speakeasy SDK instance and controller
pub type SpeakeasySdk = GenericSpeakeasySdk<GrpcClient>;

/// Middleware controller, use for setting request specific [Masking], [path-hint](MiddlewareController::set_path_hint()) and [customer-ids](MiddlewareController::set_customer_id())
pub type MiddlewareController = GenericController<GrpcClient>;

#[doc(hidden)]
/// Generic structs used to override Grpc transport
pub type GenericController<T> = controller::Controller<T>;
#[doc(hidden)]
pub type GenericSpeakeasySdk<T> = sdk::GenericSpeakeasySdk<T>;

#[cfg(feature = "tokio")]
type GrpcStatus = tonic::Status;

#[cfg(feature = "tokio02")]
type GrpcStatus = tonic03::Status;

/// General error struct for the crate
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid api key {0}")]
    InvalidApiKey(InvalidHeaderValue),
    #[error("request not saved, make sure the middleware is being used")]
    RequestNotSaved,
    #[error("invalid server address, incorrect: {0}")]
    InvalidServerError(String),
    #[error("unable to get embedded access token")]
    UnableToGetEmbeddedAccessToken(#[from] GrpcStatus),
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

/// Configuration struct for configuring the global speakeasy SDK instance
#[derive(Debug, Clone)]
pub(crate) struct RequestConfig {
    pub api_id: String,
    pub version_id: String,
}

impl From<Config> for RequestConfig {
    fn from(config: Config) -> Self {
        Self {
            api_id: config.api_id,
            version_id: config.version_id,
        }
    }
}

impl SpeakeasySdk {
    pub async fn get_embedded_access_token(
        &self,
        request: EmbedAccessTokenRequest,
    ) -> Result<EmbedAccessTokenResponse, Error> {
        self.transport.get_embedded_access_token(request).await
    }
}

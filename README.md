# speakeasy-rust-sdk

![180100416-b66263e6-1607-4465-b45d-0e298a67c397](https://user-images.githubusercontent.com/68016351/181640742-31ab234a-3b39-432e-b899-21037596b360.png)

Speakeasy is your API Platform team as a service. Use our drop in SDK to manage all your API Operations including embeds for request logs and usage dashboards, test case generation from traffic, and understanding API drift.

The Speakeasy Rust SDK for evaluating API requests/responses. Currently compatible with actix3.

## Requirements

Supported Frameworks:

- Axum
- Actix 3

## Usage

Available on crates: [crates.io/crates/speakeasy-rust-sdk](https://crates.io/crates/speakeasy-rust-sdk)

Documentation available at: [docs.rs/speakeasy-rust-sdk](https://docs.rs/speakeasy-rust-sdk)

Run:

```
cargo add speakeasy-rust-sdk --features actix3
```

Or add it directly to your `Cargo.toml`

```toml
speakeasy-rust-sdk = {version = "0.2.0", features = ["actix3"]}
```

### Minimum configuration

[Sign up for free on our platform](https://www.speakeasyapi.dev/). After you've created a workspace and generated an API key enable Speakeasy in your API as follows:

_(for axum configuration see the example at [examples/axum/](examples/axum/src/main.rs))_

Configure Speakeasy at the start of your `main()` function:

```rust
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

```rust
// The path template "/v1/users/{id}" is captured automatically by the SDK
#[get("v1/users/{id}")]
async fn handler_function(id: web::Path<String>) -> impl Responder {
    // handler function code
}
```

This isn't always successful or even possible, meaning requests received by Speakeasy will be marked as `unmatched`, and potentially not associated with your Api, Version or ApiEndpoints in the Speakeasy Dashboard.

To help the SDK in these situations you can provide path hints per request handler that match the paths in your OpenAPI Schema:

```rust
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

```rust
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

```rust
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
            ...
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

But if you would like to be more selective you can mask certain sensitive data using our middleware controller allowing you to mask fields as needed in different handlers:

```rust
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

The `Masking` struct can be set with a number of different options to mask sensitive data in the request:

- `masking.with_query_string_mask` - **with_query_string_mask** will mask the specified query strings with an optional mask string.
- `masking.with_request_header_mask` - **with_request_header_mask** will mask the specified request headers with an optional mask string.
- `masking.with_response_header_mask` - **with_response_header_mask** will mask the specified response headers with an optional mask string.
- `masking.with_request_cookie_mask` - **with_request_cookie_mask** will mask the specified request cookies with an optional mask string.
- `masking.with_response_cookie_mask` - **with_response_cookie_mask** will mask the specified response cookies with an optional mask string.
- `masking.with_request_field_mask_string` - **with_request_field_mask_string** will mask the specified request body fields with an optional mask. Supports string fields only. Matches using regex.
- `masking.with_request_field_mask_number` - **with_request_field_mask_number** will mask the specified request body fields with an optional mask. Supports number fields only. Matches using regex.
- `masking.with_response_field_mask_string` - **with_response_field_mask_string** will mask the specified response body fields with an optional mask. Supports string fields only. Matches using regex.
- `masking.with_response_field_mask_number` - **with_response_field_mask_number** will mask the specified response body fields with an optional mask. Supports number fields only. Matches using regex.

For complete docs on masking see the [docs.rs/speakeasy-rust-sdk](https://docs.rs/speakeasy-rust-sdk/latest/speakeasy_rust_sdk/)

### Examples

- Axum - [examples/axum/](examples/axum/)
- Actix3 - [examples/actix3/](examples/actix3/)
- Actix3 Test Server - [testserver/](testserver/)

use actix_web::{dev::Service, web, App, HttpServer};
use futures_util::FutureExt as _;
use speakeasy_rust_sdk::{
    middleware::actix3::Middleware, Config, SpeakeasySdk, StringMaskingOption,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        let config = Config {
            api_key: "YOUR API KEY HERE".to_string(), // retrieve from Speakeasy API dashboard.
            api_id: "YOUR API ID HERE".to_string(), // enter a name that you'd like to associate captured requests with.
            // This name will show up in the Speakeasy dashboard. e.g. "PetStore" might be a good ApiID for a Pet Store's API.
            // No spaces allowed.
            version_id: "YOUR VERSION ID HERE".to_string(), // enter a version that you would like to associate captured requests with.
            // The combination of ApiID (name) and VersionID will uniquely identify your requests in the Speakeasy Dashboard.
            // e.g. "v1.0.0". You can have multiple versions for the same ApiID (if running multiple versions of your API)
            port: 3000, // The port number your express app is listening on (required to build full URLs on non-standard ports)
        };

        // Create a new Speakeasy SDK instance
        let mut sdk = SpeakeasySdk::new(config);

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

        log::info!("starting HTTP server at http://localhost:8080");

        let speakeasy_middleware = Middleware::new(sdk);

        App::new()
            .wrap(speakeasy_middleware.request_capture)
            .wrap(speakeasy_middleware.response_capture)
            .wrap_fn(|req, srv| {
                println!("Hi from start. You requested: {}", req.path());

                srv.call(req).map(|res| {
                    println!("Hi from response");
                    res
                })
            })
            .service(
                web::resource("/").to(|| async {
                    "Hello, middleware! Check the console where the server is run."
                }),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

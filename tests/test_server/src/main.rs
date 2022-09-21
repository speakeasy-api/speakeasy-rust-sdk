use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use har::Har;
use speakeasy_protos::ingest::IngestRequest;
use speakeasy_rust_sdk::{
    middleware::actix3::Middleware, sdk, transport::Transport, Config, StringMaskingOption,
};

async fn index_get(text: Option<String>) -> impl Responder {
    match text {
        Some(text) => HttpResponse::Ok().body(text),
        None => HttpResponse::Ok().body(""),
    }
}

async fn index_post() -> impl Responder {
    format!("test")
}

#[derive(Debug, Clone)]
pub struct GrpcMock {}

impl GrpcMock {
    pub fn new() -> Self {
        Self {}
    }
}

impl Transport for GrpcMock {
    type Output = ();
    type Error = ();

    fn send(&self, request: IngestRequest) -> Result<Self::Output, Self::Error> {
        let har: Har = serde_json::from_str(&request.har).unwrap();

        println!("{}", get_test_name(har));

        Ok(())
    }
}

fn get_test_name(har: Har) -> String {
    match har.log {
        har::Spec::V1_2(log) => log
            .entries
            .first()
            .unwrap()
            .request
            .headers
            .iter()
            .find(|h| h.name == "x-speakeasy-test-name")
            .unwrap()
            .value
            .clone(),
        har::Spec::V1_3(_) => todo!(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let config = Config {
            api_key: "test".to_string(),
            api_id: "test".to_string(),
            version_id: "test".to_string(),
        };

        // let (sender, mut receiver) = crate::async_runtime::channel(1);
        let grpc_mock = GrpcMock::new();

        // Create a new Speakeasy SDK instance
        let mut sdk = sdk::SpeakeasySdk::new_with_transport(config, grpc_mock);

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

        let speakeasy_middleware = Middleware::new(sdk);
        let (request_capture, response_capture) = speakeasy_middleware.init();

        App::new()
            .app_data(web::PayloadConfig::new(3_145_728))
            .wrap(request_capture)
            .wrap(response_capture)
            .route("/test", web::get().to(index_get))
            .route("/test", web::post().to(index_post))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}

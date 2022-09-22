use std::{fs::File, io::Write};

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use har::Har;
use speakeasy_protos::ingest::IngestRequest;
use speakeasy_rust_sdk::{
    middleware::actix3::Middleware, sdk, transport::Transport, Config, StringMaskingOption,
};

const TEST_NAME_HEADER: &str = "x-speakeasy-test-name";

async fn index_get(text: Option<String>, req: HttpRequest) -> impl Responder {
    // get header and apply masking
    let test_name = req
        .headers()
        .get(TEST_NAME_HEADER)
        .unwrap()
        .to_str()
        .unwrap();

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
        let test_name = get_test_name(har.clone());

        let test_data_folder = format!("{}/testresults", env!("CARGO_MANIFEST_DIR"));
        let test_result_file = format!("{}/{}.har", test_data_folder, test_name);

        let mut file = File::create(&test_result_file).unwrap();
        file.write_all(request.har.as_bytes()).unwrap();

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
            .find(|h| h.name == TEST_NAME_HEADER)
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

#[cfg(test)]
mod test;

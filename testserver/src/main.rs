use std::{collections::HashMap, fs::File, io::Write};

use actix_web::{
    web::{self, ReqData},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use har::Har;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use speakeasy_protos::ingest::IngestRequest;
use speakeasy_rust_sdk::{
    middleware::actix3::Middleware, sdk, transport::Transport, Config, MiddlewareController,
    StringMaskingOption,
};

const TEST_NAME_HEADER: &str = "x-speakeasy-test-name";

pub static TEST_DATA: Lazy<(HashMap<String, TestInput>, HashMap<String, har::Har>)> =
    Lazy::new(|| {
        let tests_data_folder = format!("{}/testdata", env!("CARGO_MANIFEST_DIR"));
        let mut test_inputs: HashMap<String, TestInput> = HashMap::new();
        let mut test_outputs: HashMap<String, har::Har> = HashMap::new();

        for entry in std::fs::read_dir(tests_data_folder).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            if file_name.ends_with("_input.json") {
                let file = std::fs::File::open(path).unwrap();
                let test_input: TestInput = serde_json::from_reader(file).unwrap();

                test_inputs.insert(file_name.clone().replace("_input.json", ""), test_input);
            } else if file_name.ends_with("_output.json") {
                let file = std::fs::File::open(path).unwrap();
                let test_output: har::Har = serde_json::from_reader(file).unwrap();

                test_outputs.insert(file_name.clone().replace("_output.json", ""), test_output);
            }
        }

        (test_inputs, test_outputs)
    });

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fields {
    max_capture_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Args {
    method: String,
    url: String,
    #[serde(default)]
    headers: Vec<Header>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    response_status: Option<i32>,
    #[serde(default)]
    response_body: Option<String>,
    #[serde(default)]
    response_headers: Option<Vec<Header>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    key: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInput {
    name: String,
    fields: Fields,
    args: Args,
}

async fn index_get(
    text: Option<String>,
    req: HttpRequest,
    controller: ReqData<MiddlewareController>,
) -> impl Responder {
    // get header and apply masking
    let test_data = &TEST_DATA;

    let test_name = req
        .headers()
        .get(TEST_NAME_HEADER)
        .unwrap()
        .to_str()
        .unwrap();

    let test_inputs = test_data.0.clone();

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

pub fn get_entry(har: Har) -> har::v1_2::Entries {
    match har.log {
        har::Spec::V1_2(log) => log.entries.first().unwrap().clone(),
        har::Spec::V1_3(_) => todo!(),
    }
}

fn get_test_name(har: Har) -> String {
    get_entry(har)
        .request
        .headers
        .iter()
        .find(|h| h.name == TEST_NAME_HEADER)
        .unwrap()
        .value
        .clone()
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

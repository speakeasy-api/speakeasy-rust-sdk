use std::{collections::HashMap, sync::Arc};

#[cfg(test)]
use actix3::{get, HttpResponse, Responder};
use actix3::{test, web, App};
use serde::{Deserialize, Serialize};

use crate::{
    middleware::actix3::Middleware, sdk::SpeakeasySdk, transport::tests::GrpcMock, Config,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Fields {
    max_capture_size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    key: String,
    values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestInput {
    name: String,
    fields: Fields,
    args: Args,
}

async fn index_get(text: Option<String>) -> impl Responder {
    match text {
        Some(text) => HttpResponse::Ok().body(text),
        None => HttpResponse::Ok().body(""),
    }
}

async fn index_post() -> impl Responder {
    format!("test")
}

impl SpeakeasySdk<Arc<GrpcMock>> {
    fn new(config: Config, transport: Arc<GrpcMock>) -> Self {
        Self {
            transport,
            config: config.into(),
            masking: Default::default(),
        }
    }
}

#[actix_rt::test]
async fn integration_tests() {
    let tests_data_folder = format!("{}/tests/testdata", env!("CARGO_MANIFEST_DIR"));
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

    for (test_name, test_input) in test_inputs {
        println!("running test: {}", test_name);

        let config = Config {
            api_key: "test".to_string(),
            api_id: "test".to_string(),
            version_id: "test".to_string(),
        };

        let (sender, mut receiver) = crate::async_runtime::channel(1);

        let grpc_mock = Arc::new(GrpcMock::new(sender));

        // Create a new Speakeasy SDK instance
        let sdk = SpeakeasySdk::new(config, grpc_mock.clone());

        let speakeasy_middleware = Middleware::new(sdk);
        let (request_capture, response_capture) = speakeasy_middleware.init();

        let mut app = test::init_service(
            App::new()
                .route("/test", web::get().to(index_get))
                .route("/test", web::post().to(index_post))
                .wrap(request_capture)
                .wrap(response_capture),
        )
        .await;

        let method = test_input
            .args
            .method
            .to_uppercase()
            .as_str()
            .parse()
            .unwrap();

        let mut req_builder = test::TestRequest::with_uri(&test_input.args.url).method(method);

        for header in &test_input.args.headers {
            for value in &header.values {
                req_builder = req_builder.header(header.key.clone(), &**value);
            }
        }

        let req = req_builder.to_request();

        let _resp = test::call_service(&mut app, req).await;
        let args = test_input.args;

        if let Some(har) = receiver.recv().await {
            println!("HAR {:#?}", har);
        }

        // assert!(grpc_mock.lock().unwrap().response.is_some());
    }
}

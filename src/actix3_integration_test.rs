use std::{collections::HashMap, sync::Arc};

use actix3::{
    client::{Client, ClientRequest},
    rt::Arbiter,
    rt::System,
    test, web, App, HttpServer,
};
#[cfg(test)]
use actix3::{get, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio02 as tokio;
use tokio02::runtime::Runtime;

use crate::{
    async_runtime, middleware::actix3::Middleware, sdk::SpeakeasySdk, transport::tests::GrpcMock,
    Config, StringMaskingOption,
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

impl SpeakeasySdk<Arc<GrpcMock>> {
    fn new(config: Config, transport: Arc<GrpcMock>) -> Self {
        Self {
            transport,
            config: config.into(),
            masking: Default::default(),
        }
    }
}

#[test]
fn integration_tests() {
    let mut system = actix_rt::System::new("test");
    let arbiter = Arbiter::new();

    system.block_on(async {
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

            let mut client = Client::default().get("http://localhost:8080/test");

            for header in &test_input.args.headers {
                for value in &header.values {
                    client = client.header(header.key.clone(), &**value);
                }
            }

            client = client.header("x-speakeasy-test-name", &*test_name);
            client.send().await;

            // if let Some(har) = receiver.recv().await {
            //     println!("HAR {:#?}", har);
            // }

            // assert!(grpc_mock.lock().unwrap().response.is_some());
        }
    });
}

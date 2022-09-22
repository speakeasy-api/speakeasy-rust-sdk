use crate::{get_entry, TestInput, TEST_DATA};
use actix_web::client::Client;
use har::{v1_2::Headers, Har};
use pretty_assertions::assert_eq;
use std::{collections::HashMap, io::Read};

#[test]
fn integration_tests() {
    let mut system = actix_rt::System::new("test");

    system.block_on(async {
        let tests_results_folder = format!("{}/testresults", env!("CARGO_MANIFEST_DIR"));
        let test_data = &TEST_DATA;
        let test_inputs = test_data.0.clone();
        let test_outputs = test_data.1.clone();

        for (test_name, test_input) in test_inputs {
            println!("running test: {}", test_name);

            let mut client = Client::default().get("http://localhost:8080/test");

            for header in &test_input.args.headers {
                for value in &header.values {
                    client = client.header(header.key.clone(), &**value);
                }
            }

            client = client.header("x-speakeasy-test-name", &*test_name);
            let res = client.send().await.unwrap();
            println!("response: {:#?}", res.headers());
        }

        for (test_name, test_output) in test_outputs {
            println!("checking response for: {}", test_name);

            let want_har = test_output;

            let got_har_file_name = format!("{}/{}.har", tests_results_folder, test_name);
            let mut got_har_file = std::fs::File::open(&got_har_file_name).unwrap();
            let mut got_har_string = String::new();

            got_har_file.read_to_string(&mut got_har_string).unwrap();
            let got_har: Har = serde_json::from_str(&got_har_string).unwrap();

            let got_har_entry = get_entry(got_har);
            let want_har_entry = get_entry(want_har);

            // check headers
            assert_eq!(
                got_har_entry
                    .request
                    .headers
                    .clone()
                    .into_iter()
                    .filter(|h| h.name != "x-speakeasy-test-name")
                    .filter(|h| if h.name == "content-length" && h.value == "0" {
                        false
                    } else {
                        true
                    })
                    .filter(|h| h.name != "date")
                    .collect::<Vec<_>>(),
                want_har_entry
                    .request
                    .headers
                    .clone()
                    .into_iter()
                    .filter(|h| h.name != "connection")
                    .collect::<Vec<_>>()
            )
        }
    });
}

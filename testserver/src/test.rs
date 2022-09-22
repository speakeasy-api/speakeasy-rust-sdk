use crate::{get_entry, TestInput, TEST_DATA};
use actix_web::{client::Client, http::Cookie};
use har::{v1_2::Headers, Har};
use pretty_assertions::assert_eq;
use std::{collections::HashMap, io::Read, time::Duration};

#[test]
fn integration_tests() {
    let mut system = actix_rt::System::new("test");

    system.block_on(async {
        let tests_results_folder = format!("{}/testresults", env!("CARGO_MANIFEST_DIR"));

        std::fs::remove_dir_all(&tests_results_folder).unwrap();
        std::fs::create_dir(&tests_results_folder).unwrap();

        let test_data = &TEST_DATA;
        let test_inputs = test_data.0.clone();
        let test_outputs = test_data.1.clone();

        for (test_name, test_input) in test_inputs {
            println!("running test: {}", test_name);

            let mut client = if test_input.args.method == "POST" {
                Client::default().post("http://localhost:8080/test")
            } else {
                Client::default().get("http://localhost:8080/test")
            };

            for header in &test_input.args.headers {
                for value in &header.values {
                    client = client.header(header.key.clone(), &**value);

                    if header.key == "Cookie" {
                        client =
                            client.cookie(Cookie::build(header.key.clone(), &**value).finish());
                    }
                }
            }

            client = client.header("x-speakeasy-test-name", &*test_name);

            let _res = if test_input.args.method == "POST" {
                client
                    .send_body(test_input.args.body.clone().unwrap())
                    .await
                    .unwrap()
            } else {
                client.send().await.unwrap()
            };
        }

        // wait for files to be created
        std::thread::sleep(Duration::from_secs(1));

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

            let mut got_headers = got_har_entry.request.headers.clone();
            got_headers.sort_by_key(|h| h.name.clone());

            let mut want_headers = want_har_entry.request.headers.clone();
            want_headers.sort_by_key(|h| h.name.clone());

            // check request headers
            assert_eq!(
                got_headers
                    .into_iter()
                    .filter(|h| h.name != "x-speakeasy-test-name")
                    .filter(|h| if h.name == "content-length" && h.value == "0" {
                        false
                    } else {
                        true
                    })
                    .filter(|h| h.name != "date")
                    .collect::<Vec<_>>(),
                want_headers
                    .into_iter()
                    .filter(|h| h.name != "connection")
                    .collect::<Vec<_>>()
            );

            // check response headers
            let mut got_headers = got_har_entry.response.headers.clone();
            got_headers.sort_by_key(|h| h.name.clone());

            let mut want_headers = want_har_entry.response.headers.clone();
            want_headers.sort_by_key(|h| h.name.clone());

            // check request headers
            assert_eq!(
                got_headers
                    .into_iter()
                    .filter(|h| h.name != "date")
                    .collect::<Vec<_>>(),
                want_headers.into_iter().collect::<Vec<_>>()
            )
        }
    });
}

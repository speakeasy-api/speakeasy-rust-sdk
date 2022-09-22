use crate::{get_entry, get_log, TEST_DATA};
use actix_web::client::Client;
use har::Har;
use itertools::Itertools;
use pretty_assertions::assert_eq;
use std::{io::Read, time::Duration};

#[test]
fn integration_tests() {
    let mut system = actix_rt::System::new("test");

    system.block_on(async {
        let tests_results_folder = format!("{}/testresults", env!("CARGO_MANIFEST_DIR"));

        let _ = std::fs::remove_dir_all(&tests_results_folder);
        std::fs::create_dir(&tests_results_folder).unwrap();

        let test_data = &TEST_DATA;
        let test_inputs = test_data.0.clone();
        let test_outputs = test_data.1.clone();

        for (test_name, test_input) in test_inputs {
            println!("running test: {}", test_name);

            let mut client = if test_input.args.method == "POST" {
                Client::default().post(test_input.args.url)
            } else {
                Client::default().get(test_input.args.url)
            };

            for header in &test_input.args.headers {
                for value in &header.values {
                    client = client.header(header.key.clone(), &**value);
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

            let got_har_log = get_log(got_har.clone());
            let want_har_log = get_log(want_har.clone());

            let got_har_entry = get_entry(got_har);
            let want_har_entry = get_entry(want_har);

            // check log parent fields
            assert_eq!(got_har_log.creator, want_har_log.creator);
            assert_eq!(got_har_log.comment, want_har_log.comment);

            // check request headers
            let mut got_headers = got_har_entry.request.headers.clone();
            got_headers.sort_by_key(|h| h.name.clone());

            let mut want_headers = want_har_entry.request.headers.clone();
            want_headers.sort_by_key(|h| h.name.clone());

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

            // check request cookies
            let mut got_cookies = got_har_entry.request.cookies.clone();
            got_cookies.sort_by_key(|h| h.name.clone());

            let mut want_cookies = want_har_entry.request.cookies.clone();
            want_cookies.sort_by_key(|h| h.name.clone());

            assert_eq!(
                got_cookies
                    .into_iter()
                    .filter(|h| h.name != "x-speakeasy-test-name")
                    .filter(|h| if h.name == "content-length" && h.value == "0" {
                        false
                    } else {
                        true
                    })
                    .filter(|h| h.name != "date")
                    .collect::<Vec<_>>(),
                want_cookies
                    .into_iter()
                    .filter(|h| h.name != "connection")
                    .collect::<Vec<_>>()
            );

            assert_eq!(
                got_har_entry.request.query_string,
                want_har_entry.request.query_string
            );
            assert_eq!(
                got_har_entry.request.body_size,
                want_har_entry.request.body_size
            );
            assert_eq!(got_har_entry.request.method, want_har_entry.request.method);
            assert_eq!(got_har_entry.request.url, want_har_entry.request.url);
            assert_eq!(
                got_har_entry.server_ip_address,
                want_har_entry.server_ip_address
            );
            assert_eq!(
                got_har_entry.request.http_version,
                want_har_entry.request.http_version
            );

            // RESPONSE TESTS
            assert_eq!(
                got_har_entry.response.redirect_url.unwrap_or_default(),
                want_har_entry.response.redirect_url.unwrap_or_default()
            );

            // check request headers
            let mut got_headers = got_har_entry.response.headers.clone();
            got_headers.sort_by_key(|h| h.value.clone());

            let mut want_headers = want_har_entry.response.headers.clone();
            want_headers.sort_by_key(|h| h.value.clone());

            assert_eq!(
                got_headers
                    .into_iter()
                    .map(|mut h| {
                        if h.name == "set-cookie" {
                            h.value = h.value.chars().sorted().rev().collect::<String>();
                        }
                        h
                    })
                    .collect::<Vec<_>>(),
                want_headers
                    .into_iter()
                    .map(|mut h| {
                        if h.name == "set-cookie" {
                            h.value = h.value.chars().sorted().rev().collect::<String>();
                        }
                        h
                    })
                    .collect::<Vec<_>>()
            );

            // check response cookies
            let mut got_cookies = got_har_entry.response.cookies.clone();
            got_cookies.sort_by_key(|h| h.name.clone());

            let mut want_cookies = want_har_entry.response.cookies.clone();
            want_cookies.sort_by_key(|h| h.name.clone());

            assert_eq!(
                got_cookies,
                want_cookies
                    .into_iter()
                    .map(|mut c| {
                        if c.expires
                            .as_deref()
                            .unwrap_or_default()
                            .starts_with("2020-01-01T01:00:00")
                        {
                            c.expires = None
                        }
                        c
                    })
                    .collect::<Vec<_>>()
            );

            assert_eq!(
                got_har_entry.response.http_version,
                want_har_entry.response.http_version
            );

            assert_eq!(
                got_har_entry.response.http_version,
                want_har_entry.response.http_version
            );

            assert_eq!(
                got_har_entry.response.status_text,
                want_har_entry.response.status_text
            );
        }
    });
}

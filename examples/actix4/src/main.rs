use std::sync::{Arc, RwLock};

use actix_web::{
    get, post,
    web::{self, ReqData},
    App, HttpResponse, HttpServer, Responder,
};
use log::info;
use speakeasy_rust_sdk::{
    masking::StringMaskingOption, middleware::actix4::Middleware, Config, Masking,
    MiddlewareController, SpeakeasySdk,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Person {
    name: String,
    age: i32,
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[post("/")]
async fn index(item: web::Json<Person>) -> HttpResponse {
    println!("json: {:?}", &item);
    HttpResponse::Ok().json(item.0)
}

#[post("/upload")]
async fn upload(item: web::Bytes) -> impl Responder {
    println!("bytes length: {:?}", item.len());
    use std::{fs::File, io::Write};

    let mut file = File::create("uploads/copied.mov").unwrap();
    file.write_all(&item).unwrap();

    "Uploaded".to_string()
}

#[post("/use_controller")]
async fn use_controller(
    item: web::Json<Person>,
    controller: ReqData<Arc<RwLock<MiddlewareController>>>,
) -> HttpResponse {
    println!("json: {:?}", &item);

    // create a specific masking for this request/response
    let mut masking = Masking::default();
    masking.with_request_field_mask_string("name", "NoOne");
    masking.with_response_field_mask_number("age", 22);

    controller
        .write()
        .unwrap()
        .set_path_hint("/use_controller/*");

    controller.write().unwrap().set_masking(masking);

    controller
        .write()
        .unwrap()
        .set_customer_id("123customer_id".to_string());

    HttpResponse::Ok().json(item.0)
}

#[get("/print_access_token")]
async fn print_access_token(app_state: web::Data<AppState>) -> impl Responder {
    use speakeasy_rust_sdk::speakeasy_protos::embedaccesstoken::{
        embed_access_token_request::Filter, EmbedAccessTokenRequest,
    };

    let request = EmbedAccessTokenRequest {
        filters: vec![Filter {
            key: "customer_id".to_string(),
            operator: "=".to_string(),
            value: "a_customer_id".to_string(),
        }],
        ..Default::default()
    };

    let token_response = app_state
        .speakeasy_sdk
        .get_embedded_access_token(request)
        .await
        .unwrap();

    format!("Access token: {}", token_response.access_token)
}

struct AppState {
    speakeasy_sdk: SpeakeasySdk,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    info!("starting HTTP server at http://localhost:8080");

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
        sdk.masking().with_query_string_mask("secret", "********");
        sdk.masking()
            .with_query_string_mask("password", StringMaskingOption::default());

        // Configure masking for request
        sdk.masking()
            .with_request_field_mask_string("password", StringMaskingOption::default());

        // Configure masking for response
        sdk.masking()
            .with_response_field_mask_string("secret", StringMaskingOption::default());

        // AppState
        let app_state = AppState {
            speakeasy_sdk: sdk.clone(),
        };

        let speakeasy_middleware = Middleware::new(sdk);
        let (request_capture, response_capture) = speakeasy_middleware.into();

        App::new()
            .app_data(web::PayloadConfig::new(3_145_728))
            .app_data(web::Data::new(app_state))
            .wrap(request_capture)
            .wrap(response_capture)
            .service(greet)
            .service(index)
            .service(upload)
            .service(use_controller)
            .service(print_access_token)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

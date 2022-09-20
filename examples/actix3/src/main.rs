use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::info;
use speakeasy_rust_sdk::{
    middleware::actix3::Middleware, Config, Masking, MiddlewareController, SpeakeasySdk,
    StringMaskingOption,
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

    let mut file = File::create("uploads/copied.png").unwrap();
    file.write_all(&item).unwrap();

    format!("Uploaded!")
}

#[post("/use_controller")]
async fn use_controller(item: web::Json<Person>, req: HttpRequest) -> HttpResponse {
    println!("json: {:?}", &item);

    let ext = req.head().extensions();
    let controller = ext.get::<MiddlewareController>().unwrap();

    // create a specific masking for this request/response
    let mut masking = Masking::default();
    masking.with_request_field_mask_string("name", "NoOne");
    masking.with_response_field_mask_number("age", 22);

    controller.set_path_hint("/use_controller/*").await;
    controller.set_masking(masking).await;

    HttpResponse::Ok().json(item.0)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    info!("starting HTTP server at http://localhost:8080");

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

        let speakeasy_middleware = Middleware::new(sdk);
        let (request_capture, response_capture) = speakeasy_middleware.init();

        App::new()
            .app_data(web::PayloadConfig::new(3_145_728))
            .wrap(request_capture)
            .wrap(response_capture)
            .service(greet)
            .service(index)
            .service(upload)
            .service(use_controller)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

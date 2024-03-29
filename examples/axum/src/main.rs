use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use speakeasy_rust_sdk::{
    masking::StringMaskingOption, middleware::axum::Middleware, Config, Masking,
    MiddlewareController, SpeakeasySdk,
};

use tower::ServiceBuilder;

use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Person {
    name: String,
    age: i32,
}

async fn greet(name: Path<String>) -> impl IntoResponse {
    format!("Hello {}!", name.as_str())
}

async fn index(item: Json<Person>) -> impl IntoResponse {
    println!("json: {:?}", &item);
    (StatusCode::CREATED, Json(item.0))
}

async fn upload(item: Bytes) -> impl IntoResponse {
    println!("bytes length: {:?}", item.len());
    use std::{fs::File, io::Write};

    let mut file = File::create("uploads/copied.mov").unwrap();
    file.write_all(&item).unwrap();

    "Uploaded".to_string()
}

async fn use_controller(
    Extension(controller): Extension<Arc<RwLock<MiddlewareController>>>,
    Json(item): Json<Person>,
) -> impl IntoResponse {
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

    (StatusCode::CREATED, Json(item))
}

async fn print_access_token(State(app_state): State<AppState>) -> impl IntoResponse {
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

#[derive(Debug, Clone)]
struct AppState {
    speakeasy_sdk: Arc<SpeakeasySdk>,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

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

    let app_state = AppState {
        speakeasy_sdk: Arc::new(sdk.clone()),
    };

    let speakeasy_middleware = Middleware::new(sdk);
    let (request_capture, response_capture) = speakeasy_middleware.into();

    // build our application with a route
    let app = Router::new()
        .route("/", post(index))
        .route("/greet/:name", get(greet))
        .route("/use_controller", post(use_controller))
        .route("/upload", post(upload))
        .route("/print_access_token", get(print_access_token))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 5))
        .layer(ServiceBuilder::new().layer(request_capture))
        .layer(ServiceBuilder::new().layer(response_capture))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

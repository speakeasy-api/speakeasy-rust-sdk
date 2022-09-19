use crate::generic_http::GenericRequest;
use actix3::dev::ServiceRequest;

impl From<ServiceRequest> for GenericRequest {
    fn from(request: ServiceRequest) -> Self {
        GenericRequest {
            method: request.method().to_string(),
            hostname: Some(request.connection_info().host().to_string()),
            url: request.uri().to_string(),
            protocol: todo!(),
            http_version: todo!(),
            headers: todo!(),
            cookies: todo!(),
            port: todo!(),
            body: todo!(),
        }
    }
}

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, web};
use opentelemetry_semantic_conventions as semconv;
use serde::{Deserialize, Serialize};
use tapo::ApiClient;
use tracing::instrument;

use crate::settings::Tapo;
use crate::system::api::errors::ApiError;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiStatusResponse {
    pub code: u16,
    pub message: String,
}

impl ApiStatusResponse {
    pub fn new(status_code: StatusCode, message: &str) -> Self {
        Self {
            code: status_code.as_u16(),
            message: message.to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct SetDevicePayload {
    ip_address: String,
    device_on: bool,
}

#[derive(Deserialize)]
pub struct GetDevicePayload {
    ip_address: String,
}

#[derive(Serialize)]
pub struct DeviceResponse {
    ip_address: String,
    device_on: Option<bool>,
}

#[instrument(name = "health_check", skip_all, fields(
    http.request.method = "GET",
    http.route = "/health-check",
    http.response.status_code = tracing::field::Empty,
    otel.kind = "server"
))]
pub async fn health_check() -> HttpResponse {
    let body = ApiStatusResponse::new(StatusCode::OK, "OK");

    tracing::Span::current().record(semconv::attribute::HTTP_RESPONSE_STATUS_CODE, 200);
    HttpResponse::Ok().json(body)
}

#[instrument(name = "get_device", skip_all, fields(
    http.request.method = "GET",
    http.route = "/device", 
    device.ip_address = %device.ip_address,
    http.response.status_code = tracing::field::Empty,
    otel.kind = "server"
))]
pub async fn get_device(
    config: web::Data<Tapo>,
    device: web::Json<GetDevicePayload>,
) -> Result<HttpResponse, ApiError> {
    let client = ApiClient::new(config.username.clone(), config.password.clone());
    let handler = client
        .generic_device(device.ip_address.clone())
        .await
        .map_err(|_| ApiError::BadRequest("failed to connect to the device".to_string()))?;

    let device_info = handler
        .get_device_info()
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let result = DeviceResponse {
        ip_address: device.ip_address.clone(),
        device_on: device_info.device_on,
    };

    tracing::Span::current().record(semconv::attribute::HTTP_RESPONSE_STATUS_CODE, 200);
    Ok(HttpResponse::Ok().json(result))
}

#[instrument(name = "set_device", skip_all, fields(
    http.request.method = "POST",
    http.route = "/device",
    device.ip_address = %device.ip_address,
    device.state = %device.device_on,
    http.response.status_code = tracing::field::Empty,
    otel.kind = "server"
))]
pub async fn set_device(
    config: web::Data<Tapo>,
    device: web::Json<SetDevicePayload>,
) -> Result<HttpResponse, ApiError> {
    let client = ApiClient::new(config.username.clone(), config.password.clone());
    let handler = client
        .generic_device(device.ip_address.clone())
        .await
        .map_err(|_| ApiError::BadRequest("failed to connect to the device".to_string()))?;

    if device.device_on {
        handler
            .on()
            .await
            .map_err(|_| ApiError::InternalServerError)?
    } else {
        handler
            .off()
            .await
            .map_err(|_| ApiError::InternalServerError)?
    }

    let result = DeviceResponse {
        ip_address: device.ip_address.clone(),
        device_on: Some(device.device_on),
    };

    tracing::Span::current().record(semconv::attribute::HTTP_RESPONSE_STATUS_CODE, 200);
    Ok(HttpResponse::Ok().json(result))
}

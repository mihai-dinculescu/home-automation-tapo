use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tapo::{ApiClient, GenericDevice};

use crate::settings::Tapo;
use crate::system::api::errors::ApiError;

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
    device_on: bool,
}

pub async fn get_device(
    config: web::Data<Tapo>,
    device: web::Json<GetDevicePayload>,
) -> Result<HttpResponse, ApiError> {
    let client = ApiClient::<GenericDevice>::new(
        device.ip_address.clone(),
        config.username.clone(),
        config.password.clone(),
        true,
    )
    .await
    .map_err(|_| ApiError::BadRequest("failed to connect to the device".to_string()))?;

    let device_info = client
        .get_device_info()
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let result = DeviceResponse {
        ip_address: device.ip_address.clone(),
        device_on: device_info.device_on,
    };

    Ok(HttpResponse::Ok().json(result))
}

pub async fn set_device(
    config: web::Data<Tapo>,
    device: web::Json<SetDevicePayload>,
) -> Result<HttpResponse, ApiError> {
    let client = ApiClient::<GenericDevice>::new(
        device.ip_address.clone(),
        config.username.clone(),
        config.password.clone(),
        true,
    )
    .await
    .map_err(|_| ApiError::BadRequest("failed to connect to the device".to_string()))?;

    if device.device_on {
        client
            .on()
            .await
            .map_err(|_| ApiError::InternalServerError)?
    } else {
        client
            .off()
            .await
            .map_err(|_| ApiError::InternalServerError)?
    }

    let result = DeviceResponse {
        ip_address: device.ip_address.clone(),
        device_on: device.device_on,
    };

    Ok(HttpResponse::Ok().json(result))
}

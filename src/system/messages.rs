use std::sync::Arc;

use actix::Message;
use tapo::{ApiClient, DeviceUsageResult, GenericDevice};

use crate::settings::Device;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DevicesHealthCheckMessage;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GetDeviceDataMessage {
    pub client: Arc<ApiClient<GenericDevice>>,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DeviceUsageMessage {
    pub device: Device,
    pub device_usage: DeviceUsageResult,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct StopMessage;

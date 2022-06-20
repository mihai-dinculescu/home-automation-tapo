use actix::Message;
use tapo::DeviceUsageResult;

use crate::settings::Device;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DevicesHealthCheckMessage;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GetDeviceDataMessage;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DeviceUsageMessage {
    pub device: Device,
    pub device_usage: DeviceUsageResult,
}

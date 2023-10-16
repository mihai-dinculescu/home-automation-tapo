use actix::Message;
use tapo::responses::DeviceUsageEnergyMonitoringResult;

use crate::settings::Device;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HealthCheckMessage;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GetDeviceDataMessage;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DeviceUsageMessage {
    pub device: Device,
    pub device_usage: DeviceUsageEnergyMonitoringResult,
}

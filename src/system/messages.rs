use actix::Message;
use tapo::responses::DeviceUsageEnergyMonitoringResult;

use crate::settings::Device;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HealthCheckMessage {
    pub span_context: opentelemetry::Context,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GetDeviceDataMessage {
    pub span_context: opentelemetry::Context,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DeviceUsageMessage {
    pub span_context: opentelemetry::Context,
    pub device: Device,
    pub device_usage: DeviceUsageEnergyMonitoringResult,
}

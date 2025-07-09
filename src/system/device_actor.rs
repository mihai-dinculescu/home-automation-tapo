use std::time::Duration;

use actix::{Actor, Addr, AsyncContext, Context, Handler, WrapFuture, clock::interval};
use opentelemetry_semantic_conventions as semconv;
use tapo::ApiClient;
use tracing::{Instrument, debug, error, instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

use crate::{
    settings::{Device, Tapo},
    system::messages::{DeviceUsageMessage, GetDeviceDataMessage},
};

use super::coordinator_actor::CoordinatorActor;

#[derive(Debug)]
pub struct DeviceActor {
    coordinator_actor_addr: Addr<CoordinatorActor>,
    config: Tapo,
    device: Device,
}

impl DeviceActor {
    #[instrument(name = "DeviceActor::new", skip_all, fields(
        device.name = %device.name,
        device.ip_address = %device.ip_address,
    ))]
    pub fn new(
        coordinator_actor_addr: Addr<CoordinatorActor>,
        config: Tapo,
        device: Device,
    ) -> Self {
        Self {
            coordinator_actor_addr,
            config,
            device,
        }
    }

    #[instrument(name = "DeviceActor::query_device_usage", skip_all, fields(
        device.name = %device.name,
        device.ip_address = %device.ip_address,
        otel.status_code = tracing::field::Empty,
        error.type = tracing::field::Empty,
        exception.type = tracing::field::Empty,
        exception.message = tracing::field::Empty,
        exception.stacktrace = tracing::field::Empty,
    ))]
    async fn query_device_usage(
        device: Device,
        tapo_username: String,
        tapo_password: String,
        coordinator_actor_addr: Addr<CoordinatorActor>,
    ) {
        let span = tracing::Span::current();

        let result = async {
            let client = ApiClient::new(tapo_username, tapo_password);
            let handler = client.p110(device.ip_address.clone()).await?;

            let device_usage = handler.get_device_usage().await?;

            Ok::<_, tapo::Error>(device_usage)
        }
        .await;

        match result {
            Ok(device_usage) => {
                let result = coordinator_actor_addr.try_send(DeviceUsageMessage {
                    span_context: span.context(),
                    device,
                    device_usage,
                });

                if let Err(e) = result {
                    span.record(semconv::attribute::OTEL_STATUS_CODE, "ERROR");
                    span.record(
                        semconv::attribute::ERROR_TYPE,
                        "SendError<DeviceUsageMessage>",
                    );
                    span.record(
                        semconv::attribute::EXCEPTION_TYPE,
                        "SendError<DeviceUsageMessage>",
                    );
                    span.record(semconv::attribute::EXCEPTION_MESSAGE, e.to_string());
                    span.record(semconv::attribute::EXCEPTION_STACKTRACE, format!("{e:?}"));
                } else {
                    span.record(semconv::attribute::OTEL_STATUS_CODE, "OK");
                }
            }
            Err(e) => {
                error!(
                    "Failed to query device usage for '{}': {:?}",
                    device.name, e
                );
                span.record(semconv::attribute::OTEL_STATUS_CODE, "ERROR");
                span.record(semconv::attribute::ERROR_TYPE, "tapo::Error");
                span.record(semconv::attribute::EXCEPTION_TYPE, "tapo::Error");
                span.record(semconv::attribute::EXCEPTION_MESSAGE, e.to_string());
                span.record(semconv::attribute::EXCEPTION_STACKTRACE, format!("{e:?}"));
            }
        }
    }
}

impl Actor for DeviceActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Device Actor for '{}' started...", self.device.name);

        let addr = ctx.address();
        let refresh_rate = Duration::from_secs(self.config.refresh_rate_s);

        let device_name = self.device.name.clone();
        let device_ip_address = self.device.ip_address.clone();

        let fut = async move {
            let mut interval = interval(refresh_rate);

            loop {
                interval.tick().await;

                let span = tracing::info_span!(
                    "DeviceActor::IntervalTick",
                    otel.kind = "producer",
                    messaging.system = "actix",
                    messaging.message.id = "GetDeviceDataMessage",
                    messaging.operation.name = "send",
                    messaging.operation.type = "send",
                    messaging.destination.name = "DeviceActor",
                    device.name = %device_name,
                    device.ip_address = %device_ip_address,
                    status_code = tracing::field::Empty,
                    error.type = tracing::field::Empty,
                    exception.type = tracing::field::Empty,
                    exception.message = tracing::field::Empty,
                    exception.stacktrace = tracing::field::Empty,
                );
                let _enter = span.enter();

                if let Err(e) = addr.try_send(GetDeviceDataMessage {
                    span_context: span.context(),
                }) {
                    span.record(
                        semconv::attribute::ERROR_TYPE,
                        "SendError<GetDeviceDataMessage>",
                    );
                    span.record(
                        semconv::attribute::EXCEPTION_TYPE,
                        "SendError<GetDeviceDataMessage>",
                    );
                    span.record(semconv::attribute::EXCEPTION_MESSAGE, e.to_string());
                    span.record(semconv::attribute::EXCEPTION_STACKTRACE, format!("{e:?}"));
                } else {
                    span.record(semconv::attribute::OTEL_STATUS_CODE, "OK");
                }
            }
        }
        .into_actor(self);

        ctx.spawn(fut);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("Device Actor for '{}' stopped.", self.device.name);
    }
}

impl Handler<GetDeviceDataMessage> for DeviceActor {
    type Result = ();

    #[instrument(
        name = "DeviceActor::Handler<GetDeviceDataMessage>",
        skip_all,
        fields(
            otel.kind = "consumer",
            messaging.system = "actix",
            messaging.message.id = "GetDeviceDataMessage",
            messaging.operation.name = "poll",
            messaging.operation.type = "receive",
            messaging.destination.name = "DeviceActor",
            device.name = %self.device.name,
            device.ip_address = %self.device.ip_address,
        )
    )]
    fn handle(&mut self, message: GetDeviceDataMessage, ctx: &mut Context<Self>) -> Self::Result {
        let span = tracing::Span::current();
        span.set_parent(message.span_context);

        let device = self.device.clone();
        let tapo_username = self.config.username.clone();
        let tapo_password = self.config.password.clone();
        let coordinator_actor_addr = self.coordinator_actor_addr.clone();

        let fut =
            Self::query_device_usage(device, tapo_username, tapo_password, coordinator_actor_addr)
                .instrument(span)
                .into_actor(self);

        ctx.spawn(fut);
    }
}

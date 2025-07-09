use std::collections::HashMap;
use std::time::Duration;

use actix::clock::interval;
use actix::{Actor, Addr, AsyncContext, Context, Handler, WrapFuture};
use opentelemetry_semantic_conventions as semconv;
use tracing::{debug, info, instrument, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::settings::Settings;
use crate::system::api::api_actor::ApiActor;
use crate::system::device_actor::DeviceActor;
use crate::system::messages::{DeviceUsageMessage, HealthCheckMessage};
use crate::system::mqtt_actor::MqttActor;

#[derive(Debug)]
pub struct CoordinatorActor {
    settings: Settings,
    api_actor_addr: Addr<ApiActor>,
    mqtt_actor_addr: Addr<MqttActor>,
    device_actors: HashMap<String, Addr<DeviceActor>>,
}

impl CoordinatorActor {
    pub fn new(settings: Settings) -> Self {
        let mqtt_actor = MqttActor::new(settings.mqtt.clone());
        let mqtt_actor_addr = mqtt_actor.start();

        let api_actor = ApiActor::new(settings.api.clone(), settings.tapo.clone());
        let api_actor_addr = api_actor.start();

        Self {
            settings,
            api_actor_addr,
            mqtt_actor_addr,
            device_actors: HashMap::new(),
        }
    }
}

impl Actor for CoordinatorActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Coordinator Actor started...");

        let addr = ctx.address();

        let fut = async move {
            let mut interval = interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let span = tracing::info_span!(
                    "CoordinatorActor::IntervalTick",
                    otel.kind = "producer",
                    messaging.system = "actix",
                    messaging.message.id = "HealthCheckMessage",
                    messaging.operation.name = "send",
                    messaging.operation.type = "send",
                    messaging.destination.name = "CoordinatorActor",
                    error.type = tracing::field::Empty,
                    exception.type = tracing::field::Empty,
                    exception.message = tracing::field::Empty,
                    exception.stacktrace = tracing::field::Empty,
                );
                let _enter = span.enter();

                if let Err(e) = addr.try_send(HealthCheckMessage {
                    span_context: span.context(),
                }) {
                    span.record(
                        semconv::attribute::ERROR_TYPE,
                        "SendError<HealthCheckMessage>",
                    );
                    span.record(
                        semconv::attribute::EXCEPTION_TYPE,
                        "SendError<HealthCheckMessage>",
                    );
                    span.record(semconv::attribute::EXCEPTION_MESSAGE, e.to_string());
                    span.record(semconv::attribute::EXCEPTION_STACKTRACE, format!("{e:?}"));
                }
            }
        }
        .into_actor(self);

        ctx.spawn(fut);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("Coordinator Actor stopped.");
    }
}

impl Handler<HealthCheckMessage> for CoordinatorActor {
    type Result = ();

    #[instrument(
        name = "CoordinatorActor::Handler<HealthCheckMessage>",
        skip_all,
        fields(
            otel.kind = "consumer",
            messaging.system = "actix",
            messaging.message.id = "HealthCheckMessage",
            messaging.operation.name = "poll",
            messaging.operation.type = "receive",
            messaging.destination.name = "CoordinatorActor",
        )
    )]
    fn handle(&mut self, message: HealthCheckMessage, ctx: &mut Context<Self>) -> Self::Result {
        tracing::Span::current().set_parent(message.span_context);

        // check api
        if !self.api_actor_addr.connected() {
            warn!("API Actor is not connected, restarting...");
            let api_actor = ApiActor::new(self.settings.api.clone(), self.settings.tapo.clone());
            self.api_actor_addr = api_actor.start();
        }

        // check mqtt
        if !self.mqtt_actor_addr.connected() {
            warn!("MQTT Actor is not connected, restarting...");
            let mqtt_actor = MqttActor::new(self.settings.mqtt.clone());
            self.mqtt_actor_addr = mqtt_actor.start();
        }

        // check devices
        let addr = ctx.address();

        for device in &self.settings.devices {
            if self.device_actors.contains_key(&device.ip_address) {
                let device_actor = self.device_actors.get(&device.ip_address);

                if let Some(device_actor) = device_actor {
                    if device_actor.connected() {
                        // device actor is alive and well, nothing to do here
                        continue;
                    } else {
                        warn!(
                            device.name = device.name,
                            device.ip_address = device.ip_address,
                            "Device actor is not connected, restarting...",
                        );
                    }
                }
            } else {
                info!(
                    device.name = device.name,
                    device.ip_address = device.ip_address,
                    "Device actor not found, creating a new one...",
                );
            }

            // device actor hasn't been created yet or has died -> (re)create
            let device_actor =
                DeviceActor::new(addr.clone(), self.settings.tapo.clone(), device.clone());
            let device_actor_addr = device_actor.start();

            self.device_actors
                .insert(device.ip_address.clone(), device_actor_addr);
        }
    }
}

impl Handler<DeviceUsageMessage> for CoordinatorActor {
    type Result = ();

    #[instrument(
        name = "CoordinatorActor::Handler<DeviceUsageMessage>",
        skip_all,
        fields(
            otel.kind = "consumer",
            messaging.system = "actix",
            messaging.message.id = "DeviceUsageMessage",
            messaging.operation.name = "poll",
            messaging.operation.type = "receive",
            messaging.destination.name = "CoordinatorActor",
            device.name = %message.device.name,
            device.ip_address = %message.device.ip_address,
            error.type = tracing::field::Empty,
            exception.type = tracing::field::Empty,
            exception.message = tracing::field::Empty,
            exception.stacktrace = tracing::field::Empty,
        )
    )]
    fn handle(&mut self, message: DeviceUsageMessage, _: &mut Context<Self>) -> Self::Result {
        let span = tracing::Span::current();
        span.set_parent(message.span_context);

        let result = self.mqtt_actor_addr.try_send({
            DeviceUsageMessage {
                span_context: span.context(),
                device: message.device,
                device_usage: message.device_usage,
            }
        });

        if let Err(e) = result {
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
        }
    }
}

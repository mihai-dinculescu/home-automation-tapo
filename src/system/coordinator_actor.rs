use std::collections::HashMap;
use std::time::Duration;

use actix::clock::interval;
use actix::{Actor, Addr, AsyncContext, Context, Handler, WrapFuture};
use log::{debug, info};

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
                addr.try_send(HealthCheckMessage)
                    .expect("failed to send message to CoordinatorActor");
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

    fn handle(&mut self, _: HealthCheckMessage, ctx: &mut Context<Self>) -> Self::Result {
        info!("HealthCheckMessage");

        // check api
        if !self.api_actor_addr.connected() {
            let api_actor = ApiActor::new(self.settings.api.clone(), self.settings.tapo.clone());
            self.api_actor_addr = api_actor.start();
        }

        // check mqtt
        if !self.mqtt_actor_addr.connected() {
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
                    }
                }
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

    fn handle(&mut self, message: DeviceUsageMessage, _: &mut Context<Self>) -> Self::Result {
        info!(
            "device usage for {} ({}): {:?}",
            message.device.name, message.device.ip_address, message.device_usage
        );
        self.mqtt_actor_addr
            .try_send(message)
            .expect("failed to send message to MqttActor");
    }
}

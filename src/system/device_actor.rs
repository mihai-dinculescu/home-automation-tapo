use std::{sync::Arc, time::Duration};

use actix::{
    clock::interval, Actor, ActorContext, Addr, AsyncContext, Context, Handler, WrapFuture,
};
use log::{debug, error};
use tapo::{ApiClient, GenericDevice};

use crate::{
    settings::{Device, Tapo},
    system::messages::{DeviceUsageMessage, GetDeviceDataMessage, StopMessage},
};

use super::coordinator_actor::CoordinatorActor;

#[derive(Debug)]
pub struct DeviceActor {
    coordinator_actor_addr: Addr<CoordinatorActor>,
    config: Tapo,
    device: Device,
}

impl DeviceActor {
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
}

impl Actor for DeviceActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Device Actor for '{}' started...", self.device.name);

        let addr = ctx.address();
        let refresh_rate = Duration::from_secs(self.config.refresh_rate_s as u64);

        let device_name = self.device.name.clone();
        let ip_address = self.device.ip_address.clone();
        let tapo_username = self.config.username.clone();
        let tapo_password = self.config.password.clone();

        let fut = async move {
            let client = ApiClient::<GenericDevice>::new(
                ip_address.clone(),
                tapo_username,
                tapo_password,
                true,
            )
            .await;

            match client {
                Ok(client) => {
                    let client = Arc::new(client);
                    let mut interval = interval(refresh_rate);

                    loop {
                        interval.tick().await;
                        addr.try_send(GetDeviceDataMessage {
                            client: client.clone(),
                        })
                        .expect("failed to send message to DeviceActor");
                    }
                }
                Err(e) => {
                    error!("failed to create Tapo client for {device_name} ({ip_address}): {e:?}");
                    addr.try_send(StopMessage)
                        .expect("failed to send message to DeviceActor");
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

    fn handle(&mut self, message: GetDeviceDataMessage, ctx: &mut Context<Self>) -> Self::Result {
        debug!(
            "Getting device data from {} ({})...",
            self.device.name, self.device.ip_address
        );

        let device = self.device.clone();
        let coordinator_actor_addr = self.coordinator_actor_addr.clone();

        let fut = async move {
            let device_usage = message.client.get_device_usage().await;

            match device_usage {
                Ok(device_usage) => {
                    coordinator_actor_addr
                        .try_send(DeviceUsageMessage {
                            device,
                            device_usage,
                        })
                        .expect("failed to send message to CoordinatorActor");
                }
                Err(e) => {
                    error!(
                        "failed to get device usage for {} ({}): {:?}",
                        device.name, device.ip_address, e
                    )
                }
            }
        }
        .into_actor(self);

        ctx.spawn(fut);
    }
}

impl Handler<StopMessage> for DeviceActor {
    type Result = ();

    fn handle(&mut self, _: StopMessage, ctx: &mut Context<Self>) -> Self::Result {
        ctx.stop();
    }
}

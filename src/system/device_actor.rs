use std::time::Duration;

use actix::{clock::interval, Actor, Addr, AsyncContext, Context, Handler, WrapFuture};
use log::{debug, error};
use tapo::{ApiClient, GenericDevice};

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

        let fut = async move {
            let mut interval = interval(refresh_rate);

            loop {
                interval.tick().await;
                addr.try_send(GetDeviceDataMessage)
                    .expect("failed to send message to DeviceActor");
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

    fn handle(&mut self, _: GetDeviceDataMessage, ctx: &mut Context<Self>) -> Self::Result {
        debug!(
            "Getting device data from {} ({})...",
            self.device.name, self.device.ip_address
        );

        let device = self.device.clone();
        let tapo_username = self.config.username.clone();
        let tapo_password = self.config.password.clone();
        let coordinator_actor_addr = self.coordinator_actor_addr.clone();

        let fut = async move {
            let result = async {
                let client = ApiClient::<GenericDevice>::new(
                    device.ip_address.clone(),
                    tapo_username,
                    tapo_password,
                    true,
                )
                .await?;

                let device_usage = client.get_device_usage().await?;

                Ok::<_, anyhow::Error>(device_usage)
            }
            .await;

            match result {
                Ok(device_usage) => {
                    coordinator_actor_addr
                        .try_send(DeviceUsageMessage {
                            device,
                            device_usage,
                        })
                        .expect("failed to send message to CoordinatorActor");
                }
                Err(e) => error!(
                    "failed to get device usage for {} ({}): {:?}",
                    device.name, device.ip_address, e
                ),
            }
        }
        .into_actor(self);

        ctx.spawn(fut);
    }
}

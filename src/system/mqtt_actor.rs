use actix::{Actor, AsyncContext, Context, Handler, WrapFuture};
use log::{debug, error, info};
use paho_mqtt::{AsyncClient, Message, QOS_1};
use serde::Serialize;
use serde_json::json;
use tapo::responses::DeviceUsageEnergyMonitoringResult;

use crate::{
    settings::{Device, Mqtt},
    system::messages::DeviceUsageMessage,
};

pub struct MqttActor {
    config: Mqtt,
    client: AsyncClient,
}

impl MqttActor {
    pub fn new(config: Mqtt) -> Self {
        let client =
            AsyncClient::new(config.address.clone()).expect("failed to create the MQTT client");

        Self { config, client }
    }
}

impl Actor for MqttActor {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        debug!("MQTT Actor started...");
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("MQTT Actor stopped.");
    }
}

impl Handler<DeviceUsageMessage> for MqttActor {
    type Result = ();

    fn handle(&mut self, message: DeviceUsageMessage, ctx: &mut Context<Self>) -> Self::Result {
        debug!(
            "sending mqtt message with device usage for {} ({}): {:?}",
            message.device.name, message.device.ip_address, message.device_usage
        );

        let client = self.client.clone();
        let topic_name = self.config.topic_name.clone();

        let fut = async move {
            let payload: MqttMessagePayload = (message.device, message.device_usage).into();
            let payload = json!(payload).to_string();

            let result = async {
                if !client.is_connected() {
                    client.connect(None).await?;
                }

                let message = Message::new(topic_name, payload.clone(), QOS_1);
                let delivery_token = client.publish(message);
                delivery_token.wait()?;

                Ok::<_, anyhow::Error>(())
            }
            .await;

            match result {
                Ok(_) => info!("Sent MQTT message: {payload}"),
                Err(e) => error!("failed to publish to MQTT: {e:?}"),
            }
        }
        .into_actor(self);

        ctx.spawn(fut);
    }
}

#[derive(Serialize)]
struct MqttMessagePayload {
    device_name: String,
    // Today's time usage in minutes
    time_usage_today: Option<u64>,
    // Past 7 days time usage in minutes
    time_usage_past7: Option<u64>,
    // Past 30 days time usage in minutes
    time_usage_past30: Option<u64>,
    // Today's power usage in watt-hour (Wh)
    power_usage_today: Option<u64>,
    // Past 7 days power usage in watt-hour (Wh)
    power_usage_past7: Option<u64>,
    // Today's power usage in watt-hour (Wh)
    power_usage_past30: Option<u64>,
}

impl From<(Device, DeviceUsageEnergyMonitoringResult)> for MqttMessagePayload {
    fn from(data: (Device, DeviceUsageEnergyMonitoringResult)) -> Self {
        let (device, dur) = data;

        MqttMessagePayload {
            device_name: device.name,
            time_usage_today: match device.record_time_usage {
                true => dur.time_usage.today,
                false => None,
            },
            time_usage_past7: match device.record_time_usage {
                true => dur.time_usage.past7,
                false => None,
            },
            time_usage_past30: match device.record_time_usage {
                true => dur.time_usage.past30,
                false => None,
            },
            power_usage_today: dur.power_usage.today,
            power_usage_past7: dur.power_usage.past7,
            power_usage_past30: dur.power_usage.past30,
        }
    }
}

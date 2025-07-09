use actix::{Actor, AsyncContext, Context, WrapFuture};
use tracing::debug;

use crate::settings::{Api, Tapo};
use crate::system::api::web_server::WebServer;

#[derive(Debug)]
pub struct ApiActor {
    config_api: Api,
    config_tapo: Tapo,
}

impl ApiActor {
    pub fn new(config_api: Api, config_tapo: Tapo) -> Self {
        Self {
            config_api,
            config_tapo,
        }
    }
}

impl Actor for ApiActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Api Actor started...");

        let host = self.config_api.host.clone();
        let port = self.config_api.port;

        let tapo = self.config_tapo.clone();

        let fut = async move {
            let web_server = WebServer::new(&host, port, tapo)
                .await
                .expect("failed to create the API");

            web_server
                .run_until_stopped()
                .await
                .expect("failed to start the API");
        }
        .into_actor(self);

        ctx.wait(fut);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("Api Actor stopped.");
    }
}

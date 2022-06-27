use actix::{Actor, AsyncContext, Context, WrapFuture};
use actix_web::{web, App, HttpServer};
use log::debug;

use crate::settings::{Api, Tapo};
use crate::system::api::handlers;

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

        let addr = format!("{}:{}", self.config_api.host, self.config_api.port);
        let tapo = self.config_tapo.clone();

        let fut = async move {
            let data = web::Data::new(tapo);

            HttpServer::new(move || {
                App::new()
                    .app_data(data.clone())
                    .route("/device", web::get().to(handlers::get_device))
                    .route("/device", web::post().to(handlers::set_device))
            })
            .bind(addr)
            .expect("failed to bind to the API socket")
            .run()
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

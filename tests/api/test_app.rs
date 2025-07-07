use home_automation_tapo::{settings::Tapo, system::api::web_server::WebServer};

pub struct TestApp {
    pub address: String,
}

impl TestApp {
    pub async fn new() -> Self {
        let tapo = Tapo {
            username: "".to_string(),
            password: "".to_string(),
            refresh_rate_s: 60,
        };

        let web_server = WebServer::new("localhost", 0, tapo)
            .await
            .expect("Failed to build API");

        let port = web_server.port();
        tokio::spawn(web_server.run_until_stopped());

        Self {
            address: format!("http://localhost:{port}"),
        }
    }
}

use crate::config::Config;
use axum::{
    Router,
    body::Body,
    extract::{Query, State, WebSocketUpgrade, ws::WebSocket},
    response::Response,
    routing::get,
};
use serde::Deserialize;
use tokio::{io::AsyncReadExt, net::TcpStream, select};

mod config;

#[derive(Deserialize)]
struct Destination {
    to: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::init();
    let config = Config::default();

    let listener = tokio::net::TcpListener::bind(config.endpoint.clone())
        .await
        .unwrap();

    let router = Router::new()
        .route(&config.route, get(handle_connection))
        .with_state(config);

    tracing::info!("Start serving requests");
    axum::serve(listener, router).await.unwrap();
    Ok(())
}

async fn handle_connection(
    config: State<Config>,
    destination: Query<Destination>,
    ws: WebSocketUpgrade,
) -> Response {
    tracing::debug!("Received a new connection request to {:?}", destination.to);

    if !config.enabled_urls.iter().any(|url| url == &destination.to) {
        tracing::debug!("Connection request rejected since it does not target enabled URL");

        return Response::builder()
            .status(400)
            .body(Body::from("Requested destination is not enabled"))
            .unwrap();
    }

    let Ok(mut stream) = TcpStream::connect(&destination.to).await else {
        return Response::builder()
            .status(400)
            .body(Body::from("Requested destination is not enabled"))
            .unwrap();
    };
    tracing::debug!("TCP stream to target established");

    ws.on_upgrade(async move |mut ws: WebSocket| {
        let mut buf = [0; 1500];

        loop {
            select! {
                data = ws.recv() => {todo!()},
                read = stream.read(&mut buf) => {todo!()},
            }
        }
    })
}

use crate::config::Config;
use axum::{
    Router,
    body::Body,
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::get,
};
use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    select,
};

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

    // Check that destination is enabled in config
    if !config.enabled_urls.iter().any(|url| url == &destination.to) {
        tracing::debug!("Connection request rejected since it does not target enabled URL");

        return Response::builder()
            .status(400)
            .body(Body::from("Requested destination is not enabled"))
            .unwrap();
    }

    // Connect to destination
    let Ok(mut stream) = TcpStream::connect(&destination.to).await else {
        return Response::builder()
            .status(400)
            .body(Body::from("Requested destination is not enabled"))
            .unwrap();
    };
    tracing::debug!("TCP stream to target established");

    // TODO: Close WS with a reason
    // TODO: Check on results
    ws.on_upgrade(async move |mut ws: WebSocket| {
        let mut buf = [0; 1500];

        loop {
            select! {
                // Handle receiving data from the web socket side
                data = ws.recv() => {
                    match data {
                        None => {
                            tracing::debug!("Shutting down conntextion to {:?}", destination.to);
                            let _ = stream.shutdown().await;
                            break;
                        },
                        Some(data) => match data {
                            Err(err) =>{
                                tracing::warn!("Error while reading from websocket: {:?}", err);
                                //let _ = ws.send(Message::Close(None)).await;
                                let _ = stream.shutdown().await;
                                break;
                            },
                            Ok(data) => match data {
                                Message::Binary(bytes) => {
                                    tracing::trace!("Received {} bytes of data from websocket", bytes.len());
                                    let _ = stream.write_all(&bytes).await;
                                    //tracing::trace!("Forwarded {} bytes", bytes.len());
                                },
                                Message::Close(_) => {
                                    tracing::debug!("Shutting down conntextion to {:?}", destination.to);
                                    let _ = stream.shutdown().await;
                                    break;
                                },
                                Message::Ping(bytes) => {
                                    tracing::debug!("Received ping");
                                    let _ = ws.send(Message::Pong(bytes)).await;
                                },
                                Message::Text(_) => tracing::warn!("Received unexpected text data"),
                                Message::Pong(_) => tracing::debug!("Received pong"),
                            },
                        },
                    }
                },
                read = stream.read(&mut buf) => {
                    match read {
                        Err(err) => {
                            tracing::warn!("Error while reading TCP stream: {:?}", err);
                            let _ = ws.send(Message::Close(None)).await;
                            break;
                        },
                        Ok(read) => {
                            tracing::trace!("Read {} bytes of data", read);
                            let new_buf = buf[..read].to_vec();
                            let _ = ws.send(Message::Binary(new_buf.into())).await;
                        },
                    }
                },
            }
        }
    })
}

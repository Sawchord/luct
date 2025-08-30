use crate::config::{Config, is_valid_destination};
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
use url::{Host, Url};

mod config;

#[derive(Deserialize)]
struct Destination {
    to: Url,
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
        .with_state(config.clone());

    tracing::info!("Serving requests at {}:{}", config.endpoint, config.route);
    axum::serve(listener, router).await.unwrap();
    Ok(())
}

async fn handle_connection(
    config: State<Config>,
    destination: Query<Destination>,
    ws: WebSocketUpgrade,
) -> Response {
    tracing::debug!(
        "Received a new connection request to {}",
        destination.to.as_str()
    );
    //tracing::debug!("Enabled: {:?}", config.enabled_urls);

    // Check that destination is enabled in config
    if !config
        .enabled_urls
        .iter()
        .any(|url| is_valid_destination(url, &destination.to))
    {
        tracing::debug!(
            "Connection request rejected since {} is not target enabled URL",
            destination.to
        );

        return Response::builder()
            .status(400)
            .body(Body::from("Requested destination is not enabled"))
            .unwrap();
    }

    // Connect to destination
    let stream = match (
        destination.to.host(),
        destination.to.port_or_known_default(),
    ) {
        (Some(Host::Domain(domain)), Some(port)) => TcpStream::connect((domain, port)).await,
        (Some(Host::Ipv4(addr)), Some(port)) => TcpStream::connect((addr, port)).await,
        (Some(Host::Ipv6(addr)), Some(port)) => TcpStream::connect((addr, port)).await,
        _ => {
            tracing::debug!("Failed to parse destination");
            return Response::builder()
                .status(400)
                .body(Body::from("Failed to parse destination"))
                .unwrap();
        }
    };
    let Ok(mut stream) = stream else {
        tracing::debug!("Failed to connect to server");
        return Response::builder()
            .status(400)
            .body(Body::from("Failed to connect to destination"))
            .unwrap();
    };
    tracing::debug!("TCP stream to target established");

    // TODO: Close WS with a reason
    // TODO: Check on results
    // TODO: Separate TCP and WS sides and connect them with a bounded channel
    ws.on_upgrade(async move |mut ws: WebSocket| {
        let mut buf = [0; 1500];

        loop {
            select! {
                // Handle receiving data from the web socket side
                data = ws.recv() => {
                    match data {
                        None => {
                            tracing::debug!("Shutting down connction to {:?}", destination.to);
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

use axum::{
    Error,
    body::Body,
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    select,
};
use url::{Host, Url};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destination {
    to: Url,
}

impl Destination {
    pub fn dst(&self) -> &Url {
        &self.to
    }
}

pub async fn handle_connection<F>(destination: Url, ws: WebSocketUpgrade, access: F) -> Response
where
    F: Fn(&Url) -> bool,
{
    // Check access
    if !access(&destination) {
        tracing::debug!(
            "Connection request rejected since {:?} is not target enabled URL",
            destination
        );

        return Response::builder()
            .status(400)
            .body(Body::from("Requested destination is not enabled"))
            .unwrap();
    }

    // Connect to destination
    let stream = match (destination.host(), destination.port_or_known_default()) {
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
                    if !handle_websocket_receive(data, &mut ws, &mut stream, &destination).await {
                        break;
                    }
                },
                // Handle receiving data from the tcp socket side
                read = stream.read(&mut buf) => {
                    if !handle_tcp_stream_receive(read, &buf, &mut ws, &mut stream, &destination).await {
                        break;
                    }
                },
            }
        }
    })
}

async fn handle_websocket_receive(
    data: Option<Result<Message, Error>>,
    ws: &mut WebSocket,
    stream: &mut TcpStream,
    destination: &Url,
) -> bool {
    match data {
        None => {
            tracing::debug!("Shutting down connction to {:?}", destination);
            let _ = stream.shutdown().await;
            false
        }
        Some(data) => match data {
            Err(err) => {
                tracing::warn!("Error while reading from websocket: {:?}", err);
                //let _ = ws.send(Message::Close(None)).await;
                let _ = stream.shutdown().await;
                false
            }
            Ok(data) => match data {
                Message::Binary(bytes) => {
                    tracing::trace!("Received {} bytes of data from websocket", bytes.len());
                    let _ = stream.write_all(&bytes).await;
                    //tracing::trace!("Forwarded {} bytes", bytes.len());
                    true
                }
                Message::Close(_) => {
                    tracing::debug!("Shutting down conntextion to {:?}", destination);
                    let _ = stream.shutdown().await;
                    false
                }
                Message::Ping(bytes) => {
                    tracing::debug!("Received ping");
                    let _ = ws.send(Message::Pong(bytes)).await;
                    true
                }
                Message::Text(_) => {
                    tracing::warn!("Received unexpected text data");
                    true
                }
                Message::Pong(_) => {
                    tracing::debug!("Received pong");
                    true
                }
            },
        },
    }
}

async fn handle_tcp_stream_receive(
    read: tokio::io::Result<usize>,
    buf: &[u8],
    ws: &mut WebSocket,
    _stream: &mut TcpStream,
    _destination: &Url,
) -> bool {
    match read {
        Err(err) => {
            tracing::warn!("Error while reading TCP stream: {:?}", err);
            let _ = ws.send(Message::Close(None)).await;
            false
        }
        Ok(read) => {
            tracing::trace!("Read {} bytes of data", read);

            if read == 0 {
                let _ = ws.send(Message::Close(None)).await;
                false
            } else {
                let new_buf = buf[..read].to_vec();
                let _ = ws.send(Message::Binary(new_buf.into())).await;
                true
            }
        }
    }
}

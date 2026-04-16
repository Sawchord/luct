use axum::{
    Error,
    extract::{
        WebSocketUpgrade,
        ws::{CloseFrame, Message, WebSocket},
    },
    response::Response,
};
use otlsp_core::OtlspErrorCode;
use serde::{Deserialize, Serialize};
use std::io::{self, ErrorKind};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    select,
};
use url::{Host, Url};

const FRAME_SIZE: usize = 1500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destination {
    to: String,
}

impl Destination {
    pub fn dst(&self) -> &str {
        &self.to
    }
}

// TODO: Url parsing should happen here, such that we can do better error handling on bad url
pub async fn handle_connection<F>(
    destination: Destination,
    ws: WebSocketUpgrade,
    access: F,
) -> Response
where
    F: Fn(Url) -> bool + Send + 'static,
{
    ws.on_upgrade(async move |mut ws: WebSocket| {
        let destination = destination.dst();
        let Ok(destination) = Url::parse(destination) else {
            tracing::debug!("Failed to parse destination url {}", destination);

            let _ = ws
                .send(Message::Close(Some(io_error_to_close_msg(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Destination url {} could not be parsed", destination),
                )))))
                .await;
            return;
        };

        // Check access
        if !access(destination.clone()) {
            tracing::debug!(
                "Connection request rejected since {:?} is not target enabled URL",
                destination
            );

            let _ = ws
                .send(Message::Close(Some(io_error_to_close_msg(io::Error::new(
                    ErrorKind::PermissionDenied,
                    format!("Destination {} is disabled by proxy", destination),
                )))))
                .await;
            return;
        }

        // Connect to destination
        let stream = match (destination.host(), destination.port_or_known_default()) {
            (Some(Host::Domain(domain)), Some(port)) => TcpStream::connect((domain, port)).await,
            (Some(Host::Ipv4(addr)), Some(port)) => TcpStream::connect((addr, port)).await,
            (Some(Host::Ipv6(addr)), Some(port)) => TcpStream::connect((addr, port)).await,
            _ => {
                tracing::debug!("Failed to parse destination {}", destination);
                let _ = ws
                    .send(Message::Close(Some(io_error_to_close_msg(io::Error::new(
                        ErrorKind::InvalidInput,
                        format!(
                            "Destination {} cannot be parsed as OTLSP destionation",
                            destination
                        ),
                    )))))
                    .await;
                return;
            }
        };

        let stream = match stream {
            Err(err) => {
                tracing::debug!("Failed to connect to connection: {}", destination);
                let _ = ws
                    .send(Message::Close(Some(io_error_to_close_msg(err))))
                    .await;
                return;
            }
            Ok(stream) => stream,
        };
        tracing::debug!("TCP stream to target established");

        let _ = ws.send(Message::Text("accept".into())).await;
        tracing::debug!("OTLSP connection accepted");

        connection_loop(ws, stream, destination).await;
    })
}

async fn connection_loop(mut ws: WebSocket, mut stream: TcpStream, destination: Url) {
    // TODO: Make size configurable
    let (to_server_tx, mut to_server_rx) =
        tokio::sync::mpsc::channel::<Option<Result<Message, Error>>>(100);
    let (to_client_tx, mut to_client_rx) =
        tokio::sync::mpsc::channel::<(tokio::io::Result<usize>, [u8; 1500])>(100);

    // TODO: Close WS with a reason
    // TODO: Error handling in the ws.send calls

    let mut buf = [0; FRAME_SIZE];

    loop {
        select! {
            biased;

            // Handle receiving data from the web socket side
            data = to_server_rx.recv() => {
                if !handle_websocket_receive(data.flatten(), &mut ws, &mut stream, &destination).await {
                    break;
                }
            },
            // Handle receiving data from the tcp socket side
            read = to_client_rx.recv() => {
                if !handle_tcp_stream_receive(read, &mut ws, &mut stream, &destination).await {
                    break;
                }
            },
            read = stream.read(&mut buf) => {
                to_client_tx.send((read, buf))
                .await
                .expect("to_client_rx dropped unexpectedly");
            }
            data = ws.recv() => {
                to_server_tx.send(data)
                .await
                .expect("to_client_rx dropped unexpectedly");
            },
        }
    }
}

async fn handle_websocket_receive(
    data: Option<Result<Message, Error>>,
    ws: &mut WebSocket,
    stream: &mut TcpStream,
    destination: &Url,
) -> bool {
    match data {
        None => {
            tracing::warn!("Channel to server closed unexpectedly");

            let _ = stream.shutdown().await;
            false
        }
        Some(data) => match data {
            Err(err) => {
                tracing::warn!("Error while reading from websocket: {:?}", err);
                let _ = stream.shutdown().await;
                false
            }
            Ok(data) => match data {
                Message::Binary(bytes) => {
                    tracing::trace!("Received {} bytes of data from websocket", bytes.len());
                    let _ = stream.write_all(&bytes).await;
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
    read: Option<(tokio::io::Result<usize>, [u8; FRAME_SIZE])>,
    ws: &mut WebSocket,
    _stream: &mut TcpStream,
    _destination: &Url,
) -> bool {
    match read {
        None => {
            tracing::warn!("Channel to client closed unexpectedly");

            let _ = ws.send(Message::Close(None)).await;
            false
        }
        Some((Err(err), _)) => {
            tracing::warn!("Error while reading TCP stream: {:?}", err);
            let _ = ws
                .send(Message::Close(Some(io_error_to_close_msg(err))))
                .await;
            false
        }
        Some((Ok(read), buf)) => {
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

fn io_error_to_close_msg(error: io::Error) -> CloseFrame {
    let code = OtlspErrorCode::from(error.kind());
    let reason = error
        .into_inner()
        .map(|err| format!("{}", err))
        .unwrap_or_default();

    CloseFrame {
        code: code.into(),
        reason: reason.into(),
    }
}

use crate::{OtlspError, WebsocketStream, async_stream::WsAsyncStream};
use futures::{SinkExt, StreamExt};
use hyper::{body::Body, client::conn::http1::Connection};
use std::{
    collections::VecDeque,
    io,
    sync::{Arc, RwLock},
    task::{Context, Waker},
};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use url::Url;
#[derive(Debug, Clone)]
pub struct NativeWebsocketStream {
    sender: UnboundedSender<Message>,
    inner: Arc<RwLock<NativeWebsocketInner>>,
}

#[derive(Debug)]
struct NativeWebsocketInner {
    input_buffer: VecDeque<u8>,
    waker: Vec<Waker>,
    connection_status: Option<io::Result<()>>,
}

//(WebSocketStream<MaybeTlsStream<TcpStream>>);

impl WebsocketStream for NativeWebsocketStream {
    async fn new(proxy: Url, mut dst: Url) -> Result<Self, OtlspError> {
        dst.set_path("");
        let request_string = format!("{}?to={}", proxy.as_str(), dst.as_str());

        // Connect the web socket to the proxy
        let (ws_stream, _response) = connect_async(&request_string)
            .await
            .map_err(|err| OtlspError::UnreachableStd(Arc::new(err)))?;

        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let stream = Self {
            sender,
            inner: Arc::new(RwLock::new(NativeWebsocketInner {
                input_buffer: VecDeque::new(),
                waker: vec![],
                connection_status: None,
            })),
        };

        let (mut write, mut read) = ws_stream.split();

        // Handle the outbound traffic
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match write.send(msg).await {
                    Ok(()) => (),
                    Err(err) => {
                        tracing::error!("Error while sending data via websocket: {:?}", err);
                        // TODO: Need to set error?
                        break;
                    }
                }
            }
        });

        // Handle the inbound traffic
        let mut stream2 = stream.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                stream2.handle_inbound(msg).await
            }
        });

        Ok(stream)
    }

    fn close(&self) -> io::Result<()> {
        todo!()
    }

    fn enqueue_waker(&self, cx: &Context<'_>) {
        self.inner.write().unwrap().waker.push(cx.waker().clone());
    }

    fn spawn<B>(connection: Connection<WsAsyncStream<Self>, B>)
    where
        B: Body + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        tokio::spawn(async move {
            if let Err(err) = connection.await {
                tracing::error!("Connection failed: {:?}", err)
            }
        });
    }
}

impl NativeWebsocketStream {
    async fn handle_inbound(&mut self, msg: Result<Message, tungstenite::Error>) {
        match msg {
            Ok(msg) => match msg {
                Message::Binary(bytes) => {
                    tracing::trace!("Received {} bytes", bytes.len());

                    if !bytes.is_empty() {
                        let mut inner = self.inner.write().unwrap();
                        inner.input_buffer.extend(bytes.iter());
                        inner.wake_all();
                    }
                }
                Message::Close(close_frame) => todo!(),
                Message::Ping(bytes) => {
                    tracing::debug!("Received a ping");
                    if let Err(err) = self.sender.send(Message::Pong(bytes)) {
                        tracing::error!("Failed to send a pong message: {:?}", err);
                    }
                }
                Message::Pong(_bytes) => {
                    tracing::debug!("Received a pong message")
                }
                Message::Text(txt) => {
                    tracing::warn!("received unexpected Text: {:?}", txt);
                }
                Message::Frame(_frame) => tracing::error!("Received a raw frame. This is a bug"),
            },
            Err(err) => todo!(),
        };
    }
}

impl io::Read for NativeWebsocketStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }
}

impl io::Write for NativeWebsocketStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl NativeWebsocketInner {
    fn wake_all(&mut self) {
        for w in self.waker.drain(..) {
            w.wake();
        }
        tracing::trace!("wake_all called");
    }
}

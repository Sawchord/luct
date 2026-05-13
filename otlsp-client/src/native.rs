use crate::{OtlspError, WebsocketStream, async_stream::WsAsyncStream};
use futures::{SinkExt, StreamExt, channel::mpsc::UnboundedSender};
use hyper::{body::Body, client::conn::http1::Connection};
use std::{
    collections::VecDeque,
    io,
    sync::{Arc, RwLock},
    task::{Context, Waker},
};
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

        // Create unbounded channel and put into stream implementation
        let (sender, receiver) = futures::channel::mpsc::unbounded();
        let stream = Self {
            sender,
            inner: Arc::new(RwLock::new(NativeWebsocketInner {
                input_buffer: VecDeque::new(),
                waker: vec![],
                connection_status: None,
            })),
        };

        // Split websocket stream apart, connect outbound end to channel
        // This allows to send data from different places
        let (write, mut read) = ws_stream.split();
        let write_future = receiver.map(Ok).forward(write);
        tokio::spawn(write_future);
        
        let mut stream2 = stream.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                stream2.handle_msg(msg).await
            }
        });

        Ok(stream)
    }

    fn close(&self) -> io::Result<()> {
        todo!()
    }

    fn enqueue_waker(&self, cx: &Context<'_>) {
        todo!()
    }

    fn spawn<B>(connection: Connection<WsAsyncStream<Self>, B>)
    where
        B: Body,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        todo!()
    }
}

impl NativeWebsocketStream {
    async fn handle_msg(&mut self, msg: Result<Message, tungstenite::Error>) {
        match msg {
            Err(err) => todo!(),
            Ok(msg) => match msg {
                Message::Binary(bytes) => todo!(),
                Message::Close(close_frame) => todo!(),
                Message::Ping(bytes) => {
                    tracing::debug!("Received a ping");
                    self.sender.send(Message::Pong(bytes)).await;
                }
                Message::Pong(bytes) => todo!(),
                Message::Text(utf8_bytes) => todo!(),
                Message::Frame(frame) => tracing::error!("Received a raw frame. This is a bug"),
            },
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

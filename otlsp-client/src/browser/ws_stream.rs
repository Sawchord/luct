use crate::error::OtlspError;
use js_sys::{ArrayBuffer, JsString, Promise, Uint8Array};
use otlsp_core::OtlspErrorCode;
use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{self, ErrorKind},
    rc::Rc,
    task::Waker,
};
use url::Url;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::{BinaryType, Blob, CloseEvent, MessageEvent, WebSocket};

#[derive(Debug, Clone)]
pub(crate) struct WsStream {
    websocket: WebSocket,

    // TODO: Use a vectored buffer to avoid extensive copying
    input_buffer: Rc<RefCell<VecDeque<u8>>>,

    /// Since hyper expects async channels, we need to have the ability to
    /// wake channels, if there is new data available
    waker: Rc<RefCell<Vec<Waker>>>,

    /// State of the connection
    ///
    /// `None` indicates it is still open
    /// `Some(Ok(()))` indicates it closed without error
    /// `Some(Err(...))` is an error that needs to be propagated to the upper layers
    connection_status: Rc<RefCell<Option<io::Result<()>>>>,

    /// Handle to the onmessage callback function.
    ///
    /// Since [`WsStream`] gets closed by dropping it, holding on to the onmessage
    /// callback like this ensures, that the closure exists long enough to be always
    /// callable by the websocket connection.
    _onmessage: Rc<Closure<dyn FnMut(MessageEvent)>>,

    /// Handle to the onclose callback function.
    ///
    /// We hold on to it for the same reason
    _onclose: Rc<Closure<dyn FnMut(MessageEvent)>>,
}

impl WsStream {
    pub async fn new(proxy: Url, mut dst: Url) -> Result<Self, OtlspError> {
        let waker = Rc::new(RefCell::new(vec![]));
        let input_buffer = Rc::new(RefCell::new(VecDeque::<u8>::new()));

        dst.set_path("");
        let request_string = format!("{}?to={}", proxy.as_str(), dst.as_str());
        tracing::debug!("Connecting to: {:?}", request_string);

        let websocket = WebSocket::new(&request_string).unwrap();
        websocket.set_binary_type(BinaryType::Arraybuffer);

        Self::await_opened(&websocket).await?;

        let cloned_buffer = input_buffer.clone();
        let waker_cloned = waker.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
            if let Ok(abuf) = event.data().dyn_into::<ArrayBuffer>() {
                let array = Uint8Array::new(&abuf);
                let len = array.byte_length() as usize;
                tracing::trace!("ArrayBuffer received {} bytes", len);

                if len > 0 {
                    cloned_buffer.borrow_mut().extend(array.to_vec());
                    Self::wake_all(&waker_cloned);
                }
            } else if let Ok(blob) = event.data().dyn_into::<Blob>() {
                tracing::warn!("received unexpected Blob: {:?}", blob);
            } else if let Ok(txt) = event.data().dyn_into::<JsString>() {
                tracing::warn!("received unexpected Text: {:?}", txt);
            } else {
                tracing::warn!("received unexpected Unknown: {:?}", event.data());
            }
        });
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

        let connection_status = Rc::new(RefCell::new(None));
        let connection_status_clone = connection_status.clone();
        let waker_cloned = waker.clone();

        let onclose_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
            tracing::debug!("Received close event: {:?}", event.data());

            let mut connection_status = connection_status_clone.borrow_mut();
            *connection_status = match event.dyn_into::<CloseEvent>().ok() {
                None => Some(Ok(())),
                Some(close) => Some(Err(close_event_to_io_err(close))),
            };

            Self::wake_all(&waker_cloned);
        });
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));

        Ok(Self {
            websocket,
            input_buffer,
            waker,
            connection_status,
            #[allow(clippy::arc_with_non_send_sync)]
            _onmessage: Rc::new(onmessage_callback),
            #[allow(clippy::arc_with_non_send_sync)]
            _onclose: Rc::new(onclose_callback),
        })
    }

    pub fn waker(&self) -> Rc<RefCell<Vec<Waker>>> {
        self.waker.clone()
    }

    /// Set up the connection or error out
    async fn await_opened(websocket: &WebSocket) -> Result<(), OtlspError> {
        // These are here to hold the closures until the promise is resolved
        let mut open_cb = None;
        let mut error_cb = None;
        let mut message_cb = None;
        let mut close_cb = None;

        let opened = Promise::new(&mut |ok, err| {
            let err_clone = err.clone();
            let onerror_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
                tracing::warn!("Error while opening websocket");
                err_clone.call1(&JsValue::null(), &event).unwrap();
            });

            websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            error_cb = Some(onerror_callback);

            let onclose_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
                tracing::warn!("Websocket closed unexpectedly");
                err.call1(&JsValue::null(), &event).unwrap();
            });
            websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            close_cb = Some(onclose_callback);

            let onopen_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
                tracing::debug!(
                    "Opened websocket connection: {:?}",
                    event.data().as_string()
                );
            });
            websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            open_cb = Some(onopen_callback);

            let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
                tracing::debug!(
                    "Opened websocket connection: {:?}",
                    event.data().as_string()
                );
                if let Ok(str) = event.data().dyn_into::<JsString>()
                    && str == "accept"
                {
                    tracing::debug!(
                        "Websocket connection opened: {:?}",
                        event.data().as_string()
                    );
                    ok.call0(&JsValue::null()).unwrap();
                } else {
                    tracing::warn!("Ignoring unknown data");
                }
                ok.call0(&JsValue::null()).unwrap();
            });
            websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            message_cb = Some(onmessage_callback);
        });

        // Await the promise
        let result = JsFuture::from(opened).await;

        // Unset the callbacks
        websocket.set_onerror(None);
        websocket.set_onclose(None);
        websocket.set_onopen(None);
        websocket.set_onmessage(None);

        // Check for errors
        result.map_err(|err| {
            tracing::warn!("Failed to establish websocket connection");
            match err.dyn_into::<CloseEvent>().ok() {
                None => OtlspError::Unknown,
                Some(err) => close_event_to_io_err(err).into(),
            }
        })?;

        Ok(())
    }

    pub(crate) fn close(&self) -> io::Result<()> {
        self.websocket
            .close_with_code(1000)
            .expect("Failed to close websocket");

        match self.connection_status.borrow_mut().take() {
            Some(status) => status,
            None => {
                tracing::trace!("ws stream close: would block");
                Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Waiting on shutdown".to_string(),
                ))
            }
        }
    }

    /// Wake all wakers in the list
    fn wake_all(waker: &Rc<RefCell<Vec<Waker>>>) {
        let mut waker = waker.borrow_mut();
        for w in waker.drain(..) {
            w.wake()
        }
        tracing::trace!("wake_all called")
    }
}

impl io::Read for WsStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        tracing::trace!("Reading from ws stream");

        let mut input = self.input_buffer.borrow_mut();
        let new_bytes_len = std::cmp::min(buf.len(), input.len());

        for (idx, byte) in input.drain(..new_bytes_len).enumerate() {
            buf[idx] = byte;
        }

        if new_bytes_len == 0 {
            match self.connection_status.borrow_mut().take() {
                // Connection closed without error
                Some(Ok(())) => Ok(0),
                // There was an error, transmit it to upper layer
                Some(Err(err)) => {
                    tracing::trace!("ws stream read: sending error {} to upper layer", err);
                    Err(err)
                }
                // Connection is still open, send a would block
                None => {
                    tracing::trace!("ws stream read: would block");
                    Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "No new data available".to_string(),
                    ))
                }
            }
        } else {
            tracing::trace!("ws stream read {} bytes", new_bytes_len);
            Ok(new_bytes_len)
        }
    }
}

impl io::Write for WsStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.websocket
            .send_with_js_u8_array(&Uint8Array::from(buf))
            .map_err(|err| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    err.as_string()
                        .unwrap_or("Failed to send to websocket".to_string()),
                )
            })?;

        tracing::trace!("ws stream wrote {} bytes", buf.len());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // NOTE: The Javascript engine has buffered the data
        // We do not have any way of checking, whether it has already arrived.
        // However, the data will still be sent out even if we drop WsStream, as long
        // the connection to the server persists.
        // Therefore for our purposes, we can consider the data flushed.
        Ok(())
    }
}

impl Drop for WsStream {
    fn drop(&mut self) {
        tracing::debug!("Dropping ws stream");
    }
}

fn close_event_to_io_err(close: CloseEvent) -> io::Error {
    let code = OtlspErrorCode::from(close.code());

    let reason = close.reason();
    let kind: ErrorKind = code.clone().into();

    match kind {
        ErrorKind::Other => {
            io::Error::other(format!("Websocket error[{}]: {}", u16::from(code), reason))
        }
        kind => io::Error::new(kind, reason),
    }
}

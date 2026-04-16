use crate::error::OtlspError;
use js_sys::{ArrayBuffer, JsString, Promise, Uint8Array};
use std::{
    cell::RefCell,
    collections::VecDeque,
    io,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    task::Waker,
};
use url::Url;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::{BinaryType, Blob, MessageEvent, WebSocket};

#[derive(Debug, Clone)]
pub(crate) struct WsStream {
    websocket: WebSocket,

    // TODO: Use a vectored buffer to avoid extensive copying
    input_buffer: Rc<RefCell<VecDeque<u8>>>,

    /// Since hyper expects async channels, we need to have the ability to
    /// wake channels, if there is new data available
    waker: Rc<RefCell<Vec<Waker>>>,

    is_closed: Rc<AtomicBool>,

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

        // Initialize on_close to set the is_closed atomic bool to true
        let is_closed = Rc::new(AtomicBool::new(false));
        let is_closed_clone = is_closed.clone();
        let onclose_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
            tracing::debug!("Received close event: {:?}", event.data());
            is_closed_clone.store(true, Ordering::Relaxed);
        });
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));

        Ok(Self {
            websocket,
            input_buffer,
            waker,
            is_closed,
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
                tracing::warn!("Error while opening websocket: {:?}", event);
                err_clone.call1(&JsValue::null(), &event.data()).unwrap();
            });
            websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            error_cb = Some(onerror_callback);

            let onclose_callback = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent| {
                tracing::warn!("Websocket closed unexpectedly: {:?}", event);
                err.call1(&JsValue::null(), &event.data()).unwrap();
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

        // Await the promise and check the errors
        match JsFuture::from(opened).await {
            Ok(_) => (),
            Err(err) => {
                tracing::warn!("Failed to establish websocket connection: {:?}", err);
                return Err(OtlspError::Unreachable(
                    err.as_string().unwrap_or("Failed to connect".to_string()),
                ));
            }
        };

        // Unset the callbacks
        websocket.set_onerror(None);
        websocket.set_onclose(None);
        websocket.set_onopen(None);
        websocket.set_onmessage(None);

        Ok(())
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
        tracing::trace!("Reading from ws stream");

        let mut input = self.input_buffer.borrow_mut();
        let new_bytes_len = std::cmp::min(buf.len(), input.len());

        for (idx, byte) in input.drain(..new_bytes_len).enumerate() {
            buf[idx] = byte;
        }

        // If there were no bytes in the input buffer, but the connection is still open,
        // we need to return an interrupted error
        if new_bytes_len == 0 && !self.is_closed.load(Ordering::Relaxed) {
            tracing::trace!("ws stream read: would block");
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "No new data available".to_string(),
            ))
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
                std::io::Error::new(
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
        tracing::debug!("Closing ws stream");

        // Need to close the WS stream, to make sure that onmessage will never be called again
        self.websocket.close().unwrap();

        // Set the stream to closed, then wake up all the wakers, so they read the EOF
        self.is_closed.store(true, Ordering::SeqCst);
        Self::wake_all(&self.waker);
    }
}

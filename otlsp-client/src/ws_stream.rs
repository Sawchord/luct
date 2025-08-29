use crate::{console_log, error::OtlspError};
use js_sys::{ArrayBuffer, JsString, Uint8Array};
use std::{
    collections::VecDeque,
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::Waker,
};
use url::Url;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{BinaryType, Blob, MessageEvent, WebSocket};

#[derive(Debug, Clone)]
pub(crate) struct WsStream {
    websocket: WebSocket,

    // TODO: Use a vectored buffer to avoid extensive copying
    input_buffer: Arc<Mutex<VecDeque<u8>>>,

    /// Since hyper expects async channels, we need to have the ability to
    /// wake channels, if there is new data available
    waker: Arc<Mutex<Vec<Waker>>>,

    is_closed: Arc<AtomicBool>,

    /// Handle to the onmessage callback function.
    ///
    /// Since [`WsStream`] gets closed by dropping it, holding on to the onmessage
    /// callback like this ensures, that the closure exists long enough to be always
    /// callable by the websocket connection.
    _onmessage: Arc<Closure<dyn FnMut(MessageEvent)>>,

    /// Handle to the onclose callback function.
    ///
    /// We hold on to it for the same reason
    _onclose: Arc<Closure<dyn FnMut(MessageEvent)>>,
}

impl WsStream {
    pub async fn new(proxy: Url, dst: Url) -> Result<Self, OtlspError> {
        let input_buffer = Arc::new(Mutex::new(VecDeque::<u8>::new()));
        let waker = Arc::new(Mutex::new(vec![]));

        let request_string = format!("{}?to=\"{}\"", proxy.as_str(), dst.as_str());
        console_log!("Connecting to: {:?}", request_string);

        let websocket = WebSocket::new(&request_string).unwrap();
        websocket.set_binary_type(BinaryType::Arraybuffer);

        // TODO: Should we await the opening of the channel here?

        let cloned_buffer = input_buffer.clone();
        let waker_cloned = waker.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(abuf) = e.data().dyn_into::<ArrayBuffer>() {
                console_log!("message event, received arraybuffer: {:?}", abuf);
                let array = Uint8Array::new(&abuf);
                let len = array.byte_length() as usize;
                console_log!("ArrayBuffer received {}bytes: {:?}", len, array.to_vec());

                cloned_buffer.lock().unwrap().extend(array.to_vec());
                wake_all(&waker_cloned);
            } else if let Ok(blob) = e.data().dyn_into::<Blob>() {
                console_log!("message event, received Blob: {:?}", blob);
            } else if let Ok(txt) = e.data().dyn_into::<JsString>() {
                console_log!("message event, received Text: {:?}", txt);
            } else {
                console_log!("message event, received Unknown: {:?}", e.data());
            }
        });
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

        // Initialize on_close to set the is_closed atomic bool to true
        let is_closed = Arc::new(AtomicBool::new(false));
        let is_closed_clone = is_closed.clone();
        let onclose_callback = Closure::<dyn FnMut(_)>::new(move |_: MessageEvent| {
            is_closed_clone.store(true, Ordering::Relaxed);
        });
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));

        Ok(Self {
            websocket,
            input_buffer,
            waker,
            is_closed,
            #[allow(clippy::arc_with_non_send_sync)]
            _onmessage: Arc::new(onmessage_callback),
            #[allow(clippy::arc_with_non_send_sync)]
            _onclose: Arc::new(onclose_callback),
        })
    }

    pub fn waker(&self) -> Arc<Mutex<Vec<Waker>>> {
        self.waker.clone()
    }
}

impl io::Read for WsStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut lock: std::sync::MutexGuard<'_, VecDeque<u8>> = self.input_buffer.lock().unwrap();
        let new_elems = lock.drain(..buf.len()).collect::<Vec<_>>();
        buf.copy_from_slice(&new_elems);

        // If there were no bytes in the input buffer, but the connection is still open,
        // we need to return an interrupted error
        if new_elems.is_empty() && !self.is_closed.load(Ordering::Relaxed) {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "No new data available".to_string(),
            ))
        } else {
            Ok(new_elems.len())
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
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // FIXME: We would need to wait until the websocket has sent all the data
        // There seems to be no way of acquiring this information
        Ok(())
    }
}

impl Drop for WsStream {
    fn drop(&mut self) {
        // Need to close the WS stream, to make sure that onmessage will never be called again
        self.websocket.close().unwrap();

        // Set the stream to closed, then wake up all the wakers, so they read the EOF
        self.is_closed.store(true, Ordering::SeqCst);
        wake_all(&self.waker);
    }
}

fn wake_all(waker: &Arc<Mutex<Vec<Waker>>>) {
    let mut waker = waker.lock().unwrap();
    for w in waker.drain(..) {
        w.wake()
    }
}

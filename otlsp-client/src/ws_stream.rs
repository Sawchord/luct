use crate::error::OtlspError;
use js_sys::{ArrayBuffer, JsString, Uint8Array};
use std::{
    collections::VecDeque,
    io,
    sync::{Arc, Mutex},
};
use url::Url;
use wasm_bindgen::{
    JsCast,
    prelude::{Closure, wasm_bindgen},
};
use web_sys::{BinaryType, Blob, FileReader, MessageEvent, ProgressEvent, WebSocket};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub(crate) struct WsStream {
    input_buffer: Arc<Mutex<VecDeque<u8>>>,
    websocket: WebSocket,
    _cb: Closure<dyn FnMut(MessageEvent)>,
}

impl WsStream {
    pub fn new(proxy: Url) -> Result<Self, OtlspError> {
        let input_buffer = Arc::new(Mutex::new(VecDeque::<u8>::new()));

        let websocket = WebSocket::new(proxy.as_str()).unwrap();
        websocket.set_binary_type(BinaryType::Arraybuffer);

        let cloned_buffer = input_buffer.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(abuf) = e.data().dyn_into::<ArrayBuffer>() {
                console_log!("message event, received arraybuffer: {:?}", abuf);
                let array = Uint8Array::new(&abuf);
                let len = array.byte_length() as usize;
                console_log!("Arraybuffer received {}bytes: {:?}", len, array.to_vec());

                cloned_buffer.lock().unwrap().extend(array.to_vec());
            } else if let Ok(blob) = e.data().dyn_into::<Blob>() {
                console_log!("message event, received blob: {:?}", blob);
                let fr = FileReader::new().unwrap();
                let fr_c = fr.clone();

                let cloned_buffer = cloned_buffer.clone();
                let onloadend_cb = Closure::<dyn FnMut(_)>::new(move |_e: ProgressEvent| {
                    let array = Uint8Array::new(&fr_c.result().unwrap());
                    let len = array.byte_length() as usize;
                    console_log!("Blob received {}bytes: {:?}", len, array.to_vec());

                    cloned_buffer.lock().unwrap().extend(array.to_vec());
                });
                fr.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
                fr.read_as_array_buffer(&blob).expect("blob not readable");

                // TODO: We should not forget these
                onloadend_cb.forget();
            } else if let Ok(txt) = e.data().dyn_into::<JsString>() {
                console_log!("message event, received Text: {:?}", txt);
            } else {
                console_log!("message event, received Unknown: {:?}", e.data());
            }
        });
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

        Ok(Self {
            input_buffer,
            websocket,
            _cb: onmessage_callback,
        })
    }
}

impl io::Read for WsStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut lock: std::sync::MutexGuard<'_, VecDeque<u8>> = self.input_buffer.lock().unwrap();
        let new_elems = lock.drain(..buf.len()).collect::<Vec<_>>();
        buf.copy_from_slice(&new_elems);
        Ok(new_elems.len())
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
        Ok(())
    }
}

impl Drop for WsStream {
    fn drop(&mut self) {
        // Need to close the WS stream, to make sure that onmessage will never be called again
        self.websocket.close().unwrap();
    }
}

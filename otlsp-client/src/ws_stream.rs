use crate::error::OtlspError;
use js_sys::{ArrayBuffer, JsString, Uint8Array};
use rustls::lock::Mutex;
use std::{collections::VecDeque, io, sync::Arc};
use url::Url;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{BinaryType, Blob, FileReader, MessageEvent, ProgressEvent, WebSocket};

pub(crate) struct WsStream {
    input_buffer: Arc<Mutex<VecDeque<u8>>>,
    websocket: WebSocket,
    onmessage_callback: Closure<dyn FnMut(MessageEvent)>,
}

impl WsStream {
    pub async fn new(proxy: Url) -> Result<Self, OtlspError> {
        let input_buffer = Arc::new(Mutex::new(VecDeque::<u8>::new()));

        let websocket = WebSocket::new(proxy.as_str()).unwrap();
        websocket.set_binary_type(BinaryType::Arraybuffer);

        let cloned_buffer = input_buffer.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(abuf) = e.data().dyn_into::<ArrayBuffer>() {
                // console_log!("message event, received arraybuffer: {:?}", abuf);
                let array = Uint8Array::new(&abuf);
                // let len = array.byte_length() as usize;
                // console_log!("Arraybuffer received {}bytes: {:?}", len, array.to_vec());

                cloned_buffer
                    .lock()
                    .unwrap()
                    .extend(array.to_vec().into_iter());
            } else if let Ok(blob) = e.data().dyn_into::<Blob>() {
                //console_log!("message event, received blob: {:?}", blob);
                let fr = FileReader::new().unwrap();
                let fr_c = fr.clone();

                let cloned_buffer = cloned_buffer.clone();
                let onloadend_cb = Closure::<dyn FnMut(_)>::new(move |_e: ProgressEvent| {
                    let array = Uint8Array::new(&fr_c.result().unwrap());
                    // let len = array.byte_length() as usize;
                    // console_log!("Blob received {}bytes: {:?}", len, array.to_vec());

                    cloned_buffer
                        .lock()
                        .unwrap()
                        .extend(array.to_vec().into_iter());
                });
                fr.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
                fr.read_as_array_buffer(&blob).expect("blob not readable");

                // TODO: We should not forget these
                onloadend_cb.forget();
            } else if let Ok(_txt) = e.data().dyn_into::<JsString>() {
                //console_log!("message event, received Text: {:?}", txt);
            } else {
                //console_log!("message event, received Unknown: {:?}", e.data());
            }
        });
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

        Ok(Self {
            input_buffer,
            websocket,
            onmessage_callback,
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
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl Drop for WsStream {
    fn drop(&mut self) {
        // Need to close the WS stream, to make sure that onmessage will never be called again
        self.websocket.close().unwrap();
    }
}

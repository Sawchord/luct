use js_sys::Function;
use wasm_bindgen::{JsThreadLocal, prelude::wasm_bindgen};

pub use crate::extension_sys::storage::{Storage, StorageArea};

mod storage;

pub fn browser() -> &'static JsThreadLocal<Browser> {
    &BROWSER
}

#[wasm_bindgen]
extern "C" {
    pub type Browser;

    #[cfg(feature = "firefox")]
    #[wasm_bindgen(thread_local_v2, js_name = browser)]
    static BROWSER: Browser;

    // #[wasm_bindgen(method, getter)]
    // pub fn action(this: &Browser) -> Action;

    // #[cfg(feature = "firefox")]
    // #[wasm_bindgen(method, getter, js_name = browserAction)]
    // pub fn browser_action(this: &Browser) -> BrowserAction;

    // #[wasm_bindgen(method, getter)]
    // pub fn cookies(this: &Browser) -> Cookies;

    // #[cfg(feature = "firefox")]
    // #[wasm_bindgen(method, getter, js_name = contextualIdentities)]
    // pub fn contextual_identities(this: &Browser) -> ContextualIdentities;

    // #[wasm_bindgen(method, getter)]
    // pub fn downloads(this: &Browser) -> Downloads;

    // #[wasm_bindgen(method, getter)]
    // pub fn runtime(this: &Browser) -> Runtime;

    // #[wasm_bindgen(method, getter)]
    // pub fn sessions(this: &Browser) -> Sessions;

    // #[cfg(feature = "firefox")]
    // #[wasm_bindgen(method, getter, js_name = sidebarAction)]
    // pub fn sidebar_action(this: &Browser) -> SidebarAction;

    #[wasm_bindgen(method, getter)]
    pub fn storage(this: &Browser) -> Storage;

    // #[wasm_bindgen(method, getter)]
    // pub fn tabs(this: &Browser) -> Tabs;

    // #[cfg(feature = "firefox")]
    // #[wasm_bindgen(method, getter)]
    // pub fn theme(this: &Browser) -> BrowserTheme;

    // #[wasm_bindgen(method, getter)]
    // pub fn windows(this: &Browser) -> Windows;

    // #[wasm_bindgen(method, getter)]
    // pub fn scripting(this: &Browser) -> Scripting;

    // #[wasm_bindgen(method, getter)]
    // pub fn history(this: &Browser) -> History;

    // #[wasm_bindgen(method, getter)]
    // pub fn bookmarks(this: &Browser) -> Bookmarks;

    // #[wasm_bindgen(method, getter)]
    // pub fn commands(this: &Browser) -> Commands;

    // #[wasm_bindgen(method, getter)]
    // pub fn identity(this: &Browser) -> Identity;

    // #[wasm_bindgen(method, getter)]
    // pub fn omnibox(this: &Browser) -> Omnibox;

    // #[wasm_bindgen(method, getter, js_name = contextMenus)]
    // pub fn context_menus(this: &Browser) -> ContextMenus;
}

#[wasm_bindgen]
extern "C" {
    pub type EventTarget;

    #[wasm_bindgen(method, js_name = addListener)]
    pub fn add_listener(this: &EventTarget, listener: &Function);

    #[wasm_bindgen(method, js_name = removeListener)]
    pub fn remove_listener(this: &EventTarget, listener: &Function);

    #[wasm_bindgen(method, js_name = hasListener)]
    pub fn has_listener(this: &EventTarget, listener: &Function) -> bool;
}

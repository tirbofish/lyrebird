//! The executable that runs the scene files. Connects directly to the [lyrebird-runtime] package, 
//! and removes any debug code, making it fast for production. 

#![windows_subsystem = "windows"]

#[tokio::main]
async fn main() {
    lyrebird_renderer::run::<lyrebird_runtime::scene::Runtime>().unwrap();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsValue;

    console_error_panic_hook::set_once();
    if let Err(err) = run::<scene::Runtime>() {
        log::error!("{err:?}");
        return Err(JsValue::from_str(&format!("{err:?}")));
    }

    Ok(())
}
#[cfg(feature = "debug")]
compile_error!("\n\nThis binary is compiled with the `debug` feature, which is not allowed to happen. \nlyrebird-runtime can only have this feature as a library target, not an executable.\n\n\n");

fn main() {
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
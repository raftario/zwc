use wasm_bindgen::prelude::*;

fn err_to_js<E: std::error::Error>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[wasm_bindgen]
pub fn camouflage(
    payload: String,
    dummy: String,
    key: Option<String>,
    compression_level: i32,
) -> Result<String, JsValue> {
    zwc::camouflage(
        payload.into_bytes(),
        &dummy,
        key.as_ref().map(AsRef::as_ref),
        Some(compression_level),
    )
    .map_err(err_to_js)
}

#[wasm_bindgen]
pub fn decamouflage(camouflaged: String, key: Option<String>) -> Result<String, JsValue> {
    match zwc::decamouflage(&camouflaged, key.as_ref().map(AsRef::as_ref)) {
        Ok(d) => match String::from_utf8(d) {
            Ok(s) => Ok(s),
            Err(_) => Err(JsValue::from_str(
                "The web version doesn't currently support binary data",
            )),
        },
        Err(e) => Err(err_to_js(e)),
    }
}

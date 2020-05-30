#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

static ORIGINAL: &str = include_str!("../src/lib.rs");

#[wasm_bindgen_test]
fn roundtrip() {
    let camouflaged = zwc_wasm::camouflage(
        ORIGINAL.to_owned(),
        "Hello, World!".to_owned(),
        Some("secret".to_owned()),
        10,
    )
    .unwrap();
    let decamouflaged = zwc_wasm::decamouflage(camouflaged, Some("secret".to_owned())).unwrap();
    assert_eq!(decamouflaged.as_str(), ORIGINAL);
}

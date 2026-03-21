use wasm_bindgen::prelude::*;
use js_sys;
use web_sys;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    web_sys::console::log_1(&"Hello from Rust!".into());
    Ok(())
}

// TronLink wallet integration
#[wasm_bindgen]
pub fn get_tron_address() -> Option<String> {
    let window = web_sys::window()?;
    let tron_web = js_sys::Reflect::get(&window, &JsValue::from_str("tronWeb")).ok()?;
    let default_address = js_sys::Reflect::get(&tron_web, &JsValue::from_str("defaultAddress")).ok()?;
    let base58 = js_sys::Reflect::get(&default_address, &JsValue::from_str("base58")).ok()?;
    base58.as_string()
}

// Three.js starter
#[wasm_bindgen]
pub fn start_three(canvas_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let init_func = js_sys::Reflect::get(&window, &JsValue::from_str("initThree"))?;
    let init_func = init_func.dyn_into::<js_sys::Function>()?;
    init_func.call1(&window, &JsValue::from_str(canvas_id))?;
    Ok(())
}

// FRACTRAN interpreter
#[wasm_bindgen]
pub fn run_fractran(program: &str, mut n: u64, steps: usize) -> u64 {
    let fractions: Vec<(u64, u64)> = program
        .split_whitespace()
        .filter_map(|f| {
            let (num, den) = f.split_once('/')?;
            Some((num.parse().ok()?, den.parse().ok()?))
        })
        .collect();

    for _ in 0..steps {
        if let Some((num, den)) = fractions.iter().find(|(_, den)| n % den == 0) {
            n = n / den * num;
        } else {
            break;
        }
    }
    n
}

//! Browser-only native bridge for the on-screen control pad.
//!
//! The HTML shell owns `window.__tapperInput`; reading a plain object is far
//! more reliable than synthesising `KeyboardEvent`s that winit may ignore.

#[derive(Default)]
pub struct RawTouch {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub pour: bool,
    pub up_seq: u64,
    pub down_seq: u64,
    pub left_seq: u64,
    pub right_seq: u64,
    pub pour_seq: u64,
    pub start_seq: u64,
    pub pause_seq: u64,
    pub mute_seq: u64,
    pub touch_mode: bool,
}

#[cfg(target_arch = "wasm32")]
pub fn read() -> RawTouch {
    use wasm_bindgen::JsValue;

    let global = js_sys::global();
    let input = js_sys::Reflect::get(&global, &JsValue::from_str("__tapperInput"))
        .unwrap_or(JsValue::UNDEFINED);
    let num = |key: &str| -> f64 {
        if input.is_undefined() || input.is_null() {
            return 0.0;
        }
        js_sys::Reflect::get(&input, &JsValue::from_str(key))
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    };
    RawTouch {
        up: num("up") != 0.0,
        down: num("down") != 0.0,
        left: num("left") != 0.0,
        right: num("right") != 0.0,
        pour: num("pour") != 0.0,
        up_seq: num("upSeq") as u64,
        down_seq: num("downSeq") as u64,
        left_seq: num("leftSeq") as u64,
        right_seq: num("rightSeq") as u64,
        pour_seq: num("pourSeq") as u64,
        start_seq: num("startSeq") as u64,
        pause_seq: num("pauseSeq") as u64,
        mute_seq: num("muteSeq") as u64,
        touch_mode: num("touch") != 0.0,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read() -> RawTouch {
    RawTouch::default()
}

#[allow(clippy::too_many_arguments)]
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn publish_state(
    score: u32,
    lives: u32,
    screen: usize,
    loop_no: u32,
    phase: u8,
    lane: usize,
    slot: f32,
    patrons: usize,
    mugs: usize,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsValue;
        let obj = js_sys::Object::new();
        let mut set = |key: &str, value: f64| {
            let _ = js_sys::Reflect::set(&obj, &JsValue::from_str(key), &JsValue::from_f64(value));
        };
        set("score", score as f64);
        set("lives", lives as f64);
        set("screen", screen as f64);
        set("loop", loop_no as f64);
        set("phase", phase as f64);
        set("lane", lane as f64);
        set("slot", slot as f64);
        set("patrons", patrons as f64);
        set("mugs", mugs as f64);
        let _ = js_sys::Reflect::set(&js_sys::global(), &JsValue::from_str("__tapperState"), &obj);
    }
}

use wasm_bindgen::prelude::*;
use std::{
    future::Future,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    collections::VecDeque,
};
//use parking_lot::{Mutex, RawMutex};
use half::f16;
use three_d::*;

use web_sys::{ Blob, Url };
use js_sys::Array;


#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


#[wasm_bindgen(module = "/helper.js")]
extern "C" {
    pub fn get_canvas_width() -> u32;
    pub fn get_canvas_height() -> u32;
    pub fn cpu_cores() -> u32;
    pub fn get_time_milliseconds() -> f64;
    pub fn get_webgl1_version() -> String;
    pub fn get_webgl2_version() -> String;
    pub fn get_url_param() -> String;
    pub fn get_position_param() -> JsValue;
    pub fn get_target_param() -> JsValue;
    pub fn get_up_param() -> JsValue;
    pub async fn sleep_js(ms: u32);
}


// Convert Javascript's 64 bit float array to f32s expected by Vec3
#[inline(always)]
fn convert_js_array_to_vector3(js_array: JsValue) -> Vec3 {
    let array: Array = js_sys::Array::from(&js_array);
    let x = array.get(0).as_f64().unwrap_or(0.0) as f32;
    let y = array.get(1).as_f64().unwrap_or(0.0) as f32;
    let z = array.get(2).as_f64().unwrap_or(0.0) as f32;
    vec3(x, y, z)
}


// Get the position JsVal and convert into a Vec3
#[inline(always)]
pub fn get_position() -> Vec3 {
    let position_val = get_position_param();
    convert_js_array_to_vector3(position_val)
}


// Get the target JsVal and convert into a Vec3
#[inline(always)]
pub fn get_target() -> Vec3 {
    let target_val = get_target_param();
    convert_js_array_to_vector3(target_val)
}


// Get the up JsVal and convert into a Vec3
#[inline(always)]
pub fn get_up() -> Vec3 {
    let up_val = get_up_param();
    convert_js_array_to_vector3(up_val)
}


/// Enable better error messages if our code ever panics
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}


/// Sets error flag and message for an egui window
#[inline(always)]
pub fn set_error_for_egui(flag: &Arc<AtomicBool>, msg: &Arc<Mutex<String>>, s: String) {
    flag.store(true, Ordering::Relaxed);
    {
        let mut mutex = msg.lock().unwrap();
        *mutex += s.as_str();
    }
}


/// Executes an async Future on the current thread
#[inline(always)]
pub fn execute_future<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}


/// Transmutes a slice
#[inline(always)]
pub fn transmute_slice<S, T>(slice: &[S]) -> &[T] {
    let ptr = slice.as_ptr() as *const T;
    let len = std::mem::size_of_val(slice) / std::mem::size_of::<T>();
    unsafe { std::slice::from_raw_parts(ptr, len) }
}


/// Transmutes a mutable slice
#[inline(always)]
pub fn transmute_slice_mut<S, T>(slice: &mut [S]) -> &mut [T] {
    let ptr = slice.as_mut_ptr() as *mut T;
    let len = std::mem::size_of_val(slice) / std::mem::size_of::<T>();
    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
}


/// Packs two f32s as two f16s combined together
#[inline(always)]
pub fn pack_half_2x16(x: f32, y: f32) -> u32 {
    let x_half = f16::from_f32(x);
    let y_half = f16::from_f32(y);
    let result = u32::from(x_half.to_bits()) | (u32::from(y_half.to_bits()) << 16);
    result// & 0xFFFFFFFF
}


/// Check if a float is zero
#[inline(always)]
pub fn is_float_zero(x: f32, threshold: f32) -> bool {
    return x.abs() < threshold;
}


/// Check if two floats are equal
#[inline(always)]
pub fn are_floats_equal(x: f32, y: f32, threshold: f32) -> bool {
    return is_float_zero(x-y, threshold);
}


/// Creates URL of a byte array
#[inline(always)]
pub fn create_url_byte_array(bytes: Vec<u8>) -> Result<String, JsValue> {
    let array = js_sys::Uint8Array::from(bytes.as_slice());
    let array = js_sys::Array::of1(&JsValue::from(array));
    let blob = Blob::new_with_u8_array_sequence(&array)?;
    let url = Url::create_object_url_with_blob(&blob)?;

    Ok(url)
}


/// Incremental Moving Average
pub struct IncrementalMA {
    v: VecDeque<f64>,
    sum: f64,
}
impl IncrementalMA {
    pub fn new(window_size: usize) -> Self {
        IncrementalMA {
            v: VecDeque::with_capacity(window_size),
            sum: 0_f64,
        }
    }

    pub fn add(&mut self, value: f64) -> f64 {
        if self.v.len() == self.v.capacity() {
            self.sum -= self.v.pop_front().unwrap();
        }

        self.v.push_back(value);
        self.sum += value;

        self.sum / (self.v.len() as f64)
    }

    pub fn calc(&self) -> f64 {
        if self.v.is_empty() {
            0_f64
        } else {
            self.sum / (self.v.len() as f64)
        }
    }
}


/*
// TODO
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
*/

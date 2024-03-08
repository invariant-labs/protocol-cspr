use crate::types::sqrt_price::get_max_tick;
use js_sys::BigInt;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;
pub const MAX_TICK: i32 = 221_818;
pub const MIN_TICK: i32 = -MAX_TICK;

pub const MAX_SQRT_PRICE: u128 = 65535383934512647000000000000;
pub const MIN_SQRT_PRICE: u128 = 15258932000000000000;

pub const TICK_SEARCH_RANGE: i32 = 256;
pub const CHUNK_SIZE: i32 = 64;

#[wasm_wrapper]
pub fn get_global_max_sqrt_price() -> u128 {
    MAX_SQRT_PRICE
}

#[wasm_wrapper]
pub fn get_global_min_sqrt_price() -> u128 {
    MIN_SQRT_PRICE
}

#[wasm_wrapper]
pub fn get_tick_search_range() -> i32 {
    TICK_SEARCH_RANGE
}

#[wasm_wrapper]
pub fn tick_to_chunk(tick: i32, tick_spacing: i32) -> u16 {
    let bitmap_index = (tick + MAX_TICK) / tick_spacing;
    let chunk: u16 = (bitmap_index / CHUNK_SIZE) as u16;
    chunk
}

#[wasm_wrapper]
pub fn tick_to_pos(tick: i32, tick_spacing: i32) -> u8 {
    let bitmap_index = (tick + MAX_TICK) / tick_spacing;
    let pos: u8 = (bitmap_index % CHUNK_SIZE) as u8;
    pos
}

#[wasm_wrapper]
pub fn get_max_chunk(tick_spacing: u32) -> u32 {
    let max_tick = get_max_tick(tick_spacing);
    let max_bitmap_index = (max_tick + MAX_TICK) / tick_spacing as i32;
    let max_chunk_index = max_bitmap_index / CHUNK_SIZE;
    max_chunk_index as u32
}

#[wasm_wrapper]
pub fn get_chunk_size() -> i32 {
    CHUNK_SIZE
}

#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// You can add safe Rust wrappers here
pub struct Can2040 {
    inner: can2040,
}

impl Can2040 {
    pub fn new(pio_num: u32) -> Self {
        let mut inner: can2040 = unsafe { core::mem::zeroed() };
        unsafe {
            can2040_setup(&mut inner, pio_num);
        }
        Self { inner }
    }

    pub fn start(&mut self, sys_clock: u32, bitrate: u32, gpio_rx: u32, gpio_tx: u32) {
        unsafe {
            can2040_start(&mut self.inner, sys_clock, bitrate, gpio_rx, gpio_tx);
        }
    }

    // Add more safe wrapper methods as needed
}

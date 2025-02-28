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

    pub fn stop(&mut self) {
        unsafe {
            can2040_stop(&mut self.inner);
        }
    }

    pub fn check_transmit(&mut self) -> i32 {
        unsafe { can2040_check_transmit(&mut self.inner) }
    }

    pub fn transmit(&mut self, msg: &mut can2040_msg) -> Result<(), i32> {
        let result = unsafe { can2040_transmit(&mut self.inner, msg) };
        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    /// Configure a callback function for receiving CAN messages and notifications
    pub fn set_callback(&mut self, callback: can2040_rx_cb) {
        unsafe {
            can2040_callback_config(&mut self.inner, callback);
        }
    }

    /// Get statistics about CAN bus operations
    pub fn get_statistics(&mut self) -> can2040_stats {
        let mut stats = can2040_stats::default();
        unsafe {
            can2040_get_statistics(&mut self.inner, &mut stats);
        }
        stats
    }

    /// Handle PIO interrupts - should be called from the PIO interrupt handler
    pub fn handle_interrupt(&mut self) {
        unsafe {
            can2040_pio_irq_handler(&mut self.inner);
        }
    }

    /// Helper method to create an extended frame format (EFF) ID
    pub fn make_eff_id(id: u32) -> u32 {
        id | CAN2040_ID_EFF as u32
    }

    /// Helper method to create a remote transmission request (RTR) ID
    pub fn make_rtr_id(id: u32) -> u32 {
        id | CAN2040_ID_RTR as u32
    }

    /// Setup the CAN peripheral
    pub fn setup(&mut self) {
        unsafe {
            can2040_setup(&mut self.inner, self.inner.pio_num);
        }
    }
}

/// Notification types that can be received in callbacks
pub mod notify {
    use super::*;
    pub const RX: u32 = CAN2040_NOTIFY_RX;
    pub const TX: u32 = CAN2040_NOTIFY_TX;
    pub const ERROR: u32 = CAN2040_NOTIFY_ERROR;
}

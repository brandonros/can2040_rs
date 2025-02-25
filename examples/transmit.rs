#![no_std]
#![no_main]

use can2040_rs::{notify, Can2040};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0, PIO1, UART1};
use embassy_rp::pio::{self, Pio};
use embassy_rp::uart::{self};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use {defmt_serial as _, panic_probe as _};

// Program metadata for `picotool info`.
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"can2040_example"),
    embassy_rp::binary_info::rp_program_description!(c"can2040 example"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// interrupt handlers
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
});

// cyw43 task
#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

// blinky task
#[embassy_executor::task]
async fn blinky_task(control: &'static mut cyw43::Control<'static>) {
    loop {
        control.gpio_set(0, true).await;
        Timer::after(Duration::from_millis(1000)).await;
        control.gpio_set(0, false).await;
        Timer::after(Duration::from_millis(1000)).await;
    }
}

// CAN message callback
extern "C" fn can_callback(
    _cd: *mut can2040_rs::can2040,
    notify: u32,
    msg: *mut can2040_rs::can2040_msg,
) {
    if notify == notify::RX {
        defmt::info!("CAN message received");
        // Safety: msg is valid when notification is RX
        let msg = unsafe { &*msg };
        defmt::info!("ID: {}, DLC: {}", msg.id, msg.dlc);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // init peripherals
    let p = embassy_rp::init(Default::default());

    // init uart
    static UART: StaticCell<uart::Uart<'static, UART1, uart::Blocking>> = StaticCell::new();
    let uart1 = UART.init(uart::Uart::new_blocking(
        p.UART1,
        p.PIN_4, // tx, blue, goes to rx
        p.PIN_5, // rx, white, goes to tx
        uart::Config::default(),
    ));

    // init defmt serial
    defmt_serial::defmt_serial(uart1);

    // init cyw43
    let fw = include_bytes!("../../rp2350-ble/cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../../rp2350-ble/cyw43-firmware/43439A0_clm.bin");
    let btfw = include_bytes!("../../rp2350-ble/cyw43-firmware/43439A0_btfw.bin");
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, _bt_device, mut control, runner) =
        cyw43::new_with_bluetooth(state, pwr, spi, fw, btfw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));
    control.init(clm).await;

    // init blinky task
    static CONTROL: StaticCell<cyw43::Control<'static>> = StaticCell::new();
    let control = CONTROL.init(control);
    unwrap!(spawner.spawn(blinky_task(control)));

    // Initialize CAN
    let mut pio1 = Pio::new(p.PIO1, Irqs);
    let mut can = Can2040::new(1); // Use PIO1
    can.setup();
    can.set_callback(Some(can_callback));

    // Start CAN with 500kbit/s bitrate, using GPIO 10 (RX) and 11 (TX)
    can.start(125_000_000, 500_000, 10, 11);

    // Example: Send a CAN message every second
    loop {
        let mut msg = can2040_rs::can2040_msg::default();
        msg.id = 0x7e0; // Standard ID
        msg.dlc = 8; // 8 bytes of data
                     // Set data using the union
        unsafe {
            msg.__bindgen_anon_1.data = [0x02, 0x3e, 0x00, 0x55, 0x55, 0x55, 0x55, 0x55];
        }

        if can.check_transmit() > 0 {
            match can.transmit(&msg) {
                Ok(_) => defmt::info!("Message sent"),
                Err(e) => defmt::error!("Failed to send: {}", e),
            }
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}

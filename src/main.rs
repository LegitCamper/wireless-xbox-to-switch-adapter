#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::{Duration, Timer};
use embassy_usb::{Builder, Config};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

mod switch;
use switch::*;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    spawner.spawn(usb(p.USB)).unwrap();
}

// handles all the usb communication to the switch
#[embassy_executor::task]
async fn usb(usb: USB) {
    let usb = Driver::new(usb, Irqs);

    // Create embassy-usb Config
    let mut config = Config::new(0x0F0D, 0x0092);
    config.manufacturer = Some("HORI CO.,LTD.");
    config.product = Some("POKKEN CONTROLLER");
    config.serial_number = None;
    config.max_packet_size_0 = 64;
    config.max_power = 100;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut builder = Builder::new(
        usb,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
}

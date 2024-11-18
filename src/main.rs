#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::{Duration, Timer};
use embassy_usb::class::hid::HidReader;
use embassy_usb::class::hid::HidWriter;
use embassy_usb::class::hid::{self, HidReaderWriter};
use embassy_usb::{Builder, Config};
use gpio::{Level, Output};
use joycon_sys::input::ButtonsStatus;
use joycon_sys::input::DeviceStatus;
use joycon_sys::input::NormalInputReport;
use joycon_sys::input::StandardInputReport;
use {defmt_rtt as _, panic_probe as _};

use joycon_sys::{InputReport, OutputReport};

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
    config.device_class = 0x03;
    config.device_sub_class = 0x00;
    config.device_protocol = 0x00;
    config.device_release = 0x01;
    config.max_power = 300; // might need to be 500

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = hid::State::new();

    let report = InputReport::new();

    let mut builder = Builder::new(
        usb,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let config = hid::Config {
        report_descriptor: report.as_bytes(),
        request_handler: None,
        poll_ms: 0x05,
        max_packet_size: 64,
    };
    let mut hid_writer = HidWriter::<_, 1>::new(&mut builder, &mut state, config);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let in_fut = async {
        loop {
            // every 1 second
            _ = Timer::after_secs(1).await;
            let report = switch::get_test_report();
            // Send the report.
            match hid_writer.write(&report.as_bytes()).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            }
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, in_fut).await;
}

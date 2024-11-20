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
    let mut config = Config::new(0x057e, 0x2009);
    config.manufacturer = Some("Nintendo Co., Ltd.");
    config.product = Some("Pro Controller");
    config.serial_number = Some("000000000001");
    config.max_packet_size_0 = 64;
    config.device_class = 0x00;
    config.device_sub_class = 0x00;
    config.device_protocol = 0x00;
    config.device_release = 0x0200;
    config.max_power = 500;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = hid::State::new();

    let mut request_handler = UsbRequestHandler {};
    // looks like the switch doesnt expect anything sent over the control endpoint
    // let mut device_handler = switch::UsbDeviceHandler {};

    let mut builder = Builder::new(
        usb,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // builder.handler(&mut device_handler);

    builder.function(0x21, 0x00, 0x00);
    builder.function(0x22, 0x00, 0x00);

    let config = hid::Config {
        report_descriptor: &HID_DESCRIPTOR,
        request_handler: None,
        poll_ms: 0x05,
        max_packet_size: 64,
    };

    let hid = HidReaderWriter::<_, 64, 64>::new(&mut builder, &mut state, config);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let (reader, mut writer) = hid.split();

    // Do stuff with the class!
    let in_fut = async {
        loop {
            // every 1 second
            _ = Timer::after_secs(1).await;
            let report = ProControllerReport {
                button: Button::SWITCH_A,
                DPAD: Dpad::DPAD_TOP,
                LX: 0,
                LY: 0,
                RX: 0,
                RY: 0,
                VendorSpec: 0,
            };
            // Send the report.
            match writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            }
        }
    };

    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, join(in_fut, out_fut)).await;
}

#![no_std]
#![no_main]

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio;
use embassy_rp::peripherals::{DMA_CH0, PIO0, USB};
use embassy_rp::pio::{self, Pio};
use embassy_rp::usb::{self, Driver};
use embassy_time::Timer;
use embassy_usb::class::hid;
use embassy_usb::class::hid::HidWriter;
use embassy_usb::{Builder, Config};
use gpio::{Level, Output};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod switch;
use switch::*;
// mod xbox;
// use xbox::*;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    spawner.spawn(usb_task(p.USB)).unwrap();

    // // spawn xbox controller task
    // {
    //     // https://github.com/embassy-rs/embassy/tree/main/cyw43-firmware
    //     // let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    //     // let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");
    //     // let btfw = include_bytes!("../cyw43-firmware/43439A0_btfw.bin");

    //     // To make flashing faster for development, you may want to flash the firmwares independently
    //     // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     //     probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     //     probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    //     //     probe-rs download 43439A0_btfw.bin --binary-format bin --chip RP2040 --base-address 0x10141400
    //     let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 224190) };
    //     let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };
    //     let btfw = unsafe { core::slice::from_raw_parts(0x10141400 as *const u8, 6164) };

    //     let pwr = Output::new(p.PIN_23, Level::Low);
    //     let cs = Output::new(p.PIN_25, Level::High);
    //     let mut pio = Pio::new(p.PIO0, Irqs);
    //     let spi = PioSpi::new(
    //         &mut pio.common,
    //         pio.sm0,
    //         pio.irq0,
    //         cs,
    //         p.PIN_24,
    //         p.PIN_29,
    //         p.DMA_CH0,
    //     );

    //     static STATE: StaticCell<cyw43::State> = StaticCell::new();
    //     let state = STATE.init(cyw43::State::new());
    //     info!("before");
    //     let (_net_device, bt_device, mut control, runner) =
    //         cyw43::new_with_bluetooth(state, pwr, spi, fw, btfw).await;
    //     info!("after");
    //     unwrap!(spawner.spawn(cyw43_task(runner)));
    //     control.init(clm).await;

    //     info!("setting up");
    //     bluetooth_setup(bt_device).await;
    // }
}

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn usb_task(usb: USB) {
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
    config.device_release = 0x02;
    config.composite_with_iads;
    config.max_power = 500;
    config.supports_remote_wakeup = true;
    config.self_powered = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 64];
    let mut bos_descriptor = [0; 41];
    let mut control_buf = [0; 64];

    let mut state = hid::State::new();

    let mut request_handler = UsbRequestHandler {};
    // let mut device_handler = switch::UsbDeviceHandler {};

    let mut builder = Builder::new(
        usb,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // add the usb interface class
    let mut func = builder.function(0x03, 0x00, 0x00);
    let mut interface = func.interface();
    let interface_num = interface.interface_number();
    let interface_str = interface.string();
    info!("interface index: {}", interface_num.0);
    info!("interface string index: {}", interface_str.0);
    drop(func);

    // builder.handler(&mut device_handler);

    let config = hid::Config {
        report_descriptor: &HID_DESCRIPTOR,
        request_handler: Some(&mut request_handler),
        poll_ms: 0x05,
        max_packet_size: 64,
    };

    let mut hid_writer = HidWriter::<_, 64>::new(&mut builder, &mut state, config);

    let mut usb = builder.build();
    let usb_fut = usb.run();

    let in_fut = async {
        loop {
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
            match hid_writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            }
            info!("sent button, again");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, in_fut).await;
}

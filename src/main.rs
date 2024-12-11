#![no_std]
#![no_main]

use core::borrow::Borrow;

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio;
use embassy_rp::peripherals::{DMA_CH0, PIO0, USB};
use embassy_rp::pio::{self, Pio};
use embassy_rp::usb::{self, Driver};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Receiver;
use embassy_sync::channel::Sender;
use embassy_sync::mutex::Mutex;
use embassy_sync::once_lock::OnceLock;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embassy_usb::class::hid::{self, HidReader, HidReaderWriter, HidWriter};
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::UsbVersion;
use embassy_usb::{Builder, Config};
use gpio::{Level, Output};
use joycon_sys;
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

const USB_RESPONSE_CHANNEL_SIZE: usize = 10;

static CONTROLLER_STATE: OnceLock<Mutex<NoopRawMutex, ControllerState>> = OnceLock::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    CONTROLLER_STATE
        .init(Mutex::new(ControllerState::new()))
        .expect("Failed to init Controller State");

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
    //     control.init(clm).await;t.is_special()
    //     info!("setting up");
    //     bluetooth_setup(bt_device).await;
    // }

    // spawns usb tasks
    {
        let usb = Driver::new(p.USB, Irqs);

        // Create embassy-usb Config
        let mut config = Config::new(joycon_sys::NINTENDO_VENDOR_ID, joycon_sys::PRO_CONTROLLER);
        config.manufacturer = Some("Nintendo Co., Ltd.");
        config.product = Some("Pro Controller");
        config.serial_number = Some("000000000001");
        config.max_packet_size_0 = 64;
        config.device_class = 0x00;
        config.device_sub_class = 0x00;
        config.device_protocol = 0x00;
        config.device_release = 0x0200;
        config.bcd_usb = UsbVersion::Two;
        config.composite_with_iads;
        config.max_power = 500;
        config.supports_remote_wakeup = true;
        config.self_powered = false;

        // Create embassy-usb DeviceBuilder using the driver and config.
        // It needs some buffers for building the descriptors.
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 64]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        static STATE: StaticCell<hid::State> = StaticCell::new();

        let mut builder = Builder::new(
            usb,
            config,
            CONFIG_DESCRIPTOR.init([0; 64]),
            &mut [], // pro controller does not implement bos
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );

        let config = hid::Config {
            report_descriptor: &HID_DESCRIPTOR,
            request_handler: None,
            poll_ms: 0x08,
            max_packet_size: 64,
        };

        let hid =
            HidReaderWriter::<_, 64, 64>::new(&mut builder, STATE.init(hid::State::new()), config);

        let mut usb = builder.build();
        let usb_fut = usb.run();

        static CHANNEL: StaticCell<
            Channel<NoopRawMutex, joycon_sys::InputReport, USB_RESPONSE_CHANNEL_SIZE>,
        > = StaticCell::new();
        let channel = CHANNEL.init(Channel::<
            NoopRawMutex,
            joycon_sys::InputReport,
            USB_RESPONSE_CHANNEL_SIZE,
        >::new());

        info!("Usb setup and running");
        let (reader, writer) = hid.split();
        unwrap!(spawner.spawn(hid_reader(reader, channel.sender())));
        unwrap!(spawner.spawn(hid_writer(writer, channel.receiver())));
        unwrap!(spawner.spawn(notify(channel.sender())));

        usb_fut.await;
    }
}

fn buf_to_report(buf: [u8; 64], report: &mut joycon_sys::OutputReport) {
    info!(
        "buf len: {}, report len: {}",
        buf.len(),
        report.as_bytes().len()
    );
    for idx in 0..report.byte_size() {
        report.as_bytes_mut()[idx] = buf[idx]
    }
}

#[embassy_executor::task]
async fn hid_reader(
    mut reader: HidReader<'static, Driver<'static, USB>, 64>,
    channel: Sender<'static, NoopRawMutex, joycon_sys::InputReport, USB_RESPONSE_CHANNEL_SIZE>,
) -> ! {
    reader.ready().await;
    let mut output_report = joycon_sys::OutputReport::new();
    let mut buf = [0; 64];
    loop {
        match reader.read(&mut buf).await {
            Ok(_) => {
                buf_to_report(buf, &mut output_report);
                if let Ok(e) = joycon_sys::output::OutputReportEnum::try_from(output_report) {
                    match e {
                        joycon_sys::output::OutputReportEnum::RumbleAndSubcmd(
                            subcommand_request,
                        ) => {
                            if let Ok(cmd) = joycon_sys::output::SubcommandRequestEnum::try_from(
                                subcommand_request,
                            ) {
                                match cmd {
    joycon_sys::output::SubcommandRequestEnum::GetOnlyControllerState(_) => info!("wants controller state"),
    joycon_sys::output::SubcommandRequestEnum::BluetoothManualPairing(_) => (),
    joycon_sys::output::SubcommandRequestEnum::RequestDeviceInfo(_) => info!("wants devidce info" ),
    joycon_sys::output::SubcommandRequestEnum::SetInputReportMode(raw_id) => (),
    joycon_sys::output::SubcommandRequestEnum::GetTriggerButtonsElapsedTime(_) => (),
    joycon_sys::output::SubcommandRequestEnum::SetShipmentMode(raw_id) => (),
    joycon_sys::output::SubcommandRequestEnum::SPIRead(spiread_request) => (),
    joycon_sys::output::SubcommandRequestEnum::SPIWrite(spiwrite_request) => (),
    joycon_sys::output::SubcommandRequestEnum::SetMCUConf(mcucommand) => (),
    joycon_sys::output::SubcommandRequestEnum::SetMCUState(raw_id) => (),
    joycon_sys::output::SubcommandRequestEnum::SetUnknownData(_) => (),
    joycon_sys::output::SubcommandRequestEnum::SetPlayerLights(player_lights) => (),
    joycon_sys::output::SubcommandRequestEnum::SetHomeLight(home_light) => (),
    joycon_sys::output::SubcommandRequestEnum::SetIMUMode(raw_id) => (),
    joycon_sys::output::SubcommandRequestEnum::SetIMUSens(sensitivity) => (),
    joycon_sys::output::SubcommandRequestEnum::EnableVibration(raw_id) => (),
    joycon_sys::output::SubcommandRequestEnum::MaybeAccessory(accessory_command) => (),
    joycon_sys::output::SubcommandRequestEnum::Unknown0x59(_) => (),
    joycon_sys::output::SubcommandRequestEnum::Unknown0x5a(_) => (),
    joycon_sys::output::SubcommandRequestEnum::Unknown0x5b(_) => (),
    joycon_sys::output::SubcommandRequestEnum::Unknown0x5c(_) => (),
}
                            }
                        }
                        joycon_sys::output::OutputReportEnum::MCUFwUpdate(_) => {
                            info!("Got update")
                        }
                        joycon_sys::output::OutputReportEnum::RumbleOnly(_) => {
                            info!("Got rumble only ")
                        }
                        joycon_sys::output::OutputReportEnum::RequestMCUData(mcurequest) => {
                            info!("Got mcu")
                        }
                    }
                }
            }
            Err(error) => warn!("usb read error: {}", error),
        }
    }
}

pub static NOTIFY_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[embassy_executor::task]
async fn notify(
    channel: Sender<'static, NoopRawMutex, joycon_sys::InputReport, USB_RESPONSE_CHANNEL_SIZE>,
) -> ! {
    // wait till handshakes are done
    NOTIFY_SIGNAL.wait().await;
    loop {
        Timer::after_millis(8).await;
        let report = joycon_sys::input::InputReportEnum::StandardAndSubcmd((
            CONTROLLER_STATE.get().await.lock().await.standard(),
            joycon_sys::input::SubcommandReply::from(
                joycon_sys::input::SubcommandReplyEnum::RequestDeviceInfo(device_info()),
            ),
        ));
        channel.send(joycon_sys::InputReport::from(report)).await;
    }
}

#[embassy_executor::task]
async fn hid_writer(
    mut writer: HidWriter<'static, Driver<'static, USB>, 64>,
    channel: Receiver<'static, NoopRawMutex, joycon_sys::InputReport, USB_RESPONSE_CHANNEL_SIZE>,
) -> ! {
    writer.ready().await;

    // sends connection status
    let report = joycon_sys::input::InputReportEnum::StandardAndSubcmd((
        CONTROLLER_STATE.get().await.lock().await.standard(),
        joycon_sys::input::SubcommandReply::from(
            joycon_sys::input::SubcommandReplyEnum::RequestDeviceInfo(device_info()),
        ),
    ));
    unwrap!(
        writer
            .write(joycon_sys::InputReport::from(report).as_bytes())
            .await
    );

    loop {
        let input = channel.receive().await;
        unwrap!(writer.write(input.as_bytes()).await);
    }
}

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

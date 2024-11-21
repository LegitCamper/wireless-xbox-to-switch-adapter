#![no_std]
#![no_main]

use bt_hci::controller::ExternalController;
use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::join::join3;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio;
use embassy_rp::peripherals::{DMA_CH0, PIO0, USB};
use embassy_rp::pio::{self, Pio};
use embassy_rp::usb::{self, Driver};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};
use embassy_usb::class::hid::HidReader;
use embassy_usb::class::hid::HidWriter;
use embassy_usb::class::hid::{self, HidReaderWriter};
use embassy_usb::{Builder, Config};
use gpio::{Level, Output};
use static_cell::StaticCell;
use trouble_host::advertise::{
    AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
};
use trouble_host::attribute::{AttributeTable, CharacteristicProp, Service, Uuid};
use trouble_host::gatt::GattEvent;
use trouble_host::{Address, BleHost, BleHostResources, PacketQos};
use {defmt_rtt as _, panic_probe as _};

mod switch;
use switch::*;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    spawner.spawn(usb_task(p.USB)).unwrap();

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, bt_device, mut control, runner) =
        cyw43::new_with_bluetooth(state, pwr, spi, fw, btfw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));
    control.init(clm).await;

    spawner
        .spawn(bluetooth_task(Output::new(p.PIN_23, Level::Low), spi))
        .unwrap();
}

#[embassy_executor::task]
async fn bluetooth_task(pwr: Output<'static>, spi: PioSpi<'static, PIO0, 0, DMA_CH0>) {
    // https://github.com/embassy-rs/embassy/tree/main/cyw43-firmware
    // let fw = include_bytes!("../../../../cyw43-firmware/43439A0.bin");
    // let clm = include_bytes!("../../../../cyw43-firmware/43439A0_clm.bin");
    // let btfw = include_bytes!("../../../../cyw43-firmware/43439A0_btfw.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    //     probe-rs download 43439A0_btfw.bin --binary-format bin --chip RP2040 --base-address 0x10141400
    let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 224190) };
    let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };
    let btfw = unsafe { core::slice::from_raw_parts(0x10141400 as *const u8, 6164) };

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    control.init(clm).await;

    let controller: ExternalController<_, 10> = ExternalController::new(bt_device);
    static HOST_RESOURCES: StaticCell<BleHostResources<4, 32, 27>> = StaticCell::new();
    let host_resources = HOST_RESOURCES.init(BleHostResources::new(PacketQos::None));

    let mut ble: BleHost<'_, _> = BleHost::new(controller, host_resources);

    ble.set_random_address(Address::random([0xff, 0x9f, 0x1a, 0x05, 0xe4, 0xff]));
    let mut table: AttributeTable<'_, NoopRawMutex, 10> = AttributeTable::new();

    // Generic Access Service (mandatory)
    let id = b"Pico W Bluetooth";
    let appearance = [0x80, 0x07];
    let mut bat_level = [0; 1];
    let handle = {
        let mut svc = table.add_service(Service::new(0x1800));
        let _ = svc.add_characteristic_ro(0x2a00, id);
        let _ = svc.add_characteristic_ro(0x2a01, &appearance[..]);
        svc.build();

        // Generic attribute service (mandatory)
        table.add_service(Service::new(0x1801));

        // Battery service
        let mut svc = table.add_service(Service::new(0x180f));

        svc.add_characteristic(
            0x2a19,
            &[CharacteristicProp::Read, CharacteristicProp::Notify],
            &mut bat_level,
        )
        .build()
    };

    let mut adv_data = [0; 31];
    AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[Uuid::Uuid16([0x0f, 0x18])]),
            AdStructure::CompleteLocalName(b"Pico W Bluetooth"),
        ],
        &mut adv_data[..],
    )
    .unwrap();

    let server = ble.gatt_server(&table);

    info!("Starting advertising and GATT service");
    let _ = join3(
        ble.run(),
        async {
            loop {
                match server.next().await {
                    Ok(GattEvent::Write {
                        handle,
                        connection: _,
                    }) => {
                        let _ = table.get(handle, |value| {
                            info!("Write event. Value written: {:?}", value);
                        });
                    }
                    Ok(GattEvent::Read { .. }) => {
                        info!("Read event");
                    }
                    Err(e) => {
                        error!("Error processing GATT events: {:?}", e);
                    }
                }
            }
        },
        async {
            let mut advertiser = ble
                .advertise(
                    &Default::default(),
                    Advertisement::ConnectableScannableUndirected {
                        adv_data: &adv_data[..],
                        scan_data: &[],
                    },
                )
                .await
                .unwrap();
            let conn = advertiser.accept().await.unwrap();
            // Keep connection alive
            let mut tick: u8 = 0;
            loop {
                Timer::after(Duration::from_secs(10)).await;
                tick += 1;
                server.notify(handle, &conn, &[tick]).await.unwrap();
            }
        },
    )
    .await;
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
    info!("interface index: {:?}", interface_num);
    info!("interface string index: {:?}", interface_str);
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

use defmt::info;
use defmt::*;
use embassy_time::Timer;
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{
    class::hid::{ReportId, RequestHandler},
    Handler,
};
use embassy_usb::{Builder, Config};
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};

mod hid_descriptor;
pub use hid_descriptor::UsbSwitchProControllerReport;
pub use hid_descriptor::HID_DESCRIPTOR;
use joycon_sys::input::*;

pub fn device_info() -> DeviceInfo {
    DeviceInfo::new(
        FirmwareVersion([0x03, 0x48]),
        WhichController::ProController.try_into().unwrap(),
        MACAddress([0x7c, 0xbb, 0x8a, 0xea, 0x30, 0x57]),
        UseSPIColors::No.into(),
    )
}

#[derive(Debug)]
pub struct ControllerState {
    buttons: ButtonsStatus,
    left_stick: Stick,
    right_stick: Stick,
    status: DeviceStatus,
}

impl ControllerState {
    pub fn new() -> Self {
        Self {
            buttons: Default::default(),
            left_stick: Stick::new(),
            right_stick: Stick::new(),
            status: DeviceStatus(0),
        }
    }

    pub fn standard(&self) -> StandardInputReport {
        StandardInputReport {
            timer: 0,
            info: self.status,
            buttons: self.buttons,
            left_stick: self.left_stick,
            right_stick: self.right_stick,
            vibrator: 0,
        }
    }
}

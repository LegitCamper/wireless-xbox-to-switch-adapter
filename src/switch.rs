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
pub use hid_descriptor::SwitchProControllerReport;
pub use hid_descriptor::HID_DESCRIPTOR;

pub enum ResponseType {
    Bytes([u8; 64]),
    ControllerUpdate,
}

pub enum ReportType {
    Nintendo(NintendoReportType),
    Hid,
}

impl ReportType {
    pub fn parse(msg: &[u8]) -> Option<Self> {
        if msg[0] == 0x80 {
            if msg[1] == 0x02 {
                Some(Self::Nintendo(NintendoReportType::Handshake))
            } else if msg[1] == 0x03 {
                Some(Self::Nintendo(NintendoReportType::Baudrate))
            } else if msg[1] == 0x04 {
                Some(Self::Nintendo(NintendoReportType::NoTimeout))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub enum NintendoReportType {
    Handshake,
    Baudrate,
    NoTimeout,
}

impl NintendoReportType {
    pub fn resp(&self) -> [u8; 64] {
        let mut resp = [0; 64];
        match self {
            NintendoReportType::Handshake => {
                resp[0] = 0x81;
                resp[1] = 0x02;
                resp
            }
            NintendoReportType::Baudrate => {
                resp[0] = 0x81;
                resp[1] = 0x03;
                resp
            }
            NintendoReportType::NoTimeout => {
                resp[0] = 0x81;
                resp[1] = 0x02;
                resp
            }
        }
    }
}

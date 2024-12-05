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

#[derive(Debug)]
pub enum ReportType {
    Nintendo(NintendoReportType),
    Hid(HidReportType),
}

impl ReportType {
    pub fn parse(msg: &[u8]) -> Option<Self> {
        if msg[0] == 0x80 {
            if msg[1] == 0x01 {
                Some(Self::Nintendo(NintendoReportType::Status))
            } else if msg[1] == 0x02 {
                Some(Self::Nintendo(NintendoReportType::Handshake))
            } else if msg[1] == 0x03 {
                Some(Self::Nintendo(NintendoReportType::Baudrate))
            } else if msg[1] == 0x04 {
                Some(Self::Nintendo(NintendoReportType::NoTimeout))
            } else {
                None
            }
        } else if msg[0] == 0x01 {
            if msg[0] == 0x01 {
                if msg[1] == 0x00 {
                    Some(Self::Hid(HidReportType::Zero))
                } else if msg[1] == 0x01 {
                    Some(Self::Hid(HidReportType::One))
                } else if msg[1] == 0x02 {
                    Some(Self::Hid(HidReportType::Two))
                } else if msg[1] == 0x03 {
                    Some(Self::Hid(HidReportType::Three))
                } else if msg[1] == 0x04 {
                    Some(Self::Hid(HidReportType::Four))
                } else if msg[1] == 0x05 {
                    Some(Self::Hid(HidReportType::Five))
                } else if msg[1] == 0x06 {
                    Some(Self::Hid(HidReportType::Six))
                } else if msg[1] == 0x07 {
                    Some(Self::Hid(HidReportType::Seven))
                } else if msg[1] == 0x08 {
                    Some(Self::Hid(HidReportType::Eight))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum HidReportType {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

impl HidReportType {
    pub fn resp(&self) -> Option<[u8; 64]> {
        let mut resp = [0; 64];
        match self {
            HidReportType::Zero => {
                resp[..27].copy_from_slice(&[
                    0x21, 0x44, 0x91, 0x0, 0x80, 0x0, 0x11, 0xf8, 0x7c, 0x3b, 0x88, 0x82, 0x0,
                    0x82, 0x2, 0x3, 0x48, 0x3, 0x2, 0xdc, 0x68, 0xeb, 0xed, 0x5c, 0x79, 0x1, 0x1,
                ]);
                Some(resp)
            }
            HidReportType::One => {
                resp[..22].copy_from_slice(&[
                    0x21, 0x4a, 0x91, 0x0, 0x80, 0x0, 0x10, 0x38, 0x7d, 0x3d, 0x88, 0x82, 0x0,
                    0x90, 0x10, 0x10, 0x80, 0x0, 0x0, 0x2, 0xff, 0xff,
                ]);
                Some(resp)
            }
            HidReportType::Two => {
                resp[..22].copy_from_slice(&[
                    0x21, 0x54, 0x91, 0x0, 0x80, 0x0, 0x10, 0x48, 0x7d, 0x3d, 0x88, 0x82, 0x0,
                    0x90, 0x10, 0x1b, 0x80, 0x0, 0x0, 0x2, 0xb2, 0xa1,
                ]);
                Some(resp)
            }
            HidReportType::Three => {
                resp[..29].copy_from_slice(&[
                    0x21, 0x5b, 0x91, 0x0, 0x80, 0x0, 0xf, 0x28, 0x7d, 0x3b, 0x78, 0x82, 0xb, 0x90,
                    0x10, 0x3d, 0x60, 0x0, 0x0, 0x9, 0xe2, 0x95, 0x6a, 0x25, 0x98, 0x7e, 0xfd, 0x5,
                    0x5f,
                ]);
                Some(resp)
            }
            HidReportType::Four => {
                resp[..29].copy_from_slice(&[
                    0x21, 0x64, 0x91, 0x0, 0x80, 0x0, 0x11, 0x38, 0x7d, 0x3d, 0x88, 0x82, 0x9,
                    0x90, 0x10, 0x1d, 0x80, 0x0, 0x0, 0x9, 0xf8, 0x97, 0x80, 0x43, 0x86, 0x61,
                    0xc1, 0x55, 0x62,
                ]);
                Some(resp)
            }
            HidReportType::Five => {
                resp[..22].copy_from_slice(&[
                    0x21, 0x6d, 0x91, 0x0, 0x80, 0x0, 0xf, 0x58, 0x7d, 0x3d, 0x98, 0x82, 0x9, 0x90,
                    0x10, 0x26, 0x80, 0x0, 0x0, 0x2, 0xb2, 0xa1,
                ]);
                Some(resp)
            }
            HidReportType::Six => {
                resp[..44].copy_from_slice(&[
                    0x21, 0x75, 0x91, 0x0, 0x80, 0x0, 0xe, 0x78, 0x7d, 0x3a, 0x98, 0x82, 0xa, 0x90,
                    0x10, 0x28, 0x80, 0x0, 0x0, 0x18, 0xf, 0x1, 0x3a, 0x0, 0xb7, 0x0, 0x0, 0x40,
                    0x0, 0x40, 0x0, 0x40, 0x1f, 0x0, 0xd9, 0xff, 0xd5, 0xff, 0x3b, 0x34, 0x3b,
                    0x34, 0x3b, 0x34,
                ]);
                Some(resp)
            }
            HidReportType::Seven => {
                resp[..15].copy_from_slice(&[
                    0x21, 0x7e, 0x91, 0x0, 0x80, 0x0, 0xf, 0x38, 0x7d, 0x3c, 0x88, 0x82, 0xa, 0x80,
                    0x40,
                ]);
                Some(resp)
            }
            HidReportType::Eight => {
                resp[..15].copy_from_slice(&[
                    0x21, 0x87, 0x91, 0x0, 0x80, 0x0, 0x10, 0xf8, 0x7c, 0x3d, 0x98, 0x82, 0xb,
                    0x80, 0x3,
                ]);
                Some(resp)
            }
            HidReportType::Nine => {
                resp[..64].copy_from_slice(&[]);
                Some(resp)
            }
            HidReportType::Ten => {
                resp[..64].copy_from_slice(&[]);
                Some(resp)
            }
        }
    }
}

#[derive(Debug)]
pub enum NintendoReportType {
    Status,
    Handshake,
    Baudrate,
    NoTimeout,
}

impl NintendoReportType {
    pub fn resp(&self) -> Option<[u8; 64]> {
        let mut resp = [0; 64];
        match self {
            NintendoReportType::Status => {
                resp[..10]
                    .copy_from_slice(&[0x81, 0x1, 0x0, 0x3, 0x79, 0x5c, 0xed, 0xeb, 0x68, 0xdc]);
                Some(resp)
            }
            NintendoReportType::Handshake => {
                resp[..2].copy_from_slice(&[0x81, 0x02]);
                Some(resp)
            }
            NintendoReportType::Baudrate => {
                resp[..2].copy_from_slice(&[0x81, 0x03]);
                Some(resp)
            }
            NintendoReportType::NoTimeout => None,
        }
    }
}

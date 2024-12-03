use defmt::info;
use defmt::*;
use embassy_time::Timer;
use embassy_usb::class::hid::{self, HidReader, HidReaderWriter, HidWriter};
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::{Driver, Endpoint, EndpointIn, EndpointOut};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{
    class::hid::{ReportId, RequestHandler},
    Handler,
};
use embassy_usb::{Builder, Config};

mod hid_descriptor;
use hid_descriptor::SwitchProControllerReport;
use serde::Serialize;
use usbd_hid::descriptor::SerializedDescriptor;

pub struct HidEndpoints<'d, D: Driver<'d>> {
    writer: HidWriter<'d, D, 64>,
    reader: HidReader<'d, D, 64>,
    handshake: bool,
}

impl<'d, D: Driver<'d>> HidEndpoints<'d, D> {
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut hid::State<'d>) -> Self {
        // add the hid interface class
        // let mut func = builder.function(0x03, 0x00, 0x00);
        // let mut interface = func.interface();
        // let interface_num = interface.interface_number();
        // let interface_str = interface.string();
        // info!("interface index: {}", interface_num.0);
        // info!("interface string index: {}", interface_str.0);
        // drop(func);

        let config = hid::Config {
            report_descriptor: &SwitchProControllerReport::desc(),
            request_handler: None,
            poll_ms: 0x08,
            max_packet_size: 64,
        };

        let hid = HidReaderWriter::<_, 64, 64>::new(builder, state, config);
        let (reader, writer) = hid.split();

        HidEndpoints {
            reader,
            writer,
            handshake: false,
        }
    }

    // Wait until the device's endpoints are enabled.
    pub async fn wait_connected(&mut self) {
        self.reader.ready().await;
        self.writer.ready().await;
    }

    pub async fn test(&mut self) {
        // try read
        let mut buf = [0; 64];
        match self.reader.read(&mut buf).await {
            Ok(_) => {
                info!("resp: {:?}", buf);
                // is nintendo protocol
                if buf[0] == 0x80 {
                    if buf[1] == 0x02 {
                        // complete handshake
                        self.handshake = true;
                        info!("completing handshake");
                        unwrap!(self.writer.write(&[0x81, 0x02]).await);
                    } else if buf[1] == 0x02 {
                        unwrap!(self.writer.write(&[0x81, 0x03]).await);
                    }
                }
            }

            Err(_) => (),
        }

        if self.handshake {
            let mut report = SwitchProControllerReport::new();
            report.press_button(hid_descriptor::Button::A);
            match self.writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            }
            info!("sent button, again");
        }
    }
}

/// Handle CONTROL endpoint requests and responses. For many simple requests and responses
/// you can get away with only using the control endpoint.
pub struct ControlHandler {
    pub if_num: InterfaceNumber,
}

impl ControlHandler {
    pub fn new() -> Self {
        Self {
            if_num: InterfaceNumber(0),
        }
    }
}

impl Handler for ControlHandler {
    /// Respond to HostToDevice control messages, where the host sends us a command and
    /// optionally some data, and we can only acknowledge or reject it.
    fn control_out<'a>(&'a mut self, req: Request, buf: &'a [u8]) -> Option<OutResponse> {
        // Log the request before filtering to help with debugging.
        // info!("Got control_out, request={}, buf={:a}", req, buf);

        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor || req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != self.if_num.0 as u16 {
            return None;
        }

        // Accept request 100, value 200, reject others.
        if req.request == 100 && req.value == 200 {
            Some(OutResponse::Accepted)
        } else {
            Some(OutResponse::Rejected)
        }
    }

    /// Respond to DeviceToHost control messages, where the host requests some data from us.
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        // info!("Got control_in, request={}", req);

        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor || req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != self.if_num.0 as u16 {
            return None;
        }

        // Respond "hello" to request 101, value 201, when asked for 5 bytes, otherwise reject.
        if req.request == 101 && req.value == 201 && req.length == 5 {
            buf[..5].copy_from_slice(b"hello");
            Some(InResponse::Accepted(&buf[..5]))
        } else {
            Some(InResponse::Rejected)
        }
    }
}

pub struct UsbRequestHandler {}

impl RequestHandler for UsbRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        // info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        // info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        // info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        // info!("Get idle rate for {:?}", id);
        None
    }
}

pub static HID_DESCRIPTOR: &'static [u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)        0
    0x15, 0x00, // Logical Minimum (0)                 2
    0x09, 0x04, // Usage (Joystick)                    4
    0xa1, 0x01, // Collection (Application)            6
    0x85, 0x30, //  Report ID (48)                     8
    0x05, 0x01, //  Usage Page (Generic Desktop)       10
    0x05, 0x09, //  Usage Page (Button)                12
    0x19, 0x01, //  Usage Minimum (1)                  14
    0x29, 0x0a, //  Usage Maximum (10)                 16
    0x15, 0x00, //  Logical Minimum (0)                18
    0x25, 0x01, //  Logical Maximum (1)                20
    0x75, 0x01, //  Report Size (1)                    22
    0x95, 0x0a, //  Report Count (10)                  24
    0x55, 0x00, //  Unit Exponent (0)                  26
    0x65, 0x00, //  Unit (None)                        28
    0x81, 0x02, //  Input (Data,Var,Abs)               30
    0x05, 0x09, //  Usage Page (Button)                32
    0x19, 0x0b, //  Usage Minimum (11)                 34
    0x29, 0x0e, //  Usage Maximum (14)                 36
    0x15, 0x00, //  Logical Minimum (0)                38
    0x25, 0x01, //  Logical Maximum (1)                40
    0x75, 0x01, //  Report Size (1)                    42
    0x95, 0x04, //  Report Count (4)                   44
    0x81, 0x02, //  Input (Data,Var,Abs)               46
    0x75, 0x01, //  Report Size (1)                    48
    0x95, 0x02, //  Report Count (2)                   50
    0x81, 0x03, //  Input (Cnst,Var,Abs)               52
    0x0b, 0x01, 0x00, 0x01, 0x00, //  Usage (Vendor Usage 0x10001)       54
    0xa1, 0x00, //  Collection (Physical)              59
    0x0b, 0x30, 0x00, 0x01, 0x00, //   Usage (Vendor Usage 0x10030)      61
    0x0b, 0x31, 0x00, 0x01, 0x00, //   Usage (Vendor Usage 0x10031)      66
    0x0b, 0x32, 0x00, 0x01, 0x00, //   Usage (Vendor Usage 0x10032)      71
    0x0b, 0x35, 0x00, 0x01, 0x00, //   Usage (Vendor Usage 0x10035)      76
    0x15, 0x00, //   Logical Minimum (0)               81
    0x27, 0xff, 0xff, 0x00, 0x00, //   Logical Maximum (65535)           83
    0x75, 0x10, //   Report Size (16)                  88
    0x95, 0x04, //   Report Count (4)                  90
    0x81, 0x02, //   Input (Data,Var,Abs)              92
    0xc0, //  End Collection                     94
    0x0b, 0x39, 0x00, 0x01, 0x00, //  Usage (Vendor Usage 0x10039)       95
    0x15, 0x00, //  Logical Minimum (0)                100
    0x25, 0x07, //  Logical Maximum (7)                102
    0x35, 0x00, //  Physical Minimum (0)               104
    0x46, 0x3b, 0x01, //  Physical Maximum (315)             106
    0x65, 0x14, //  Unit (EnglishRotation: deg)        109
    0x75, 0x04, //  Report Size (4)                    111
    0x95, 0x01, //  Report Count (1)                   113
    0x81, 0x02, //  Input (Data,Var,Abs)               115
    0x05, 0x09, //  Usage Page (Button)                117
    0x19, 0x0f, //  Usage Minimum (15)                 119
    0x29, 0x12, //  Usage Maximum (18)                 121
    0x15, 0x00, //  Logical Minimum (0)                123
    0x25, 0x01, //  Logical Maximum (1)                125
    0x75, 0x01, //  Report Size (1)                    127
    0x95, 0x04, //  Report Count (4)                   129
    0x81, 0x02, //  Input (Data,Var,Abs)               131
    0x75, 0x08, //  Report Size (8)                    133
    0x95, 0x34, //  Report Count (52)                  135
    0x81, 0x03, //  Input (Cnst,Var,Abs)               137
    0x06, 0x00, 0xff, //  Usage Page (Vendor Defined Page 1) 139
    0x85, 0x21, //  Report ID (33)                     142
    0x09, 0x01, //  Usage (Vendor Usage 1)             144
    0x75, 0x08, //  Report Size (8)                    146
    0x95, 0x3f, //  Report Count (63)                  148
    0x81, 0x03, //  Input (Cnst,Var,Abs)               150
    0x85, 0x81, //  Report ID (129)                    152
    0x09, 0x02, //  Usage (Vendor Usage 2)             154
    0x75, 0x08, //  Report Size (8)                    156
    0x95, 0x3f, //  Report Count (63)                  158
    0x81, 0x03, //  Input (Cnst,Var,Abs)               160
    0x85, 0x01, //  Report ID (1)                      162
    0x09, 0x03, //  Usage (Vendor Usage 0x03)          164
    0x75, 0x08, //  Report Size (8)                    166
    0x95, 0x3f, //  Report Count (63)                  168
    0x91, 0x83, //  Output (Cnst,Var,Abs,Vol)          170
    0x85, 0x10, //  Report ID (16)                     172
    0x09, 0x04, //  Usage (Vendor Usage 0x04)          174
    0x75, 0x08, //  Report Size (8)                    176
    0x95, 0x3f, //  Report Count (63)                  178
    0x91, 0x83, //  Output (Cnst,Var,Abs,Vol)          180
    0x85, 0x80, //  Report ID (128)                    182
    0x09, 0x05, //  Usage (Vendor Usage 0x05)          184
    0x75, 0x08, //  Report Size (8)                    186
    0x95, 0x3f, //  Report Count (63)                  188
    0x91, 0x83, //  Output (Cnst,Var,Abs,Vol)          190
    0x85, 0x82, //  Report ID (130)                    192
    0x09, 0x06, //  Usage (Vendor Usage 0x06)          194
    0x75, 0x08, //  Report Size (8)                    196
    0x95, 0x3f, //  Report Count (63)                  198
    0x91, 0x83, //  Output (Cnst,Var,Abs,Vol)          200
    0xc0, // End Collection                      202
];

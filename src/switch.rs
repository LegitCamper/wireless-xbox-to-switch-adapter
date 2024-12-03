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

        HidEndpoints { reader, writer }
    }

    // Wait until the device's endpoints are enabled.
    pub async fn wait_connected(&mut self) {
        self.reader.ready().await;
        self.writer.ready().await;
    }

    pub async fn run(&mut self) {
        let mut buf = [0; 64];

        let mut controller = SwitchProControllerReport::new();

        info!("usb descriptor: {:x}", SwitchProControllerReport::desc());

        loop {
            match self.reader.read(&mut buf).await {
                Ok(_) => {
                    // is nintendo protocol
                    if buf[0] == 0x80 {
                        if buf[1] == 0x02 {
                            // complete handshake
                            info!("completing handshake");
                            unwrap!(self.writer.write(&[0x81, 0x02]).await);
                        } else if buf[1] == 0x02 {
                            unwrap!(self.writer.write(&[0x81, 0x03]).await);
                        }
                    } else if buf[0] == 1 {
                        info!("got hid command");
                        unwrap!(self.writer.write_serialize(&controller).await);
                    }
                }

                Err(error) => info!("usb read error: {}", error),
            }

            // if self.handshake {
            //     let mut report = SwitchProControllerReport::new();
            //     report.press_button(hid_descriptor::Button::A);
            //     match self.writer.write_serialize(&report).await {
            //         Ok(()) => {}
            //         Err(e) => warn!("Failed to send report: {:?}", e),
            //     }
            //     info!("sent button, again");
            // }
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

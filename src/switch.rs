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
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};

mod hid_descriptor;
use hid_descriptor::SwitchProControllerReport;
use hid_descriptor::HID_DESCRIPTOR;

pub enum NintendoReportType {
    Handshake,
    Baudrate,
    NoTimeout,
}

impl NintendoReportType {
    pub fn parse(msg: &[u8]) -> Option<Self> {
        if msg[0] == 0x80 {
            if msg[1] == 0x02 {
                Some(Self::Handshake)
            } else if msg[1] == 0x03 {
                Some(Self::Baudrate)
            } else if msg[1] == 0x04 {
                Some(Self::NoTimeout)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct HidEndpoints<'d, D: Driver<'d>> {
    writer: HidWriter<'d, D, 64>,
    reader: HidReader<'d, D, 64>,
    read_buf: [u8; 64],
    update_host: bool,
}

impl<'d, D: Driver<'d>> HidEndpoints<'d, D> {
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut hid::State<'d>) -> Self {
        let config = hid::Config {
            // report_descriptor: &SwitchProControllerReport::desc(),
            report_descriptor: &HID_DESCRIPTOR,
            request_handler: None,
            poll_ms: 0x08,
            max_packet_size: 64,
        };

        let hid = HidReaderWriter::<_, 64, 64>::new(builder, state, config);
        let (reader, writer) = hid.split();

        let read_buf = [0; 64];

        HidEndpoints {
            writer,
            reader,
            read_buf,
            update_host: false,
        }
    }

    // Wait until the device's endpoints are enabled.
    pub async fn wait_connected(&mut self) {
        self.reader.ready().await;
        self.writer.ready().await;
    }

    pub async fn handshake(&mut self) {
        match self.reader.read(&mut self.read_buf).await {
            Ok(_) => {
                // is nintendo protocol
                if self.read_buf[0] == 0x80 {
                    info!(
                        "Got Nintendo request: {} {}",
                        self.read_buf[0], self.read_buf[1]
                    );
                    // handles connection status
                    if self.read_buf[1] == 0x01 {
                        self.write(&[
                            // just a test
                            0x81, 0x01, 0x00, 0x02, 0x57, 0x30, 0xea, 0x8a, 0xbb, 0x7c,
                        ])
                        .await
                    }
                    if self.read_buf[1] == 0x02 {
                        self.write(&[0x81, 0x02]).await;
                    } else if self.read_buf[1] == 0x03 {
                        self.write(&[0x81, 0x03]).await;
                    } else if self.read_buf[1] == 0x04 {
                        return;
                    }
                }
            }

            Err(error) => warn!("usb read error: {}", error),
        }
    }

    // Sends current connection status, and if the Joy-Con are connected,
    // a MAC address and the type of controller.
    pub async fn init(&mut self) {
        self.write(&[
            0x81, 0x1, 0x0, 0x3, 0x79, 0x5c, 0xed, 0xeb, 0x68, 0xdc, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ])
        .await;
    }

    pub async fn write(&mut self, msg: &[u8]) {
        match self.writer.write(msg).await {
            Ok(_) => (),
            Err(error) => warn!("Usb write error: {}", error),
        }
    }

    pub async fn write_serialize<IR: AsInputReport>(&mut self, msg: &IR) {
        match self.writer.write_serialize(msg).await {
            Ok(_) => (),
            Err(error) => warn!("Usb write error: {}", error),
        }
    }

    pub async fn run(&mut self) {
        let mut controller = SwitchProControllerReport::new();

        info!("usb descriptor: {:x}", SwitchProControllerReport::desc());

        loop {
            if self.update_host {
                loop {
                    Timer::after_millis(8).await;
                    self.write(&[
                        0x30, 0xd7, 0x91, 0x0, 0x80, 0x0, 0x10, 0x38, 0x7d, 0x3c, 0x88, 0x82, 0xc,
                        0x90, 0xfe, 0x4, 0x1, 0x11, 0x10, 0x29, 0x0, 0xd9, 0xff, 0xe0, 0xff, 0x92,
                        0xfe, 0x3, 0x1, 0x10, 0x10, 0x29, 0x0, 0xd8, 0xff, 0xde, 0xff, 0x94, 0xfe,
                        0x2, 0x1, 0x11, 0x10, 0x2a, 0x0, 0xd9, 0xff, 0xde, 0xff, 0x0, 0x0, 0x0,
                        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                    ])
                    .await
                }
            }
        }
    }
}

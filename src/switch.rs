use embassy_usb::{class::hid::RequestHandler, Handler};

pub struct UsbRequestHandler {}

impl RequestHandler for UsbRequestHandler {
    fn get_report(
        &mut self,
        id: embassy_usb::class::hid::ReportId,
        buf: &mut [u8],
    ) -> Option<usize> {
        let _ = (id, buf);
        None
    }

    fn set_report(
        &mut self,
        id: embassy_usb::class::hid::ReportId,
        data: &[u8],
    ) -> embassy_usb::control::OutResponse {
        let _ = (id, data);
        embassy_usb::control::OutResponse::Rejected
    }

    fn get_idle_ms(&mut self, id: Option<embassy_usb::class::hid::ReportId>) -> Option<u32> {
        let _ = id;
        None
    }

    fn set_idle_ms(&mut self, id: Option<embassy_usb::class::hid::ReportId>, duration_ms: u32) {
        let _ = (id, duration_ms);
    }
}

pub struct UsbDeviceHandler {}

impl Handler for UsbDeviceHandler {
    fn enabled(&mut self, _enabled: bool) {}

    fn reset(&mut self) {}

    fn addressed(&mut self, _addr: u8) {}

    fn configured(&mut self, _configured: bool) {}

    fn suspended(&mut self, _suspended: bool) {}

    fn remote_wakeup_enabled(&mut self, _enabled: bool) {}

    fn set_alternate_setting(
        &mut self,
        iface: embassy_usb::types::InterfaceNumber,
        alternate_setting: u8,
    ) {
        let _ = iface;
        let _ = alternate_setting;
    }

    fn control_out(
        &mut self,
        req: embassy_usb::control::Request,
        data: &[u8],
    ) -> Option<embassy_usb::control::OutResponse> {
        let _ = (req, data);
        None
    }

    fn control_in<'a>(
        &'a mut self,
        req: embassy_usb::control::Request,
        buf: &'a mut [u8],
    ) -> Option<embassy_usb::control::InResponse<'a>> {
        let _ = (req, buf);
        None
    }

    fn get_string(&mut self, index: embassy_usb::types::StringIndex, lang_id: u16) -> Option<&str> {
        // let _ = (index, embassy_usb::descriptor::lang_id);
        None
    }
}

pub static HID_DESCRIPTOR: &'static [u8] = &[
    // HID Descriptor
    0x05, 0x01, // Usage Page (Generic Desktop Ctrls)
    0x15, 0x00, // Logical Minimum (0)
    0x09, 0x04, // Usage (Joystick)
    0xA1, 0x01, // Collection (Application)
    0x85, 0x30, //   Report ID (48)
    0x05, 0x01, //   Usage Page (Generic Desktop Ctrls)
    0x05, 0x09, //   Usage Page (Button)
    0x19, 0x01, //   Usage Minimum (0x01)
    0x29, 0x0A, //   Usage Maximum (0x0A)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x0A, //   Report Count (10)
    0x55, 0x00, //   Unit Exponent (0)
    0x65, 0x00, //   Unit (None)
    0x81, 0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x05, 0x09, //   Usage Page (Button)
    0x19, 0x0B, //   Usage Minimum (0x0B)
    0x29, 0x0E, //   Usage Maximum (0x0E)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x04, //   Report Count (4)
    0x81, 0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x03, //   Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x0B, 0x01, 0x00, 0x01, 0x00, //   Usage (0x010001)
    0xA1, 0x00, //   Collection (Physical)
    0x0B, 0x30, 0x00, 0x01, 0x00, //     Usage (0x010030)
    0x0B, 0x31, 0x00, 0x01, 0x00, //     Usage (0x010031)
    0x0B, 0x32, 0x00, 0x01, 0x00, //     Usage (0x010032)
    0x0B, 0x35, 0x00, 0x01, 0x00, //     Usage (0x010035)
    0x15, 0x00, //     Logical Minimum (0)
    0x27, 0xFF, 0xFF, 0x00, 0x00, //     Logical Maximum (65534)
    0x75, 0x10, //     Report Size (16)
    0x95, 0x04, //     Report Count (4)
    0x81, 0x02, //     Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0, //   End Collection
    0x0B, 0x39, 0x00, 0x01, 0x00, //   Usage (0x010039)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x07, //   Logical Maximum (7)
    0x35, 0x00, //   Physical Minimum (0)
    0x46, 0x3B, 0x01, //   Physical Maximum (315)
    0x65, 0x14, //   Unit (System: English Rotation, Length: Centimeter)
    0x75, 0x04, //   Report Size (4)
    0x95, 0x01, //   Report Count (1)
    0x81, 0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x05, 0x09, //   Usage Page (Button)
    0x19, 0x0F, //   Usage Minimum (0x0F)
    0x29, 0x12, //   Usage Maximum (0x12)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x04, //   Report Count (4)
    0x81, 0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x34, //   Report Count (52)
    0x81, 0x03, //   Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x06, 0x00, 0xFF, //   Usage Page (Vendor Defined 0xFF00)
    0x85, 0x21, //   Report ID (33)
    0x09, 0x01, //   Usage (0x01)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x3F, //   Report Count (63)
    0x81, 0x03, //   Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x85, 0x81, //   Report ID (-127)
    0x09, 0x02, //   Usage (0x02)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x3F, //   Report Count (63)
    0x81, 0x03, //   Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x85, 0x01, //   Report ID (1)
    0x09, 0x03, //   Usage (0x03)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x3F, //   Report Count (63)
    0x91,
    0x83, //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Volatile)
    0x85, 0x10, //   Report ID (16)
    0x09, 0x04, //   Usage (0x04)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x3F, //   Report Count (63)
    0x91,
    0x83, //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Volatile)
    0x85, 0x80, //   Report ID (-128)
    0x09, 0x05, //   Usage (0x05)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x3F, //   Report Count (63)
    0x91,
    0x83, //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Volatile)
    0x85, 0x82, //   Report ID (-126)
    0x09, 0x06, //   Usage (0x06)
    0x75, 0x08, //   Report Size (8)
    0x95, 0x3F, //   Report Count (63)
    0x91,
    0x83, //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Volatile)
    0xC0, // End Collection
];

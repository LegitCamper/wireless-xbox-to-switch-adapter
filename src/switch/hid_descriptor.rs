use serde::ser::{Serialize, SerializeTuple, Serializer};
use usbd_hid::descriptor::gen_hid_descriptor;
use usbd_hid::descriptor::generator_prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum Button {
    A,
    B,
    X,
    Y,
    L,
    R,
    ZL,
    ZR,
    Plus,
    Minus,
    LeftStick,
    RightStick,
    Home,
    Capture,
}

impl Button {
    /// Returns the bit mask for the corresponding button.
    pub fn as_value(self) -> (u8, u16) {
        match self {
            Button::A => (0, 1 << 0),
            Button::B => (0, 1 << 1),
            Button::X => (0, 1 << 2),
            Button::Y => (0, 1 << 3),
            Button::L => (0, 1 << 4),
            Button::R => (0, 1 << 5),
            Button::ZL => (0, 1 << 6),
            Button::ZR => (0, 1 << 7),
            Button::Plus => (0, 1 << 8),
            Button::Minus => (0, 1 << 9),
            Button::LeftStick => (1, 1 << 0),
            Button::RightStick => (1, 1 << 1),
            Button::Home => (1, 1 << 2),
            Button::Capture => (1, 1 << 3),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Dpad {
    Neutral = 0x08, // No direction pressed
    Up = 0x00,
    UpRight = 0x01,
    Right = 0x02,
    DownRight = 0x03,
    Down = 0x04,
    DownLeft = 0x05,
    Left = 0x06,
    UpLeft = 0x07,
}

pub const JOYSTICK_CENTER: u16 = 32767;

// source: <https://gist.github.com/ToadKing/b883a8ccfa26adcc6ba9905e75aeb4f2>
#[gen_hid_descriptor(
    (usage_page = GENERIC_DESKTOP, logical_min = 0, usage = JOYSTICK, collection = APPLICATION) = {
        (usage_page = GENERIC_DESKTOP, usage_page = BUTTON, usage_min = 0x01, usage_max = 0x0A, logical_min = 0, logical_max = 1, unit_exponent = 0) = { // report_id = 48 
            #[data, variable, absolute, no_wrap, linear, preferred, non_null] buttons=input;
        };
        (usage_page = BUTTON, usage_min = 0x0B, usage_max = 0x0E, logical_min = 0, logical_max = 1) = {
            #[data, variable, absolute, no_wrap, linear, preferred, non_null] special_buttons=input;
        };
        (usage = 0x010030, logical_min = 0, logical_max = 65534 ) = {
            #[item_settings data, variable] left_x=input;
        };
        (usage = 0x010031, logical_min = 0, logical_max = 65534) = {
            #[item_settings data, variable] left_y=input;
        };
        (usage = 0x010032, logical_min = 0, logical_max = 65534) = {
            #[item_settings data, variable] right_y=input;
        };
        (usage = 0x010035, logical_min = 0, logical_max = 65534) = {
            #[item_settings data, variable] right_y=input;
        };
        (usage = 0x010039, logical_min = 0, logical_max = 7) = {
            #[item_settings data, variable] d_pad=input;
        };
    }
)]
pub struct SwitchProControllerReport {
    pub buttons: u16,        // First 10 buttons
    pub special_buttons: u8, // Next 4 special buttons
    pub left_x: u16,         // Left joystick X
    pub left_y: u16,         // Left joystick Y
    pub right_x: u16,        // Right joystick X
    pub right_y: u16,        // Right joystick Y
    pub d_pad: u8,           // Hat (D-Pad)
}

impl SwitchProControllerReport {
    /// Creates a new, empty controller report.
    pub fn new() -> Self {
        Self {
            buttons: 0,
            special_buttons: 0,
            left_x: JOYSTICK_CENTER, // Center position for 16-bit joystick values
            left_y: JOYSTICK_CENTER,
            right_x: JOYSTICK_CENTER,
            right_y: JOYSTICK_CENTER,
            d_pad: Dpad::Neutral as u8, // Neutral position for hat switch (8 means no direction pressed)
        }
    }

    /// Press a named button.
    pub fn press_button(&mut self, button: Button) {
        let (group, mask) = button.as_value();
        if group == 0 {
            self.buttons |= mask;
        } else {
            self.special_buttons |= mask as u8;
        }
    }

    /// Release a named button.
    pub fn release_button(&mut self, button: Button) {
        let (group, mask) = button.as_value();
        if group == 0 {
            self.buttons &= !mask;
        } else {
            self.special_buttons &= !(mask as u8);
        }
    }

    /// Set the hat switch to a named direction.
    pub fn set_dpad(&mut self, position: Dpad) {
        self.d_pad = position as u8;
    }

    /// Set the joystick positions (left stick: x, y; right stick: z, rz).
    pub fn set_joystick(&mut self, lx: u16, ly: u16, rx: u16, ry: u16) {
        self.left_x = lx;
        self.left_y = ly;
        self.right_x = rx;
        self.right_y = ry;
    }

    /// Convenience method to reset all buttons and set joysticks to neutral position.
    pub fn reset(&mut self) {
        self.buttons = 0;
        self.special_buttons = 0;
        self.left_x = JOYSTICK_CENTER;
        self.left_y = JOYSTICK_CENTER;
        self.right_x = JOYSTICK_CENTER;
        self.right_y = JOYSTICK_CENTER;
        self.d_pad = Dpad::Neutral as u8;
    }
}

pub static HID_DESCRIPTOR: [u8; 203] = [
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

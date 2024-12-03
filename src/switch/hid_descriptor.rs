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

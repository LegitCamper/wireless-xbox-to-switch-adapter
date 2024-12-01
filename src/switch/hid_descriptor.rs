use serde::ser::{Serialize, SerializeTuple, Serializer};
use usbd_hid::descriptor::gen_hid_descriptor;
use usbd_hid::descriptor::generator_prelude::*;

// source: <https://gist.github.com/ToadKing/b883a8ccfa26adcc6ba9905e75aeb4f2>
#[gen_hid_descriptor(
   (usage_page = GENERIC_DESKTOP, logical_min = 1, usage = JOYSTICK, collection = APPLICATION) = {
      (usage_page = GENERIC_DESKTOP, usage_page = BUTTON, report_id = 0x30, usage_min = 0x01, usage_max = 0x0A, logical_min = 0, logical_max = 1, unit_exponent = 0) = {
        #[data, variable, absolute, no_wrap, linear, preferred, not_null] ten_buttons=input;
      };
      (usage_page = BUTTON, usage_min = 0x0B, usage_max = 0x0E, logical_min = 0, logical_max = 1) = {
        #[data, variable, absolute, no_wrap, linear, preferred, not_null] four_buttons=input;
      };

      // () = {
      // #[data, variable, absolute, no_wrap, linear, preferred, not_null] four_buttons=input;
      // }
      // there is another one here?

      (usage = 0x010001, collection = PHYSICAL, usage = 0x010030, usage = 0x010031, usage = 0x010032, usage = 0x010035, logical_min = 0, logical_max = 65534) = {
        #[data, variable, absolute, no_wrap, linear, preferred, not_null] four_buttons=input;
      };
    }
)]
pub struct ProControllerReport {
    pub ten_buttons: u8,
    pub four_buttons: u8,
    pub x: i8,
    pub y: i8,
}

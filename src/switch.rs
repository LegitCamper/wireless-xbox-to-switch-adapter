use super::CONTROLLER_STATE;
use defmt::*;
use joycon_sys::input::*;
use joycon_sys::mcu::*;
use joycon_sys::output::*;
use joycon_sys::U16LE;

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

pub async fn handle_request(request: OutputReportEnum) -> Option<InputReport> {
    let report = match request {
        OutputReportEnum::RumbleAndSubcmd(subcommand_request) => {
            if let Ok(cmd) = SubcommandRequestEnum::try_from(subcommand_request) {
                let reply = match cmd {
                    SubcommandRequestEnum::GetOnlyControllerState(_) => {
                        Some(SubcommandReplyEnum::GetOnlyControllerState(()))
                    }
                    SubcommandRequestEnum::BluetoothManualPairing(_) => {
                        Some(SubcommandReplyEnum::BluetoothManualPairing(()))
                    }
                    SubcommandRequestEnum::RequestDeviceInfo(_) => {
                        Some(SubcommandReplyEnum::RequestDeviceInfo(device_info()))
                    }
                    SubcommandRequestEnum::SetInputReportMode(raw_id) => {
                        Some(SubcommandReplyEnum::SetInputReportMode(()))
                    }
                    SubcommandRequestEnum::GetTriggerButtonsElapsedTime(_) => Some(
                        SubcommandReplyEnum::GetTriggerButtonsElapsedTime([U16LE::default(); 7]),
                    ),
                    SubcommandRequestEnum::SetShipmentMode(raw_id) => {
                        Some(SubcommandReplyEnum::SetShipmentMode(()))
                    }
                    SubcommandRequestEnum::SPIRead(spiread_request) => {
                        // SubcommandReplyEnum::SPIRead()
                        None
                    }
                    SubcommandRequestEnum::SPIWrite(spiwrite_request) => None,
                    SubcommandRequestEnum::SetMCUConf(mcucommand) => {
                        Some(SubcommandReplyEnum::SetMCUConf(MCUReport::new()))
                    }
                    SubcommandRequestEnum::SetMCUState(raw_id) => {
                        Some(SubcommandReplyEnum::SetMCUState(()))
                    }
                    SubcommandRequestEnum::SetUnknownData(_) => {
                        Some(SubcommandReplyEnum::SetUnknownData(()))
                    }
                    SubcommandRequestEnum::SetPlayerLights(player_lights) => {
                        Some(SubcommandReplyEnum::SetPlayerLights(()))
                    }
                    SubcommandRequestEnum::SetHomeLight(home_light) => {
                        Some(SubcommandReplyEnum::SetHomeLight(()))
                    }
                    SubcommandRequestEnum::SetIMUMode(raw_id) => {
                        Some(SubcommandReplyEnum::SetIMUMode(()))
                    }
                    SubcommandRequestEnum::SetIMUSens(sensitivity) => {
                        Some(SubcommandReplyEnum::SetIMUSens(()))
                    }
                    SubcommandRequestEnum::EnableVibration(raw_id) => {
                        Some(SubcommandReplyEnum::EnableVibration(()))
                    }
                    SubcommandRequestEnum::MaybeAccessory(accessory_command) => None,
                    SubcommandRequestEnum::Unknown0x59(_) => None,
                    SubcommandRequestEnum::Unknown0x5a(_) => None,
                    SubcommandRequestEnum::Unknown0x5b(_) => None,
                    SubcommandRequestEnum::Unknown0x5c(_) => None,
                };
                if let Some(reply) = reply {
                    Some(InputReportEnum::StandardAndSubcmd((
                        CONTROLLER_STATE.get().await.lock().await.standard(),
                        reply.into(),
                    )))
                } else {
                    None
                }
            } else {
                warn!("could not read subcommand request");
                None
            }
        }
        OutputReportEnum::MCUFwUpdate(_) => None,
        OutputReportEnum::RumbleOnly(_) => None,
        OutputReportEnum::RequestMCUData(mcurequest) => {
            if let Ok(request) = MCURequestEnum::try_from(mcurequest) {
                match request {
                    MCURequestEnum::GetMCUStatus(_) => None,
                    MCURequestEnum::GetNFCData(_) => None,
                    MCURequestEnum::GetIRData(_irrequest) => None,
                }
            } else {
                warn!("Failed to read mcu report");
                None
            }
        }
    };

    if let Some(report) = report {
        Some(InputReport::from(report))
    } else {
        None
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

const VENDOR_ID: u16 = 0x057E;
const PRODUCT_ID: u16 = 0x2009;
const PACKET_SIZE: u16 = 64;
const CALIBRATION_OFFSET: u16 = 0x603D;
const CALIBRATION_LENGTH: u16 = 0x12;
const COMMAND_RETRIES: u16 = 10;
// const RUMBLE_NEUTRAL: u16 = (0x00, 0x01, 0x40, 0x40);
// const RUMBLE: u16 = (0x74, 0xBE, 0xBD, 0x6F);
// const DEFAULT_IMU_SENSITIVITY: u16 = (0x03, 0x00, 0x00, 0x01);

mod output_report_id {
    const RUMBLE_SUBCOMMAND: u16 = 0x01;
    const RUMBLE: u16 = 0x10;
    const COMMAND: u16 = 0x80;
}

mod input_report_id {
    const SUBCOMMAND_REPLY: u16 = 0x21;
    const CONTROLLER_STATE: u16 = 0x30;
    const COMMAND_ACK: u16 = 0x81;
}

mod command_id {
    const HANDSHAKE: u16 = 0x02;
    const HIGH_SPEED: u16 = 0x03;
    const FORCE_USB: u16 = 0x04;
}

mod subcommand_id {
    const SET_INPUT_REPORT_MODE: u16 = 0x03;
    const SPI_FLASH_READ: u16 = 0x10;
    const SET_PLAYER_LIGHTS: u16 = 0x30;
    const SET_HOME_LIGHT: u16 = 0x38;
    const ENABLE_IMU: u16 = 0x40;
    const SET_IMU_SENSITIVITY: u16 = 0x41;
    const ENABLE_VIBRATION: u16 = 0x48;
}

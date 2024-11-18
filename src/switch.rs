use joycon_sys::input::*;

pub fn get_test_report() -> InputReport {
    let mut input = NormalInputReport::default();
    input.buttons = [1, 1];
    input.stick = 1;
    InputReportEnum::Normal(input).into()
}

use super::constants::*;
use super::sysex::append_14bit;
use super::types::PinMode;

pub fn query_firmware() -> Vec<u8> {
    vec![START_SYSEX, REPORT_FIRMWARE, END_SYSEX]
}

pub fn query_capabilities() -> Vec<u8> {
    vec![START_SYSEX, CAPABILITY_QUERY, END_SYSEX]
}

pub fn query_analog_mapping() -> Vec<u8> {
    vec![START_SYSEX, ANALOG_MAPPING_QUERY, END_SYSEX]
}

pub fn pin_state_query(pin: u8) -> Vec<u8> {
    vec![START_SYSEX, PIN_STATE_QUERY, pin, END_SYSEX]
}

pub fn sampling_interval(ms: u16) -> Vec<u8> {
    let mut buf = vec![START_SYSEX, SAMPLING_INTERVAL];
    append_14bit(&mut buf, ms);
    buf.push(END_SYSEX);
    buf
}

pub fn i2c_config(delay_us: u16) -> Vec<u8> {
    let mut buf = vec![START_SYSEX, I2C_CONFIG];
    append_14bit(&mut buf, delay_us);
    buf.push(END_SYSEX);
    buf
}

pub fn i2c_read(address: u8, size: u16) -> Vec<u8> {
    let mut buf = vec![START_SYSEX, I2C_REQUEST, address, I2C_READ << 3];
    append_14bit(&mut buf, size);
    buf.push(END_SYSEX);
    buf
}

pub fn i2c_write(address: u8, data: &[u8]) -> Vec<u8> {
    let mut buf = vec![START_SYSEX, I2C_REQUEST, address, I2C_WRITE << 3];
    super::sysex::append_7bit_data(&mut buf, data);
    buf.push(END_SYSEX);
    buf
}

pub fn report_digital(port: u8, enabled: bool) -> Vec<u8> {
    vec![REPORT_DIGITAL | port, u8::from(enabled)]
}

pub fn report_analog(analog_pin: u8, enabled: bool) -> Vec<u8> {
    vec![REPORT_ANALOG | analog_pin, u8::from(enabled)]
}

pub fn set_pin_mode(pin: u8, mode: PinMode) -> Vec<u8> {
    vec![SET_PIN_MODE, pin, mode.as_u8()]
}

pub fn analog_write(pin: u8, level: u16) -> Vec<u8> {
    let mut buf = vec![ANALOG_MESSAGE | pin];
    append_14bit(&mut buf, level);
    buf
}

pub fn digital_port_write(port: u8, mask: u16) -> Vec<u8> {
    let mut buf = vec![DIGITAL_MESSAGE | port];
    append_14bit(&mut buf, mask);
    buf
}

pub fn set_digital_pin_value(pin: u8, value: u8) -> Vec<u8> {
    vec![SET_DIGITAL_PIN_VALUE, pin, value]
}

pub fn extended_analog_write(pin: u8, level: u16) -> Vec<u8> {
    let mut buf = vec![START_SYSEX, EXTENDED_ANALOG, pin];
    append_14bit(&mut buf, level);
    buf.push(END_SYSEX);
    buf
}

pub fn servo_config(pin: u8, min_pulse: u16, max_pulse: u16) -> Vec<u8> {
    let mut buf = vec![START_SYSEX, SERVO_CONFIG, pin];
    append_14bit(&mut buf, min_pulse);
    append_14bit(&mut buf, max_pulse);
    buf.push(END_SYSEX);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::PinMode;

    #[test]
    fn set_pin_mode_output() {
        assert_eq!(set_pin_mode(13, PinMode::Output), vec![0xF4, 13, 1]);
    }

    #[test]
    fn report_digital_port_0() {
        assert_eq!(report_digital(0, true), vec![0xD0, 1]);
    }

    #[test]
    fn report_analog_a0() {
        assert_eq!(report_analog(0, true), vec![0xC0, 1]);
    }

    #[test]
    fn set_digital_pin_value_high() {
        assert_eq!(set_digital_pin_value(13, 1), vec![0xF5, 13, 1]);
    }

    #[test]
    fn query_firmware_bytes() {
        assert_eq!(
            query_firmware(),
            vec![START_SYSEX, REPORT_FIRMWARE, END_SYSEX]
        );
    }

    #[test]
    fn extended_analog_write_bytes() {
        let data = super::extended_analog_write(3, 128);
        assert_eq!(data[0], START_SYSEX);
        assert_eq!(data[1], EXTENDED_ANALOG);
        assert_eq!(data[2], 3);
    }

    #[test]
    fn servo_config_bytes() {
        let data = super::servo_config(3, 600, 2400);
        assert_eq!(data[0], START_SYSEX);
        assert_eq!(data[1], SERVO_CONFIG);
    }

    #[test]
    fn sampling_interval_bytes() {
        let data = super::sampling_interval(10);
        assert_eq!(data[0], START_SYSEX);
        assert_eq!(data[1], SAMPLING_INTERVAL);
    }

    #[test]
    fn pin_state_query_bytes() {
        assert_eq!(
            super::pin_state_query(13),
            vec![START_SYSEX, PIN_STATE_QUERY, 13, END_SYSEX]
        );
    }
}

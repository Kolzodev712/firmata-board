use crate::pin::{AnalogPin, DigitalPin};

pub trait BoardProfile {
    const NAME: &'static str;
    const PIN_COUNT: u8;
    const ANALOG_PIN_COUNT: u8;
    const DIGITAL_PORT_COUNT: u8;
    const DEFAULT_BAUD: u32;

    fn digital_pin(pin: u8) -> Option<DigitalPin>;
    fn analog_pin(index: u8) -> Option<AnalogPin>;
    fn supports_pwm(pin: DigitalPin) -> bool;
    fn analog_to_firmata_pin(analog: AnalogPin) -> u8 {
        analog.0 + crate::protocol::ANALOG_PIN_OFFSET
    }
}

pub mod uno;

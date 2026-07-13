use crate::board::profiles::BoardProfile;
use crate::pin::{AnalogPin, DigitalPin};

pub struct Uno;

impl BoardProfile for Uno {
    const NAME: &'static str = "Arduino Uno";
    const PIN_COUNT: u8 = 20;
    const ANALOG_PIN_COUNT: u8 = 6;
    const DIGITAL_PORT_COUNT: u8 = 2;
    const DEFAULT_BAUD: u32 = 57_600;

    fn digital_pin(pin: u8) -> Option<DigitalPin> {
        if pin <= 13 {
            Some(DigitalPin(pin))
        } else {
            None
        }
    }

    fn analog_pin(index: u8) -> Option<AnalogPin> {
        if index < Self::ANALOG_PIN_COUNT {
            Some(AnalogPin(index))
        } else {
            None
        }
    }

    fn supports_pwm(pin: DigitalPin) -> bool {
        matches!(pin.0, 3 | 5 | 6 | 9 | 10 | 11)
    }
}

pub fn d(pin: u8) -> DigitalPin {
    Uno::digital_pin(pin).expect("valid Uno digital pin")
}

pub fn a(pin: u8) -> AnalogPin {
    Uno::analog_pin(pin).expect("valid Uno analog pin")
}

#[cfg(feature = "serial")]
mod serial_board {
    use crate::board::profiles::BoardProfile;
    use crate::board::Board;
    use crate::error::Result;
    use crate::pin::{I2cHandle, PinHandle, PinRef};
    use crate::transport::serial::SerialTransport;

    use super::Uno;

    pub struct ArduinoUno {
        board: Board<SerialTransport>,
    }

    impl ArduinoUno {
        pub fn open(path: &str) -> Result<Self> {
            let transport = SerialTransport::open(path, Uno::DEFAULT_BAUD)?;
            let board = Board::connect(transport)?;
            Ok(Self { board })
        }

        pub fn open_with_retry(path: &str) -> Result<Self> {
            let transport = SerialTransport::open(path, Uno::DEFAULT_BAUD)?;
            let board = Board::connect_with_retry(transport)?;
            Ok(Self { board })
        }

        pub fn pin<'a>(
            &'a mut self,
            pin: impl Into<PinRef>,
        ) -> PinHandle<'a, SerialTransport, Uno> {
            PinHandle::new(&mut self.board, pin.into())
        }

        pub fn i2c<'a>(&'a mut self) -> I2cHandle<'a, SerialTransport> {
            I2cHandle::new(&mut self.board)
        }

        pub fn board(&self) -> &Board<SerialTransport> {
            &self.board
        }

        pub fn board_mut(&mut self) -> &mut Board<SerialTransport> {
            &mut self.board
        }
    }
}

#[cfg(feature = "serial")]
pub use serial_board::ArduinoUno;

//! Multi-board Firmata host adapter for Arduino and compatible boards.
//!
//! See the [architecture guide](docs/architecture.md) for design details.

pub mod board;
pub mod error;
pub mod pin;
pub mod protocol;
pub mod transport;

#[cfg(feature = "serial")]
pub use board::profiles::uno::ArduinoUno;
pub use board::profiles::uno::{a, d, Uno};
pub use board::Board;
pub use error::{Error, Result};
pub use pin::{AnalogPin, DigitalPin, I2cHandle, PinHandle, PinRef};
pub use protocol::{encode, Decoder, I2cReply, Message, PinMode, PinState};
pub use protocol::{
    ANALOG_PIN_OFFSET, PIN_MODE_ANALOG, PIN_MODE_INPUT, PIN_MODE_OUTPUT, PIN_MODE_PULLUP,
    PIN_MODE_PWM, PIN_MODE_SERVO,
};
pub use transport::{FlushTransport, MockTransport, Transport};

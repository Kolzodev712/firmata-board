use snafu::prelude::*;

use crate::board::handshake::{perform_handshake, DEFAULT_HANDSHAKE_TIMEOUT};
use crate::board::retry::retry_with_backoff;
use crate::board::state::BoardState;
use crate::error::{Result, StdIoSnafu};
use crate::protocol::decode::Decoder;
use crate::protocol::types::PinMode;
use crate::protocol::{encode, Message};
use crate::transport::Transport;

pub mod handshake;
pub mod profiles;
pub mod retry;
pub mod state;

#[derive(Debug)]
pub struct Board<T: Transport> {
    transport: T,
    decoder: Decoder,
    state: BoardState,
}

impl<T: Transport> Board<T> {
    pub fn from_transport(transport: T) -> Self {
        Self {
            transport,
            decoder: Decoder::new(),
            state: BoardState::new(),
        }
    }

    pub fn connect(transport: T) -> Result<Self> {
        let mut board = Self::from_transport(transport);
        perform_handshake(&mut board, DEFAULT_HANDSHAKE_TIMEOUT)?;
        Ok(board)
    }

    pub fn connect_with_retry(transport: T) -> Result<Self> {
        let mut board = Self::from_transport(transport);
        retry_with_backoff(|| perform_handshake(&mut board, DEFAULT_HANDSHAKE_TIMEOUT))?;
        Ok(board)
    }

    pub fn state(&self) -> &BoardState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut BoardState {
        &mut self.state
    }

    #[tracing::instrument(skip(self, data), err, level = "DEBUG")]
    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        self.transport.write_all(data).context(StdIoSnafu)?;
        self.transport.flush_transport().context(StdIoSnafu)?;
        Ok(())
    }

    #[tracing::instrument(skip(self), err, level = "DEBUG")]
    pub fn read_message(&mut self) -> Result<Message> {
        let mut byte = [0u8; 1];
        loop {
            self.transport.read_exact(&mut byte).context(StdIoSnafu)?;
            if let Some(message) = self.decoder.feed(byte[0])? {
                self.state.apply(message.clone())?;
                return Ok(message);
            }
        }
    }

    pub fn set_pin_mode(&mut self, pin: u8, mode: PinMode) -> Result<()> {
        self.state.validate_pin(pin)?;
        self.send(&encode::set_pin_mode(pin, mode))?;
        let pin_state = &mut self.state.pins[pin as usize];
        pin_state.mode = mode;
        pin_state.modes = vec![mode];
        Ok(())
    }

    pub fn digital_write(&mut self, pin: u8, level: u8) -> Result<()> {
        self.state.validate_pin(pin)?;
        self.state.pins[pin as usize].value = i32::from(level);
        self.send(&encode::set_digital_pin_value(pin, level))
    }

    pub fn digital_port_write(&mut self, pin: u8, level: u8) -> Result<()> {
        self.state.validate_pin(pin)?;
        let port = pin / 8;
        let mut mask = 0u16;
        for i in 0..8u8 {
            let p = (port * 8) + i;
            if self
                .state
                .pins
                .get(p as usize)
                .is_some_and(|s| s.value != 0)
            {
                mask |= 1 << i;
            }
        }
        let bit = pin % 8;
        if level != 0 {
            mask |= 1 << bit;
        } else {
            mask &= !(1 << bit);
        }
        self.state.pins[pin as usize].value = i32::from(level);
        self.send(&encode::digital_port_write(port, mask))
    }

    pub fn analog_write(&mut self, pin: u8, level: u16) -> Result<()> {
        self.state.validate_pin(pin)?;
        self.state.pins[pin as usize].value = i32::from(level);
        if pin < 16 {
            self.send(&encode::analog_write(pin, level))
        } else {
            self.send(&encode::extended_analog_write(pin, level))
        }
    }

    pub fn report_digital(&mut self, port: u8, enabled: bool) -> Result<()> {
        self.send(&encode::report_digital(port, enabled))
    }

    pub fn report_analog(&mut self, analog_pin: u8, enabled: bool) -> Result<()> {
        self.send(&encode::report_analog(analog_pin, enabled))
    }

    pub fn i2c_config(&mut self, delay_us: u16) -> Result<()> {
        self.send(&encode::i2c_config(delay_us))
    }

    pub fn i2c_read(&mut self, address: u8, size: u16) -> Result<()> {
        self.send(&encode::i2c_read(address, size))
    }

    pub fn i2c_write(&mut self, address: u8, data: &[u8]) -> Result<()> {
        self.send(&encode::i2c_write(address, data))
    }

    pub fn sampling_interval(&mut self, ms: u16) -> Result<()> {
        self.send(&encode::sampling_interval(ms))
    }

    pub fn servo_config(&mut self, pin: u8, min_pulse: u16, max_pulse: u16) -> Result<()> {
        self.state.validate_pin(pin)?;
        self.send(&encode::servo_config(pin, min_pulse, max_pulse))
    }

    pub fn pin_state_query(&mut self, pin: u8) -> Result<()> {
        self.state.validate_pin(pin)?;
        self.send(&encode::pin_state_query(pin))
    }

    pub fn take_i2c_reply(&mut self) -> Option<crate::protocol::I2cReply> {
        if self.state.i2c_replies.is_empty() {
            None
        } else {
            Some(self.state.i2c_replies.remove(0))
        }
    }

    pub fn firmware_name(&self) -> &str {
        &self.state.firmware_name
    }

    pub fn firmware_version(&self) -> &str {
        &self.state.firmware_version
    }

    pub fn protocol_version(&self) -> &str {
        &self.state.protocol_version
    }
}

impl<T: Transport> std::fmt::Display for Board<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Board {{ firmware={}, version={}, protocol={} }}",
            self.state.firmware_name, self.state.firmware_version, self.state.protocol_version
        )
    }
}

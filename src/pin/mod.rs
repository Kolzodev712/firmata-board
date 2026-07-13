use crate::board::profiles::BoardProfile;
use crate::board::Board;
use crate::error::Result;
use crate::protocol::types::PinMode;
use crate::transport::Transport;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinRef {
    Digital(DigitalPin),
    Analog(AnalogPin),
    Raw(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DigitalPin(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnalogPin(pub u8);

impl From<DigitalPin> for PinRef {
    fn from(value: DigitalPin) -> Self {
        Self::Digital(value)
    }
}

impl From<AnalogPin> for PinRef {
    fn from(value: AnalogPin) -> Self {
        Self::Analog(value)
    }
}

impl From<u8> for PinRef {
    fn from(value: u8) -> Self {
        Self::Raw(value)
    }
}

pub struct PinHandle<'a, T: Transport, P: BoardProfile> {
    board: &'a mut Board<T>,
    firmata_pin: u8,
    analog_index: Option<u8>,
    _profile: std::marker::PhantomData<P>,
}

impl<'a, T: Transport, P: BoardProfile> PinHandle<'a, T, P> {
    pub fn new(board: &'a mut Board<T>, pin: PinRef) -> Self {
        let (firmata_pin, analog_index) = match pin {
            PinRef::Digital(d) => (d.0, None),
            PinRef::Analog(a) => (P::analog_to_firmata_pin(a), Some(a.0)),
            PinRef::Raw(p) => (p, None),
        };
        Self {
            board,
            firmata_pin,
            analog_index,
            _profile: std::marker::PhantomData,
        }
    }

    pub fn firmata_pin(&self) -> u8 {
        self.firmata_pin
    }

    pub fn mode(&mut self, mode: PinMode) -> Result<&mut Self> {
        self.board.set_pin_mode(self.firmata_pin, mode)?;
        Ok(self)
    }

    pub fn input(&mut self) -> Result<&mut Self> {
        self.mode(PinMode::Input)
    }

    pub fn output(&mut self) -> Result<&mut Self> {
        self.mode(PinMode::Output)
    }

    pub fn pwm(&mut self) -> Result<&mut Self> {
        self.mode(PinMode::Pwm)
    }

    pub fn servo(&mut self) -> Result<&mut Self> {
        self.mode(PinMode::Servo)
    }

    pub fn analog_input(&mut self) -> Result<&mut Self> {
        self.mode(PinMode::Analog)
    }

    pub fn pullup(&mut self) -> Result<&mut Self> {
        self.mode(PinMode::Pullup)
    }

    pub fn write_digital(&mut self, high: bool) -> Result<()> {
        self.board.digital_write(self.firmata_pin, u8::from(high))
    }

    pub fn write_analog(&mut self, level: u16) -> Result<()> {
        self.board.analog_write(self.firmata_pin, level)
    }

    pub fn enable_reporting(&mut self) -> Result<()> {
        if let Some(analog_index) = self.analog_index {
            self.board.report_analog(analog_index, true)
        } else {
            let port = self.firmata_pin / 8;
            self.board.report_digital(port, true)
        }
    }

    pub fn disable_reporting(&mut self) -> Result<()> {
        if let Some(analog_index) = self.analog_index {
            self.board.report_analog(analog_index, false)
        } else {
            let port = self.firmata_pin / 8;
            self.board.report_digital(port, false)
        }
    }

    pub fn value(&self) -> i32 {
        self.board
            .state()
            .pins
            .get(self.firmata_pin as usize)
            .map(|p| p.value)
            .unwrap_or(0)
    }
}

pub struct I2cHandle<'a, T: Transport> {
    board: &'a mut Board<T>,
}

impl<'a, T: Transport> I2cHandle<'a, T> {
    pub fn new(board: &'a mut Board<T>) -> Self {
        Self { board }
    }

    pub fn config(&mut self, delay_us: u16) -> Result<()> {
        self.board.i2c_config(delay_us)
    }

    pub fn write(&mut self, address: u8, data: &[u8]) -> Result<()> {
        self.board.i2c_write(address, data)
    }

    pub fn read(&mut self, address: u8, size: u16) -> Result<crate::protocol::I2cReply> {
        self.board.i2c_read(address, size)?;
        self.wait_for_reply()
    }

    pub fn wait_for_reply(&mut self) -> Result<crate::protocol::I2cReply> {
        loop {
            if let Some(reply) = self.board.take_i2c_reply() {
                return Ok(reply);
            }
            self.board.read_message()?;
        }
    }
}

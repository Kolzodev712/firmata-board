use super::constants::DEFAULT_ANALOG_RESOLUTION;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PinMode {
    Input = 0,
    Output = 1,
    Analog = 2,
    Pwm = 3,
    Servo = 4,
    Shift = 5,
    I2c = 6,
    OneWire = 7,
    Stepper = 8,
    Encoder = 9,
    Serial = 10,
    Pullup = 11,
    Ignore = 0x7F,
}

impl PinMode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Input),
            1 => Some(Self::Output),
            2 => Some(Self::Analog),
            3 => Some(Self::Pwm),
            4 => Some(Self::Servo),
            5 => Some(Self::Shift),
            6 => Some(Self::I2c),
            7 => Some(Self::OneWire),
            8 => Some(Self::Stepper),
            9 => Some(Self::Encoder),
            10 => Some(Self::Serial),
            11 => Some(Self::Pullup),
            0x7F => Some(Self::Ignore),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinCapabilities {
    pub modes: Vec<PinMode>,
    pub resolution: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinState {
    pub mode: PinMode,
    pub resolution: u8,
    pub modes: Vec<PinMode>,
    pub value: i32,
}

impl Default for PinState {
    fn default() -> Self {
        Self {
            mode: PinMode::Analog,
            modes: vec![PinMode::Analog],
            resolution: DEFAULT_ANALOG_RESOLUTION,
            value: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct I2cReply {
    pub address: u16,
    pub register: u16,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    ProtocolVersion { major: u8, minor: u8 },
    Analog { pin: u8, value: u16 },
    Digital { port: u8, mask: u16 },
    EmptyResponse,
    AnalogMappingResponse { mapping: Vec<u8> },
    CapabilityResponse { pins: Vec<PinCapabilities> },
    PinStateResponse { pin: u8, mode: PinMode, value: i32 },
    ReportFirmware { major: u8, minor: u8, name: String },
    I2cReply(I2cReply),
}

impl Message {
    pub fn is_handshake_firmware(&self) -> bool {
        matches!(
            self,
            Message::ReportFirmware { .. } | Message::ProtocolVersion { .. }
        )
    }

    pub fn is_handshake_capabilities(&self) -> bool {
        matches!(
            self,
            Message::CapabilityResponse { .. } | Message::AnalogMappingResponse { .. }
        )
    }
}

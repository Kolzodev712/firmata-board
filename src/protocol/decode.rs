use snafu::prelude::*;

use crate::error::{Error, MessageTooShortSnafu, Result, Utf8Snafu};

use super::constants::*;
use super::sysex::decode_14bit;
use super::types::{I2cReply, Message, PinCapabilities, PinMode};

#[derive(Debug, Default)]
enum DecodeState {
    #[default]
    Idle,
    TwoOfThree {
        first: u8,
        second: u8,
    },
    OneOfThree(u8),
    Sysex(Vec<u8>),
}

#[derive(Debug, Default)]
pub struct Decoder {
    state: DecodeState,
}

impl Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, byte: u8) -> Result<Option<Message>> {
        let state = std::mem::replace(&mut self.state, DecodeState::Idle);
        match state {
            DecodeState::Idle => match byte {
                START_SYSEX => {
                    self.state = DecodeState::Sysex(vec![START_SYSEX]);
                    Ok(None)
                }
                REPORT_VERSION
                | ANALOG_MESSAGE..=ANALOG_MESSAGE_BOUND
                | DIGITAL_MESSAGE..=DIGITAL_MESSAGE_BOUND => {
                    self.state = DecodeState::OneOfThree(byte);
                    Ok(None)
                }
                _ => Err(Error::BadByte { byte }),
            },
            DecodeState::OneOfThree(first) => {
                self.state = DecodeState::TwoOfThree {
                    first,
                    second: byte,
                };
                Ok(None)
            }
            DecodeState::TwoOfThree { first, second } => {
                let msg = parse_three_byte(&[first, second, byte])?;
                Ok(Some(msg))
            }
            DecodeState::Sysex(mut buf) => {
                buf.push(byte);
                if byte == END_SYSEX {
                    let msg = parse_sysex(&buf)?;
                    Ok(Some(msg))
                } else {
                    self.state = DecodeState::Sysex(buf);
                    Ok(None)
                }
            }
        }
    }
}

fn parse_three_byte(buf: &[u8]) -> Result<Message> {
    match buf[0] {
        REPORT_VERSION => Ok(Message::ProtocolVersion {
            major: buf[1],
            minor: buf[2],
        }),
        cmd if (ANALOG_MESSAGE..=ANALOG_MESSAGE_BOUND).contains(&cmd) => {
            let analog_index = cmd & 0x0F;
            let pin = analog_index + ANALOG_PIN_OFFSET;
            let value = decode_14bit(buf[1], buf[2]);
            Ok(Message::Analog { pin, value })
        }
        cmd if (DIGITAL_MESSAGE..=DIGITAL_MESSAGE_BOUND).contains(&cmd) => {
            let port = cmd & 0x0F;
            let mask = decode_14bit(buf[1], buf[2]);
            Ok(Message::Digital { port, mask })
        }
        byte => Err(Error::BadByte { byte }),
    }
}

fn parse_sysex(buf: &[u8]) -> Result<Message> {
    if buf.len() < 2 {
        return Err(Error::MessageTooShort);
    }

    match buf[1] {
        END_SYSEX => Ok(Message::EmptyResponse),
        ANALOG_MAPPING_RESPONSE => parse_analog_mapping(buf),
        CAPABILITY_RESPONSE => parse_capability_response(buf),
        REPORT_FIRMWARE => parse_report_firmware(buf),
        I2C_REPLY => parse_i2c_reply(buf),
        PIN_STATE_RESPONSE => parse_pin_state_response(buf),
        code => Err(Error::UnknownSysEx { code }),
    }
}

fn parse_analog_mapping(buf: &[u8]) -> Result<Message> {
    let mapping = buf[2..buf.len() - 1].to_vec();
    Ok(Message::AnalogMappingResponse { mapping })
}

fn parse_capability_response(buf: &[u8]) -> Result<Message> {
    let mut pins = Vec::new();
    let mut i = 2;
    let mut modes = Vec::new();
    let mut resolution = None;

    while i < buf.len() - 1 {
        if buf[i] == PIN_MODE_IGNORE {
            let first_mode = modes
                .first()
                .copied()
                .and_then(PinMode::from_u8)
                .ok_or(Error::MessageTooShort)?;
            let res = resolution.ok_or(Error::MessageTooShort)?;
            let pin_modes: Vec<PinMode> =
                modes.iter().filter_map(|&m| PinMode::from_u8(m)).collect();
            pins.push(PinCapabilities {
                modes: if pin_modes.is_empty() {
                    vec![first_mode]
                } else {
                    pin_modes
                },
                resolution: res,
            });
            modes.clear();
            resolution = None;
            i += 1;
        } else {
            modes.push(buf[i]);
            if resolution.is_none() {
                resolution = Some(buf[i + 1]);
            }
            i += 2;
        }
    }

    Ok(Message::CapabilityResponse { pins })
}

fn parse_report_firmware(buf: &[u8]) -> Result<Message> {
    let major = *buf.get(2).context(MessageTooShortSnafu)?;
    let minor = *buf.get(3).context(MessageTooShortSnafu)?;
    let name = if buf.len() > 5 {
        std::str::from_utf8(&buf[4..buf.len() - 1])
            .context(Utf8Snafu)?
            .to_string()
    } else {
        String::new()
    };
    Ok(Message::ReportFirmware { major, minor, name })
}

fn parse_i2c_reply(buf: &[u8]) -> Result<Message> {
    if buf.len() < 8 {
        return Err(Error::MessageTooShort);
    }
    let mut reply = I2cReply {
        address: decode_14bit(buf[2], buf[3]),
        register: decode_14bit(buf[4], buf[5]),
        data: vec![decode_14bit(buf[6], buf[7]) as u8],
    };
    let mut i = 8;
    while i + 1 < buf.len() - 1 {
        if buf[i] == END_SYSEX {
            break;
        }
        reply.data.push(decode_14bit(buf[i], buf[i + 1]) as u8);
        i += 2;
    }
    Ok(Message::I2cReply(reply))
}

fn parse_pin_state_response(buf: &[u8]) -> Result<Message> {
    if buf.len() < 4 {
        return Err(Error::MessageTooShort);
    }
    let pin = buf[2];
    if buf[3] == END_SYSEX {
        return Ok(Message::PinStateResponse {
            pin,
            mode: PinMode::Input,
            value: 0,
        });
    }
    let mode = PinMode::from_u8(buf[3]).ok_or(Error::BadByte { byte: buf[3] })?;
    let value = i32::from(buf.get(4).copied().unwrap_or(0));
    Ok(Message::PinStateResponse { pin, mode, value })
}

pub fn decode_all(data: &[u8]) -> Result<Vec<Message>> {
    let mut decoder = Decoder::new();
    let mut messages = Vec::new();
    for &byte in data {
        if let Some(msg) = decoder.feed(byte)? {
            messages.push(msg);
        }
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::encode;

    #[test]
    fn decode_protocol_version() {
        let msgs = decode_all(&[REPORT_VERSION, 2, 5]).unwrap();
        assert_eq!(msgs, vec![Message::ProtocolVersion { major: 2, minor: 5 }]);
    }

    #[test]
    fn decode_analog_message() {
        let data = encode::analog_write(0, 512);
        let msgs = decode_all(&data).unwrap();
        assert_eq!(msgs.len(), 1);
        match &msgs[0] {
            Message::Analog { pin, value } => {
                assert_eq!(*pin, 14);
                assert_eq!(*value, 512);
            }
            _ => panic!("expected analog"),
        }
    }

    #[test]
    fn decode_digital_message() {
        let data = encode::digital_port_write(0, 0b1010);
        let msgs = decode_all(&data).unwrap();
        match &msgs[0] {
            Message::Digital { port, mask } => {
                assert_eq!(*port, 0);
                assert_eq!(*mask, 0b1010);
            }
            _ => panic!("expected digital"),
        }
    }
}

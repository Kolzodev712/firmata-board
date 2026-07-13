use crate::error::{Error, Result};
use crate::protocol::{Message, PinCapabilities, PinMode, PinState};

#[derive(Debug, Default)]
pub struct BoardState {
    pub pins: Vec<PinState>,
    pub i2c_replies: Vec<crate::protocol::I2cReply>,
    pub protocol_version: String,
    pub firmware_name: String,
    pub firmware_version: String,
}

impl BoardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pin_count(&self) -> usize {
        self.pins.len()
    }

    pub fn validate_pin(&self, pin: u8) -> Result<()> {
        if pin as usize >= self.pins.len() {
            return Err(Error::PinOutOfBounds {
                pin,
                max: self.pins.len().saturating_sub(1) as u8,
            });
        }
        Ok(())
    }

    pub fn supports_mode(&self, pin: u8, mode: PinMode) -> Result<bool> {
        self.validate_pin(pin)?;
        Ok(self.pins[pin as usize].modes.contains(&mode))
    }

    pub fn apply(&mut self, message: Message) -> Result<()> {
        match message {
            Message::ProtocolVersion { major, minor } => {
                self.protocol_version = format!("{major}.{minor}");
            }
            Message::ReportFirmware { major, minor, name } => {
                self.firmware_version = format!("{major}.{minor}");
                self.firmware_name = name;
            }
            Message::CapabilityResponse { pins } => {
                self.pins = vec![PinState::default()];
                for cap in pins {
                    self.pins.push(capabilities_to_pin_state(cap));
                }
            }
            Message::AnalogMappingResponse { mapping } => {
                for (pin_idx, &value) in mapping.iter().enumerate() {
                    if value != crate::protocol::PIN_MODE_IGNORE {
                        if let Some(state) = self.pins.get_mut(pin_idx) {
                            state.mode = PinMode::Analog;
                            state.modes = vec![PinMode::Analog];
                            state.resolution = crate::protocol::DEFAULT_ANALOG_RESOLUTION;
                        }
                    }
                }
            }
            Message::Analog { pin, value } => {
                if let Some(state) = self.pins.get_mut(pin as usize) {
                    state.value = i32::from(value);
                }
            }
            Message::Digital { port, mask } => {
                for i in 0..8u8 {
                    let pin = (port * 8) + i;
                    if let Some(state) = self.pins.get_mut(pin as usize) {
                        if state.mode == PinMode::Input {
                            state.value = i32::from((mask >> i) & 1);
                        }
                    }
                }
            }
            Message::PinStateResponse { pin, mode, value } => {
                if let Some(state) = self.pins.get_mut(pin as usize) {
                    state.mode = mode;
                    state.modes = vec![mode];
                    state.value = value;
                }
            }
            Message::I2cReply(reply) => {
                self.i2c_replies.push(reply);
            }
            Message::EmptyResponse => {}
        }
        Ok(())
    }

    pub fn port_count(&self) -> u8 {
        if self.pins.is_empty() {
            return 0;
        }
        ((self.pins.len() - 1) as u8).div_ceil(8)
    }
}

fn capabilities_to_pin_state(cap: PinCapabilities) -> PinState {
    let mode = cap.modes.first().copied().unwrap_or(PinMode::Input);
    PinState {
        mode,
        modes: cap.modes,
        resolution: cap.resolution,
        value: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_capability_response() {
        let mut state = BoardState::new();
        state
            .apply(Message::CapabilityResponse {
                pins: vec![
                    PinCapabilities {
                        modes: vec![PinMode::Input, PinMode::Output],
                        resolution: 1,
                    },
                    PinCapabilities {
                        modes: vec![PinMode::Input, PinMode::Output, PinMode::Pwm],
                        resolution: 10,
                    },
                ],
            })
            .unwrap();
        assert_eq!(state.pins.len(), 3);
        assert_eq!(state.pins[1].modes.len(), 2);
    }

    #[test]
    fn pin_out_of_bounds() {
        let state = BoardState::new();
        assert!(state.validate_pin(0).is_err());
    }
}

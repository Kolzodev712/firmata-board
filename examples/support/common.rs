use firmata_board::ArduinoUno;
use std::env;

pub fn default_port() -> String {
    env::var("FIRMATA_PORT").unwrap_or_else(|_| "/dev/ttyACM0".into())
}

pub fn open_uno() -> firmata_board::Result<ArduinoUno> {
    ArduinoUno::open(&default_port())
}

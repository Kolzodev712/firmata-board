#[path = "support/common.rs"]
mod common;

use firmata_board::d;

fn main() {
    tracing_subscriber::fmt::init();

    let mut uno = common::open_uno().expect("open Arduino Uno");

    let pin = d(3);

    uno.pin(pin).pwm().expect("pwm mode");
    uno.pin(pin).write_analog(0).expect("initial write");

    tracing::info!("Starting loop...");

    loop {
        for value in (0..255).step_by(5) {
            uno.pin(pin).write_analog(value).expect("pwm write");
            tracing::info!("{}", value);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}

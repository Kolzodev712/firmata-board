#[path = "support/common.rs"]
mod common;

use firmata_board::d;

fn main() {
    tracing_subscriber::fmt::init();

    let mut uno = common::open_uno().expect("open Arduino Uno");

    let pin = d(3);

    uno.pin(pin).servo().expect("servo mode");

    tracing::info!("Starting loop...");

    loop {
        for value in 0..180 {
            uno.pin(pin).write_analog(value).expect("servo write");
            tracing::info!("{}", value);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

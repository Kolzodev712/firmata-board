#[path = "support/common.rs"]
mod common;

use firmata_board::a;

fn main() {
    tracing_subscriber::fmt::init();

    let mut uno = common::open_uno().expect("open Arduino Uno");

    uno.pin(a(0))
        .analog_input()
        .expect("set analog input")
        .enable_reporting()
        .expect("enable reporting");

    loop {
        uno.board_mut().read_message().expect("read message");
        tracing::info!("analog value: {}", uno.pin(a(0)).value());
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[path = "support/common.rs"]
mod common;

use firmata_board::d;

fn main() {
    tracing_subscriber::fmt::init();

    let mut uno = common::open_uno().expect("open Arduino Uno");

    let led = d(13);
    let button = d(2);

    uno.pin(led).output().expect("led output");
    uno.pin(button).input().expect("button input");
    uno.pin(button)
        .enable_reporting()
        .expect("button reporting");

    tracing::info!("Starting loop...");

    loop {
        uno.board_mut().read_message().expect("read message");
        if uno.pin(button).value() == 0 {
            tracing::info!("off");
            uno.pin(led).write_digital(false).expect("write");
        } else {
            tracing::info!("on");
            uno.pin(led).write_digital(true).expect("write");
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

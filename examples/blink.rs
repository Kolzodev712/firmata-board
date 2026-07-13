#[path = "support/common.rs"]
mod common;

use firmata_board::d;

fn main() {
    tracing_subscriber::fmt::init();

    let mut uno = common::open_uno().expect("open Arduino Uno");

    uno.pin(d(13))
        .output()
        .expect("set output")
        .write_digital(true)
        .expect("write");

    let mut level = true;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(400));
        level = !level;
        uno.pin(d(13)).write_digital(level).expect("digital write");
    }
}

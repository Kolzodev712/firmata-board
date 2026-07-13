#[path = "support/common.rs"]
mod common;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn init(uno: &Arc<Mutex<firmata_board::ArduinoUno>>) {
    let mut u = uno.lock().expect("lock");
    u.i2c().config(0).expect("i2c delay");
    u.i2c().write(0x09, b"o").expect("i2c write");
    thread::sleep(Duration::from_millis(10));
}

fn set_rgb(uno: &Arc<Mutex<firmata_board::ArduinoUno>>, rgb: [u8; 3]) {
    let mut u = uno.lock().expect("lock");
    u.i2c().write(0x09, b"n").expect("i2c write");
    u.i2c().write(0x09, &rgb).expect("i2c write");
}

fn read_rgb(uno: &Arc<Mutex<firmata_board::ArduinoUno>>) -> Vec<u8> {
    {
        let mut u = uno.lock().expect("lock");
        u.i2c().write(0x09, b"g").expect("i2c write");
        u.i2c().read(0x09, 3).expect("i2c read");
    }
    loop {
        let mut u = uno.lock().expect("lock");
        if let Ok(reply) = u.i2c().wait_for_reply() {
            return reply.data;
        }
        drop(u);
        thread::sleep(Duration::from_millis(10));
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let uno = Arc::new(Mutex::new(common::open_uno().expect("open Arduino Uno")));

    {
        let reader = uno.clone();
        thread::spawn(move || loop {
            if let Ok(mut u) = reader.lock() {
                let _ = u.board_mut().read_message();
            }
            thread::sleep(Duration::from_millis(10));
        });
    }

    init(&uno);

    set_rgb(&uno, [255, 0, 0]);
    tracing::info!("rgb: {:?}", read_rgb(&uno));
    thread::sleep(Duration::from_millis(1000));

    set_rgb(&uno, [0, 255, 0]);
    tracing::info!("rgb: {:?}", read_rgb(&uno));
    thread::sleep(Duration::from_millis(1000));

    set_rgb(&uno, [0, 0, 255]);
    tracing::info!("rgb: {:?}", read_rgb(&uno));
    thread::sleep(Duration::from_millis(1000));
}

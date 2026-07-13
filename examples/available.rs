fn main() {
    tracing_subscriber::fmt::init();

    for port in serialport::available_ports().expect("list ports") {
        tracing::info!("{}: {:?}", port.port_name, port.port_type);
    }
}

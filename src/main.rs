use qrcode::QrCode;

fn main() {
    let code = QrCode::new(b"444").unwrap();
    let string = code.render::<char>()
        .quiet_zone(false)
        .module_dimensions(1, 1)
        .build();
    println!("{}", string);
}

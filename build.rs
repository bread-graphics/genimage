// BSL 1.0 License

/// Help us know whether or not we can use certain features.
fn main() {
    let c = autocfg::new();
    c.emit_rustc_version(1, 57);
}

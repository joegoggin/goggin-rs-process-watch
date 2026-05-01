fn main() {
    if let Err(error) = goggin_rs_console::run() {
        error.exit();
    }
}

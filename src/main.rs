fn main() {
    if let Err(error) = goggin_rs_process_watch::run() {
        error.exit();
    }
}

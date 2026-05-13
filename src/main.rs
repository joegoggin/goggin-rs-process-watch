fn main() {
    if let Err(error) = goggin_rs_process_watch::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

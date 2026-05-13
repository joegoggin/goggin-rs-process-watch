//! Binary entry point for process watch.
//!
//! This binary delegates all application behavior to the library crate and
//! handles final error reporting and process exit status.

/// Runs the process-watch binary and maps application errors to exit status 1.
fn main() {
    if let Err(error) = goggin_rs_process_watch::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

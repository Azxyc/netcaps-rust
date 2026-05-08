#[cfg(target_os = "macos")]
mod app;
#[cfg(target_os = "macos")]
mod caps;
#[cfg(target_os = "macos")]
mod cli;
#[cfg(target_os = "macos")]
mod ffi;
#[cfg(target_os = "macos")]
mod hid;
#[cfg(target_os = "macos")]
mod network;

#[cfg(target_os = "macos")]
fn main() {
    if let Err(error) = app::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("netcaps is only supported on macOS.");
    std::process::exit(1);
}

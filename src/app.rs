use std::thread;
use std::time::Duration;

use crate::caps::BlinkInterval;
use crate::cli::{CliCommand, CliConfig};
use crate::hid::CapsLockLedManager;
use crate::network::NetworkMonitor;

const MAX_CHECKS_BEFORE_SLOWDOWN: u32 = 7_500;

pub fn run() -> Result<(), String> {
    match crate::cli::parse(std::env::args()) {
        CliCommand::Run(config) => run_monitor(config),
        CliCommand::Version => {
            print_version();
            Ok(())
        }
        CliCommand::Help { message, exit_code } => {
            print_help(message.as_deref());
            std::process::exit(exit_code);
        }
    }
}

fn run_monitor(config: CliConfig) -> Result<(), String> {
    if config.silent {
        lower_process_priority();
    }

    let mut led_manager = CapsLockLedManager::new(config.silent)?;
    let mut network = NetworkMonitor::new(config.interface_filter);
    let mut blink_interval = BlinkInterval::default();
    let mut previous_bytes = network.byte_counts();
    let mut checks_without_activity = 0;

    loop {
        blink_interval.refresh();
        thread::sleep(blink_interval.current());

        let current_bytes = network.byte_counts();
        if current_bytes.has_activity_since(previous_bytes) {
            if !config.silent {
                println!("RX: {}, TX: {}", current_bytes.rx, current_bytes.tx);
            }
            led_manager.blink(1, blink_interval.current());
            checks_without_activity = 0;
        } else {
            checks_without_activity += 1;
            if checks_without_activity >= MAX_CHECKS_BEFORE_SLOWDOWN {
                thread::sleep(Duration::from_millis(50));
            }
        }

        previous_bytes = current_bytes;
    }
}

fn lower_process_priority() {
    unsafe {
        let _ = libc::setpriority(libc::PRIO_PROCESS, 0, 15);
    }
}

fn print_version() {
    println!("netcaps version {}", env!("CARGO_PKG_VERSION"));
    println!("    Made by Taj C (forcequit)");
    println!("    Check this out on GitHub, at https://github.com/forcequitOS/netcaps");
}

fn print_help(message: Option<&str>) {
    println!();
    println!("Usage:");
    println!("    netcaps [arguments]");
    println!();
    println!("Arguments:");
    println!("    --silent, -s         - silences command-line output");
    println!("    --local-only, -L     - only listen on loX");
    println!("    --en-only, -E        - only listen on enX");
    println!("    --peers-only, -P     - only listen on awdlX and llwX");
    println!("    --utun-only, -U      - only listen on utunX");
    println!("    --local-included, -l - listen on loX in addition to others");
    println!("    --utun-included, -u  - listen on utunX in addition to others");
    println!("    --version, -v        - displays current version of netcaps");
    println!("    --help, -h           - shows this help menu");
    println!();

    if let Some(message) = message {
        println!("Error: {message}");
        println!();
    }
}

use crate::network::{InterfaceFilter, InterfaceKind};

pub enum CliCommand {
    Run(CliConfig),
    Version,
    Help {
        message: Option<String>,
        exit_code: i32,
    },
}

pub struct CliConfig {
    pub silent: bool,
    pub interface_filter: InterfaceFilter,
}

#[derive(Default)]
struct Flags {
    silent: bool,
    help: bool,
    version: bool,
    local_only: bool,
    en_only: bool,
    peers_only: bool,
    utun_only: bool,
    local_included: bool,
    utun_included: bool,
}

pub fn parse(args: impl IntoIterator<Item = String>) -> CliCommand {
    let args = args.into_iter().collect::<Vec<_>>();
    let mut flags = Flags::default();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--silent" | "-s" => flags.silent = true,
            "--help" | "-h" => flags.help = true,
            "--version" | "-v" => flags.version = true,
            "--local-only" | "-L" => flags.local_only = true,
            "--en-only" | "-E" => flags.en_only = true,
            "--peers-only" | "-P" => flags.peers_only = true,
            "--utun-only" | "-U" => flags.utun_only = true,
            "--local-included" | "-l" => flags.local_included = true,
            "--utun-included" | "-u" => flags.utun_included = true,
            _ => {}
        }
    }

    if (flags.help || flags.version) && args.len() > 2 {
        return CliCommand::Help {
            message: Some("--help and --version cannot be combined with other flags.".to_owned()),
            exit_code: 1,
        };
    }

    if flags.version {
        return CliCommand::Version;
    }

    let only_flag_count = [
        flags.local_only,
        flags.en_only,
        flags.peers_only,
        flags.utun_only,
    ]
    .into_iter()
    .filter(|flag| *flag)
    .count();

    if only_flag_count > 1 {
        return CliCommand::Help {
            message: Some("Cannot use multiple 'only' flags together.".to_owned()),
            exit_code: 0,
        };
    }

    if only_flag_count > 0 && (flags.local_included || flags.utun_included) {
        return CliCommand::Help {
            message: Some("Cannot combine 'included' flags with 'only' flags.".to_owned()),
            exit_code: 0,
        };
    }

    if flags.help {
        return CliCommand::Help {
            message: None,
            exit_code: 0,
        };
    }

    CliCommand::Run(CliConfig {
        silent: flags.silent,
        interface_filter: interface_filter(flags),
    })
}

fn interface_filter(flags: Flags) -> InterfaceFilter {
    if flags.local_only {
        return InterfaceFilter::only([InterfaceKind::Loopback]);
    }

    if flags.en_only {
        return InterfaceFilter::only([InterfaceKind::Ethernet]);
    }

    if flags.peers_only {
        return InterfaceFilter::only([InterfaceKind::Awdl, InterfaceKind::Llw]);
    }

    if flags.utun_only {
        return InterfaceFilter::only([InterfaceKind::Utun]);
    }

    let mut filter = InterfaceFilter::default();
    if flags.local_included {
        filter.insert(InterfaceKind::Loopback);
    }
    if flags.utun_included {
        filter.insert(InterfaceKind::Utun);
    }
    filter
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> CliCommand {
        parse(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn parses_version() {
        assert!(matches!(
            parse_args(&["netcaps", "--version"]),
            CliCommand::Version
        ));
    }

    #[test]
    fn rejects_help_combined_with_other_flags() {
        match parse_args(&["netcaps", "--help", "--silent"]) {
            CliCommand::Help {
                message: Some(message),
                exit_code: 1,
            } => assert!(message.contains("cannot be combined")),
            _ => panic!("expected a failing help command"),
        }
    }

    #[test]
    fn rejects_multiple_only_flags() {
        match parse_args(&["netcaps", "--local-only", "--en-only"]) {
            CliCommand::Help {
                message: Some(message),
                exit_code: 0,
            } => assert!(message.contains("multiple 'only' flags")),
            _ => panic!("expected a help command for conflicting only flags"),
        }
    }

    #[test]
    fn allows_both_included_flags() {
        match parse_args(&["netcaps", "--local-included", "--utun-included"]) {
            CliCommand::Run(config) => {
                assert!(config.interface_filter.allows_name("lo0"));
                assert!(config.interface_filter.allows_name("utun4"));
                assert!(config.interface_filter.allows_name("en0"));
            }
            _ => panic!("expected a run command"),
        }
    }
}

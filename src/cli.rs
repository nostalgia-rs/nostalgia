use clap::{Arg, ArgMatches, Command};
use std::ffi::OsString;

/// Parses the supplied process arguments using Nostalgia's command-line definition.
pub fn get_cli_arguments<'a, I, T>(args: I) -> ArgMatches
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone + 'a,
{
    let command: Command = build_command();
    command.get_matches_from(args)
}

#[must_use]
/// Builds the Clap command used by the executable.
pub fn build_command() -> Command {
    Command::new("nostalgia")
        .next_line_help(true)
        .hide_possible_values(true)
        .about("CLI to help bring some nostalgia to your day.")
        .help_expected(true)
        .max_term_width(80)
        .arg(
            Arg::new("get-config-location")
                // .short('c')
                .long("get-config-location")
                .required(false)
                .num_args(0)
                .help("Prints the location of the .nostalgia folder used to store images, database, and configuration file.")
        ).arg(
            Arg::new("print-config")
                // .short('p')
                .long("print-config")
                .required(false)
                .num_args(0)
                .help("Prints the current configuration.")
        )
}

#[cfg(test)]
mod tests {
    use super::get_cli_arguments;

    #[test]
    fn parses_both_supported_flags() {
        let matches = get_cli_arguments(["nostalgia", "--get-config-location", "--print-config"]);

        assert!(matches.get_flag("get-config-location"));
        assert!(matches.get_flag("print-config"));
    }

    #[test]
    fn leaves_flags_disabled_by_default() {
        let matches = get_cli_arguments(["nostalgia"]);

        assert!(!matches.get_flag("get-config-location"));
        assert!(!matches.get_flag("print-config"));
    }
}

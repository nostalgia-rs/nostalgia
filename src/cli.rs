use clap::{Arg, ArgAction, ArgMatches, Command};
use std::ffi::OsString;

/// Parses the supplied process arguments using Nostalgia's command-line definition.
///
/// The first item in `args` should be the executable name, following Clap's
/// standard argument convention.
pub fn get_cli_arguments<'a, I, T>(args: I) -> ArgMatches
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone + 'a,
{
    let command: Command = build_command();
    command.get_matches_from(args)
}

#[must_use]
/// Builds the Clap command and help text used by the executable.
///
/// Defines verbosity, configuration, database validation, and asset download
/// options.
pub fn build_command() -> Command {
    Command::new("nostalgia")
        .version(env!("CARGO_PKG_VERSION"))
        .next_line_help(true)
        .hide_possible_values(true)
        .about("CLI to help bring some nostalgia to your day.")
        .help_expected(true)
        .max_term_width(80)
        .after_help(
            "Data: Put CSV files under ~/.nostalgia/ with columns \
             category,subcategory,index,name,dark_threshold,background_color,foreground_color,type,second_type. \
             Match each row with ~/.nostalgia/<category>/<subcategory>/<index>.jpg. \
             See the README for details.",
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .action(ArgAction::Count)
                .help("Increase logging verbosity (-v for info, -vv for debug)."),
        )
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
        ).arg(
            Arg::new("validate-database")
                .long("validate-database")
                .required(false)
                .num_args(0)
                .help("Loads and displays all rows from the database CSV files.")
        ).arg(
            Arg::new("download-assets")
                .long("download-assets")
                .required(false)
                .num_args(0)
                .help("Downloads and extracts the latest image assets.")
        )
}

#[cfg(test)]
mod tests {
    use super::get_cli_arguments;

    #[test]
    fn parses_both_supported_flags() {
        let matches = get_cli_arguments([
            "nostalgia",
            "--get-config-location",
            "--print-config",
            "--validate-database",
            "--download-assets",
        ]);

        assert!(matches.get_flag("get-config-location"));
        assert!(matches.get_flag("print-config"));
        assert!(matches.get_flag("validate-database"));
        assert!(matches.get_flag("download-assets"));
    }

    #[test]
    fn leaves_flags_disabled_by_default() {
        let matches = get_cli_arguments(["nostalgia"]);

        assert!(!matches.get_flag("get-config-location"));
        assert!(!matches.get_flag("print-config"));
        assert!(!matches.get_flag("validate-database"));
        assert!(!matches.get_flag("download-assets"));
        assert_eq!(matches.get_count("verbose"), 0);
    }

    #[test]
    fn counts_repeated_verbosity_flags() {
        assert_eq!(
            get_cli_arguments(["nostalgia", "-v"]).get_count("verbose"),
            1
        );
        assert_eq!(
            get_cli_arguments(["nostalgia", "-vv"]).get_count("verbose"),
            2
        );
    }
}

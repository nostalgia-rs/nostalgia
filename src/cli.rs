use std::ffi::OsString;
// use clap::{Arg, ArgAction, ArgMatches, Command, ValueHint, builder::NonEmptyStringValueParser};
use clap::{Arg, ArgMatches, Command};

pub fn get_cli_arguments<'a, I, T>(args: I) -> ArgMatches
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone + 'a,
{
    let command: Command = build_command();
    command.get_matches_from(args)
}

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

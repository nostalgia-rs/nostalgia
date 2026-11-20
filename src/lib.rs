use anyhow::Result;
use std::env;
pub mod adaptors;
pub mod cli;
pub mod config;
pub mod database;
use crate::adaptors::Adaptor;
use crate::adaptors::{AdaptorConfig, get_adaptor};
use crate::config::{NostalgiaConfig, get_config};
use crate::database::get_random_row;

/// Runs the Nostalgia command.
///
/// # Errors
///
/// Returns an error if configuration or database operations fail.
pub fn run() -> Result<()> {
    let cli_arguments = cli::get_cli_arguments(env::args_os());
    let config: NostalgiaConfig = get_config()?;
    println!("Categories: {}", config.categories.join(", "));

    if cli_arguments.get_flag("get-config-location") {
        let config_location = dirs::home_dir()
            .map(|home| home.join(".nostalgia"))
            .ok_or_else(|| anyhow::anyhow!("HOME environment variable is not set"))?;
        println!("Config location: {}", config_location.display());
        return Ok(());
    }

    if cli_arguments.get_flag("print-config") {
        println!("Current configuration: {config:#?}");
        return Ok(());
    }

    let rand = get_random_row(
        &config.categories,
        &config.subcategories,
        &config.names,
        &config.types,
    )?;
    // println!("Random row: {:?}", rand);
    let category: String = rand.column("category")?.get(0)?.str_value().to_string();
    println!("Category: {category}");
    let subcategory: String = rand.column("subcategory")?.get(0)?.str_value().to_string();
    println!("Subcategory: {subcategory}");
    let index: String = rand.column("index")?.get(0)?.str_value().to_string();
    println!("Index: {index}");

    let image_location =
        format!("/Users/villanueva/.nostalgia/{category}/{subcategory}/{index}.jpg");

    let background_color: String = rand
        .column("background_color")?
        .get(0)?
        .str_value()
        .to_string();
    let foreground_color: String = rand
        .column("foreground_color")?
        .get(0)?
        .str_value()
        .to_string();
    let adaptor_config = AdaptorConfig {
        settings: &config,
        background_color: Some(&background_color),
        foreground_color: Some(&foreground_color),
    };

    for adaptor_name in &config.enabled_adaptors {
        let adaptor: Box<dyn Adaptor> = get_adaptor(adaptor_name)?;
        let result = adaptor.execute(&image_location, &adaptor_config)?;
        println!("{}: {result}", adaptor.get_name());
    }

    Ok(())
}

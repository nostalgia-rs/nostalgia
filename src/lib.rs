//! Nostalgia selects a random entry from the user's image database and applies
//! it through the configured wallpaper or terminal adaptors.
#![deny(missing_docs)]

use anyhow::Result;
use std::env;

/// Adaptors that apply an image to a wallpaper or terminal.
pub mod adaptors;
/// Command-line argument parsing.
pub mod cli;
/// User configuration loading and defaults.
pub mod config;
/// Image database filtering and random selection.
pub mod database;
use crate::adaptors::Adaptor;
use crate::adaptors::{AdaptorConfig, get_adaptor};
use crate::config::{NostalgiaConfig, get_config};
use crate::database::get_random_row;

fn config_location(home: Option<std::path::PathBuf>) -> Result<std::path::PathBuf> {
    home.map(|home| home.join(".nostalgia"))
        .ok_or_else(|| anyhow::anyhow!("HOME environment variable is not set"))
}

fn build_image_location(
    home: Option<std::path::PathBuf>,
    category: &str,
    subcategory: &str,
    index: &str,
) -> Result<String> {
    let home = home.ok_or_else(|| anyhow::anyhow!("HOME environment variable is not set"))?;
    home.join(".nostalgia")
        .join(category)
        .join(subcategory)
        .join(format!("{index}.jpg"))
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow::anyhow!("Image path is not valid UTF-8"))
}

/// Runs the Nostalgia command using the process arguments and user files.
///
/// # Errors
///
/// Returns an error if configuration or database operations fail.
pub fn run() -> Result<()> {
    let cli_arguments = cli::get_cli_arguments(env::args_os());
    let config: NostalgiaConfig = get_config()?;
    println!("Categories: {}", config.categories.join(", "));

    if cli_arguments.get_flag("get-config-location") {
        let config_location = config_location(dirs::home_dir())?;
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

    let image_location = build_image_location(dirs::home_dir(), &category, &subcategory, &index)?;

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

#[cfg(test)]
mod tests {
    use super::{build_image_location, config_location};

    #[test]
    fn builds_image_paths_from_home_and_reports_missing_or_invalid_paths() {
        let home = std::env::temp_dir();
        assert_eq!(
            build_image_location(Some(home.clone()), "pokemon", "fire", "1").unwrap(),
            home.join(".nostalgia/pokemon/fire/1.jpg").to_str().unwrap()
        );
        assert!(build_image_location(None, "pokemon", "fire", "1").is_err());

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;

            let invalid_home = std::path::PathBuf::from(std::ffi::OsString::from_vec(vec![0xff]));
            assert!(build_image_location(Some(invalid_home), "pokemon", "fire", "1").is_err());
        }
    }

    #[test]
    fn resolves_the_config_location_or_reports_an_unavailable_home() {
        let home = std::env::temp_dir();
        assert_eq!(
            config_location(Some(home.clone())).unwrap(),
            home.join(".nostalgia")
        );
        assert!(config_location(None).is_err());
    }
}

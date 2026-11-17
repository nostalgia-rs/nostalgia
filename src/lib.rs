use std::env;
use anyhow::Result;
use clap::builder::Str;
pub mod cli;
pub mod adaptors;
pub mod database;
pub mod config;
use crate::config::get_config;
use crate::adaptors::get_adaptor;
use crate::database::get_random_row;
use crate::adaptors::Adaptor;

pub fn run() -> Result<()> {
    let cli_arguments = cli::get_cli_arguments(env::args_os());
    let config = get_config()?;
    println!("Config: {:?}", config.all_categories);

    if cli_arguments.get_flag("get-config-location") {
        let config_location = dirs::home_dir()
            .map(|home| home.join(".nostalgia"))
            .ok_or_else(|| anyhow::anyhow!("HOME environment variable is not set"))?;
        println!("Config location: {:?}", config_location);
        return Ok(());
    }
    // let name = cli_arguments
    //     .get_one::<String>("name")
    //     .expect("name has a default value");

    let adaptor: Box<dyn Adaptor> = get_adaptor("darwin");

    let rand = get_random_row()?;
    // println!("Random row: {:?}", rand);
    let category: String = rand.column("category")?.get(0)?.str_value().to_string();
    println!("Category: {:?}", category);
    let subcategory: String = rand.column("subcategory")?.get(0)?.str_value().to_string();
    println!("Subcategory: {:?}", subcategory);
    let index: String = rand.column("index")?.get(0)?.str_value().to_string();
    println!("Index: {:?}", index);

    let image_location = format!(
        "/Users/villanueva/.nostalgia/{}/{}/{}.jpg",
        category,
        subcategory,
        index,
    );
    adaptor.execute(&image_location)?;
    println!("Wallpaper set to {image_location}");

    // let name: &str = *cli_arguments.get_one::<String>("name").expect("name is required");
    // println!("Hello {:?}!", name);

    // if let Some(name) = cli_arguments.get_one::<String>("name") {
    //     println!("Hello: {}", name);
    // }



    // let rand = get_random_row();
    // println!("Random row: {:?}", rand);

    Ok(())
}
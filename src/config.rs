use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct NostalgiaConfig {
    pub change_terminal_background_color: bool,
    pub change_terminal_background_transparency: u8,
    pub change_terminal_foreground_color: bool,
    pub enabled_adaptors: Vec<String>,

    pub categories: Vec<String>,
    pub subcategories: Vec<String>,
    pub names: Vec<String>,
    pub types: Vec<String>,
}

/// `NostalgiaConfig` implements `Default`
impl ::core::default::Default for NostalgiaConfig {
    fn default() -> Self {
        Self {
            enabled_adaptors: vec!["wallpaper".to_owned()],
            change_terminal_background_color: true,
            change_terminal_background_transparency: 100,
            change_terminal_foreground_color: true,

            categories: vec!["ALL".to_owned()],
            subcategories: vec!["ALL".to_owned()],
            names: vec!["ALL".to_owned()],
            types: vec!["ALL".to_owned()],
        }
    }
}

fn load_config(path: &Path) -> Result<NostalgiaConfig, confy::ConfyError> {
    confy::load_path(path)
}

/// Loads the Nostalgia configuration from the user's home directory.
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined or the configuration cannot be loaded.
pub fn get_config() -> Result<NostalgiaConfig, confy::ConfyError> {
    let home = dirs::home_dir().ok_or_else(|| {
        confy::ConfyError::BadConfigDirectory("Could not determine home directory".to_owned())
    })?;
    load_config(&home.join(".nostalgia").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::{NostalgiaConfig, load_config};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_values_from_the_nostalgia_config_file() {
        let home = std::env::temp_dir().join(format!(
            "nostalgia-config-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = home.join(".nostalgia").join("config.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            r#"
enabled_adaptors = ["wallpaper", "warp", "ghostty", "apple"]
change_terminal_background_transparency = 42
categories = ["pokemon"]
"#,
        )
        .unwrap();

        let config = load_config(&path).unwrap();

        assert_eq!(
            config.enabled_adaptors,
            ["wallpaper", "warp", "ghostty", "apple"]
        );
        assert_eq!(config.change_terminal_background_transparency, 42);
        assert_eq!(config.categories, ["pokemon"]);

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn defaults_to_the_platform_wallpaper_adaptor() {
        assert_eq!(NostalgiaConfig::default().enabled_adaptors, ["wallpaper"]);
    }
}

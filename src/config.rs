use std::path::Path;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
#[non_exhaustive]
/// User-configurable filters and application behavior.
pub struct NostalgiaConfig {
    /// Whether adaptors should change the terminal background color.
    pub change_terminal_background_color: bool,
    /// Terminal background image transparency, from 0 to 100.
    pub change_terminal_background_transparency: u8,
    /// Whether adaptors should change the terminal foreground color.
    pub change_terminal_foreground_color: bool,
    /// Names of enabled adaptors.
    pub enabled_adaptors: Vec<String>,

    /// Category filters; `"ALL"` disables filtering for this field.
    pub categories: Vec<String>,
    /// Subcategory filters; `"ALL"` disables filtering for this field.
    pub subcategories: Vec<String>,
    /// Name filters; `"ALL"` disables filtering for this field.
    pub names: Vec<String>,
    /// Type filters; `"ALL"` disables filtering for this field.
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

fn config_path(home: Option<PathBuf>) -> Result<PathBuf, confy::ConfyError> {
    let home = home.ok_or_else(|| {
        confy::ConfyError::BadConfigDirectory("Could not determine home directory".to_owned())
    })?;
    Ok(home.join(".nostalgia").join("config.toml"))
}

/// Loads the Nostalgia configuration from `~/.nostalgia/config.toml`.
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined or the configuration cannot be loaded.
pub fn get_config() -> Result<NostalgiaConfig, confy::ConfyError> {
    get_config_from_home(dirs::home_dir())
}

fn get_config_from_home(home: Option<PathBuf>) -> Result<NostalgiaConfig, confy::ConfyError> {
    load_config(&config_path(home)?)
}

#[cfg(test)]
mod tests {
    use super::{NostalgiaConfig, config_path, get_config_from_home, load_config};
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
        let config = NostalgiaConfig::default();
        assert_eq!(config.enabled_adaptors, ["wallpaper"]);
        assert!(config.change_terminal_background_color);
        assert_eq!(config.change_terminal_background_transparency, 100);
        assert!(config.change_terminal_foreground_color);
        assert_eq!(config.categories, ["ALL"]);
        assert_eq!(config.subcategories, ["ALL"]);
        assert_eq!(config.names, ["ALL"]);
        assert_eq!(config.types, ["ALL"]);
    }

    #[test]
    fn rejects_malformed_configuration_files() {
        let home = std::env::temp_dir().join(format!(
            "nostalgia-invalid-config-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&home).unwrap();
        let path = home.join("config.toml");
        fs::write(&path, "change_terminal_background_transparency = nope").unwrap();

        assert!(load_config(&path).is_err());

        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn resolves_the_config_path_and_reports_an_unavailable_home() {
        let home = std::env::temp_dir();
        assert_eq!(
            config_path(Some(home.clone())).unwrap(),
            home.join(".nostalgia/config.toml")
        );
        assert!(config_path(None).is_err());
        assert!(get_config_from_home(None).is_err());
    }
}

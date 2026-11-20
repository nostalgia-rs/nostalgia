//! Configuration keys are taken from Ghostty's official option reference:
//! https://ghostty.org/docs/config/reference (notably `background-image`,
//! `background-opacity`, `background`, and `foreground`).

use crate::adaptors::{Adaptor, AdaptorConfig};

use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub struct Ghostty {}

impl Adaptor for Ghostty {
    fn get_name(&self) -> &str {
        "ghostty"
    }

    fn execute(&self, image_location: &str, config: &AdaptorConfig<'_>) -> Result<String, Error> {
        self.update_config(image_location, config)
    }
}

impl Ghostty {
    fn update_config(
        &self,
        image_location: &str,
        config: &AdaptorConfig<'_>,
    ) -> Result<String, Error> {
        let config_paths = config_paths()?;
        let mut config_path = None;
        for path in config_paths.iter().rev() {
            match fs::metadata(path) {
                Ok(metadata) if metadata.is_file() => {
                    config_path = Some(path.clone());
                    break;
                }
                Ok(_) => {}
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        let config_path = config_path
            .or_else(|| config_paths.first().cloned())
            .ok_or_else(|| Error::other("No Ghostty config path is available"))?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = match fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error),
        };
        fs::write(
            &config_path,
            update_config(&contents, image_location, config)?,
        )?;

        Ok("success".to_owned())
    }
}

fn config_paths() -> Result<Vec<PathBuf>, Error> {
    let xdg_config_dir = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "Could not determine Ghostty config directory",
            )
        })?;

    let mut paths = vec![
        xdg_config_dir.join("ghostty/config.ghostty"),
        xdg_config_dir.join("ghostty/config"),
    ];

    #[cfg(target_os = "macos")]
    if let Some(home) = dirs::home_dir() {
        let macos_config_dir = home.join("Library/Application Support/com.mitchellh.ghostty");
        paths.push(macos_config_dir.join("config.ghostty"));
        paths.push(macos_config_dir.join("config"));
    }

    Ok(paths)
}

fn update_config(
    contents: &str,
    image_location: &str,
    config: &AdaptorConfig<'_>,
) -> Result<String, Error> {
    let transparency = config.settings.change_terminal_background_transparency;
    if transparency > 100 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Ghostty background transparency must be between 0 and 100",
        ));
    }

    let escaped_location = image_location.replace('\\', "\\\\").replace('"', "\\\"");
    let mut updated = update_config_setting(
        contents,
        "background-image",
        &format!("\"{escaped_location}\""),
    );

    updated = update_config_setting(
        &updated,
        "background-opacity",
        &format!("{:.2}", f32::from(transparency) / 100.0),
    );

    if config.settings.change_terminal_background_color {
        let background_color = config.background_color.ok_or_else(|| {
            Error::new(ErrorKind::InvalidInput, "Background color is not available")
        })?;
        updated = update_config_setting(&updated, "background", background_color);
    }

    if config.settings.change_terminal_foreground_color {
        let foreground_color = config.foreground_color.ok_or_else(|| {
            Error::new(ErrorKind::InvalidInput, "Foreground color is not available")
        })?;
        updated = update_config_setting(&updated, "foreground", foreground_color);
    }

    Ok(updated)
}

fn update_config_setting(contents: &str, key: &str, value: &str) -> String {
    let setting = format!("{key} = {value}");
    let mut updated = String::with_capacity(contents.len() + setting.len() + 1);
    let mut found_setting = false;

    for line in contents.split_inclusive('\n') {
        let line_content = line.trim_end_matches('\n').trim_end_matches('\r');
        let matches_key = line_content
            .split_once('=')
            .is_some_and(|(line_key, _)| line_key.trim() == key);

        if matches_key {
            if !found_setting {
                updated.push_str(&setting);
                updated.push_str(&line[line_content.len()..]);
                found_setting = true;
            }
        } else {
            updated.push_str(line);
        }
    }

    if !found_setting {
        if !updated.is_empty() && !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(&setting);
        updated.push('\n');
    }

    updated
}

#[cfg(test)]
mod tests {
    use super::update_config;
    use crate::adaptors::AdaptorConfig;
    use crate::config::NostalgiaConfig;

    #[test]
    fn updates_image_colors_and_background_opacity_without_changing_other_config() {
        let contents = "# terminal theme\nfont-size = 14\nbackground-image = old.png\n";
        let mut settings = NostalgiaConfig::default();
        settings.change_terminal_background_transparency = 75;
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("282c34"),
            foreground_color: Some("abb2bf"),
        };

        assert_eq!(
            update_config(contents, "/images/new image.png", &config).unwrap(),
            "# terminal theme\nfont-size = 14\nbackground-image = \"/images/new image.png\"\nbackground-opacity = 0.75\nbackground = 282c34\nforeground = abb2bf\n"
        );
    }

    #[test]
    fn replaces_duplicate_settings_and_escapes_path() {
        let contents = "background-image=old.png\nbackground-image = older.png\nbackground=ffffff\nbackground = 000000";
        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("#282c34"),
            foreground_color: Some("#abb2bf"),
        };

        assert_eq!(
            update_config(contents, r#"C:\images\new "image".png"#, &config).unwrap(),
            "background-image = \"C:\\\\images\\\\new \\\"image\\\".png\"\nbackground = #282c34\nbackground-opacity = 1.00\nforeground = #abb2bf\n"
        );
    }

    #[test]
    fn leaves_disabled_colors_unchanged() {
        let contents = "background = 282c34\n";
        let mut settings = NostalgiaConfig::default();
        settings.change_terminal_background_color = false;
        settings.change_terminal_foreground_color = false;
        let config = AdaptorConfig {
            settings: &settings,
            background_color: None,
            foreground_color: None,
        };

        assert_eq!(
            update_config(contents, "/images/new.png", &config).unwrap(),
            "background = 282c34\nbackground-image = \"/images/new.png\"\nbackground-opacity = 1.00\n"
        );
    }

    #[test]
    fn rejects_transparency_above_100() {
        let mut settings = NostalgiaConfig::default();
        settings.change_terminal_background_transparency = 101;
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("282c34"),
            foreground_color: Some("abb2bf"),
        };

        assert_eq!(
            update_config("", "/images/new.png", &config)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
    }
}

//! Configuration keys are taken from Ghostty's official option reference:
//! <https://ghostty.org/docs/config/reference> (notably `background-image`,
//! `background-opacity`, `background`, and `foreground`).

use crate::adaptors::{Adaptor, AdaptorConfig};

use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub struct Ghostty {}

impl Adaptor for Ghostty {
    fn get_name(&self) -> &'static str {
        "ghostty"
    }

    fn execute(&self, image_location: &str, config: &AdaptorConfig<'_>) -> Result<String, Error> {
        update_config_file(image_location, config)
    }
}

fn update_config_file(image_location: &str, config: &AdaptorConfig<'_>) -> Result<String, Error> {
    update_config_file_from_paths(config_paths(), image_location, config)
}

fn update_config_file_from_paths(
    config_paths: Result<Vec<PathBuf>, Error>,
    image_location: &str,
    config: &AdaptorConfig<'_>,
) -> Result<String, Error> {
    update_config_file_at_paths(&config_paths?, image_location, config)
}

fn update_config_file_at_paths(
    config_paths: &[PathBuf],
    image_location: &str,
    config: &AdaptorConfig<'_>,
) -> Result<String, Error> {
    update_config_file_at_paths_with(
        config_paths,
        image_location,
        config,
        create_config_directory,
        write_config_file,
    )
}

fn create_config_directory(path: &std::path::Path) -> Result<(), Error> {
    fs::create_dir_all(path)
}

fn write_config_file(path: &std::path::Path, contents: String) -> Result<(), Error> {
    fs::write(path, contents)
}

fn update_config_file_at_paths_with(
    config_paths: &[PathBuf],
    image_location: &str,
    config: &AdaptorConfig<'_>,
    create_directory: impl FnOnce(&std::path::Path) -> Result<(), Error>,
    write_file: impl FnOnce(&std::path::Path, String) -> Result<(), Error>,
) -> Result<String, Error> {
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
        create_directory(parent)?;
    }

    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    write_file(
        &config_path,
        update_config(&contents, image_location, config)?,
    )?;

    Ok("success".to_owned())
}

fn config_paths() -> Result<Vec<PathBuf>, Error> {
    let xdg_config_dir = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    config_paths_from(xdg_config_dir, dirs::home_dir())
}

fn config_paths_from(
    xdg_config_dir: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Result<Vec<PathBuf>, Error> {
    let xdg_config_dir = xdg_config_dir
        .or_else(|| home.clone().map(|home| home.join(".config")))
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
    if let Some(home) = home {
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
    use super::{
        config_paths_from, update_config, update_config_file_at_paths,
        update_config_file_at_paths_with, update_config_file_from_paths, write_config_file,
    };
    use crate::adaptors::AdaptorConfig;
    use crate::config::NostalgiaConfig;
    use std::path::PathBuf;

    #[test]
    fn updates_image_colors_and_background_opacity_without_changing_other_config() {
        let contents = "# terminal theme\nfont-size = 14\nbackground-image = old.png\n";
        let settings = NostalgiaConfig {
            change_terminal_background_transparency: 75,
            ..NostalgiaConfig::default()
        };
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
        let settings = NostalgiaConfig {
            change_terminal_background_color: false,
            change_terminal_foreground_color: false,
            ..NostalgiaConfig::default()
        };
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
        let settings = NostalgiaConfig {
            change_terminal_background_transparency: 101,
            ..NostalgiaConfig::default()
        };
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

    #[test]
    fn rejects_missing_enabled_colors() {
        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: None,
            foreground_color: None,
        };
        assert_eq!(
            update_config("", "/images/new.png", &config)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );

        let settings = NostalgiaConfig {
            change_terminal_background_color: false,
            change_terminal_foreground_color: true,
            ..NostalgiaConfig::default()
        };
        let config = AdaptorConfig {
            settings: &settings,
            background_color: None,
            foreground_color: None,
        };
        assert_eq!(
            update_config("", "/images/new.png", &config)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn resolves_xdg_and_home_config_locations() {
        let xdg = std::env::temp_dir().join("nostalgia-xdg");
        let home = std::env::temp_dir().join("nostalgia-home");

        assert_eq!(
            config_paths_from(Some(xdg.clone()), None).unwrap()[0],
            xdg.join("ghostty/config.ghostty")
        );
        assert_eq!(
            config_paths_from(None, Some(home.clone())).unwrap()[0],
            home.join(".config/ghostty/config.ghostty")
        );
        assert!(config_paths_from(None, None).is_err());
    }

    #[test]
    fn reports_empty_and_unreadable_configuration_paths() {
        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("112233"),
            foreground_color: Some("aabbcc"),
        };
        assert!(update_config_file_at_paths(&[], "/images/new.jpg", &config).is_err());
        assert!(
            update_config_file_at_paths(&[PathBuf::from("/")], "/images/new.jpg", &config).is_err()
        );
    }

    #[test]
    fn reports_metadata_errors_for_invalid_parent_paths() {
        use std::fs;

        let path = std::env::temp_dir().join(format!(
            "nostalgia-ghostty-file-parent-{}",
            std::process::id()
        ));
        fs::write(&path, "not a directory").unwrap();
        let config_path = path.join("ghostty/config");
        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("112233"),
            foreground_color: Some("aabbcc"),
        };

        assert!(update_config_file_at_paths(&[config_path], "/images/new.jpg", &config).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn propagates_update_and_write_errors() {
        use std::fs;

        let update_path = std::env::temp_dir().join(format!(
            "nostalgia-ghostty-update-error-{}",
            std::process::id()
        ));
        let settings = NostalgiaConfig::default();
        let missing_colors = AdaptorConfig {
            settings: &settings,
            background_color: None,
            foreground_color: None,
        };
        assert!(
            update_config_file_at_paths(
                std::slice::from_ref(&update_path),
                "/images/new.jpg",
                &missing_colors,
            )
            .is_err()
        );

        let config_path = std::env::temp_dir().join(format!(
            "nostalgia-ghostty-write-error-{}.conf",
            std::process::id()
        ));
        fs::write(&config_path, "existing = true\n").unwrap();
        let valid_colors = AdaptorConfig {
            settings: &settings,
            background_color: Some("112233"),
            foreground_color: Some("aabbcc"),
        };
        assert!(
            update_config_file_at_paths_with(
                std::slice::from_ref(&config_path),
                "/images/new.jpg",
                &valid_colors,
                |path| fs::create_dir_all(path),
                |_, _| Err(std::io::Error::other("simulated write failure")),
            )
            .is_err()
        );
        fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn propagates_configuration_directory_creation_errors() {
        let config_path = std::env::temp_dir().join(format!(
            "nostalgia-ghostty-directory-error-{}.conf",
            std::process::id()
        ));
        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("112233"),
            foreground_color: Some("aabbcc"),
        };

        assert!(
            update_config_file_at_paths_with(
                std::slice::from_ref(&config_path),
                "/images/new.jpg",
                &config,
                |_| Err(std::io::Error::other(
                    "simulated directory creation failure"
                )),
                write_config_file,
            )
            .is_err()
        );
    }

    #[test]
    fn propagates_configuration_path_resolution_errors() {
        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("112233"),
            foreground_color: Some("aabbcc"),
        };
        assert!(
            update_config_file_from_paths(
                Err(std::io::Error::other("simulated path resolution failure")),
                "/images/new.jpg",
                &config,
            )
            .is_err()
        );
    }
}

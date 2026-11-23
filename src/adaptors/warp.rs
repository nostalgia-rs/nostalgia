//! Warp theme fields follow its custom-theme YAML reference:
//! <https://docs.warp.dev/terminal/appearance/custom-themes/>
//! Warp's documented background image uses a JPEG path relative to
//! `~/.warp/themes/` and an opacity from 0 to 100. The generated theme must be
//! selected once in Warp's theme picker; Warp exposes no supported CLI setting
//! for changing the active theme.

use crate::adaptors::{Adaptor, AdaptorConfig};

use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

#[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
use std::env;

pub struct Warp {}

impl Adaptor for Warp {
    fn get_name(&self) -> &'static str {
        "warp"
    }

    fn execute(&self, image_location: &str, config: &AdaptorConfig<'_>) -> Result<String, Error> {
        execute_with_theme_dir(image_location, config, themes_dir)
    }
}

fn execute_with_theme_dir(
    image_location: &str,
    config: &AdaptorConfig<'_>,
    themes_dir: impl FnOnce() -> Result<PathBuf, Error>,
) -> Result<String, Error> {
    let transparency = config.settings.change_terminal_background_transparency;
    if transparency > 100 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Warp background image opacity must be between 0 and 100",
        ));
    }

    let image_path = Path::new(image_location);
    if !image_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg")
        })
    {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Warp background images must be JPEG files",
        ));
    }

    let background_color = config
        .settings
        .change_terminal_background_color
        .then_some(config.background_color)
        .flatten()
        .map(warp_color)
        .transpose()?;
    if config.settings.change_terminal_background_color && background_color.is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Background color is not available",
        ));
    }

    let foreground_color = config
        .settings
        .change_terminal_foreground_color
        .then_some(config.foreground_color)
        .flatten()
        .map(warp_color)
        .transpose()?;
    if config.settings.change_terminal_foreground_color && foreground_color.is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Foreground color is not available",
        ));
    }

    let themes_dir = themes_dir()?;
    write_warp_theme(
        image_path,
        &themes_dir,
        background_color.as_deref(),
        foreground_color.as_deref(),
        transparency,
    )
}

fn themes_dir() -> Result<PathBuf, Error> {
    #[cfg(target_os = "macos")]
    {
        return themes_dir_from_home(dirs::home_dir());
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = env::var_os("APPDATA")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::NotFound,
                    "Could not determine Warp data directory",
                )
            })?;
        return Ok(app_data
            .join("warp")
            .join("Warp")
            .join("data")
            .join("themes"));
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let data_dir = match env::var_os("XDG_DATA_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
        {
            Some(data_dir) => data_dir,
            None => home_dir()?.join(".local/share"),
        };
        return Ok(data_dir.join("warp-terminal/themes"));
    }

    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        all(unix, not(target_os = "macos"))
    )))]
    Err(Error::new(
        ErrorKind::Unsupported,
        "Warp theme directory is not configured for this platform",
    ))
}

#[cfg(target_os = "macos")]
fn themes_dir_from_home(home: Option<PathBuf>) -> Result<PathBuf, Error> {
    Ok(home_dir_from(home)?.join(".warp/themes"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn home_dir() -> Result<PathBuf, Error> {
    home_dir_from(dirs::home_dir())
}

#[cfg(any(test, target_os = "macos", all(unix, not(target_os = "macos"))))]
fn home_dir_from(home: Option<PathBuf>) -> Result<PathBuf, Error> {
    home.ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "Could not determine the home directory",
        )
    })
}

fn write_warp_theme(
    image_path: &Path,
    themes_dir: &Path,
    background_color: Option<&str>,
    foreground_color: Option<&str>,
    transparency: u8,
) -> Result<String, Error> {
    fs::create_dir_all(themes_dir)?;

    let background_image_name = "nostalgia.jpg";
    let background_image_path = themes_dir.join(background_image_name);
    if image_path != background_image_path {
        fs::copy(image_path, &background_image_path)?;
    }

    let theme_path = themes_dir.join("nostalgia.yaml");
    fs::write(
        &theme_path,
        theme_yaml(
            background_color,
            foreground_color,
            background_image_name,
            transparency,
        ),
    )?;

    eprintln!(
        "Created Warp theme at {}; select 'Nostalgia' in Warp's theme picker to apply it",
        theme_path.display()
    );
    Ok("Warp theme created".to_owned())
}

fn warp_color(color: &str) -> Result<String, Error> {
    let hex = color.strip_prefix('#').unwrap_or(color);
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Warp colors must be six-digit hexadecimal RGB values",
        ));
    }
    Ok(format!("#{hex}"))
}

fn theme_yaml(
    background_color: Option<&str>,
    foreground_color: Option<&str>,
    background_image_name: &str,
    opacity: u8,
) -> String {
    let mut theme = String::from("name: Nostalgia\ndetails: darker\n");
    if let Some(color) = background_color {
        theme.push_str(&format!("background: '{color}'\n"));
    }
    if let Some(color) = foreground_color {
        theme.push_str(&format!("foreground: '{color}'\n"));
        theme.push_str(&format!("accent: '{color}'\n"));
    }
    theme.push_str(&format!(
        "background_image:\n  path: {background_image_name}\n  opacity: {opacity}\n"
    ));
    theme
}

#[cfg(test)]
mod tests {
    use super::{execute_with_theme_dir, home_dir_from, theme_yaml, warp_color, write_warp_theme};

    #[test]
    fn formats_warp_theme_using_documented_background_image_fields() {
        assert_eq!(
            theme_yaml(Some("#112233"), Some("#aabbcc"), "nostalgia.jpg", 75),
            "name: Nostalgia\ndetails: darker\nbackground: '#112233'\nforeground: '#aabbcc'\naccent: '#aabbcc'\nbackground_image:\n  path: nostalgia.jpg\n  opacity: 75\n"
        );
    }

    #[test]
    fn normalizes_and_validates_warp_colors() {
        assert_eq!(warp_color("aAbBcC").unwrap(), "#aAbBcC");
        assert_eq!(
            warp_color("bad-color").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
        assert_eq!(
            warp_color("zzzzzz").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
        assert_eq!(
            warp_color("12zzzz").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn omits_optional_colors_from_the_theme() {
        assert_eq!(
            theme_yaml(None, None, "nostalgia.jpg", 0),
            "name: Nostalgia\ndetails: darker\nbackground_image:\n  path: nostalgia.jpg\n  opacity: 0\n"
        );
    }

    #[test]
    fn rejects_invalid_execution_inputs_before_touching_files() {
        use super::Warp;
        use crate::adaptors::{Adaptor, AdaptorConfig};
        use crate::config::NostalgiaConfig;

        let settings = NostalgiaConfig {
            change_terminal_background_transparency: 101,
            ..NostalgiaConfig::default()
        };
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("112233"),
            foreground_color: Some("aabbcc"),
        };
        assert_eq!(
            Warp {}.execute("image.jpg", &config).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );

        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("invalid"),
            foreground_color: Some("aabbcc"),
        };
        assert_eq!(
            Warp {}.execute("image.jpg", &config).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );

        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("112233"),
            foreground_color: Some("aabbcc"),
        };
        assert_eq!(
            Warp {}.execute("image.png", &config).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );

        let config = AdaptorConfig {
            settings: &settings,
            background_color: None,
            foreground_color: Some("aabbcc"),
        };
        assert_eq!(
            Warp {}.execute("image.jpg", &config).unwrap_err().kind(),
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
            foreground_color: Some("12zzzz"),
        };
        assert_eq!(
            Warp {}.execute("image.jpg", &config).unwrap_err().kind(),
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
            Warp {}.execute("image.jpg", &config).unwrap_err().kind(),
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
            foreground_color: Some("invalid"),
        };
        assert_eq!(
            Warp {}.execute("image.jpg", &config).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn propagates_theme_directory_lookup_errors() {
        use crate::adaptors::AdaptorConfig;
        use crate::config::NostalgiaConfig;

        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("112233"),
            foreground_color: Some("aabbcc"),
        };

        assert_eq!(
            execute_with_theme_dir("image.jpg", &config, || {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "simulated theme directory error",
                ))
            })
            .unwrap_err()
            .kind(),
            std::io::ErrorKind::NotFound
        );
    }

    #[test]
    fn writes_the_theme_without_copying_an_image_already_in_place() {
        use std::fs;

        let themes_dir =
            std::env::temp_dir().join(format!("nostalgia-warp-test-{}", std::process::id()));
        fs::create_dir_all(&themes_dir).unwrap();
        let image_path = themes_dir.join("nostalgia.jpg");
        fs::write(&image_path, b"existing image").unwrap();

        assert_eq!(
            write_warp_theme(
                &image_path,
                &themes_dir,
                Some("#112233"),
                Some("#aabbcc"),
                75
            )
            .unwrap(),
            "Warp theme created"
        );
        assert_eq!(fs::read(&image_path).unwrap(), b"existing image");
        fs::remove_dir_all(themes_dir).unwrap();
    }

    #[test]
    fn reports_theme_directory_copy_and_write_errors() {
        use std::fs;

        let root =
            std::env::temp_dir().join(format!("nostalgia-warp-errors-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("source.jpg");
        fs::write(&source, b"image").unwrap();

        let directory_path = root.join("not-a-directory");
        fs::write(&directory_path, b"file").unwrap();
        assert!(write_warp_theme(&source, &directory_path, None, None, 100).is_err());

        let themes_dir = root.join("themes");
        assert!(write_warp_theme(&root.join("missing.jpg"), &themes_dir, None, None, 100).is_err());

        fs::create_dir_all(themes_dir.join("nostalgia.yaml")).unwrap();
        assert!(write_warp_theme(&source, &themes_dir, None, None, 100).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_home_directory_errors() {
        assert!(home_dir_from(None).is_err());
        assert_eq!(
            home_dir_from(Some(std::env::temp_dir())).unwrap(),
            std::env::temp_dir()
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn resolves_warp_theme_directory_from_home() {
        use super::themes_dir_from_home;

        assert!(themes_dir_from_home(None).is_err());
        assert_eq!(
            themes_dir_from_home(Some(std::env::temp_dir())).unwrap(),
            std::env::temp_dir().join(".warp/themes")
        );
    }
}

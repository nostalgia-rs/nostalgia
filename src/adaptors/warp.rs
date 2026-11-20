//! Warp theme fields follow its custom-theme YAML reference:
//! https://docs.warp.dev/terminal/appearance/custom-themes/
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
    fn get_name(&self) -> &str {
        "warp"
    }

    fn execute(&self, image_location: &str, config: &AdaptorConfig<'_>) -> Result<String, Error> {
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
            .then(|| config.background_color)
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
            .then(|| config.foreground_color)
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
        fs::create_dir_all(&themes_dir)?;

        let background_image_name = "nostalgia.jpg";
        let background_image_path = themes_dir.join(background_image_name);
        if image_path != background_image_path {
            fs::copy(image_path, &background_image_path)?;
        }

        let theme_path = themes_dir.join("nostalgia.yaml");
        fs::write(
            &theme_path,
            theme_yaml(
                background_color.as_deref(),
                foreground_color.as_deref(),
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
}

fn themes_dir() -> Result<PathBuf, Error> {
    #[cfg(target_os = "macos")]
    {
        return Ok(home_dir()?.join(".warp/themes"));
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

    #[allow(unreachable_code)]
    Err(Error::new(
        ErrorKind::Unsupported,
        "Warp theme directory is not configured for this platform",
    ))
}

fn home_dir() -> Result<PathBuf, Error> {
    dirs::home_dir().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "Could not determine the home directory",
        )
    })
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
    use super::{theme_yaml, warp_color};

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
    }
}

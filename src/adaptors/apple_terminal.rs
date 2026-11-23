//! Apple Terminal colors are set through the `settings set` properties
//! `background color` and `normal text color` in Terminal.app's scripting
//! dictionary (`Terminal.app/Contents/Resources/Terminal.sdef`). That
//! dictionary exposes no background-image or opacity property.

use crate::adaptors::{Adaptor, AdaptorConfig};

use std::io::{Error, ErrorKind};
use std::process::Command;

pub struct AppleTerminal {}

impl Adaptor for AppleTerminal {
    fn get_name(&self) -> &'static str {
        "Apple_Terminal"
    }

    fn execute(&self, _image_location: &str, config: &AdaptorConfig<'_>) -> Result<String, Error> {
        let transparency = config.settings.change_terminal_background_transparency;
        if transparency > 100 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Apple Terminal background transparency must be between 0 and 100",
            ));
        }
        if transparency != 100 {
            eprintln!(
                "Apple Terminal does not expose background opacity through its scripting dictionary; transparency was not changed"
            );
        }
        eprintln!(
            "Apple Terminal does not expose a background-image property; only enabled profile colors will be changed"
        );

        let mut script = String::from(
            "tell application \"Terminal\"\n\
             if not (exists front window) then error \"No open Terminal window\"\n\
             set targetTab to selected tab of front window\n\
             set targetProfile to current settings of targetTab\n",
        );

        if config.settings.change_terminal_background_color {
            let background_color = config.background_color.ok_or_else(|| {
                Error::new(ErrorKind::InvalidInput, "Background color is not available")
            })?;
            let (red, green, blue) = color_components(background_color)?;
            script.push_str("set background color of targetProfile to {");
            script.push_str(&format!("{red}, {green}, {blue}"));
            script.push_str("}\n");
        }

        if config.settings.change_terminal_foreground_color {
            let foreground_color = config.foreground_color.ok_or_else(|| {
                Error::new(ErrorKind::InvalidInput, "Foreground color is not available")
            })?;
            let (red, green, blue) = color_components(foreground_color)?;
            script.push_str("set normal text color of targetProfile to {");
            script.push_str(&format!("{red}, {green}, {blue}"));
            script.push_str("}\n");
        }

        script.push_str("end tell\n");

        if config.settings.change_terminal_background_color
            || config.settings.change_terminal_foreground_color
        {
            let output = Command::new("osascript").arg("-e").arg(script).output()?;
            if !output.status.success() {
                return Err(Error::other(format!(
                    "osascript failed ({}): {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                )));
            }
        } else {
            return Ok("No Apple Terminal profile colors were enabled".to_owned());
        }

        Ok("Apple Terminal profile colors updated".to_owned())
    }
}

fn color_components(color: &str) -> Result<(u16, u16, u16), Error> {
    let hex = color.strip_prefix('#').unwrap_or(color);
    if hex.len() != 6 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Apple Terminal colors must be six-digit hexadecimal RGB values",
        ));
    }

    let channel = |start| {
        u8::from_str_radix(&hex[start..start + 2], 16)
            .map(|value| u16::from(value) * 257)
            .map_err(|_| {
                Error::new(
                    ErrorKind::InvalidInput,
                    "Apple Terminal colors must be six-digit hexadecimal RGB values",
                )
            })
    };

    Ok((channel(0)?, channel(2)?, channel(4)?))
}

#[cfg(test)]
mod tests {
    use super::{AppleTerminal, color_components};
    use crate::adaptors::{Adaptor, AdaptorConfig};
    use crate::config::NostalgiaConfig;

    #[test]
    fn converts_hex_colors_to_apple_script_rgb_components() {
        assert_eq!(
            color_components("#12aBef").unwrap(),
            (0x1212, 0xabab, 0xefef)
        );
    }

    #[test]
    fn rejects_non_hex_colors() {
        assert_eq!(
            color_components("blue").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
        assert_eq!(
            color_components("gg0000").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
        assert_eq!(
            color_components("00gg00").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
        assert_eq!(
            color_components("0000gg").unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn rejects_transparency_above_100_before_running_osascript() {
        let settings = NostalgiaConfig {
            change_terminal_background_transparency: 101,
            ..NostalgiaConfig::default()
        };
        let config = AdaptorConfig {
            settings: &settings,
            background_color: None,
            foreground_color: None,
        };
        let adaptor = AppleTerminal {};

        assert_eq!(
            adaptor.execute("", &config).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn reports_missing_enabled_profile_colors() {
        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: None,
            foreground_color: None,
        };

        assert_eq!(
            AppleTerminal {}.execute("", &config).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );

        let settings = NostalgiaConfig {
            change_terminal_background_color: false,
            ..settings
        };
        let config = AdaptorConfig {
            settings: &settings,
            background_color: None,
            foreground_color: None,
        };
        assert_eq!(
            AppleTerminal {}.execute("", &config).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn rejects_invalid_enabled_profile_colors() {
        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: Some("gg0000"),
            foreground_color: Some("aabbcc"),
        };
        assert_eq!(
            AppleTerminal {}.execute("", &config).unwrap_err().kind(),
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
            foreground_color: Some("00gg00"),
        };
        assert_eq!(
            AppleTerminal {}.execute("", &config).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn returns_without_running_osascript_when_no_profile_colors_are_enabled() {
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
            AppleTerminal {}.execute("", &config).unwrap(),
            "No Apple Terminal profile colors were enabled"
        );
    }
}

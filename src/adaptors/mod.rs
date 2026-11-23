mod apple_terminal;
mod darwin;
#[cfg(test)]
mod default;
mod ghostty;
mod warp;
mod win32;

use crate::adaptors::apple_terminal::AppleTerminal;
use crate::adaptors::darwin::Darwin;
use crate::adaptors::ghostty::Ghostty;
use crate::adaptors::warp::Warp;
use crate::adaptors::win32::Win32;
use crate::config::NostalgiaConfig;

use std::io::Error;

/// Settings and sampled colors passed to an adaptor.
pub struct AdaptorConfig<'a> {
    /// User-selected behavior and filters.
    pub settings: &'a NostalgiaConfig,
    /// Optional sampled RGB background color.
    pub background_color: Option<&'a str>,
    /// Optional sampled RGB foreground color.
    pub foreground_color: Option<&'a str>,
}

/// An implementation that applies an image or its colors to an application.
pub trait Adaptor {
    /// Returns this adaptor's stable configuration name.
    fn get_name(&self) -> &'static str;

    /// Executes this adaptor.
    ///
    /// # Errors
    ///
    /// Returns an error if the adaptor cannot apply the requested change.
    fn execute(&self, image_location: &str, config: &AdaptorConfig<'_>) -> Result<String, Error>;
}

/// Resolves a configured adaptor name to its implementation.
///
/// # Errors
///
/// Returns an error for unknown names or adaptors unsupported on the current platform.
pub fn get_adaptor(adaptor_name: &str) -> Result<Box<dyn Adaptor>, Error> {
    match adaptor_name.to_lowercase().as_str() {
        "wallpaper" => wallpaper_adaptor(),
        #[cfg(target_os = "macos")]
        "apple" | "apple_terminal" => Ok(Box::new(AppleTerminal {})),
        #[cfg(not(target_os = "macos"))]
        "apple" | "apple_terminal" => Err(Error::new(
            std::io::ErrorKind::Unsupported,
            "The apple adaptor is only supported on macOS",
        )),
        "darwin" => Ok(Box::new(Darwin {})),
        "ghostty" => Ok(Box::new(Ghostty {})),
        "warp" => Ok(Box::new(Warp {})),
        "win32" => Ok(Box::new(Win32 {})),
        _ => Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unknown adaptor: {adaptor_name}"),
        )),
    }
}

fn wallpaper_adaptor() -> Result<Box<dyn Adaptor>, Error> {
    #[cfg(target_os = "macos")]
    {
        return Ok(Box::new(Darwin {}));
    }

    #[cfg(target_os = "windows")]
    {
        return Ok(Box::new(Win32 {}));
    }

    #[allow(unreachable_code)]
    Err(Error::new(
        std::io::ErrorKind::Unsupported,
        "The wallpaper adaptor is only supported on macOS and Windows",
    ))
}

#[cfg(test)]
mod tests {
    use super::get_adaptor;

    #[test]
    fn resolves_wallpaper_to_the_current_platform() {
        #[cfg(target_os = "macos")]
        assert_eq!(get_adaptor("wallpaper").unwrap().get_name(), "darwin");

        #[cfg(target_os = "windows")]
        assert_eq!(get_adaptor("wallpaper").unwrap().get_name(), "win32");

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        assert_eq!(
            get_adaptor("wallpaper").err().unwrap().kind(),
            std::io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn resolves_named_terminal_adaptors_and_rejects_unknown_names() {
        assert_eq!(get_adaptor("warp").unwrap().get_name(), "warp");
        assert_eq!(get_adaptor("ghostty").unwrap().get_name(), "ghostty");
        assert_eq!(get_adaptor("darwin").unwrap().get_name(), "darwin");
        assert_eq!(get_adaptor("win32").unwrap().get_name(), "win32");
        #[cfg(target_os = "macos")]
        assert_eq!(get_adaptor("apple").unwrap().get_name(), "Apple_Terminal");
        #[cfg(target_os = "macos")]
        assert_eq!(
            get_adaptor("Apple_Terminal").unwrap().get_name(),
            "Apple_Terminal"
        );

        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            get_adaptor("apple").err().unwrap().kind(),
            std::io::ErrorKind::Unsupported
        );

        assert_eq!(
            get_adaptor("missing").err().unwrap().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }
}

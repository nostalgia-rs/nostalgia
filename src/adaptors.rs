mod apple_terminal;
mod darwin;
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

pub struct AdaptorConfig<'a> {
    pub settings: &'a NostalgiaConfig,
    pub background_color: Option<&'a str>,
    pub foreground_color: Option<&'a str>,
}

pub trait Adaptor {
    // Abstract methods (must be implemented)
    fn get_name(&self) -> &str;

    fn execute(&self, image_location: &str, config: &AdaptorConfig<'_>) -> Result<String, Error>;
}

pub fn get_adaptor(adaptor_name: &str) -> Result<Box<dyn Adaptor>, Error> {
    match adaptor_name {
        "wallpaper" => wallpaper_adaptor(),
        "apple" | "Apple_Terminal" => {
            if cfg!(target_os = "macos") {
                Ok(Box::new(AppleTerminal {}))
            } else {
                Err(Error::new(
                    std::io::ErrorKind::Unsupported,
                    "The apple adaptor is only supported on macOS",
                ))
            }
        }
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
        #[cfg(target_os = "macos")]
        assert_eq!(get_adaptor("apple").unwrap().get_name(), "Apple_Terminal");

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

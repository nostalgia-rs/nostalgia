use crate::adaptors::{Adaptor, AdaptorConfig};
use std::process::Command;
use std::io::Error;

pub struct Darwin {}

impl Adaptor for Darwin {

    fn get_name(&self) -> &str {
        "darwin"
    }

    fn execute(&self, image_location: &str, _config: &AdaptorConfig<'_>) -> Result<String, Error> {
        eprintln!("Setting macOS wallpaper to {image_location}");

        let osa_script = format!(r#"
            tell application "System Events"
                tell every desktop
                    set picture to "{}"
                end tell
            end tell
            "#, image_location);

        let output = Command::new("osascript")
            .arg("-e")
            .arg(osa_script)
            .arg(image_location)
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(Error::other(format!(
                "osascript failed ({}): {}",
                output.status,
                err.trim()
            )));
        }

        eprintln!("macOS wallpaper updated");

        Ok("success".to_owned())
    }
}

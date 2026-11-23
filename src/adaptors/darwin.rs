use crate::adaptors::{Adaptor, AdaptorConfig};
use std::io::Error;
use std::process::Command;

pub struct Darwin {}

impl Adaptor for Darwin {
    fn get_name(&self) -> &'static str {
        "darwin"
    }

    fn execute(&self, image_location: &str, _config: &AdaptorConfig<'_>) -> Result<String, Error> {
        tracing::info!("Setting macOS wallpaper to {image_location}");

        let osa_script = format!(
            r#"
            tell application "System Events"
                tell every desktop
                    set picture to "{image_location}"
                end tell
            end tell
            "#
        );

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

        tracing::info!("macOS wallpaper updated");

        Ok("success".to_owned())
    }
}

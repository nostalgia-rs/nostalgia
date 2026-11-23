use crate::adaptors::{Adaptor, AdaptorConfig};

use std::io::Error;

pub struct Default {}

impl Adaptor for Default {
    fn get_name(&self) -> &'static str {
        "default"
    }

    fn execute(&self, image_location: &str, _config: &AdaptorConfig<'_>) -> Result<String, Error> {
        tracing::error!("No adaptor found for image location: {image_location}");

        Err(Error::new(std::io::ErrorKind::NotFound, "No adaptor found"))
    }
}

#[cfg(test)]
mod tests {
    use super::Default;
    use crate::adaptors::{Adaptor, AdaptorConfig};
    use crate::config::NostalgiaConfig;

    #[test]
    fn reports_its_name_and_returns_not_found_when_executed() {
        let settings = NostalgiaConfig::default();
        let config = AdaptorConfig {
            settings: &settings,
            background_color: None,
            foreground_color: None,
        };
        let adaptor = Default {};

        assert_eq!(adaptor.get_name(), "default");
        assert_eq!(
            adaptor.execute("image.jpg", &config).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
    }
}

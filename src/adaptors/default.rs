use crate::adaptors::{Adaptor, AdaptorConfig};

use std::io::Error;

pub struct Default {}

impl Adaptor for Default {

    fn get_name(&self) -> &str {
        "default"
    }

    fn execute(&self, image_location: &str, _config: &AdaptorConfig<'_>) -> Result<String, Error> {
        println!("No adaptor found for image location: {}", image_location);

        Err(Error::new(std::io::ErrorKind::NotFound, "No adaptor found"))
    }
}

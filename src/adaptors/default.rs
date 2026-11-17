use crate::adaptors::Adaptor;

use std::io::Error;

pub struct Default {}

impl Adaptor for Default {

    fn get_name(&self) -> &str {
        "default"
    }

    fn execute(&self, image_location: &str) -> Result<String, Error> {
        println!("No adaptor found for image location: {}", image_location);

        Ok("success".to_string())
    }
}

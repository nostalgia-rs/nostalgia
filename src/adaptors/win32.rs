use crate::adaptors::Adaptor;

use std::io::Error;

pub struct Win32 {}

impl Adaptor for Win32 {

    fn get_name(&self) -> &str {
        "win32"
    }

    fn execute(&self, image_location: &str) -> Result<String, Error> {
        println!("Win32 executing for image location: {}", image_location);
        Ok("success".to_string())
    }
}

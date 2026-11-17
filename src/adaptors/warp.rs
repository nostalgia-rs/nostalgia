use crate::adaptors::Adaptor;

use std::io::Error;

pub struct Warp {}

impl Adaptor for Warp {

    fn get_name(&self) -> &str {
        "warp"
    }

    fn execute(&self, image_location: &str) -> Result<String, Error>{
        println!("Warp executing for image location: {}", image_location);
        Ok("success".to_string())
    }
}

mod darwin;
mod default;
mod warp;
mod win32;

use darwin::Darwin;
use default::Default;
use warp:: Warp;
use win32::Win32;

use std::io::Error;

pub trait Adaptor {
    // Abstract methods (must be implemented)
    fn get_name(&self) -> &str;

    fn execute(&self, image_location: &str) -> Result<String, Error>;
}

pub fn get_adaptor(adaptor_name: &str) -> Box<dyn Adaptor> {
    match adaptor_name {
        "darwin" => Box::new(Darwin {}),
        "warp" => Box::new(Warp {}),
        "win32" => Box::new(Win32 {}),
        &_ => Box::new(Default {}),
    }
}

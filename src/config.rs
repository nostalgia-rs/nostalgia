use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct NostalgiaConfig {
    pub all_categories: bool,
    pub all_names: bool,
    pub all_types: bool,
    pub all_second_types: bool,
    pub all_generations: bool,
}

/// `NostalgiaConfig` implements `Default`
impl ::std::default::Default for NostalgiaConfig {
    fn default() -> Self { Self {
        all_categories: true,
        all_names: true,
        all_types: true,
        all_second_types: true,
        all_generations: true }
    }
}

pub fn get_config() ->Result<NostalgiaConfig, confy::ConfyError>{
    let cfg: NostalgiaConfig = confy::load("nostalgia", None)?;
    Ok(cfg)
}
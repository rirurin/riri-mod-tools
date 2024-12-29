#![allow(dead_code, unused_imports, unused_variables)]

// See:
// https://reloaded-project.github.io/Reloaded-III/Common/Configuration/Config-Schema.html
// https://reloaded-project.github.io/Reloaded-III/Common/Configuration/Source-Generation.html

use std::{
    error::Error,
    fs,
    path::{ Path, PathBuf },
    ptr::NonNull
};
use serde::de::{ self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess };
use toml::Table;

pub struct ConfigEnum {
    choices: Vec<String>,
    default: NonNull<String>
}

pub struct ConfigInteger {
    min: u64,
    max: u64 
}

pub struct ConfigIntegerSlider {
    min: u64,
    max: u64,
    step: u64,
}

pub struct ConfigFloat {
    min: f32,
    max: f32
}

pub struct ConfigFloatSlider {
    min: f32,
    max: f32,
    step: f32
}

pub struct ConfigFile {
    filter: String,
    default: PathBuf,
    title: String,
    multiple: bool
}

pub struct ConfigFolder {
    default: PathBuf,
    title: String
}

pub enum ConfigSettingTypes {
    Boolean(bool),
    Enum(ConfigEnum),
    Integer(ConfigInteger),
    IntegerRange(ConfigIntegerSlider),
    Float(ConfigFloat),
    FloatRange(ConfigFloatSlider),
    File(ConfigFile),
    Folder(ConfigFolder),
    // Color,
    String(String),
    // StringList,
    // Url
}

pub struct ConfigSetting {
    index: usize,
    data: ConfigSettingTypes,
    name: String,
    description: String,
}

pub struct NamedConfigGroup {
    name: String,
    settings: Vec<ConfigSetting>
}

pub enum ConfigurationGroup {
    Anonymous(Vec<ConfigSetting>),
    Named(NamedConfigGroup)
}

pub struct Configuration {
    groups: Vec<ConfigurationGroup>
}

// Custom deserialization is required for config.toml: 
impl<'d> Deserialize<'d> for Configuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'d> {
        /*
        struct ConfigurationVisitor;
        impl<'d> Visitor<'d> for ConfigurationVisitor {
            type Value = 
        }
        */
        // deserializer.deserialize_seq()
        Ok(Configuration{groups: vec![]})
    }
}

// implement custom configuration for serde

pub fn generate<T: AsRef<Path>>(base: T) -> Result<(), Box<dyn Error>> {
    generate_for_reloaded_2(base)
}
fn generate_for_reloaded_2<T: AsRef<Path>>(base: T) -> Result<(), Box<dyn Error>> {
    // Read config.toml as toml table
    let config_file = base.as_ref().join("data/config/config.toml");
    // let config = toml::from_str::<Configuration>(fs::read_to_string(&config_file)?.as_str())?;
    // let config = fs::read_to_string(&config_file)?.parse::<Table>()?;
    // println!("{:#?}", config);

    // Generate Rust and C# code
    // let output_rust = base.as_ref().join("src/config.rs");
    // let output_csharp = base.as_ref().join("middata/Config.cs");
    Ok(())
}

use std::ffi;
use std::fs;
use std::io::Read;
use std::path;

use failure;
use stager;

#[cfg(feature = "json")]
use serde_json;
#[cfg(feature = "yaml")]
use serde_yaml;
#[cfg(feature = "toml")]
use toml;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub target: stager::de::Template,
    pub stage: stager::de::Staging,
}

impl Config {
    pub fn from_file(path: &path::Path) -> Result<Self, failure::Error> {
        let extension = path.extension().unwrap_or_default();
        let value = if extension == ffi::OsStr::new("yaml") || extension == ffi::OsStr::new("yml") {
            Self::load_yaml(path)
        } else if extension == ffi::OsStr::new("toml") {
            Self::load_toml(path)
        } else if extension == ffi::OsStr::new("json") {
            Self::load_json(path)
        } else {
            bail!("Unsupported file type: {:?}", extension);
        }?;

        Ok(value)
    }

    #[cfg(feature = "yaml")]
    fn load_yaml(path: &path::Path) -> Result<Self, failure::Error> {
        let f = fs::File::open(path)?;
        serde_yaml::from_reader(f).map_err(|e| e.into())
    }

    #[cfg(not(feature = "yaml"))]
    fn load_yaml(_path: &path::Path) -> Result<Self, failure::Error> {
        bail!("yaml is unsupported");
    }

    #[cfg(feature = "json")]
    fn load_json(path: &path::Path) -> Result<Self, failure::Error> {
        let f = fs::File::open(path)?;
        serde_json::from_reader(f).map_err(|e| e.into())
    }

    #[cfg(not(feature = "json"))]
    fn load_json(_path: &path::Path) -> Result<Self, failure::Error> {
        bail!("json is unsupported");
    }

    #[cfg(feature = "toml")]
    fn load_toml(path: &path::Path) -> Result<Self, failure::Error> {
        let mut f = fs::File::open(path)?;
        let mut text = String::new();
        f.read_to_string(&mut text)?;
        toml::from_str(&text).map_err(|e| e.into())
    }

    #[cfg(not(feature = "toml"))]
    fn load_toml(_path: &path::Path) -> Result<Self, failure::Error> {
        bail!("toml is unsupported");
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target: stager::de::Template::new("{{cargo.name}}-{{cargo.version}}-{{crate.target}}"),
            stage: Default::default(),
        }
    }
}

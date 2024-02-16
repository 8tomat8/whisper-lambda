use serde::Deserialize;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum Model {
    Base,
    Tiny,
    Small,
    Medium,
    Large,
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D>(deserializer: D) -> Result<Model, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        let model = Model::from_str(&s);
        match model {
            Ok(model) => {
                if !model.is_ok() {
                    return Err(serde::de::Error::custom(format!(
                        "model file not found, run `make pull`: {}",
                        s
                    )));
                }
                return Ok(model);
            }
            Err(_) => Err(serde::de::Error::custom(format!(
                "invalid model [{:?}]: {}",
                format!("{:?}", Model::list()).to_lowercase(),
                s
            ))),
        }
    }
}

impl Model {
    pub fn path(&self) -> String {
        let mut path = PathBuf::from("./models");
        path.push(self.file_name());
        return path.to_str().unwrap().to_string();
    }
}

impl Model {
    fn is_ok(&self) -> bool {
        fs::metadata(self.path().as_str()).is_ok()
    }
}

impl Model {
    fn file_name(&self) -> String {
        format!("ggml-{}.bin", self)
    }
}

impl Model {
    pub fn list() -> Vec<Model> {
        Model::iter().collect()
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Model::Base => write!(f, "base"),
            Model::Tiny => write!(f, "tiny"),
            Model::Small => write!(f, "small"),
            Model::Medium => write!(f, "medium"),
            Model::Large => write!(f, "large"),
        }
    }
}

impl FromStr for Model {
    type Err = ();
    fn from_str(input: &str) -> Result<Model, Self::Err> {
        match input.to_lowercase().as_str() {
            "base" => Ok(Model::Base),
            "tiny" => Ok(Model::Tiny),
            "small" => Ok(Model::Small),
            "medium" => Ok(Model::Medium),
            "large" => Ok(Model::Large),
            _ => Err(()),
        }
    }
}

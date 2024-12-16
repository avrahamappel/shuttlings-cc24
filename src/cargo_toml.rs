use std::collections::HashMap;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Value {
    Toml(toml::Value),
    Json(serde_json::Value),
    Yaml(serde_yml::Value),
}

#[allow(dead_code)]
enum DeserializeError {
    Toml(toml::de::Error),
    Json(serde_json::Error),
    Yaml(serde_yml::Error),
}

impl Value {
    fn try_into<T>(self) -> Result<T, DeserializeError>
    where
        T: DeserializeOwned,
    {
        match self {
            Self::Toml(value) => value.try_into().map_err(DeserializeError::Toml),
            Self::Json(value) => serde_json::from_value(value).map_err(DeserializeError::Json),
            Self::Yaml(value) => serde_yml::from_value(value).map_err(DeserializeError::Yaml),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Order {
    pub item: String,
    pub quantity: u32,
}

fn deserialize_orders<'de, D>(des: D) -> Result<Vec<Order>, D::Error>
where
    D: Deserializer<'de>,
{
    let values: Vec<Value> = Vec::deserialize(des)?;

    let mut result = Vec::new();

    for value in values {
        if let Ok(inner) = value.try_into() {
            result.push(inner);
        }
    }

    Ok(result)
}

#[derive(Deserialize, Debug, Default)]
struct Metadata {
    #[serde(default, deserialize_with = "deserialize_orders")]
    orders: Vec<Order>,
}

#[derive(Deserialize, Debug)]
struct WrappedMetadata {
    metadata: Metadata,
}

fn deserialize_metadata<'de, D>(des: D) -> Result<Metadata, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MaybeWrappedMetadata {
        Wrapped(WrappedMetadata),
        Unwrapped(Metadata),
    }

    match MaybeWrappedMetadata::deserialize(des)? {
        MaybeWrappedMetadata::Wrapped(wm) => Ok(wm.metadata),
        MaybeWrappedMetadata::Unwrapped(m) => Ok(m),
    }
}

#[derive(Debug, Deserialize)]
enum Edition {
    #[serde(rename = "2015")]
    E2015,
    #[serde(rename = "2018")]
    E2018,
    #[serde(rename = "2021")]
    E2021,
    #[serde(rename = "2024")]
    E2024,
}

fn deserialize_rust_version<'de, D>(des: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let str_o: Option<String> = Option::deserialize(des)?;
    if let Some(str) = str_o {
        f64::from_str(&str).map_err(|_| serde::de::Error::custom("Invalid version string"))?;
        Ok(Some(str))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Package {
    name: String,
    #[serde(default)]
    keywords: Vec<String>,
    // one of the tests has double nesting by mistake
    #[serde(alias = "package", default, deserialize_with = "deserialize_metadata")]
    metadata: Metadata,
    edition: Option<Edition>,
    #[serde(default, deserialize_with = "deserialize_rust_version")]
    rust_version: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Profile {
    incremental: bool,
}

#[derive(Debug, Deserialize)]
enum Resolver {
    #[serde(rename = "1")]
    R1,
    #[serde(rename = "2")]
    R2,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Workspace {
    resolver: Resolver,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct CargoToml {
    package: Package,
    profile: Option<HashMap<String, Profile>>,
    workspace: Option<Workspace>,
}

pub enum CargoOrders {
    Orders(Vec<Order>),
    KeywordMissing,
    InvalidManifest,
}

#[derive(Clone, Copy)]
pub enum ContentType {
    Yaml,
    Json,
    Toml,
}

pub fn from_str(data: &str, content_type: ContentType) -> CargoOrders {
    let cargo_res: Result<CargoToml, DeserializeError> = match content_type {
        ContentType::Yaml => serde_yml::from_str(data).map_err(DeserializeError::Yaml),
        ContentType::Json => serde_json::from_str(data).map_err(DeserializeError::Json),
        ContentType::Toml => toml::from_str(data).map_err(DeserializeError::Toml),
    };

    if let Ok(cargo_toml) = cargo_res {
        if cargo_toml
            .package
            .keywords
            .contains(&String::from("Christmas 2024"))
        {
            CargoOrders::Orders(cargo_toml.package.metadata.orders)
        } else {
            CargoOrders::KeywordMissing
        }
    } else {
        CargoOrders::InvalidManifest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_toml_doesnt_require_profiles() {
        assert!(toml::from_str::<CargoToml>(
            r#"
[package]
name = "test"
"#
        )
        .is_ok());
    }

    #[test]
    fn cargo_toml_errors_on_invalid_profiles() {
        assert!(toml::from_str::<CargoToml>(
            r#"
[package]
name = "test"

[profile.release]
incremental = "woohoo"
"#
        )
        .is_err());
    }

    #[test]
    fn cargo_toml_errors_on_invalid_rust_version_string() {
        assert!(serde_yml::from_str::<CargoToml>(
            "
package:
  name: test
  rust-version: false
"
        )
        .is_err());
    }
}

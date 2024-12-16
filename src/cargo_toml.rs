use std::collections::HashMap;

use serde::{Deserialize, Deserializer};
use serde_repr::Deserialize_repr;
use toml::Value;

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
    E2015,
    E2018,
    E2021,
    E2024,
}

fn deserialize_edition<'de, D>(des: D) -> Result<Option<Edition>, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(des)?.as_str() {
        "2015" => Ok(Some(Edition::E2015)),
        "2018" => Ok(Some(Edition::E2018)),
        "2021" => Ok(Some(Edition::E2021)),
        "2024" => Ok(Some(Edition::E2024)),
        _ => Err(serde::de::Error::custom("Invalid edition string")),
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    #[serde(default)]
    keywords: Vec<String>,
    // one of the tests has double nesting by mistake
    #[serde(alias = "package", default, deserialize_with = "deserialize_metadata")]
    metadata: Metadata,
    #[serde(default, deserialize_with = "deserialize_edition")]
    edition: Option<Edition>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Profile {
    incremental: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Workspace {
    resolver: Resolver,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
enum Resolver {
    One = 1,
    Two = 2,
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

pub fn from_str(data: &str) -> CargoOrders {
    if let Ok(cargo_toml) = toml::from_str::<CargoToml>(data) {
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
}

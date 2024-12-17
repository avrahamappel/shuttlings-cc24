use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum Conversion {
    Gallons(f64),
    Liters(f64),
}

impl Conversion {
    pub fn convert(self) -> Self {
        match self {
            Self::Gallons(g) => Self::Liters(g * 3.785_412),
            Self::Liters(l) => Self::Gallons(l * 0.264_172),
        }
    }
}

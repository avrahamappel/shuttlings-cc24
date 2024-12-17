use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum US {
    Gallons(f64),
    Liters(f64),
}

impl US {
    fn convert(self) -> Self {
        match self {
            Self::Gallons(g) => Self::Liters(g * 3.785_412),
            Self::Liters(l) => Self::Gallons(l * 0.264_172),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum UK {
    Pints(f64),
    Litres(f64),
}

impl UK {
    fn convert(self) -> Self {
        match self {
            Self::Pints(p) => Self::Litres(p * 0.568_262),
            Self::Litres(l) => Self::Pints(l * 1.759_751),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum Conversion {
    US(US),
    UK(UK),
}

impl Conversion {
    pub fn convert(self) -> Self {
        match self {
            Self::US(us) => Self::US(us.convert()),
            Self::UK(uk) => Self::UK(uk.convert()),
        }
    }
}

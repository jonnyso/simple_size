use paste::paste;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

const KB: u32 = 10;
const MB: u32 = 20;
const GB: u32 = 30;
const TB: u32 = 40;

fn err_message(s: &str) -> String {
    format!("invalid format for unit size: {s}. Acceptable formats are nB, nKB, nMB,nGB, nTB")
}

const fn down_from(lhs: f32, dec: u32) -> f32 {
    lhs * (2_u64.pow(dec) as f32)
}

const fn up_to(lhs: f32, dec: u32) -> f32 {
    lhs / (2_u64.pow(dec) as f32)
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy, Default)]
pub struct Unit(f32);

macro_rules! impl_op {
    ($trait:ident, $fname:ident, $op:tt) => {
        impl $trait for Unit {
            type Output = Self;

            fn $fname(self, rhs: Self) -> Self::Output {
                Self(self.0 $op rhs.0)
            }
        }

        impl $trait for &Unit {
            type Output = Unit;

            fn $fname(self, rhs: Self) -> Self::Output {
                Unit(self.0 $op rhs.0)
            }
        }

        paste! {
            impl [<$trait Assign>] for Unit {
                fn [<$fname _assign>](&mut self, rhs: Self) {
                    self.0 = self.0 $op rhs.0;
                }
            }
        }
    };
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Div, div, /);

impl Unit {
    pub const fn as_bytes(&self) -> f32 {
        self.0
    }

    pub const fn from_bytes(value: f32) -> Self {
        Self(value)
    }

    pub const fn from_kilo_bytes(value: f32) -> Self {
        Self(down_from(value, KB))
    }

    pub const fn from_mega_bytes(value: f32) -> Self {
        Self(down_from(value, MB))
    }

    pub const fn from_giga_bytes(value: f32) -> Self {
        Self(down_from(value, GB))
    }

    pub const fn from_tera_bytes(value: f32) -> Self {
        Self(down_from(value, TB))
    }
}

impl FromStr for Unit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index = s.rfind(char::is_numeric).ok_or_else(|| err_message(s))?;
        let (value, unit) = s
            .split_at_checked(index + 1)
            .ok_or_else(|| err_message(s))?;
        let value = value
            .trim()
            .replace(",", ".")
            .parse::<f32>()
            .map_err(|err| format!("{}: {}", err_message(s), err))?;
        let unit = unit.trim();
        Ok(match unit {
            "B" => Self::from_bytes(value),
            "KB" => Self::from_kilo_bytes(value),
            "MB" => Self::from_mega_bytes(value),
            "GB" => Self::from_giga_bytes(value),
            "TB" => Self::from_tera_bytes(value),
            _ => return Err(err_message(s)),
        })
    }
}

impl From<u64> for Unit {
    fn from(value: u64) -> Self {
        Self(value as f32)
    }
}

impl From<f32> for Unit {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = if self.0.is_sign_negative() {
            self.0 * -1_f32
        } else {
            self.0
        };
        if value >= down_from(1_f32, TB) {
            return write!(f, "{:.2}TB", up_to(self.0, TB));
        }
        if value >= down_from(1_f32, GB) {
            return write!(f, "{:.2}GB", up_to(self.0, GB));
        }
        if value >= down_from(1_f32, MB) {
            return write!(f, "{:.2}MB", up_to(self.0, MB));
        }
        if value >= down_from(1_f32, KB) {
            return write!(f, "{:.2}KB", up_to(self.0, KB));
        }
        write!(f, "{}B", self.0)
    }
}

#[cfg(feature = "serde")]
pub mod serde {
    use std::str::FromStr;

    use serde::{Deserialize, Serialize, de::Visitor};

    use crate::Unit;

    impl Serialize for Unit {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    }

    struct UnitVisitor;

    impl<'de> Visitor<'de> for UnitVisitor {
        type Value = Unit;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string with the format: 99TB|GB|MB|B")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Unit::from_str(v).map_err(E::custom)
        }
    }

    impl<'de> Deserialize<'de> for Unit {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(UnitVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        assert_eq!(Unit(1_f32), Unit::from_str("1B").unwrap());
        assert_eq!(Unit(1024_f32), Unit::from_str("1KB").unwrap());
        assert_eq!(Unit(1_048_576_f32), Unit::from_str("1MB").unwrap());
        assert_eq!(Unit(1_073_741_824_f32), Unit::from_str("1GB").unwrap());
        assert_eq!(Unit(1_099_511_627_776_f32), Unit::from_str("1TB").unwrap());
    }

    #[test]
    fn from_str_negative() {
        assert_eq!(Unit(-1_f32), Unit::from_str("-1B").unwrap());
        assert_eq!(Unit(-1024_f32), Unit::from_str("-1KB").unwrap());
        assert_eq!(Unit(-1_048_576_f32), Unit::from_str("-1MB").unwrap());
        assert_eq!(Unit(-1_073_741_824_f32), Unit::from_str("-1GB").unwrap());
        assert_eq!(
            Unit(-1_099_511_627_776_f32),
            Unit::from_str("-1TB").unwrap()
        );
    }

    #[test]
    fn to_string() {
        let size = Unit::from_str("10TB").unwrap();
        assert_eq!("10.00TB", size.to_string());
        let size = Unit::from_str("10GB").unwrap();
        assert_eq!("10.00GB", size.to_string());
        let size = Unit::from_str("10MB").unwrap();
        assert_eq!("10.00MB", size.to_string());
        let size = Unit::from_str("10KB").unwrap();
        assert_eq!("10.00KB", size.to_string());
        let size = Unit::from_str("10B").unwrap();
        assert_eq!("10B", size.to_string());
    }

    #[test]
    fn to_string_negative() {
        let size = Unit::from_str("-10TB").unwrap();
        assert_eq!("-10.00TB", size.to_string());
        let size = Unit::from_str("-10GB").unwrap();
        assert_eq!("-10.00GB", size.to_string());
        let size = Unit::from_str("-10MB").unwrap();
        assert_eq!("-10.00MB", size.to_string());
        let size = Unit::from_str("-10KB").unwrap();
        assert_eq!("-10.00KB", size.to_string());
        let size = Unit::from_str("-10B").unwrap();
        assert_eq!("-10B", size.to_string());
    }
}

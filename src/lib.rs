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

const fn down_from(lhs: f64, dec: u32) -> f64 {
    lhs * (2_u64.pow(dec) as f64)
}

const fn up_to(lhs: f64, dec: u32) -> f64 {
    lhs / (2_u64.pow(dec) as f64)
}

#[derive(PartialEq, PartialOrd)]
pub struct Unit(f64);

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
    pub const fn as_bytes(&self) -> f64 {
        self.0
    }

    pub const fn from_bytes(value: f64) -> Self {
        Self(value)
    }

    pub const fn from_kilo_bytes(value: f64) -> Self {
        Self(down_from(value, KB))
    }

    pub const fn from_mega_bytes(value: f64) -> Self {
        Self(down_from(value, MB))
    }

    pub const fn from_giga_bytes(value: f64) -> Self {
        Self(down_from(value, GB))
    }

    pub const fn from_tera_bytes(value: f64) -> Self {
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
            .parse::<f64>()
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

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 >= up_to(1_f64, TB) {
            return write!(f, "{:.2}TB", up_to(self.0, TB));
        }
        if self.0 >= up_to(1_f64, GB) {
            return write!(f, "{:.2}GB", up_to(self.0, GB));
        }
        if self.0 >= up_to(1_f64, MB) {
            return write!(f, "{:.2}MB", up_to(self.0, MB));
        }
        if self.0 >= up_to(1_f64, TB) {
            return write!(f, "{:.2}TB", up_to(self.0, TB));
        }
        write!(f, "{}B", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_bytes() {
        assert_eq!(1_f64, Unit::from_str("1B").unwrap().as_bytes());
        assert_eq!(1024_f64, Unit::from_str("1KB").unwrap().as_bytes());
        assert_eq!(1_048_576_f64, Unit::from_str("1MB").unwrap().as_bytes());
        assert_eq!(1_073_741_824_f64, Unit::from_str("1GB").unwrap().as_bytes());
        assert_eq!(
            1_099_511_627_776_f64,
            Unit::from_str("1TB").unwrap().as_bytes()
        );
    }

    #[test]
    fn to_string() {
        let size = Unit::from_str("10TB").unwrap();
        assert_eq!("10TB", size.to_string());
        let size = Unit::from_str("10GB").unwrap();
        assert_eq!("10GB", size.to_string());
        let size = Unit::from_str("10MB").unwrap();
        assert_eq!("10MB", size.to_string());
        let size = Unit::from_str("10KB").unwrap();
        assert_eq!("10KB", size.to_string());
        let size = Unit::from_str("10B").unwrap();
        assert_eq!("10B", size.to_string());
    }
}

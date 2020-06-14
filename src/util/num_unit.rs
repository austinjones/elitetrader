use std::fmt::Display;

use num::traits::Float;
use num::traits::NumCast;

use std::str::FromStr;

pub enum NumericUnit {
    Tril(f64),
    Bil(f64),
    Mil(f64),
    Kilo(f64),
    Unit(f64),
}

impl FromStr for NumericUnit {
    type Err = String;

    fn from_str(input: &str) -> Result<NumericUnit, String> {
        let numstr: &str = input.trim_matches(|c: char| !c.is_numeric() || c == '.');

        match f64::from_str(numstr) {
            Ok(num) => match input.trim().to_lowercase().chars().last() {
                Some(char) => match char {
                    't' => Ok(NumericUnit::Tril(num)),
                    'b' => Ok(NumericUnit::Bil(num)),
                    'm' => Ok(NumericUnit::Mil(num)),
                    'k' => Ok(NumericUnit::Kilo(num)),
                    '0'..='9' => Ok(NumericUnit::Unit(num)),
                    _ => Err(format!("Unknown enum variant for '{}'", input)),
                },
                None => Err("No value".to_string()),
            },
            Err(_) => Err(format!(
                "Input '{}' mut contain a single contiguious numeric portion",
                input
            )),
        }
    }
}

impl NumericUnit {
    pub fn new_string<F: NumCast + Display + Copy>(val: F, unit_description: &String) -> String {
        let en = Self::from(val);
        en.to_string(&unit_description)
    }

    pub fn from<F: NumCast + Display + Copy>(val: F) -> NumericUnit {
        let float: f64 = match NumCast::from(val) {
            Some(v) => v,
            None => panic!("Failed to convert {} to f64", val),
        };

        let (base, significand) = match float {
            0f64 => (0f64, 0f64),
            _ => {
                let log_factor = 1000f64;

                let base = float.log(log_factor).floor();
                let significand = float / log_factor.powf(base);
                (base, significand)
            }
        };

        match base as usize {
            4 => NumericUnit::Tril(significand),
            3 => NumericUnit::Bil(significand),
            2 => NumericUnit::Mil(significand),
            1 => NumericUnit::Kilo(significand),
            0 => NumericUnit::Unit(significand),
            n @ _ => panic!(
                "Numeric value {} not supported by NumericUnit - base size {}",
                val, n
            ),
        }
    }

    pub fn get_significand_mul(&self) -> (f64, f64) {
        match *self {
            NumericUnit::Tril(x) => (x, 1_000_000_000_000f64),
            NumericUnit::Bil(x) => (x, 1_000_000_000f64),
            NumericUnit::Mil(x) => (x, 1_000_000f64),
            NumericUnit::Kilo(x) => (x, 1_000f64),
            NumericUnit::Unit(x) => (x, 1f64),
        }
    }

    pub fn to_string(&self, unit_description: &String) -> String {
        let (significand, _) = self.get_significand_mul();

        let precision = match *self {
            NumericUnit::Tril(_) => 3,
            NumericUnit::Bil(_) => 2,
            NumericUnit::Mil(_) => 2,
            NumericUnit::Kilo(_) => 1,
            NumericUnit::Unit(_) => 0,
        };

        let suffix = match *self {
            NumericUnit::Tril(_) => "T",
            NumericUnit::Bil(_) => "B",
            NumericUnit::Mil(_) => "M",
            NumericUnit::Kilo(_) => "K",
            NumericUnit::Unit(_) => "",
        };

        format!(
            "{:.*} {}{}",
            precision, significand, suffix, unit_description
        )
    }

    pub fn to_num<F: NumCast>(&self) -> F {
        let (significand, mul) = self.get_significand_mul();

        let val: f64 = significand * mul;
        match F::from(val) {
            Some(v) => v,
            None => panic!("Failed to convert {} from f64", val),
        }
    }
}

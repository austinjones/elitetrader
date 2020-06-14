use std::str::FromStr;

#[derive(Copy, Clone)]
pub enum SearchQuality {
    Medium,
    High,
    Ultra,
}

impl SearchQuality {
    pub fn get_hop_width(&self) -> usize {
        // 6 is the best value.

        // higher values have shallow search areas,
        // which tend to miss good trade which are many hops away,
        // and get stuck in local maxima

        // lower values have much deeper search areas,
        // but tend to converge and get stuck in local maxima

        match *self {
            SearchQuality::Ultra => 7,
            SearchQuality::High => 6,
            SearchQuality::Medium => 5,
        }
    }

    pub fn get_depth(&self) -> usize {
        9usize
    }

    pub fn get_trade_range(&self) -> f64 {
        70f64
    }
}

impl FromStr for SearchQuality {
    type Err = String;

    fn from_str(s: &str) -> Result<SearchQuality, String> {
        match &s.to_lowercase()[..] {
            "m" | "med" => Ok(SearchQuality::Medium),
            "h" | "high" => Ok(SearchQuality::High),
            "u" | "ultra" => Ok(SearchQuality::Ultra),
            _ => Err(format!("Unknown enum variant '{}'", s)),
        }
    }
}

use std::str::FromStr;

#[derive(Copy, Clone)]
pub enum SearchQuality {
	Low,
	Medium,
	High
}

impl SearchQuality {
//	pub fn get_computation_num( &self ) -> usize {
//		// magic numbers based on performance measured on my machine.
//		// 1400 trades processed per second.
//		match *self {
//			SearchQuality::High => 60 * 839808,
//			SearchQuality::Medium => 20 * 839808,
//			SearchQuality::Low => 5 * 839808
//		}
//	}
	
	pub fn get_hop_width( &self ) -> usize {
		// 6 is the best value.
		
		// higher values have shallow search paths, 
		// which tend to miss good trade which are many hops away,
		// and get stuck in local maxima
		
		// lower values have much deeper search spaces,
		// but tend to converge and get stuck in local maxima
		6usize
	}
	
	pub fn get_depth( &self ) -> usize {
//		let target_computations = self.get_computation_num() as f64;
//		
//		let width = self.get_hop_width();
//		target_computations.log(width as f64).floor() as usize

		match *self {
			SearchQuality::High => 9,
			SearchQuality::Medium => 8,
			SearchQuality::Low => 7
		}
	}
	
	pub fn get_trade_range( &self ) -> f64 {
		80f64
	}
}

impl FromStr for SearchQuality {
    type Err = String;

    fn from_str(s: &str) -> Result<SearchQuality, String> {
        match s.to_lowercase().as_str() {
            "low" => Ok(SearchQuality::Low),
            "med" => Ok(SearchQuality::Medium),
            "high" => Ok(SearchQuality::High),
            "medium" => Ok(SearchQuality::Medium),
            "l" => Ok(SearchQuality::Low),
            "m" => Ok(SearchQuality::Medium),
            "h" => Ok(SearchQuality::High),
            _ => Err( format!("Unknown enum variant '{}'", s) ),
        }
    }
}
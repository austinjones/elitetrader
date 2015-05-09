use std::str::FromStr;

#[derive(Copy, Clone)]
pub enum SearchQuality {
	Medium,
	High,
	Ultra
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
		
		// higher values have shallow search areas, 
		// which tend to miss good trade which are many hops away,
		// and get stuck in local maxima
		
		// lower values have much deeper search areas,
		// but tend to converge and get stuck in local maxima
		6usize
	}
	
	pub fn get_depth( &self ) -> usize {
//		let target_computations = self.get_computation_num() as f64;
//		
//		let width = self.get_hop_width();
//		target_computations.log(width as f64).floor() as usize

		match *self {
			SearchQuality::Ultra => 9,
			SearchQuality::High => 8,
			SearchQuality::Medium => 7
		}
	}
	
	pub fn get_trade_range( &self ) -> f64 {
		100f64
	}
}

impl FromStr for SearchQuality {
    type Err = String;

    fn from_str(s: &str) -> Result<SearchQuality, String> {
        match s.to_lowercase().as_str() {
            "m" | "med" => Ok(SearchQuality::Medium),
            "h" | "high" => Ok(SearchQuality::High),
            "u" | "ultra" => Ok(SearchQuality::Ultra),
            _ => Err( format!("Unknown enum variant '{}'", s) ),
        }
    }
}
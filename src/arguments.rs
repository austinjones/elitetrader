use getopts::{Matches};
use search::SearchQuality;
use util::num_unit::*;
use data::trader::ShipSize;
use user_input::prompt_value;
use std::str::FromStr;

pub struct Arguments {
//	pub system: String,
	pub station: String,
	pub cargo: u32,
	pub credit_balance: u32,
	pub minimum_balance: u32,
	pub jump_range: f64,
	pub ship_size: ShipSize,
	pub search_quality: SearchQuality
}

impl Arguments {
	pub fn collect( config: &Matches ) -> Arguments {
//		let system_in = config.opt_str("s")
//			.unwrap_or( prompt_value( "current system name" ) );
		
		let station_in = match config.opt_str("t") {
			Some(t) => t,
			None => prompt_value( "t", "current station name (e.g. Git Hub)" )
		};
		
		
		let cargo_in = match config.opt_str("c") {
			Some(v) => v,
			None => prompt_value( "c", "current cargo capacity in tons (e.g. 216)" )
		};
		let cargo_capacity = match NumericUnit::from_str( cargo_in.as_str() ) {
			Ok(v) => v.to_num(),
			Err(reason) => panic!("Invalid cargo capacity '{}' - {}", cargo_in, reason)
		};
		
		
		let balance_in = match config.opt_str("b") {
			Some(v) => v,
			None => prompt_value( "b", "current credit balance (e.g. 525.4k or 525412)" )
		};
		let balance = match NumericUnit::from_str( balance_in.as_str() ) {
			Ok(v) => v.to_num(),
			Err(reason) => panic!("Invalid balance '{}' - {}", balance_in, reason)
		};
		
		
		let minimum_balance_in = match config.opt_str("m") {
			Some(v) => v,
			None => prompt_value( "b", "minimum credit balance - saftey net for rebuy (e.g. 3.2m)" )
		};
		let minimum_balance = match NumericUnit::from_str( minimum_balance_in.as_str() ) {
			Ok(v) => v.to_num(),
			Err(reason) => panic!("Invalid minimum balance '{}' - {}", minimum_balance_in, reason)
		};
		
		
		let jump_range_in = match config.opt_str("r") {
			Some(v) => v,
			None => prompt_value( "r", "current laden jump range in light years" )
		};
		let jump_range = match NumericUnit::from_str( jump_range_in.as_str() ) {
			Ok(v) => v.to_num(),
			Err(reason) => panic!("Invalid jump range '{}' - {}", jump_range_in, reason)
		};
		
		
		let ship_size_in = match config.opt_str("p") {
			Some(v) => v,
			None => prompt_value( "r", "current ship size [small|med|large], or [s|m|l]" )
		};
		let ship_size = match ShipSize::from_str( ship_size_in.as_str() ) {
			Ok(v) => v,
			Err(reason) => panic!("Invalid ship size '{}' - {}", ship_size_in, reason)
		};
		
		
		let quality_in = config.opt_str("q").unwrap_or( "high".to_string() );
		let quality : SearchQuality = match SearchQuality::from_str(quality_in.as_str()) {
			Ok(v) => v,
			Err(reason) => panic!("Invalid search quality '{}' - {}", quality_in, reason)
		};
		
		
		Arguments {
//			system: system_in,
			station: station_in,
			cargo: cargo_capacity,
			credit_balance: balance,
			jump_range: jump_range,
			minimum_balance: minimum_balance,
			ship_size: ship_size,
			search_quality: quality
		}
	}
}
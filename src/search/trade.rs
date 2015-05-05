use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::cmp::min;

use data::trader::*;
use data::IndexedUniverse;
use search::player_state::PlayerState;

#[derive(Clone)]
pub struct FullTrade<'a> {
	pub unit: UnitTrade<'a>,
	pub profit_total: u32,
	pub used_cargo: u32,
	pub profit_per_min: Option<f64>,
	pub is_valid: bool,
	state: PlayerState
}

impl<'a> FullTrade<'a> {
	pub fn new( state: &PlayerState, unit: UnitTrade<'a> ) -> FullTrade<'a> {
		let used_cargo = FullTrade::used_cargo( state, unit.buy );
		let profit_per_min = FullTrade::profit_per_min( &unit, used_cargo );
		let is_valid = unit.is_valid && used_cargo > 0;
//		println!( "Using {} of {}, profit/ton {}, profit total {}, profit/min {} over {}sec",
//			used_cargo, buy.commodity.commodity_name,
//			profit_per_ton, profit_total, profit_per_min.unwrap_or(0f64), cost_in_seconds );
		FullTrade {
			used_cargo: used_cargo,
			profit_total: unit.profit_per_ton * used_cargo,
			profit_per_min: profit_per_min,
			is_valid: is_valid,
			state: state.clone(),
			unit: unit
		}
	}
	
	pub fn state_before_trade( &self ) -> PlayerState {
		self.state.clone()
	}
	
	pub fn state_after_trade( &self ) -> PlayerState {
		self.state.with_trade( self )
	}
	
	pub fn with_sell_price( &self, sell_price: u16 ) -> FullTrade<'a> {
		let new_unit = self.unit.with_sell_price( sell_price );
		FullTrade::new( &self.state, new_unit )
	}
}

impl<'a> FullTrade<'a> {
	pub fn profit_per_min( unit: &UnitTrade, used_cargo: u32 ) -> Option<f64> {
		unit.profit_per_ton_per_min.map( |v| v*used_cargo as f64 )
	}
	
	pub fn used_cargo( state: &PlayerState, buy: &Listing ) -> u32 {
		if state.credit_balance < state.minimum_balance {
			return 0;
		}
		
		let possible_cargo = (state.credit_balance - state.minimum_balance) / buy.buy_price as u32;
		min( possible_cargo, state.cargo_capacity )
	}
}

#[derive(Clone)]
pub struct UnitTrade<'a> {
	pub commodity_name: String,
	
	pub buy: &'a Listing,
	pub buy_station: &'a Station,
	pub buy_system: &'a System,
	
	pub sell: &'a Listing,
	pub sell_station: &'a Station,
	pub sell_system: &'a System,
	
	pub buy_price: u16,
	pub sell_price: u16,
	
	pub is_valid: bool,
	pub is_prohibited: bool,

	pub profit_per_ton: u32,
	pub profit_per_ton_per_min: Option<f64>,
	
	pub distance_to_system: f64,
	pub distance_to_station: f64,
	pub distance_in_seconds: f64
}

// actual trait impl
#[allow(dead_code)]
impl<'b> UnitTrade<'b> {
	pub fn new( iuniverse: &'b IndexedUniverse, state: &PlayerState, buy: &'b Listing, sell: &'b Listing ) -> UnitTrade<'b> {
		let buy_station = iuniverse.get_station( &buy.station_id )
			.expect( format!("Unknown station id {}", buy.station_id ).as_str() );
		
		let buy_system = iuniverse.get_system( &buy.system_id )
			.expect( format!("Unknown system id {}", buy.system_id ).as_str() );
		
		let sell_station = iuniverse.get_station( &sell.station_id )
			.expect( format!("Unknown station id {}", sell.station_id ).as_str() );
		
		let sell_system = iuniverse.get_system( &sell.system_id )
			.expect( format!("Unknown system id {}", sell.system_id ).as_str() );
		
		let distance_to_system = buy_system.distance( sell_system );
		let distance_to_station = sell_station.distance_to_star.unwrap_or(100u32);
		let distance_in_seconds = UnitTrade::cost_in_seconds( state, distance_to_system, distance_to_station );
		
		let profit_per_ton = UnitTrade::profit_per_ton( &buy, &sell );
		
		let profit_per_ton_per_min = UnitTrade::profit_per_ton_per_min( &buy, &sell, distance_in_seconds );
//		println!( "Using {} of {}, profit/ton {}, profit total {}, profit/min {} over {}sec",
//			used_cargo, buy.commodity.commodity_name,
//			profit_per_ton, profit_total, profit_per_min.unwrap_or(0f64), cost_in_seconds );
		
		UnitTrade {
			commodity_name: buy.commodity.commodity_name.clone(),
			
			buy: buy,
			buy_station: buy_station,
			buy_system: buy_system,
			
			sell: sell,
			sell_station: sell_station,
			sell_system: sell_system,
			
			buy_price: buy.buy_price,
			sell_price: sell.sell_price,
			
			is_valid: UnitTrade::is_valid( &buy, &sell ),
			is_prohibited: UnitTrade::is_prohibited( &buy.commodity, &sell_station ),
			
			profit_per_ton: profit_per_ton,
			profit_per_ton_per_min: profit_per_ton_per_min,
			
			distance_to_system: distance_to_system,
			distance_to_station: distance_to_station as f64,
			distance_in_seconds: distance_in_seconds,
		}
	}
	
	pub fn score( &self ) -> Option<f64> {
		self.profit_per_ton_per_min
	}
	
	pub fn with_sell_price( &self, sell_price: u16 ) -> UnitTrade<'b> {
		let mut new = self.clone();
		let mut sell = self.sell.clone();
		sell.sell_price = sell_price;
		
		let profit_per_ton = UnitTrade::profit_per_ton( self.buy, &sell );
		let profit_per_ton_per_min = UnitTrade::profit_per_ton_per_min( self.buy, &sell, self.distance_in_seconds );
		
		new.sell_price = sell_price;
		new.profit_per_ton = profit_per_ton;
		new.profit_per_ton_per_min = profit_per_ton_per_min;
		
		new
	}
}

// static methods
impl<'b> UnitTrade<'b> {
	pub fn profit_per_ton( buy: &Listing, sell: &Listing ) -> u32 {
		if sell.sell_price > buy.buy_price {
			(sell.sell_price - buy.buy_price) as u32
		} else {
			0
		}
	}
	
	pub fn is_valid( buy: &Listing, sell: &Listing ) -> bool {
		buy.supply > 0 
			&& buy.buy_price != 0
			&& buy.buy_price < sell.sell_price 
			&& buy.commodity == sell.commodity
	}
	
	pub fn is_prohibited( commodity: &Commodity, sell_station: &Station ) -> bool {
		sell_station.prohibited_commodities.contains( &commodity.commodity_id )
	}
	
	pub fn profit_per_ton_per_min( buy: &Listing, sell: &Listing, distance_in_seconds: f64  ) -> Option<f64> {
		if !UnitTrade::is_valid( buy, sell ) {
			return None;
		}
		
		let profit_total = UnitTrade::profit_per_ton( buy, sell );
		let profit_per_min = match distance_in_seconds {
			0f64 => 60f64 * profit_total as f64,
			_ => 60f64 * profit_total as f64 / distance_in_seconds
		};
		
		Some( profit_per_min )
	}
	
	pub fn cost_in_seconds( state: &PlayerState, system_distance: f64, station_distance_ls: u32 ) -> f64 {
		let dock_time = 77f64;
		let undock_time = 29f64;
		
		let jump_time = UnitTrade::time_to_system( system_distance, state.jump_range );
		let supercruise_time = UnitTrade::time_to_station( station_distance_ls as f64 );
		
		undock_time + jump_time + supercruise_time + dock_time
	}
	
	pub fn time_to_station( station_distance_ls: f64 ) -> f64 {
		let a = 30.02805f64;
		let x = station_distance_ls;
		let b = 0.2262488;
		a * x.powf( b )
	}
	
	pub fn jump_count( system_distance: f64, jump_range: f64 ) -> f64 {
		// linear regression based on lots of time spent in the galaxy map
		// plot route. note distance and # of jumps. repeat.
		// R^2 = 0.96773.  Pretty good.
		let m = 1.281995;
		let x = system_distance / jump_range;
		let b = 0.3889492;
		m * x + b
	}
	
	fn time_to_system( system_distance: f64, jump_range: f64 ) -> f64 {
		45f64 * UnitTrade::jump_count( system_distance, jump_range )
	}
}

impl<'b> Debug for UnitTrade<'b> {
	fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
		let str = format!( "{}cr profit/ton in {}sec - buy {} for {} at {}.{}, sell for {} at {}.{}",
			self.profit_per_ton,
			self.distance_in_seconds,
			self.buy.commodity.commodity_name,
			self.buy.buy_price,
			self.buy_system.system_name,
			self.buy_station.station_name,
			self.sell.sell_price,
			self.sell_system.system_name,
			self.sell_station.station_name
		);
		
		formatter.write_str( &str )
	}
}
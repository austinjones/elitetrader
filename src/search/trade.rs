use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::cmp::min;

use data::trader::*;
use search::player_state::PlayerState;

#[derive(Clone)]
pub struct Trade<'a> {
	pub commodity_name: String,
	
	state: PlayerState<'a>,
	
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
	
	pub used_cargo: u32,
	pub profit_total: u32,
	pub profit_per_ton: u32,
	pub profit_per_min: Option<f64>,
	
	pub distance_to_system: f64,
	pub distance_to_station: f64,
	pub distance_in_seconds: f64
}

// actual trait impl
#[allow(dead_code)]
impl<'b> Trade<'b> {
	pub fn new( state: &PlayerState<'b>, buy: &'b Listing, sell: &'b Listing ) -> Trade<'b> {
		let buy_station = state.universe.get_station( &buy.station_id )
			.expect( format!("Unknown station id {}", buy.station_id ).as_str() );
		
		let buy_system = state.universe.get_system( &buy.system_id )
			.expect( format!("Unknown system id {}", buy.system_id ).as_str() );
		
		let sell_station = state.universe.get_station( &sell.station_id )
			.expect( format!("Unknown station id {}", sell.station_id ).as_str() );
		
		let sell_system = state.universe.get_system( &sell.system_id )
			.expect( format!("Unknown system id {}", sell.system_id ).as_str() );
		
		let distance_to_system = buy_system.distance( sell_system );
		let distance_to_station = sell_station.distance_to_star.unwrap_or(100u32);
		let distance_in_seconds = Trade::cost_in_seconds( state, distance_to_system, distance_to_station );
		
		let used_cargo = Trade::used_cargo( state, &buy );
		
		let profit_per_ton = Trade::profit_per_ton( &buy, &sell );
		let profit_total = profit_per_ton * used_cargo;
		
		let profit_per_min = Trade::profit_per_min( &buy, &sell, used_cargo, distance_in_seconds );
//		println!( "Using {} of {}, profit/ton {}, profit total {}, profit/min {} over {}sec",
//			used_cargo, buy.commodity.commodity_name,
//			profit_per_ton, profit_total, profit_per_min.unwrap_or(0f64), cost_in_seconds );
		
		Trade {
			commodity_name: buy.commodity.commodity_name.clone(),
			
			state: state.clone(),
			
			buy: buy,
			buy_station: buy_station,
			buy_system: buy_system,
			
			sell: sell,
			sell_station: sell_station,
			sell_system: sell_system,
			
			buy_price: buy.buy_price,
			sell_price: sell.sell_price,
			
			is_valid: Trade::is_valid( &buy, &sell, used_cargo ),
			is_prohibited: Trade::is_prohibited( &buy.commodity, &sell_station ),
			
			used_cargo: used_cargo,
			profit_total: profit_total,
			profit_per_ton: profit_per_ton,
			profit_per_min: profit_per_min,
			
			distance_to_system: distance_to_system,
			distance_to_station: distance_to_station as f64,
			distance_in_seconds: distance_in_seconds,
		}
	}
	
	pub fn score( &self ) -> Option<f64> {
		self.profit_per_min
	}
	
	pub fn state_before_trade( &self ) -> PlayerState<'b> {
		self.state.clone()
	}
	
	pub fn state_after_trade( &self ) -> PlayerState<'b> {
		self.state.with_trade( self )
	}
}

// static methods
impl<'b> Trade<'b> {
	pub fn profit_per_ton( buy: &Listing, sell: &Listing ) -> u32 {
		if sell.sell_price > buy.buy_price {
			(sell.sell_price - buy.buy_price) as u32
		} else {
			0
		}
	}
	
	pub fn is_valid( buy: &Listing, sell: &Listing, used_cargo: u32 ) -> bool {
		used_cargo > 0 
			&& buy.supply > 0 
			&& buy.buy_price != 0
			&& buy.buy_price < sell.sell_price 
			&& buy.commodity == sell.commodity
	}
	
	pub fn is_prohibited( commodity: &Commodity, sell_station: &Station ) -> bool {
		sell_station.prohibited_commodities.contains( &commodity.commodity_id )
	}
	
	pub fn used_cargo( state: &PlayerState, buy: &Listing ) -> u32 {
		if state.credit_balance < state.minimum_balance {
			return 0;
		}
		
		let possible_cargo = (state.credit_balance - state.minimum_balance) / buy.buy_price as u32;
		min( possible_cargo, state.cargo_capacity )
	}
	
	pub fn profit_per_min( buy: &Listing, sell: &Listing, used_cargo: u32, distance_in_seconds: f64  ) -> Option<f64> {
		if !Trade::is_valid( buy, sell, used_cargo ) {
			return None;
		}
		
		let profit_total = Trade::profit_per_ton( buy, sell ) * used_cargo;
		let profit_per_min = match distance_in_seconds {
			0f64 => 60f64 * profit_total as f64,
			_ => 60f64 * profit_total as f64 / distance_in_seconds
		};
		
		Some( profit_per_min )
	}
	
	pub fn cost_in_seconds( state: &PlayerState, system_distance: f64, station_distance_ls: u32 ) -> f64 {
		let dock_time = 77f64;
		let undock_time = 29f64;
		
		let jump_time = 45f64 * Trade::jump_count( system_distance, state.jump_range );
		
		let system_time = 37.39354f64 * station_distance_ls as f64;
		
		undock_time + jump_time + system_time + dock_time
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
}

impl<'b> Debug for Trade<'b> {
	fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
		let str = format!( "{}cr profit in {}sec - buy {} for {} at {}.{}, sell for {} at {}.{}",
			self.profit_total,
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
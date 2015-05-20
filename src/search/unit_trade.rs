use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;

use data::trader::*;
use data::IndexedUniverse;
use search::player_state::PlayerState;
use search::time_estimate::TimeEstimate;

use util::scored_buf::Scored;

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
	pub profit_per_ton_per_min: f64,
	
	pub normalized_time : TimeEstimate,
	pub adjusted_time: TimeEstimate
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
		
		let normalized_time = TimeEstimate::new_normalized_estimate( state, buy_system, sell_system, sell_station );
		let adjusted_time = TimeEstimate::new_adjusted_estimate( state, buy_system, sell_system, sell_station );
		
		let profit_per_ton = UnitTrade::profit_per_ton( &buy, &sell );
		
		let profit_per_ton_per_min = UnitTrade::profit_per_ton_per_min( &buy, &sell, adjusted_time.time_total );
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
			
			normalized_time: normalized_time,
			adjusted_time: adjusted_time
		}
	}
	
	pub fn with_sell_price( &self, sell_price: u16 ) -> UnitTrade<'b> {
		let mut new = self.clone();
		let mut sell = self.sell.clone();
		sell.sell_price = sell_price;
		
		let profit_per_ton = UnitTrade::profit_per_ton( self.buy, &sell );
		let profit_per_ton_per_min = UnitTrade::profit_per_ton_per_min( self.buy, &sell, self.adjusted_time.time_total );
		
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
			&& buy.commodity.commodity_id == sell.commodity.commodity_id
	}
	
	pub fn is_prohibited( commodity: &Commodity, sell_station: &Station ) -> bool {
		sell_station.prohibited_commodities.contains( &commodity.commodity_id )
	}
	
	pub fn profit_per_ton_per_min( buy: &Listing, sell: &Listing, distance_in_seconds: f64  ) -> f64 {
		if !UnitTrade::is_valid( buy, sell ) {
			return 0f64;
		}
		
		let profit_per_ton = UnitTrade::profit_per_ton( buy, sell );
		let profit_per_min = match distance_in_seconds {
			0f64 => 60f64 * profit_per_ton as f64,
			_ => 60f64 * profit_per_ton as f64 / distance_in_seconds
		};
		
		profit_per_min
	}
}

impl<'a> Scored<f64> for UnitTrade<'a> {
	fn score( &self ) -> f64 {
		self.profit_per_ton_per_min
	}
}

impl<'b> Debug for UnitTrade<'b> {
	fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
		let str = format!( "{}cr profit/ton in {}sec - buy {} for {} at {}.{}, sell for {} at {}.{}",
			self.profit_per_ton,
			self.normalized_time.time_total,
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
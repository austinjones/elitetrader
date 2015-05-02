use rand;
use rand::Rng;

use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::cmp::min;
use std::collections::LinkedList;
use std::str::FromStr;

use universe::Universe;
use data::*;
use options::{BuyOptions, SellOptions};
use scored_buf::{ScoredCircularBuffer, Sort};

pub struct Analyzer<'a> {
	pub jump_range: f64,
	pub credit_balance: u32,
	pub minimum_balance: u32,
	pub cargo_capacity: u32,
	pub universe: &'a Universe
}

#[allow(dead_code)]
impl<'a> Analyzer<'a> {
	pub fn buys_from_systems( &self, systems: Vec<&'a System> ) -> BuyOptions {
		let mut ret = BuyOptions::default();
		
		for system in systems {
			for station in &*system.stations {
				for listing in &station.listings {
					if listing.is_buy() {
						ret.push( listing );
					}
				}
			}
		}
		
		ret
	}
	
	pub fn buys_from_system( &self, system: &'a System ) -> BuyOptions {
		let mut ret = BuyOptions::default();
		
		for station in &*system.stations {
			for listing in &station.listings {
				if listing.is_buy() {
					ret.push( listing );
				}
			}
		}
		
		ret
	}
	
	pub fn buys_from_station( &self, station: &'a Station ) -> BuyOptions {
		let mut ret = BuyOptions::default();
		
		for listing in &station.listings {
			if listing.is_buy() {
				ret.push( listing );
			}
		}
		
		ret
	}
	
	pub fn sells_from_systems( &self, systems: Vec<&'a System> ) -> SellOptions {
		let mut ret = SellOptions::default();
		
		for system in systems {
			for station in &*system.stations {
				for listing in &station.listings {
					if listing.is_sell() {
						ret.push( listing );
					}
				}
			}
		}
		
		ret
	}
	
	pub fn sells_from_system( &self, system: &'a System ) -> SellOptions {
		let mut ret = SellOptions::default();
		
		for station in &*system.stations {
			for listing in &station.listings {
				if listing.is_sell() {
					ret.push( listing );
				}
			}
		}
		
		ret
	}
	
	pub fn sells_from_station( &self, station: &'a Station ) -> SellOptions {
		let mut ret = SellOptions::default();
		
		for listing in &station.listings {
			if listing.supply > 0 {
				if listing.is_sell() {
					ret.push( listing );
				}
			}
		}
		
		ret
	}
	
	pub fn best_trades_in_range(&self, station: &Station, range: f64, n: usize ) -> Vec<Trade> {
		let mut trade_buffer = ScoredCircularBuffer::new(n, Sort::Descending);
		
//		println!("best_trades_in_range - Getting systems in range");
		let system = self.universe.get_system( &station.system_id ).unwrap();
		let systems = self.universe.get_systems_in_range( &system, range );
		
//		println!("best_trades_in_range - Getting sells from systems");
		let sells = self.sells_from_systems( systems );
		
//		println!("best_trades_in_range - Grouping by commodity");
		let sells_by_commodity = sells.by_commodity();
		
//		println!("best_trades_in_range - Iterating combinations");
		for buy in self.buys_from_station(station).nodes {
			let id = buy.commodity.to_id();
			let trades = match sells_by_commodity.get( &id ) {
				Some(t) => t,
				None => { continue; }
			};
			
			for sell in trades {
				if !Trade::is_valid(&buy, sell, 1) {
					continue;
				}
				
				let sell_station = self.universe.get_station( &sell.station_id ).unwrap();
				if Trade::is_prohibited( &buy.commodity, &sell_station ) {
					continue;
				}
				
				let trade = Trade::new(self, &buy, *sell);
				let score = trade.score().unwrap_or(0f64);
				
				trade_buffer.push( trade, score );
			}
		}
		
//		println!("best_trades_in_range - Got result combinations");
		trade_buffer.sort_mut()
	}
		
	pub fn best_trades(&self, buys: &'a BuyOptions, sells: &'a SellOptions, n: usize) -> Vec<Trade> {
		let mut buffer = ScoredCircularBuffer::new(n, Sort::Descending);
		let buy_lookup = buys.by_commodity();
		
		for sell in &sells.nodes {
			let trades = match buy_lookup.get( &sell.commodity.to_id() ) {
				Some(trades) => trades,
				None => { continue; }
			};
			
			for buy in trades {
				let trade = Trade::new( &self, buy, sell );
				let score = trade.score();
				buffer.push_opt( trade, score );
			}
		}
		
		buffer.sort_mut()
	}
	
	pub fn best_next_trade<'x, 'y>(&self, station: &Station, search_range: f64, n: usize, search_depth: usize ) -> Vec<Trade> {
		match self.best_next_trade_recurse( station, search_range, n, search_depth ) {
			Some( mut routes ) => {
				let mut vec = Vec::with_capacity( routes.len() );
		
				for mut route in routes.drain() {
					match route.hops.pop_front() {
						Some( trade ) => { vec.push( trade ); }
						None => {}
					}
				}
				
				vec
			}
			None => Vec::new()
		}
	}
	
	fn best_next_trade_recurse(&self, station: &Station, range: f64, n: usize, search_depth: usize ) -> Option<Vec<TradeRoute>> {
//		let route = TradeRoute { trades: LinkedList::new() };
//		route.trades.push_front( trade );
		
		if search_depth == 0 {
			return None;
		}
		
//		println!( "depth {} - Calculating best trades from {}", search_depth, station.station_name );
		
		let best_trades = self.best_trades_in_range(station, range, n);
		let mut route_buffer = ScoredCircularBuffer::new(n, Sort::Descending);
		
		for trade in best_trades {
//			println!( "depth {} - Calculating best route for trade {:?}", search_depth, trade );
			let subroutes = match self.best_next_trade_recurse( &trade.sell_station, range, n, search_depth - 1 ) {
				Some( routes ) => routes,
				None => { 
					let mut route = TradeRoute::new();
					route.hops.push_back( trade );
					let score = route.score();
					route_buffer.push( route, score );
					continue;
				}
			};
			
			for mut route in subroutes {
				route.hops.push_front( trade.clone() );
				let score = route.score();
				route_buffer.push( route, score );
			}
		}
		
		let result = route_buffer.sort_mut();
//		println!( "result (depth {}): {:?}", search_depth, result.first() );
		Some(result)
	}
}

pub enum SearchQuality {
	Low,
	Medium,
	High
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

#[derive(Clone)]
pub struct Trade<'a> {
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
	
	pub used_cargo: u32,
	pub profit_total: u32,
	pub profit_per_ton: u32,
	pub profit_per_min: Option<f64>,
	
	pub distance_to_system: f64,
	pub distance_to_station: f64,
	pub distance_in_seconds: u32
}

#[allow(dead_code)]
impl<'b> Trade<'b> {
	pub fn new( analyzer: &Analyzer<'b>, buy: &'b Listing, sell: &'b Listing ) -> Trade<'b> {
		let buy_station = analyzer.universe.get_station( &buy.station_id ).unwrap();
		let buy_system = analyzer.universe.get_system( &buy.system_id ).unwrap();
		let sell_station = analyzer.universe.get_station( &sell.station_id ).unwrap();
		let sell_system = analyzer.universe.get_system( &sell.system_id ).unwrap();
		
		let distance_to_system = buy_system.distance( sell_system );
		let distance_to_station = sell_station.distance_to_star.unwrap_or(100u32);
		let distance_in_seconds = Trade::cost_in_seconds( analyzer, distance_to_system, distance_to_station );
		
		let used_cargo = Trade::used_cargo( analyzer, &buy );
		
		let profit_per_ton = Trade::profit_per_ton( &buy, &sell );
		let profit_total = profit_per_ton * used_cargo;
		
		let profit_per_min = Trade::profit_per_min( &buy, &sell, used_cargo, distance_in_seconds );
//		println!( "Using {} of {}, profit/ton {}, profit total {}, profit/min {} over {}sec",
//			used_cargo, buy.commodity.commodity_name,
//			profit_per_ton, profit_total, profit_per_min.unwrap_or(0f64), cost_in_seconds );
		
		Trade {
			commodity_name: buy.commodity.commodity_name.clone(),
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
	
	fn profit_per_ton( buy: &Listing, sell: &Listing ) -> u32 {
		if sell.sell_price > buy.buy_price {
			(sell.sell_price - buy.buy_price) as u32
		} else {
			0
		}
	}
	
	fn is_valid( buy: &Listing, sell: &Listing, used_cargo: u32 ) -> bool {
		used_cargo > 0 
			&& buy.supply > 0 
			&& buy.buy_price != 0
			&& buy.buy_price < sell.sell_price 
			&& buy.commodity == sell.commodity
	}
	
	fn is_prohibited( commodity: &Commodity, sell_station: &Station ) -> bool {
		sell_station.prohibited_commodities.contains( &commodity.commodity_id )
	}
	
	fn used_cargo( analyzer: &Analyzer, buy: &Listing ) -> u32 {
		let possible_cargo = (analyzer.credit_balance - analyzer.minimum_balance) / buy.buy_price as u32;
		min( possible_cargo, analyzer.cargo_capacity )
	}
	
	fn profit_per_min( buy: &Listing, sell: &Listing, used_cargo: u32, distance_in_seconds: u32  ) -> Option<f64> {
		if !Trade::is_valid( buy, sell, used_cargo ) {
			return None;
		}
		
		let profit_total = Trade::profit_per_ton( buy, sell ) * used_cargo;
		let profit_per_min = match distance_in_seconds {
			0 => 60f64 * profit_total as f64,
			_ => 60f64 * profit_total as f64 / distance_in_seconds as f64
		};
		
		// try to prevent getting stuck in local minima
		let random_factor = rand::thread_rng().gen_range(0.95f64, 1.05f64);
		
		Some( profit_per_min * random_factor )
	}
	
	fn cost_in_seconds( analyzer: &Analyzer, system_distance : f64, station_distance_ls: u32 ) -> u32 {
		let undock_time = 29u32;
		let dock_time = 77u32;
		
		let jumps = system_distance / analyzer.jump_range;
		// we won't get a perfect jump every time.  add 50% to account for this.
		let jump_time = 1.5f64 * jumps * 45f64;
		
		let system_time = 37.39354f32 * station_distance_ls as f32;
		
		undock_time + (jump_time as u32) + (system_time as u32) + dock_time
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

#[derive(Clone, Debug)]
pub struct TradeRoute<'b> {
	pub hops: LinkedList<Trade<'b>>
}

#[allow(dead_code)]
impl<'b> TradeRoute<'b> {
	pub fn new() -> TradeRoute<'b> {
		TradeRoute { hops: LinkedList::new() }
	}
	
	pub fn with_trade( trade: Trade<'b> ) -> TradeRoute<'b> {
		let mut route : TradeRoute = TradeRoute { hops: LinkedList::new() };
		route.hops.push_back( trade );
		route
	}
	
	pub fn profit_total( &self ) -> u32 {
		self.hops.iter()
			.map( |e| e.profit_total )
			.fold( 0, |a,b| a+b )
	}
	
	pub fn profit_per_ton( &self ) -> u32 {
		self.hops.iter()
			.map( |e| e.profit_per_ton )
			.fold( 0, |a,b| a+b )
	}
	
	pub fn profit_per_minute( &self ) -> f64 {
		self.hops.iter()
			.map( |e| e.profit_per_min.unwrap_or(0f64) )
			.fold( 0f64, |a,b| a+b )
	}
	
	pub fn score( &self ) -> f64 {
		self.hops.iter()
			.map( |e| e.score().unwrap_or(0f64) )
			.fold( 0f64, |a,b| a+b )
	}
	
	pub fn distance_in_seconds( &self ) -> u32 {
		self.hops.iter()
			.map( |e| e.distance_in_seconds )
			.fold( 0, |a,b| a+b )
	}
	
	pub fn distance_to_system( &self ) -> f64 {
		self.hops.iter()
			.map( |e| e.distance_to_system )
			.fold( 0f64, |a,b| a+b )
	}
	
	pub fn distance_to_station( &self ) -> f64 {
		self.hops.iter()
			.map( |e| e.distance_to_station )
			.fold( 0f64, |a,b| a+b )
	}
		
	pub fn is_valid( &self ) -> bool {
		self.hops.iter()
			.map(|e| e.is_valid )
			.fold(true, |a,b| a && b )
	}
		
	pub fn is_prohibited( &self ) -> bool {
		self.hops.iter()
			.map(|e| e.is_prohibited )
			.fold(true, |a,b| a || b )
	}
}
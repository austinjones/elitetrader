use rand::{thread_rng, Rng};
use std::collections::HashMap;

use std::thread;
use data::trader::*;
use search::trade::Trade;
use search::search_quality::SearchQuality;
use search::player_state::PlayerState;

use util::scored_buf::*;

pub struct SearchResult<'a> {
	pub trade: Trade<'a>,
	pub profit_total: u32,
	pub distance_in_seconds: f64
}

impl<'a> SearchResult<'a> {
	pub fn new( trade: Trade<'a> ) -> SearchResult<'a> {
		let profit_total = trade.profit_total;
		let distance_in_seconds = trade.distance_in_seconds;
		
		SearchResult {
			trade: trade,
			profit_total: profit_total,
			distance_in_seconds: distance_in_seconds
		}
	}
	
	pub fn with_trade( &self, trade: Trade<'a> ) -> SearchResult<'a> {
		let profit_total = self.profit_total + trade.profit_total;
		let distance_in_seconds = self.distance_in_seconds + trade.distance_in_seconds;
		
		SearchResult {
			trade: trade.clone(),
			profit_total: profit_total,
			distance_in_seconds: distance_in_seconds
		}
	}
	
	fn fudge( val: f64, fudge_factor: f64 ) -> f64 {
		val * thread_rng().gen_range( 1f64 - fudge_factor, 1f64 + fudge_factor )
	}
}

impl<'a> Scored<f64> for SearchResult<'a> {
	fn score( &self ) -> f64 {
		let val = match self.distance_in_seconds {
			0f64 => panic!("Cannot score result with 0 distance_in_seconds"),
			_ => self.profit_total as f64 / self.distance_in_seconds
		};
		
		SearchResult::fudge( val, 0.02 )
	}
}

pub struct SearchTrade<'a> {
	pub trade: Trade<'a>,
	pub sell_station: SearchStation<'a>
}


pub struct SearchStation<'a> {
	pub state: PlayerState<'a>,
	pub search_quality: SearchQuality,
	pub best_1hop_trades:  Option<Vec<SearchTrade<'a>>>
}

impl<'a> SearchStation<'a> {
	pub fn new( state: PlayerState<'a>, search_quality: SearchQuality ) -> SearchStation<'a> {
		SearchStation {
			state: state,
			search_quality: search_quality,
			best_1hop_trades: None
		}
	}
	
	fn fill_to_depth( &mut self, depth: usize ) {
//		if depth == 0 {
//			return;
//		}
		
//		let station_id = self.state.station.station_id;
		let mut cache = HashMap::new();
		
		self.fill_to_depth_recurse( &mut cache, depth );
		
//		println!("Fill complete.  Hash has {} entries", cache.len());
//		match &self.best_1hop_trades {
//			&Some(_) => {},
//			&None => {
//				let mut trades = SearchStation::best_1hop_trades( &self.state, self.search_quality );
//				cache.insert( station_id, trades.clone() );
//				let search_trades = trades.drain().map(|e| self.new_trade( e ) ).collect();
//				self.best_1hop_trades = Some(search_trades);
//			}
//		};
//		
//		let mut trades = match &mut self.best_1hop_trades {
//			&mut Some(ref mut v) => v,
//			&mut None => panic!("SearchStation.best_1hop_trades should have just been assigned")
//		};
//
////		let mut guards = Vec::with_capacity( trades.len() );
//
//		for search_trade in trades.iter_mut() {
////			let guard = thread::scoped(move || search_trade.sell_station.fill_to_depth_recurse( &mut cache, depth - 1 ) );
//			search_trade.sell_station.fill_to_depth_recurse( &mut cache, depth - 1 ) 
////			guards.push( guard );
//		}
//		
////		for guard in guards.drain() {
////			guard.join();
////		}
	}
	
	fn fill_to_depth_recurse( &mut self, cache: &mut HashMap<u32, Vec<Trade<'a>>>, depth: usize ) {
		if depth == 0 {
			return;
		}
		
//		let cache = match cache {
//			Some(v) => v,
//			None => HashMap::new()
//		};
		
		let station_id = self.state.station.station_id;
		
		let insert = cache.get(&station_id).is_none();
		if insert {
			let mut trades = SearchStation::best_1hop_trades( &self.state, self.search_quality );
			cache.insert( station_id, trades.clone() );
			
			let search_trades = trades.drain().map(|t| self.new_trade( t ) ).collect();
			self.best_1hop_trades = Some(search_trades);
		} else {
			let trades = cache.get(&station_id).unwrap();
			
			let search_trades = trades.iter().map(|t| self.new_trade( t.clone() ) ).collect();
			self.best_1hop_trades = Some(search_trades);
		};
		
		let mut trades = match &mut self.best_1hop_trades {
			&mut Some(ref mut v) => v,
			&mut None => panic!("SearchStation.best_1hop_trades should have just been assigned")
		};
		
		for search_trade in trades.iter_mut() {
			search_trade.sell_station.fill_to_depth_recurse( cache, depth - 1 );
		}
	}
		
	pub fn next_trade(mut self) -> Option<SearchTrade<'a>> {
		let depth = self.search_quality.get_depth();
		self.fill_to_depth( depth );
		
		if !self.best_1hop_trades.is_some() {
			return None;
		}
		
		//  we only need the top result
		let mut route_buffer = ScoredCircularBuffer::new( 1, Sort::Descending );
		
		let mut trades = self.best_1hop_trades.unwrap();
		let mut trade_index : Option<usize> = None;
		
		for search_trade in trades.iter() {
			let current_index = match trade_index {
				None => 0,
				Some(k) => k+1
			};
			
			trade_index = Some(current_index);
			
			let subresults = match search_trade.sell_station.next_trade_recurse() {
				Some( results ) => results,
				None => { 
					let result = SearchResult::new( search_trade.trade.clone() );
					route_buffer.push( current_index, result.score() );
					continue;
				}
			};
			
			for subresult in subresults {
				let sub = subresult.with_trade( search_trade.trade.clone() );
				route_buffer.push( current_index, sub.score() );
			}
		}
		
		let vals = route_buffer.sort_mut();
		match vals.iter().next() {
			Some(index) => trades.drain().nth( *index ),
			None => None
		}
	}
	
	fn next_trade_recurse(&self) -> Option<Vec<SearchResult<'a>>> {
		if !self.best_1hop_trades.is_some() {
			return None;
		}
		
		// we only need the top result
		let mut route_buffer = ScoredCircularBuffer::new( 1, Sort::Descending );
		
		for ref opt in self.best_1hop_trades.iter() {
			for ref search_trade in opt.iter() {
				let trade = &search_trade.trade;
				
				let subresults = match search_trade.sell_station.next_trade_recurse() {
					Some( results ) => results,
					None => { 
						route_buffer.push_scored( SearchResult::new( trade.clone() ) );
						continue;
					}
				};
				
				for subresult in subresults {
					route_buffer.push_scored( subresult.with_trade( trade.clone() ) );
				}
			}
		}
		
		Some( route_buffer.sort_mut() )
	}
	
	fn new_trade( &self, trade: Trade<'a> ) -> SearchTrade<'a> {
		if trade.buy_station.to_id() != self.state.station.to_id() {
			panic!("Cannot create trade that originates from a different station");
		}
		
		let sell_state = self.state.with_trade( &trade );
		let sell_station = SearchStation::new( sell_state, self.search_quality );
		
		SearchTrade {
			trade: trade,
			sell_station: sell_station
		}
	}
	
	fn best_1hop_trades( state: &PlayerState<'a>, search_quality: SearchQuality ) -> Vec<Trade<'a>> {
		let hop_width = search_quality.get_hop_width();
		let trade_range = search_quality.get_trade_range();
		
		let mut trade_buffer = ScoredCircularBuffer::new( hop_width, Sort::Descending );
		
//		println!("best_trades_in_range - Getting systems in range");
		let system = match state.universe.get_system( &state.station.system_id ) {
			Some(system) => system,
			None => panic!( "Unknown station id {}", &state.station.system_id )
		};
		
		let systems = state.universe.get_systems_in_range( &system, trade_range );
		
//		println!("best_trades_in_range - Getting sells from systems");
		let sells = state.universe.sells_from_systems( systems );
		
//		println!("best_trades_in_range - Grouping by commodity");
		let sells_by_commodity = sells.by_commodity();
		
//		println!("best_trades_in_range - Iterating combinations");
		for buy in state.universe.buys_from_station(state.station).nodes {
			let id = buy.commodity.to_id();
			let trades = match sells_by_commodity.get( &id ) {
				Some(t) => t,
				None => { continue; }
			};
			
			for sell in trades {
				if !Trade::is_valid(&buy, sell, 1) {
					continue;
				}
				
				let sell_station = match state.universe.get_station( &sell.station_id ) {
					Some(station) => station,
					None => { continue; }
				};
				
				if Trade::is_prohibited( &buy.commodity, &sell_station ) {
					continue;
				}
				
				let trade = Trade::new( &state, &buy, *sell);
				// this is rare, so it's not worth calculating before we build the trade
				if trade.used_cargo == 0 {
					continue;
				}
				
				let score = trade.score().unwrap_or(0f64);
				
				trade_buffer.push( trade, score );
			}
		}
		
//		println!("best_trades_in_range - Got result combinations");
		trade_buffer.sort_mut()
	}
}
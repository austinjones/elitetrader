use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::cmp::max;

use data::trader::*;
use data::IndexedUniverse;

use search::trade::FullTrade;
use search::trade::UnitTrade;
use search::search_quality::SearchQuality;
use search::player_state::PlayerState;

use util::scored_buf::*;

pub struct SearchResult<'a> {
	pub trade: FullTrade<'a>,
	pub profit_total: u32,
	pub distance_in_seconds: f64
}

impl<'a> SearchResult<'a> {
	pub fn new( trade: FullTrade<'a> ) -> SearchResult<'a> {
		let profit_total = trade.profit_total;
		let distance_in_seconds = trade.unit.distance_in_seconds;
		
		SearchResult {
			trade: trade,
			profit_total: profit_total,
			distance_in_seconds: distance_in_seconds
		}
	}
	
	pub fn with_trade( &self, trade: FullTrade<'a> ) -> SearchResult<'a> {
		let profit_total = self.profit_total + trade.profit_total;
		let distance_in_seconds = self.distance_in_seconds + trade.unit.distance_in_seconds;
		
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
	pub trade: FullTrade<'a>,
	pub sell_station: SearchStation
}

pub struct SearchStation {
	pub state: PlayerState,
	pub search_quality: SearchQuality
}

impl<'a> SearchStation {
	pub fn new( state: PlayerState, search_quality: SearchQuality ) -> SearchStation {
		SearchStation {
			state: state,
			search_quality: search_quality
		}
	}
	
	fn get_1hop_trades( &self, iuniverse: &'a IndexedUniverse, cache: &mut HashMap<u32, Vec<UnitTrade<'a>>> ) -> Vec<UnitTrade<'a>> {
		let station_id = self.state.station_id;
		
		let insert = cache.get(&station_id).is_none();
		if insert {
			let mut trades = SearchStation::best_1hop_trades( iuniverse, &self.state, self.search_quality );
			cache.insert( station_id, trades.clone() );
			
			trades
		} else {
			let trades = cache.get(&station_id).unwrap();
			
			trades.clone()
		}
	}
	
	pub fn next_trades(&mut self, iuniverse: &'a IndexedUniverse ) -> Vec<FullTrade<'a>> {
		let depth = self.search_quality.get_depth();
		
		let mut cache = HashMap::new();
		
		let trades = self.next_trades_recurse( iuniverse, &mut cache, depth );
		match trades {
			Some(mut vec) => vec.drain().map( |e| e.trade ).collect(),
			None => Vec::new()
		}
	}
	
	fn next_trades_recurse(&self, iuniverse: &'a IndexedUniverse, 
				mut cache: &mut HashMap<u32, Vec<UnitTrade<'a>>>, depth: usize) -> Option<Vec<SearchResult<'a>>> {
		if depth == 0 {
			return None;
		}
		
		// we only need the top result
		let mut route_buffer = ScoredCircularBuffer::new( self.search_quality.get_hop_width(), Sort::Descending );
		
		for unit_trade in self.get_1hop_trades( iuniverse, &mut cache ) {
			let new_search_trade = self.new_trade( unit_trade );
			if !new_search_trade.trade.is_valid {
				continue;
			}
			
			let trade = new_search_trade.trade;
			
			let subresults = match new_search_trade.sell_station.next_trades_recurse( iuniverse, cache, depth - 1 ) {
				Some( results ) => results,
				None => { 
					route_buffer.push_scored( SearchResult::new( trade.clone() ) );
					continue;
				}
			};
			
			let mut subresult_buffer = ScoredCircularBuffer::new( 1usize, Sort::Descending );
			for subresult in subresults {
				subresult_buffer.push_scored( subresult.with_trade( trade.clone() ) );
			}
			
			let mut best_subresults = subresult_buffer.sort_mut();
			match best_subresults.drain().next() {
				Some(v) => { route_buffer.push_scored(v); },
				None => {}
			};
		}
		
		Some( route_buffer.sort_mut() )
	}
	
	fn new_trade( &self, unit: UnitTrade<'a> ) -> SearchTrade<'a> {
		if unit.buy_station.to_id() != self.state.station_id {
			panic!("Cannot create trade that originates from a different station");
		}
		
		let trade = FullTrade::new( &self.state, unit );
		
		let sell_state = self.state.with_trade( &trade );
		let sell_station = SearchStation::new( sell_state, self.search_quality );
		
		SearchTrade {
			trade: trade,
			sell_station: sell_station
		}
	}
	
	fn best_1hop_trades( iuniverse : &'a IndexedUniverse, state: &PlayerState, search_quality: SearchQuality ) -> Vec<UnitTrade<'a>> {
		let hop_width = search_quality.get_hop_width();
		let trade_range = search_quality.get_trade_range();
		
		let mut trade_buffer = ScoredCircularBuffer::new( hop_width, Sort::Descending );
		
		let station = match iuniverse.get_station( &state.station_id ) {
			Some(v) => v,
			None => panic!("Unknown station id {}", &state.station_id)
		};
		
//		println!("best_trades_in_range - Getting systems in range");
		let system = match iuniverse.get_system( &state.system_id ) {
			Some(system) => system,
			None => panic!( "Unknown system id {}", &state.system_id )
		};
		
		let systems = iuniverse.get_systems_in_range( &system, trade_range );
		
//		println!("best_trades_in_range - Getting sells from systems");
		let sells = iuniverse.sells_from_systems( systems );
		
//		println!("best_trades_in_range - Grouping by commodity");
		let sells_by_commodity = sells.by_commodity();
		
//		println!("best_trades_in_range - Iterating combinations");
		for buy in iuniverse.buys_from_station(station).nodes {
			let id = buy.commodity.to_id();
			let trades = match sells_by_commodity.get( &id ) {
				Some(t) => t,
				None => { continue; }
			};
			
			for sell in trades {
				if !UnitTrade::is_valid(&buy, sell) {
					continue;
				}
				
				let sell_station = match iuniverse.get_station( &sell.station_id ) {
					Some(station) => station,
					None => { continue; }
				};
				
				if UnitTrade::is_prohibited( &buy.commodity, &sell_station ) {
					continue;
				}
				
				let trade = UnitTrade::new( &iuniverse, &state, &buy, *sell);
				let score = trade.profit_per_ton;
				
				trade_buffer.push( trade, score );
			}
		}
		
//		println!("best_trades_in_range - Got result combinations");
		trade_buffer.sort_mut()
	}
}
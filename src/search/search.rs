//TODO: optimize route cycles?

use rand::{thread_rng, Rng};
use std::collections::HashMap;

use data::trader::*;
use data::IndexedUniverse;

use search::full_trade::FullTrade;
use search::unit_trade::UnitTrade;
use search::search_quality::SearchQuality;
use search::player_state::PlayerState;

use util::scored_buf::*;

pub struct SearchCache<'a> {
	pub trade_cache: HashMap<u32, Vec<UnitTrade<'a>>>,
//	convergence_filter: VecMap<HashMap<usize, f64>>
}

impl<'a> SearchCache<'a> {
	pub fn new() -> SearchCache<'a> {
		SearchCache {
			trade_cache: HashMap::new(),
//			convergence_filter: VecMap::with_capacity(MAX_DEPTH)
		}
	}
	
//	pub fn convergence_check( &mut self, result: &SearchResult<'a>, depth: usize ) -> bool {
//		match self.convergence_filter.get( &depth ) {
//			None => {
//				let map = HashMap::new();
//				self.convergence_filter.insert(depth, map);
//			}
//			_ => {}
//		};
//		
//		let depthmap = match self.convergence_filter.get_mut( &depth ) {
//			Some(m) => m,
//			None => panic!("Should have created a depth map in convergence_filter")
//		};
//		
//		let sell_station_id = result.trade.unit.sell_station.station_id as usize;
//		let score = result.profit_total as f64 / result.distance_in_seconds;
//		
//		let is_better = match depthmap.get( &sell_station_id ) {
//			Some(best) => *best < score,
//			None => true
//		};
//		
//		if is_better {
//			depthmap.insert( sell_station_id, score );
//		}
//		
//		is_better
//	}
	
	fn get_1hop_trades( &mut self, iuniverse: &'a IndexedUniverse, 
			station: &SearchStation ) -> Vec<UnitTrade<'a>> {
		let station_id = station.state.station_id;
		
		let insert = self.trade_cache.get(&station_id).is_none();
		if insert {
			let trades = SearchCache::best_1hop_trades( iuniverse, &station.state, station.search_quality );
			self.trade_cache.insert( station_id, trades.clone() );
			
			trades
		} else {
			let trades = self.trade_cache.get(&station_id).unwrap();
			
			trades.clone()
		}
	}
			
	fn best_1hop_trades( iuniverse : &'a IndexedUniverse, state: &PlayerState, search_quality: SearchQuality ) -> Vec<UnitTrade<'a>> {
		let hop_width = search_quality.get_hop_width();
		let trade_range = search_quality.get_trade_range();
		
		let mut trade_buffer = ScoredCircularBuffer::new( hop_width, Sort::Descending );
//		let mut trade_buffer = ScoredCircularBuffer::new( hop_width, Sort::Descending );
		
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

pub struct SearchResult<'a> {
	pub trade: FullTrade<'a>,
	pub profit_total: u32,
	pub time_total: f64
}

impl<'a> SearchResult<'a> {
	pub fn new( trade: FullTrade<'a> ) -> SearchResult<'a> {
		let profit_total = trade.profit_total;
		let time_total = trade.unit.adjusted_time.time_total;
		
		SearchResult {
			trade: trade,
			profit_total: profit_total,
			time_total: time_total
		}
	}
	
	pub fn with_trade( &self, trade: &FullTrade<'a> ) -> SearchResult<'a> {
		let profit_total = self.profit_total + trade.profit_total;
		let distance_in_seconds = self.time_total + trade.unit.adjusted_time.time_total;
		
		SearchResult {
			trade: trade.clone(),
			profit_total: profit_total,
			time_total: distance_in_seconds
		}
	}
	
	pub fn with_score( &self, other: &SearchResult<'a> ) -> SearchResult<'a> {
		let profit_total = self.profit_total + other.profit_total;
		let distance_in_seconds = self.time_total + other.time_total;
		
		SearchResult {
			trade: self.trade.clone(),
			profit_total: profit_total,
			time_total: distance_in_seconds
		}
	}
	
	fn fudge( val: f64, fudge_factor: f64 ) -> f64 {
		val * thread_rng().gen_range( 1f64 - fudge_factor, 1f64 + fudge_factor )
	}
}

impl<'a> Scored<f64> for SearchResult<'a> {
	fn score( &self ) -> f64 {
		let val = match self.time_total {
			0f64 => panic!("Cannot score result with 0 distance_in_seconds"),
			_ => self.profit_total as f64 / self.time_total
		};
		
		val
//		SearchResult::fudge( val, 0.02 )
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
	
	pub fn next_trades(&mut self, iuniverse: &'a IndexedUniverse ) -> Vec<FullTrade<'a>> {
		let depth = self.search_quality.get_depth();
		
		let mut cache = SearchCache::new();
		
		let trades = self.next_trades_recurse( iuniverse, &mut cache, depth );
		match trades {
			Some(mut buffer) => {
				let t = buffer.sort_mut();
				t.iter().map(|e| e.trade.clone() ).collect()
			},
			None => Vec::new()
		}
	}
	
	fn next_trades_recurse(&self, iuniverse: &'a IndexedUniverse, 
				mut cache: &mut SearchCache<'a>, depth: usize) -> Option<ScoredCircularBuffer<f64, SearchResult<'a>>> {
		if depth == 0 {
			return None;
		}
		
		let hop_width = self.search_quality.get_hop_width();
		// we only need the top result
		let mut route_buffer = ScoredCircularBuffer::new( hop_width, Sort::Descending );
		// this method is complicated, so the number postfixes are the 'depth'
		// depth 1 is the next trade, and depth 2 is the trade after that...
		// we are looking for the best depth 1 trades with the highest score,
		// including the best depth 2, best depth 3, ... best depth N trades.
		for unit_trade_1 in cache.get_1hop_trades( iuniverse, &self ) {
			let search_trade_1 = self.new_trade( unit_trade_1 );
			if !search_trade_1.trade.is_valid {
				continue;
			}
			
			let full_trade_1 = search_trade_1.trade;
			let result_1 = SearchResult::new( full_trade_1.clone() );
			
//			if !cache.convergence_check( &result_with_trade, depth ) {
//				continue;
//			}
			
			let mut results_2 = match search_trade_1.sell_station.next_trades_recurse( iuniverse, cache, depth - 1 ) {
				Some( r ) => r,
				None => { 
					route_buffer.push_scored( SearchResult::new( full_trade_1 ) );
					continue;
				}
			};
			
			let mut best_score_with_2_buffer = ScoredCircularBuffer::new( 1usize, Sort::Descending );
			for result_2 in results_2.drain().map(|e| e.value ) {
				let result_1_with_2 = result_1.with_score( &result_2 );
				best_score_with_2_buffer.push_scored( result_1_with_2 );
			}
			
			let mut best_1 = best_score_with_2_buffer.sort_mut();
			match best_1.drain().next() {
				Some(v) => { route_buffer.push_scored(v); },
				None => {}
			};
		}
		
		if route_buffer.len() > 0 {
			Some( route_buffer )
		} else {
			None
		}
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
}
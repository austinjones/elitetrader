use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use rand::{thread_rng, Rng};

use data::trader::*;
use search::options::*;
use data::Universe;

use search::unit_trade::UnitTrade;
use search::search_quality::SearchQuality;
use search::player_state::PlayerState;
use search::search::SearchStation;

use util::scored_buf::*;

pub struct CachedTrade {
	pub buy: (usize, usize, usize),
	pub sell: (usize, usize, usize)
}

impl CachedTrade {
	pub fn to_unit_trade<'a>( &self, universe: &'a Universe, player_state: &PlayerState ) -> UnitTrade<'a> {
		let (index_buy_system, index_buy_station, index_buy_listing) = self.buy;
		let (index_sell_system, index_sell_station, index_sell_listing) = self.sell;
		
		let buy_system = universe.get_system_by_index( Some(index_buy_system) ).unwrap();
		let buy_station = &buy_system.stations[index_buy_station];
		let buy_listing = &buy_station.listings[index_buy_listing];
		
		let sell_system = universe.get_system_by_index( Some(index_sell_system) ).unwrap();
		let sell_station = &sell_system.stations[index_sell_station];
		let sell_listing = &sell_station.listings[index_sell_listing];
		
		UnitTrade::new_unpacked( player_state, 
			buy_system, buy_station, buy_listing,
			sell_system, sell_station, sell_listing )
	}	
}

pub struct SearchCache {
	trade_cache: RwLock<HashMap<u32, Vec<CachedTrade>>>,
	sell_lookup: RwLock<HashMap<u32, HashSet<u32>>>
//	convergence_filter: VecMap<HashMap<usize, f64>>
}

impl SearchCache {
	fn to_cached_trade<'a>( trade: &UnitTrade, universe: &'a Universe  ) -> CachedTrade {
		let buy_index = universe.get_index()
			.get_index_listing( trade.buy_station.station_id, trade.commodity_id );
		let sell_index = universe.get_index()
			.get_index_listing( trade.sell_station.station_id, trade.commodity_id );
		
		// TODO: handle unsafe unwrap
		CachedTrade {
			buy: buy_index.unwrap(),
			sell: sell_index.unwrap()
		}
	}
	
	pub fn new() -> SearchCache {
		SearchCache {
			trade_cache: RwLock::new(HashMap::new()),
			sell_lookup: RwLock::new(HashMap::new())
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
	
	fn update_sell_lookup( &self, trades: &Vec<UnitTrade> ) {
		let mut lookup = self.sell_lookup.write().unwrap();
		
		for trade in trades {
			let key = trade.sell_station.station_id;
			let value = trade.buy_station.station_id;
			
			let insert = lookup.get( &key ).is_none();
			
			if insert {
				let mut stations = HashSet::new();
				stations.insert( value );
				lookup.insert( key, stations );
			} else {
				let stations = lookup.get_mut( &key ).unwrap();
				stations.insert( value );
			}
		}
	}
	
	pub fn invalidate_station( &self, station_id: u32 ) {
		// TODO: automatically sync with Universe, when prices are updated
		self.trade_cache.write().unwrap()
			.remove( &station_id );
		let mut sell_lookup = self.sell_lookup.write().unwrap();
		match sell_lookup.get_mut( &station_id ) {
			Some(mut entries) => {
				for buy_station_id in entries.drain() {
					self.trade_cache.write().unwrap()
						.remove( &buy_station_id );
				}
			},
			_ => {}
		}
	}
	
	#[allow(unused)]
	pub fn random_1hop_trade<'a>( universe: &'a Universe, state: &PlayerState ) -> Option<UnitTrade<'a>> {
		let system = state.get_system( universe );
		let station = state.get_station( universe );
		
		let systems = universe.get_systems_in_range( &system, state.jump_range );
		
		let buys = BuyOptions::buys_from_station(station).nodes;
		
		let sells = SellOptions::sells_from_systems( systems );
		let sells_by_commodity = sells.by_commodity();
		
		for _ in 0..(buys.len() * sells.len()) {
			let buy = match thread_rng().choose( &buys[..] ) {
				Some(v) => v,
				None => {break;}
			};
			
			let id = buy.commodity.to_id();
			let sell_options = match sells_by_commodity.get( &id ) {
				Some(t) => t,
				None => { continue; }
			};
			
			let sell_option = match thread_rng().choose( &sell_options[..] ) {
				Some(v) => v,
				None => {continue;}
			};
			
			if !UnitTrade::is_valid(&buy, sell_option) {
				continue;
			}
			
			let sell_station = match universe.get_station( sell_option.station_id ) {
				Some(station) => station,
				None => { continue; }
			};
			
			//TODO: allow smuggling
			if UnitTrade::is_prohibited( &buy.commodity, &sell_station ) {
				continue;
			}
			
			return Some(UnitTrade::new( &universe, &state, &buy, *sell_option));
		}
		
		None
	}
	
	pub fn get_1hop_trades<'a>( &self, universe: &'a Universe, 
			station: &SearchStation ) -> Vec<UnitTrade<'a>> {
		let station_id = station.state.station_id;
		
		let insert = self.trade_cache.read().unwrap()
			.get(&station_id).is_none();
			
		if insert {
			let trades = SearchCache::best_1hop_trades( universe, &station.state, station.search_quality );
			let cache_list = trades.iter().map(|e| SearchCache::to_cached_trade( e, universe ) ).collect();
			
			self.trade_cache.write().unwrap()
				.insert( station_id, cache_list );
			self.update_sell_lookup( &trades );
			
			trades
		} else {
			let trade_cache = self.trade_cache.read().unwrap();
			let trades = trade_cache.get(&station_id).unwrap();
			
			trades.iter().map(|e| e.to_unit_trade( universe, &station.state ) ).collect()
		}
	}
	
	fn best_1hop_trades<'a>( universe : &'a Universe, state: &PlayerState, search_quality: SearchQuality ) -> Vec<UnitTrade<'a>> {
		let system = state.get_system( universe );
		let station = state.get_station( universe );
		
		let systems = universe.get_systems_in_range( &system, search_quality.get_trade_range() );
		
		let mut trade_buffer = ScoredCircularBuffer::new( search_quality.get_hop_width(), Sort::Descending );
		
//		println!("best_trades_in_range - Getting sells from systems");
		let sells = SellOptions::sells_from_systems( systems );
		
//		println!("best_trades_in_range - Grouping by commodity");
		let sells_by_commodity = sells.by_commodity();
		
//		println!("best_trades_in_range - Iterating combinations");
		for buy in BuyOptions::buys_from_station(station).nodes {
			let id = buy.commodity.to_id();
			let trades = match sells_by_commodity.get( &id ) {
				Some(t) => t,
				None => { continue; }
			};
			
			for sell in trades {
				if !UnitTrade::is_valid(&buy, sell) {
					continue;
				}
				
				let sell_station = match universe.get_station( sell.station_id ) {
					Some(station) => station,
					None => { continue; }
				};
				
				//TODO: allow smuggling
				if UnitTrade::is_prohibited( &buy.commodity, &sell_station ) {
					continue;
				}
				
				let trade = UnitTrade::new( &universe, &state, &buy, *sell);
				let score = trade.score();
				trade_buffer.push_bucket( trade, score, |t| t.sell_station.station_id );
			}
		}
		
//		println!("best_trades_in_range - Got result combinations");
//		let results = trade_buffer.sort_mut();
//		spot check the results
//		if thread_rng().gen_range( 0f64, 1f64 ) < 0.01f64 {
//			println!("best_1hop_trades for {} - {}", system.system_name, station.station_name );
//			println!("pft/min\tpft/ton\ttime\tcommodity\tsystem\tstation" );
//			for unit in results.iter() {
//				println!("{:.1}\t{:.0}\t{:.0}\t{}\t{}\t{}",
//					unit.score(),
//					unit.profit_per_ton,
//					unit.adjusted_time.time_total,
//					unit.commodity_name,
//					unit.sell_system.system_name,
//					unit.sell_station.station_name
//				);
//			}
//			
//			println!("");
//		}
//		results
		
		trade_buffer.sort_mut()
	}
	
	pub fn len(&self) -> usize {
		self.trade_cache.read().unwrap()
			.len()
	}
}
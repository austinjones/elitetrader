use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;

use crossbeam;
use num_cpus;

use std::collections::HashSet;
use rand::{thread_rng, Rng};

use data::trader::*;
use data::Universe;

use search::full_trade::FullTrade;
use search::unit_trade::UnitTrade;
use search::search_quality::SearchQuality;
use search::player_state::PlayerState;
use search::search_cache::SearchCache;

use util::num_unit::NumericUnit;
use util::scored_buf::*;

use std::cmp::*;

#[derive(Clone)]
pub struct SearchCycleTracker {
	cycles: Vec<SearchCycleElement>
}

impl SearchCycleTracker {
	pub fn new(search_quality: &SearchQuality) -> SearchCycleTracker {
		SearchCycleTracker {
			cycles: Vec::with_capacity(search_quality.get_depth())
		}
	}
	
	pub fn push( &self, trade: &FullTrade ) -> SearchCycleTracker {
		let mut new_tracker = self.clone();
		new_tracker.cycles.push( SearchCycleElement::new( trade ) );
		new_tracker
	}
	
	pub fn find_cycle( &self, trade: &FullTrade, depth_remaining: usize ) -> Option<SearchCycle> {
		let mut profit_total = 0u32;
		let mut time_total = 0f64;
		let mut cycle_length = 0usize;
		
		for elem in self.cycles.iter().rev() {
			if !elem.is_cyclic {
				return None;
			}
			
			cycle_length += 1;
			profit_total += elem.profit_total;
			time_total += elem.time_total;
			
			if elem.buy_station_id == trade.unit.buy.station_id {
				//println!("Already visited station {} - aborting route", &search_trade_1.trade.unit.sell_station.station_name );
				return Some( SearchCycle {
					profit_total: ((depth_remaining as f64 / cycle_length as f64) * profit_total as f64) as u32,
					time_total: (depth_remaining as f64 / cycle_length as f64) * time_total as f64
				} );
			}
		}
		
		None
	}
}

pub struct SearchCycle {
	pub profit_total: u32,
	pub time_total: f64
}

#[derive(Clone)]
struct SearchCycleElement {
	pub buy_station_id: u32,
	pub profit_total: u32,
	pub time_total: f64,
	pub is_cyclic: bool
}

impl SearchCycleElement {
	pub fn new<'a>( trade: &FullTrade<'a> ) -> SearchCycleElement {
		SearchCycleElement {
			buy_station_id: trade.unit.buy.station_id,
			profit_total: trade.profit_total,
			time_total: trade.unit.adjusted_time.time_total,
			is_cyclic: trade.is_cyclic
		}
	}
}
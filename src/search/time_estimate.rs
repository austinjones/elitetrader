use std::fmt::Error;

use data::trader::*;
use search::player_state::PlayerState;

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct TimeEstimate {
	pub distance_to_system: f64,
	pub distance_to_station: f64,
	pub jump_count: f64,
	pub time_to_station: f64,
	pub time_to_system: f64,
	pub time_total: f64
}
const BUY_TIME : f64 = 36.92425f64;
const UNDOCK_TIME : f64 = 77.749256f64;
const DOCK_TIME : f64 = 56.52650f64;
const SELL_TIME : f64 = 24.80750f64;

impl<'a> TimeEstimate {
	pub fn new_normalized_estimate( state: &PlayerState, buy_system: &'a System, 
			sell_system: &'a System, sell_station: &'a Station ) -> TimeEstimate {
		let distance_to_system = buy_system.distance( sell_system );
		let distance_to_station = sell_station.distance_to_star.unwrap_or(DEFAULT_STATION_DISTANCE);
		
		let jump_count = Self::jump_count( distance_to_system, state.jump_range );
		
		let time_to_system = Self::raw_time_to_system( distance_to_system, state.jump_range );
		let time_to_station = Self::raw_time_to_station( distance_to_station as f64 );
		
		TimeEstimate {
			distance_to_system: distance_to_system,
			distance_to_station: sell_station.distance_to_star.unwrap_or(0) as f64,
			jump_count: jump_count,
			time_to_system: time_to_system,
			time_to_station: time_to_station,
			time_total: time_to_system + time_to_station
		}
	}
	
	pub fn new_adjusted_estimate( state: &PlayerState, buy_system: &'a System, 
			sell_system: &'a System, sell_station: &'a Station ) -> TimeEstimate {
		let distance_to_system = buy_system.distance( sell_system );
		let distance_to_station = sell_station.distance_to_star.unwrap_or(DEFAULT_STATION_DISTANCE);
		
		let jump_count = Self::jump_count( distance_to_system, state.jump_range );
		
		let time_to_system = Self::adjusted_time_to_system( distance_to_system, state.jump_range, state.raw_adjustment_factor );
		let time_to_station = Self::adjusted_time_to_station( distance_to_station as f64, state.raw_adjustment_factor );
		
		TimeEstimate {
			distance_to_system: distance_to_system,
			distance_to_station: sell_station.distance_to_star.unwrap_or(0) as f64,
			jump_count: jump_count,
			time_to_system: time_to_system,
			time_to_station: time_to_station,
			time_total: time_to_system + time_to_station
		}
	}

	pub fn to_aboslute( &self, actual_seconds: f64 ) -> TimeEstimate {
		TimeEstimate {
			distance_to_system: self.distance_to_system,
			distance_to_station: self.distance_to_station,
			jump_count: self.jump_count,
			time_to_system: self.time_to_system,
			time_to_station: actual_seconds - self.time_to_system,
			time_total: actual_seconds
		}
	} 
}

impl TimeEstimate {
	pub fn raw_time_to_system( distance_to_system: f64, jump_range: f64 ) -> f64 {
		Self::adjusted_time_to_system( distance_to_system, jump_range, 1f64 )
	}
	
	pub fn adjusted_time_to_system( distance_to_system: f64, jump_range: f64, adjustment_factor: f64 ) -> f64 {
		(BUY_TIME + UNDOCK_TIME) * adjustment_factor + Self::jump_time( distance_to_system, jump_range )
	}
	
	pub fn raw_time_to_station( distance_to_station: f64 ) -> f64 {
		Self::adjusted_time_to_station( distance_to_station, 1f64 )
	}
	
	pub fn adjusted_time_to_station( distance_to_station: f64, adjustment_factor: f64 ) -> f64 {
		(SELL_TIME + DOCK_TIME + Self::supercruise_time( distance_to_station as f64 ) ) * adjustment_factor 
	}
	
	pub fn jump_count( system_distance: f64, jump_range: f64 ) -> f64 {
		if system_distance == 0f64 {
			return 0f64;
		}
		
		if system_distance < jump_range {
			return 1f64;
		}
		
		// linear regression based on lots of time spent in the galaxy map
		// plot route. note distance and # of jumps. repeat.
		// R^2 = 0.984.  Pretty good.
		let m = 1.354407;
		let x = system_distance / jump_range;
		let b = 0.05582672;
		(m * x + b).round()
	}
	
	pub fn jump_time( system_distance: f64, jump_range: f64 ) -> f64 {
		let jump_time = 43.16516f64 * Self::jump_count( system_distance, jump_range );
		jump_time 
	}
	
	pub fn supercruise_time( station_distance_ls: f64 ) -> f64 {
		// regression based on lots of time spent jumping to systems and flying to stations
		// this one has R^2=0.967, which is good.
		
		// this function is a symmetric sigmoid.
		
		// still, this metric can be skewed by player skill and ship performance.
		// we adapt by inferring time adjustments based on the speed the player performs at.
		// see data/time_adjustment.rs
		
		let a = -538.2561;
		let b = 0.04859748;
		let c = 340396200000f64;
		let d = 1947.062;
		
		let x = station_distance_ls;
		
		let supercruise_time = d + ( (a-d) / (1f64 + (x/c).powf(b)) );
		supercruise_time
	}
}
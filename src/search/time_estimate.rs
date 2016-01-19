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
/*
	Big table of magic numbers generated in a fancy Mathemtica environment.
	TLDR: how far you can get on each jump really depends on your jump range
	  If you have a high jump range, you can reach more stars, and those stars
	  will be closer to your destination.
	  
	  If you have a low jump range, you will be forced to 'sidetrack', 
	  due to limited jump choices.  Due to the nature of the sphere,
	  this effect gets much worse (non-linear) with smaller jump ranges.
	  
	  Lighter ships with good jump range don't experience this effect much,
	  but big ships like the T9 with poor gear really hurt.
	  
	  These numbers are calculations of mean progress per jump toward the destination,
	  from 0ly to 49.75ly in 0.25ly steps.
*/
const ADJUSTED_JUMP_RANGE : [f64; 200] = [	
	/* 0ly */0.001, 0.001, 0.001, 0.001, 
	/* 1ly */0.0017954, 0.00437344,  0.00903848, 0.0166672, 
	/* 2ly */0.0282592, 0.0449124, 0.0677932, 0.098101,  
	/* 3ly */0.13703, 0.185726, 0.245252, 0.316541, 
	/* 4ly */0.400373, 0.497345, 0.607855,  0.732094, 
	/* 5ly */0.870053, 1.02153, 1.18614, 1.36337, 
	/* 6ly */1.55257, 1.753,  1.96385, 2.18427, 
	/* 7ly */2.41342, 2.65043, 2.89449, 3.1448, 
	/* 8ly */3.40062, 3.66125, 3.92605, 4.19444, 
	/* 9ly */4.46589, 4.73993, 5.01614, 5.29415,  
	/* 10ly */5.57362, 5.85427, 6.13585, 6.41813, 
	/* 11ly */6.70093, 6.98408, 7.26745, 7.5509, 
	/* 12ly */7.83435, 8.1177, 8.40087, 8.68381, 
	/* 13ly */8.96647, 9.2488, 9.53077, 9.81234, 
	/* 14ly */10.0935, 10.3742, 10.6545, 10.9343, 
	/* 15ly */11.2137, 11.4926, 11.771, 12.049, 
	/* 16ly */12.3265, 12.6035, 12.8801, 13.1562, 
	/* 17ly */13.4319, 13.7071, 13.9819, 14.2562, 
	/* 18ly */14.5302, 14.8037, 15.0768, 15.3495, 
	/* 19ly */15.6218, 15.8937, 16.1653, 16.4365, 
	/* 20ly */16.7073, 16.9778, 17.248, 17.5178, 
	/* 21ly */17.7873, 18.0564, 18.3253, 18.5938, 
	/* 22ly */18.8621, 19.13, 19.3977, 19.6651, 
	/* 23ly */19.9322, 20.199, 20.4656, 20.732, 
	/* 24ly */20.9981, 21.2639, 21.5295, 21.7949, 
	/* 25ly */22.06, 22.325, 22.5897, 22.8542, 
	/* 26ly */23.1185, 23.3826, 23.6465, 23.9102, 
	/* 27ly */24.1737, 24.437, 24.7002, 24.9632, 
	/* 28ly */25.226, 25.4886, 25.751, 26.0133, 
	/* 29ly */26.2755, 26.5375, 26.7993, 27.061, 
	/* 30ly */27.3225, 27.5839, 27.8451, 28.1062, 
	/* 31ly */28.3672, 28.628, 28.8888, 29.1493,
	/* 32ly */29.4098, 29.6701, 29.9303, 30.1904, 
	/* 33ly */30.4504, 30.7103, 30.97, 31.2296, 
	/* 34ly */31.4892, 31.7486, 32.0079, 32.2671, 
	/* 35ly */32.5262, 32.7853, 33.0442, 33.303, 
	/* 36ly */33.5617, 33.8204, 34.0789, 34.3374, 
	/* 37ly */34.5958, 34.8541, 35.1123, 35.3704, 
	/* 38ly */35.6284, 35.8864, 36.1443, 36.4021, 
	/* 39ly */36.6598, 36.9174, 37.175, 37.4325, 
	/* 40ly */37.6899, 37.9473, 38.2046, 38.4618, 
	/* 41ly */38.719, 38.976, 39.2331, 39.49, 
	/* 42ly */39.7469, 40.0037, 40.2605, 40.5172, 
	/* 43ly */40.7739, 41.0305, 41.287, 41.5435, 
	/* 44ly */41.7999, 42.0563, 42.3126, 42.5688, 
	/* 45ly */42.825, 43.0812, 43.3373, 43.5933, 
	/* 46ly */43.8493, 44.1052, 44.3611, 44.617, 
	/* 47ly */44.8728, 45.1285, 45.3843, 45.6399, 
	/* 48ly */45.8955, 46.1511, 46.4066, 46.6621, 
	/* 49ly */46.9176, 47.173, 47.4283, 47.6836];
	
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
		BUY_TIME + UNDOCK_TIME + Self::jump_time( distance_to_system, jump_range )
	}
	
	pub fn raw_time_to_station( distance_to_station: f64 ) -> f64 {
		Self::adjusted_time_to_station( distance_to_station, 1f64 )
	}
	
	pub fn adjusted_time_to_station( distance_to_station: f64, adjustment_factor: f64 ) -> f64 {
		adjustment_factor * (SELL_TIME + DOCK_TIME + Self::supercruise_time( distance_to_station as f64 ) )
	}
		
	pub fn jump_count( system_distance: f64, jump_range: f64 ) -> f64 {
		if system_distance == 0f64 {
			return 0f64;
		}
		
		if system_distance < jump_range {
			return 1f64;
		}
		
		let index = (4f64 * jump_range) as usize;
		system_distance / ADJUSTED_JUMP_RANGE[index]
	}
	
	pub fn jump_time( system_distance: f64, jump_range: f64 ) -> f64 {
		let jump_time = 43.16516f64 * Self::jump_count( system_distance, jump_range );
		jump_time
	}
	
	pub fn supercruise_time( station_distance_ls: f64 ) -> f64 {
		if station_distance_ls == 0f64 {
			// if the station distance is not provided
			// lets return a moderate estimate
			return 240f64;
		}
		
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
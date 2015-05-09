use std::str::FromStr;

use data::Universe;
use data::IndexedUniverse;
use data::trader::Station;
use arguments::Arguments;

use search::full_trade::FullTrade;
use user_input::*;
use SEPARATOR;

#[derive(Clone)]
pub struct PlayerState {
	pub station_id: u32,
	pub system_id: u16,
	
	pub credit_balance: u32,
	pub minimum_balance: u32,
	
	pub jump_range: f64,
	pub cargo_capacity: u32,
	
	pub raw_adjustment_factor: f64
}

#[allow(dead_code)]
impl PlayerState {	
	pub fn new( arguments: &Arguments, universe: &Universe, indexed_universe: &IndexedUniverse ) -> PlayerState {
		let mut station_name = arguments.station.clone();
		let mut stations;
		
		loop {
			stations = indexed_universe.get_station_by_name( &station_name );
			
			if stations.is_some() {
				break;
			}
			
			println!( "The station '{}' was not found.", station_name );
			station_name = prompt_value( "t", "corrected station name" );
		}
		
		let stations = stations.unwrap();
		
		let station = match stations.len() {
			0 => panic!("Stations list was empty"),
			1 => stations.iter().next().unwrap(),
			_ => {
				println!("{}", SEPARATOR);
				println!("Multiple stations were found.");
				let mut print_index = 1;
				for station in stations {
					let system = indexed_universe.get_system( &station.system_id ).unwrap();
					println!("{}) {} [{}]", print_index, system.system_name, station.station_name );
					print_index += 1;
				}
				
				println!("");
				println!("Please enter the index of your station:");
				
				let index = match usize::from_str( read_line().as_str() ) {
					Ok(n) => n - 1,
					Err(_) => panic!("Invalid station index")
				};
				
				match stations.iter().nth( index ) {
					Some(s) => s,
					None => panic!("Your station was not found")
				}
			}
		};
		
		PlayerState {
			station_id: station.station_id,
			system_id: station.system_id,
			
			credit_balance: arguments.credit_balance,
			minimum_balance: arguments.minimum_balance,
			
			jump_range: arguments.jump_range,
			cargo_capacity: arguments.cargo,
			
			raw_adjustment_factor: universe.get_raw_adjustment_factor()
		}
	}
	
	pub fn refresh_time_adjustment( &self, universe: &Universe ) -> PlayerState {
		let mut new = self.clone();
		new.raw_adjustment_factor = universe.get_raw_adjustment_factor();
		new
	}
	
	
	pub fn with_station( &self, station: &Station ) -> PlayerState {
		let mut new_state = self.clone();
		new_state.station_id = station.station_id;
		new_state.system_id = station.system_id;
		new_state
	}
	
	pub fn with_trade( &self, trade: &FullTrade ) -> PlayerState {
		let mut new_state = self.with_station( trade.unit.sell_station );
		new_state.credit_balance = self.credit_balance + trade.profit_total;
		new_state
	}
}
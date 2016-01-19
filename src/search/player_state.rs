use std::str::FromStr;

use data::Universe;
use data::trader::{System, Station};
use arguments::Arguments;

use search::full_trade::FullTrade;
use user_input::*;
use SEPARATOR;

#[derive(Clone)]
pub struct PlayerState {
	pub system_id: u32,
	pub station_id: u32,
	
	pub credit_balance: u32,
	pub minimum_balance: u32,
	
	pub jump_range: f64,
	pub cargo_capacity: u32,
	
	pub raw_adjustment_factor: f64
}
//todo: refactor out jump range and cargo capacity.  mutable and immutable properties should be separate.
#[allow(dead_code)]
impl PlayerState {	
	pub fn new( arguments: &Arguments, universe: &Universe ) -> PlayerState {
		let mut station_name = arguments.station.clone();
		let mut stations;
		
		loop {
			stations = match arguments.system {
				Some(ref sys) => match universe.get_station_by_name(sys, &station_name){
					Some(station) => vec!(station),
					None => Vec::new()
				},
				None => universe.get_stations_by_name( &station_name )
			};
			
			
			if !stations.is_empty() {
				break;
			}
			
			println!( "The station '{}' was not found.", station_name );
			station_name = prompt_value( "t", "corrected station name" );
		}
		
		let station = match stations.len() {
			0 => panic!("Stations list was empty"),
			1 => stations.iter().next().unwrap(),
			_ => {
				println!("{}", SEPARATOR);
				println!("Multiple stations were found.");
				let mut print_index = 1;
				for station in stations.iter() {
					let system = universe.get_system( station.system_id ).unwrap();
					println!("{}) {} [{}]", print_index, system.system_name, station.station_name );
					print_index += 1;
				}
				
				println!("");
				println!("Please enter the index of your station:");
				
				let index = match usize::from_str( &read_line()[..] ) {
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
	
	pub fn get_station<'a>( &self, universe: &'a Universe ) -> &'a Station {
		match universe.get_station( self.station_id ) {
			Some(v) => v,
			None => panic!("Unknown station id {}", &self.station_id)
		}
	}
	
	pub fn get_system<'a>( &self, universe: &'a Universe ) -> &'a System {
		match universe.get_system( self.system_id ) {
			Some(v) => v,
			None => panic!("Unknown station id {}", &self.station_id)
		}
	}
}
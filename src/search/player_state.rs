use data::IndexedUniverse;
use data::trader::Station;
use arguments::Arguments;

use search::trade::FullTrade;
use user_input::prompt_value;

#[derive(Clone)]
pub struct PlayerState {
	pub station_id: u32,
	pub system_id: u16,
	
	pub credit_balance: u32,
	pub minimum_balance: u32,
	
	pub jump_range: f64,
	pub cargo_capacity: u32
}

#[allow(dead_code)]
impl PlayerState {	
	pub fn new( arguments: &Arguments, indexed_universe: &IndexedUniverse ) -> PlayerState {
		let mut station_name = arguments.station.clone();
		let mut station = None;
		while !station.is_some() {
			station = indexed_universe.get_station_by_name( &station_name );
			
			if !station.is_some() {
				println!( "The station '{}' was not found.", station_name );
				station_name = prompt_value( "t", "corrected station name" );
			}
		}
		
		let station = station.unwrap();
		
		PlayerState {
			station_id: station.station_id,
			system_id: station.system_id,
			
			credit_balance: arguments.credit_balance,
			minimum_balance: arguments.minimum_balance,
			
			jump_range: arguments.jump_range,
			cargo_capacity: arguments.cargo
		}
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
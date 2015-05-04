use data::Universe;
use data::trader::Station;

use search::trade::Trade;

#[derive(Clone)]
pub struct PlayerState<'a> {
	pub universe: &'a Universe,
	pub station: &'a Station,
	
	pub credit_balance: u32,
	pub minimum_balance: u32,
	
	pub jump_range: f64,
	pub cargo_capacity: u32
}

#[allow(dead_code)]
impl<'a> PlayerState<'a> {	
	pub fn with_station( &self, station: &'a Station ) -> PlayerState<'a> {
		let mut new_state = self.clone();
		new_state.station = station;
		new_state
	}
	
	pub fn with_trade( &self, trade: &Trade<'a> ) -> PlayerState<'a> {
		let mut new_state = self.clone();
		new_state.station = trade.sell_station;
		new_state.credit_balance = self.credit_balance + trade.profit_total;
		new_state
	}
}
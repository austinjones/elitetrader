use std::cmp::min;

use data::trader::*;
use search::unit_trade::UnitTrade;
use search::player_state::PlayerState;
use search::search_quality::SearchQuality;

#[derive(Clone)]
pub struct FullTrade<'a> {
	pub unit: UnitTrade<'a>,
	pub profit_total: u32,
	pub used_cargo: u32,
	pub profit_per_min: f64,
	pub is_valid: bool,
	pub is_cyclic: bool,
	state: PlayerState
}

impl<'a> FullTrade<'a> {
	pub fn new( state: &PlayerState, unit: UnitTrade<'a> ) -> FullTrade<'a> {
		let used_cargo = FullTrade::used_cargo( state, unit.buy );
		let profit_per_min = FullTrade::profit_per_min( &unit, used_cargo );
		let is_valid = unit.is_valid && used_cargo > 0;
		
		// we need to figure out whether it is possible for this trade to be cyclical, at search full depth
		// there are <max-depth> / 2 possible cycles (cycles with this station and one other, which begin at the first hop)
		// compare the possible cycles to the remaining buy supply
		
		// we also check that the player can cover the cost of all cycles
		
		let remaining_runs = unit.buy.supply as f64 / state.cargo_capacity as f64;
		let possible_cycles = SearchQuality::Ultra.get_depth() as f64 / 2f64;
		let possible_cost = (possible_cycles * state.cargo_capacity as f64 * unit.buy.buy_price as f64) as u32;
		let is_cyclic = is_valid
			&& remaining_runs > possible_cycles
			&& state.credit_balance > possible_cost;
			
//		println!( "Using {} of {}, profit/ton {}, profit total {}, profit/min {} over {}sec",
//			used_cargo, buy.commodity.commodity_name,
//			profit_per_ton, profit_total, profit_per_min.unwrap_or(0f64), cost_in_seconds );
		FullTrade {
			used_cargo: used_cargo,
			profit_total: unit.profit_per_ton * used_cargo,
			profit_per_min: profit_per_min,
			is_valid: is_valid,
			is_cyclic: is_cyclic,
			state: state.clone(),
			unit: unit
		}
	}
	
	#[allow(dead_code)]
	pub fn state_before_trade( &self ) -> PlayerState {
		self.state.clone()
	}
	
	pub fn state_after_trade( &self ) -> PlayerState {
		self.state.with_trade( self )
	}
	
	pub fn with_sell_price( &self, sell_price: u32 ) -> FullTrade<'a> {
		let new_unit = self.unit.with_sell_price( sell_price );
		FullTrade::new( &self.state, new_unit )
	}
	
	pub fn max_runs( &self ) -> f64 {
		self.unit.buy.supply as f64 / self.used_cargo as f64
	}
}

impl<'a> FullTrade<'a> {
	pub fn profit_per_min( unit: &UnitTrade, used_cargo: u32 ) -> f64 {
		unit.profit_per_ton_per_min * (used_cargo as f64)
	}
	
	pub fn used_cargo( state: &PlayerState, buy: &Listing ) -> u32 {
		if state.credit_balance < state.minimum_balance {
			return 0;
		}
		
		let possible_cargo = (state.credit_balance - state.minimum_balance) / buy.buy_price as u32;
		min( min( possible_cargo, state.cargo_capacity ), buy.supply )
	}
}
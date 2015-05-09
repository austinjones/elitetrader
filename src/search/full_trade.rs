use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::cmp::min;

use data::trader::*;
use search::unit_trade::UnitTrade;
use search::player_state::PlayerState;

#[derive(Clone)]
pub struct FullTrade<'a> {
	pub unit: UnitTrade<'a>,
	pub profit_total: u32,
	pub used_cargo: u32,
	pub profit_per_min: Option<f64>,
	pub is_valid: bool,
	state: PlayerState
}

impl<'a> FullTrade<'a> {
	pub fn new( state: &PlayerState, unit: UnitTrade<'a> ) -> FullTrade<'a> {
		let used_cargo = FullTrade::used_cargo( state, unit.buy );
		let profit_per_min = FullTrade::profit_per_min( &unit, used_cargo );
		let is_valid = unit.is_valid && used_cargo > 0;
//		println!( "Using {} of {}, profit/ton {}, profit total {}, profit/min {} over {}sec",
//			used_cargo, buy.commodity.commodity_name,
//			profit_per_ton, profit_total, profit_per_min.unwrap_or(0f64), cost_in_seconds );
		FullTrade {
			used_cargo: used_cargo,
			profit_total: unit.profit_per_ton * used_cargo,
			profit_per_min: profit_per_min,
			is_valid: is_valid,
			state: state.clone(),
			unit: unit
		}
	}
	
	pub fn state_before_trade( &self ) -> PlayerState {
		self.state.clone()
	}
	
	pub fn state_after_trade( &self ) -> PlayerState {
		self.state.with_trade( self )
	}
	
	pub fn with_sell_price( &self, sell_price: u16 ) -> FullTrade<'a> {
		let new_unit = self.unit.with_sell_price( sell_price );
		FullTrade::new( &self.state, new_unit )
	}
}

impl<'a> FullTrade<'a> {
	pub fn profit_per_min( unit: &UnitTrade, used_cargo: u32 ) -> Option<f64> {
		unit.profit_per_ton_per_min.map( |v| v*used_cargo as f64 )
	}
	
	pub fn used_cargo( state: &PlayerState, buy: &Listing ) -> u32 {
		if state.credit_balance < state.minimum_balance {
			return 0;
		}
		
		let possible_cargo = (state.credit_balance - state.minimum_balance) / buy.buy_price as u32;
		min( possible_cargo, state.cargo_capacity )
	}
}
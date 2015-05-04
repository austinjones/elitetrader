use std::collections::LinkedList;
use search::trade::Trade;

#[derive(Clone, Debug)]
pub struct TradeRoute<'b> {
	pub hops: LinkedList<Trade<'b>>
}

#[allow(dead_code)]
impl<'b> TradeRoute<'b> {
	pub fn new() -> TradeRoute<'b> {
		TradeRoute { hops: LinkedList::new() }
	}
	
	pub fn with_trade( trade: Trade<'b> ) -> TradeRoute<'b> {
		let mut route : TradeRoute = TradeRoute { hops: LinkedList::new() };
		route.hops.push_back( trade );
		route
	}
	
	pub fn profit_total( &self ) -> u32 {
		self.hops.iter()
			.map( |e| e.profit_total )
			.fold( 0, |a,b| a+b )
	}
	
	pub fn profit_per_ton( &self ) -> u32 {
		self.hops.iter()
			.map( |e| e.profit_per_ton )
			.fold( 0, |a,b| a+b )
	}
	
	pub fn profit_per_minute( &self ) -> f64 {
		self.profit_total() as f64 / (60f64 * self.distance_in_seconds() as f64)
	}
	
	pub fn score( &self ) -> f64 {
		self.hops.iter()
			.map( |e| e.score().unwrap_or(0f64) )
			.fold( 0f64, |a,b| a+b )
	}
	
	pub fn distance_in_seconds( &self ) -> f64 {
		self.hops.iter()
			.map( |e| e.distance_in_seconds )
			.fold( 0f64, |a,b| a+b )
	}
	
	pub fn distance_to_system( &self ) -> f64 {
		self.hops.iter()
			.map( |e| e.distance_to_system )
			.fold( 0f64, |a,b| a+b )
	}
	
	pub fn distance_to_station( &self ) -> f64 {
		self.hops.iter()
			.map( |e| e.distance_to_station )
			.fold( 0f64, |a,b| a+b )
	}
		
	pub fn is_valid( &self ) -> bool {
		self.hops.iter()
			.map(|e| e.is_valid )
			.fold(true, |a,b| a && b )
	}
		
	pub fn is_prohibited( &self ) -> bool {
		self.hops.iter()
			.map(|e| e.is_prohibited )
			.fold(true, |a,b| a || b )
	}
}
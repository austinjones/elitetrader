use data::adjustments::*;
use data::trader::Listing;
use time::get_time;

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct PriceAdjustment {
	pub buy_price: Option<u16>,
	pub sell_price: Option<u16>,
	pub supply: Option<u32>,
	pub system_id: u16,
	pub station_id: u32,
	pub commodity_id: u8,
	pub timestamp: u64
}

impl PriceAdjustment {
	pub fn new( supply: u32, buy_price: u16, sell_price: u16, listing: &Listing ) -> PriceAdjustment {
		PriceAdjustment {
			buy_price: Some(buy_price),
			sell_price: Some(sell_price),
			supply: Some(supply),
			system_id: listing.system_id,
			station_id: listing.station_id,
			commodity_id: listing.commodity.commodity_id,
			
			// I'm pretty sure the current timestamp isn't going to be before Jan 1st 1970...
			// lets cast it to u64
			timestamp: get_time().sec as u64
		}
	}
	
	pub fn from_sell( sell_price: u16, listing: &Listing ) -> PriceAdjustment {
		PriceAdjustment {
			buy_price: None,
			supply: None,
			sell_price: Some(sell_price),
			system_id: listing.system_id,
			station_id: listing.station_id,
			commodity_id: listing.commodity.commodity_id,
			timestamp: get_time().sec as u64
		}
	}
	
	pub fn from_buy( buy_price: u16, supply: u32, listing: &Listing ) -> PriceAdjustment {
		PriceAdjustment {
			supply: Some(supply),
			buy_price: Some(buy_price),
			sell_price: None,
			system_id: listing.system_id,
			station_id: listing.station_id,
			commodity_id: listing.commodity.commodity_id,
			timestamp: get_time().sec as u64
		}
	}
	
	pub fn load_all() -> Vec<PriceAdjustment> {
		load_adjustments("price")
	}
	
	pub fn save( &self ) {
		save_adjustment( "price", self );
	}
}
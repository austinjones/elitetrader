use data::trader::Listing;

pub struct PriceUpdate {
	pub buy_price: Option<u16>,
	pub sell_price: Option<u16>,
	pub supply: Option<u32>,
	pub system_id: u16,
	pub station_id: u32,
	pub commodity_id: u8
}

impl PriceUpdate {
	pub fn new( supply: u32, buy_price: u16, sell_price: u16, listing: &Listing ) -> PriceUpdate {
		PriceUpdate {
			buy_price: Some(buy_price),
			sell_price: Some(sell_price),
			supply: Some(supply),
			system_id: listing.system_id,
			station_id: listing.station_id,
			commodity_id: listing.commodity.commodity_id
		}
	}
	
	pub fn new_sell_update( sell_price: u16, listing: &Listing ) -> PriceUpdate {
		PriceUpdate {
			buy_price: None,
			supply: None,
			sell_price: Some(sell_price),
			system_id: listing.system_id,
			station_id: listing.station_id,
			commodity_id: listing.commodity.commodity_id
		}
	}
	
	pub fn new_buy_update( buy_price: u16, supply: u32, listing: &Listing ) -> PriceUpdate {
		PriceUpdate {
			supply: Some(supply),
			buy_price: Some(buy_price),
			sell_price: None,
			system_id: listing.system_id,
			station_id: listing.station_id,
			commodity_id: listing.commodity.commodity_id
		}
	}
}
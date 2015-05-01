use map_list::MapList;

use data::Listing;
use data::Identified;

pub struct ListingOptions<'a> {
	pub nodes: Vec<&'a Listing>
}

impl<'a> Default for ListingOptions<'a> {
	fn default() -> ListingOptions<'a> {
		ListingOptions {
			nodes: Vec::new()
		}
	}
}

#[allow(dead_code)]
impl<'a> ListingOptions<'a> {
	pub fn push( &mut self, trade: &'a Listing ) {
		self.nodes.push( trade );
	}
	
	pub fn push_all( &mut self, trades: &Vec<&'a Listing> ) {
		for trade in trades {
			self.nodes.push( trade );
		}
	}
	
	pub fn by_commodity( &self ) -> MapList<u8, &'a Listing> {
		let mut map = MapList::new();
		
		for trade in &self.nodes {
			map.insert( trade.commodity.to_id(), *trade );
		}
		
		map
	}
}

pub type BuyOptions<'a> = ListingOptions<'a>;
pub type SellOptions<'a> = ListingOptions<'a>;
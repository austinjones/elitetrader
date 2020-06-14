use crate::util::map_list::MapList;

use crate::data::trader::Identified;
use crate::data::trader::Listing;
use crate::data::trader::Station;
use crate::data::trader::System;

pub struct ListingOptions<'a> {
    pub nodes: Vec<&'a Listing>,
}

impl<'a> Default for ListingOptions<'a> {
    fn default() -> ListingOptions<'a> {
        ListingOptions { nodes: Vec::new() }
    }
}

#[allow(dead_code)]
impl<'a> ListingOptions<'a> {
    pub fn push(&mut self, trade: &'a Listing) {
        self.nodes.push(trade);
    }

    pub fn push_all(&mut self, trades: &Vec<&'a Listing>) {
        for trade in trades {
            self.nodes.push(trade);
        }
    }

    pub fn by_commodity(&self) -> MapList<u16, &'a Listing> {
        let mut map = MapList::new();

        for trade in &self.nodes {
            map.insert(trade.commodity.to_id(), *trade);
        }

        map
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

pub type BuyOptions<'a> = ListingOptions<'a>;
impl<'a> BuyOptions<'a> {
    pub fn buys_from_systems(systems: Vec<&System>) -> BuyOptions {
        let mut ret = BuyOptions::default();

        for system in systems {
            for station in &*system.stations {
                for listing in &station.listings {
                    if listing.is_buy() {
                        ret.push(listing);
                    }
                }
            }
        }

        ret
    }

    pub fn buys_from_system(system: &System) -> BuyOptions {
        let mut ret = BuyOptions::default();

        for station in &*system.stations {
            for listing in &station.listings {
                if listing.is_buy() {
                    ret.push(listing);
                }
            }
        }

        ret
    }

    pub fn buys_from_station(station: &Station) -> BuyOptions {
        let mut ret = BuyOptions::default();

        for listing in &station.listings {
            if listing.is_buy() {
                ret.push(listing);
            }
        }

        ret
    }
}

pub type SellOptions<'a> = ListingOptions<'a>;
impl<'a> SellOptions<'a> {
    pub fn sells_from_systems(systems: Vec<&System>) -> SellOptions {
        let mut ret = SellOptions::default();

        for system in systems {
            for station in &*system.stations {
                for listing in &station.listings {
                    if listing.is_sell() {
                        ret.push(listing);
                    }
                }
            }
        }

        ret
    }

    pub fn sells_from_system(system: &System) -> SellOptions {
        let mut ret = SellOptions::default();

        for station in &*system.stations {
            for listing in &station.listings {
                if listing.is_sell() {
                    ret.push(listing);
                }
            }
        }

        ret
    }

    pub fn sells_from_station(station: &Station) -> SellOptions {
        let mut ret = SellOptions::default();

        for listing in &station.listings {
            if listing.supply > 0 {
                if listing.is_sell() {
                    ret.push(listing);
                }
            }
        }

        ret
    }
}

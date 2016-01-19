extern crate time;

use std::collections::HashMap;

use spatial::octree::Index;
use spatial::octree::Octree;
use spatial::octree::Volume;

use data::universe::Universe;
use data::trader::*;
use search::options::*;

use util::map_list::MapList;

pub struct IndexedUniverse {
	// lookups for high-performance access
	pub systems: HashMap<u16, System>,
//	pub systems_by_name: HashMap<String, System>,
	
	// 3D spatial lookup.  SO COOL.
	pub octree: Octree<f64, System>,
	
	// lookups for user interaction
	pub stations: HashMap<u32, Station>,
	pub stations_by_name: MapList<String, Station>
	
	// not sure what these are good for
//	pub listings_by_system: MapList<u16, Listing>,
//	pub listings_by_station: MapList<u32, Listing>
}

#[allow(dead_code)]
impl<'a> IndexedUniverse {
	pub fn calculate( universe: &Universe ) -> IndexedUniverse {
		
//		println!( "Found {} systems.  Generating indexes...", universe.systems.len() );
		
		let mut systems_map = HashMap::new();
//		let mut systems_by_name = HashMap::new();
		
		for system in universe.systems.clone() {
//			systems_by_name.insert( system.system_name.to_lowercase(), system.clone() );
			systems_map.insert( system.to_id(), system );
		}	
			
	//	let mut system_name_map = HashMap::new();
	//	for system in systems.clone() {
	//		system_name_map.insert( system.system_name.to_lowercase(), system );
	//	}	
		
	//	let mut listings_by_system = MapList::new();
	//	for mut system in systems.clone() {
	//		let system_id = system.to_id();
	//		for mut station in system.stations.drain() {
	//			for listing in station.listings.drain() {
	//				listings_by_system.insert( system_id.clone(), listing );
	//			}
	//		}
	//	}
		
		let mut stations_map = HashMap::new();
		let mut stations_by_name = MapList::new();
		for mut system in universe.systems.clone() {
			for station in system.stations.drain() {
				stations_by_name.insert( station.station_name.to_lowercase(), station.clone() );
				stations_map.insert( station.to_id(), station );
			}
		}
		
	//	let mut listings_by_station = MapList::new();
	//	for mut system in systems.clone() {
	//		for mut station in system.stations.drain() {
	//			let station_id = station.to_id();
	//			for listing in station.listings.drain() {
	//				listings_by_station.insert( station_id.clone(), listing );
	//			}
	//		}
	//	}
		
		let octree = get_octree( universe.systems.clone() );
		
		IndexedUniverse {
			systems: systems_map,
//			systems_by_name: systems_by_name,
			stations: stations_map,
			stations_by_name: stations_by_name,
	//		listings_by_system: listings_by_system,
	//		listings_by_station: listings_by_station,
			octree: octree
		}
	}
	
	pub fn get_system( &self, id: &u16 ) -> Option<&System> {
		self.systems.get( &id )
	}
	
//	pub fn get_system_by_name( &self, system_name: &String ) -> Option<&System> {
//		self.systems_by_name.get( &system_name.to_lowercase() )
//	}
	
	pub fn get_station( &self, id: &u32 ) -> Option<&Station> {
		self.stations.get( &id )
	}
	
	pub fn get_station_by_name( &self, station_name: &String ) -> Option<&Vec<Station>> {
		self.stations_by_name.get( &station_name.to_lowercase() )
	}
	
//	pub fn get_listings_in_system( &self, id: &u16 ) -> Option<&Vec<Listing>> {
//		self.listings_by_system.get( &id )
//	}
//	
//	pub fn get_listings_in_station( &self, id: &u32 ) -> Option<&Vec<Listing>> {
//		self.listings_by_station.get( &id )
//	}
	
	pub fn get_systems_in_range( &'a self, system: &System, range: f64 ) -> Vec<&'a System> {
		self.octree.get_in_radius( system.octree_index(), range )
	}
	
	pub fn buys_from_systems( &'a self, systems: Vec<&'a System> ) -> BuyOptions {
		let mut ret = BuyOptions::default();
		
		for system in systems {
			for station in &*system.stations {
				for listing in &station.listings {
					if listing.is_buy() {
						ret.push( listing );
					}
				}
			}
		}
		
		ret
	}
	
	pub fn buys_from_system( &'a self, system: &'a System ) -> BuyOptions {
		let mut ret = BuyOptions::default();
		
		for station in &*system.stations {
			for listing in &station.listings {
				if listing.is_buy() {
					ret.push( listing );
				}
			}
		}
		
		ret
	}
	
	pub fn buys_from_station( &'a self, station: &'a Station ) -> BuyOptions {
		let mut ret = BuyOptions::default();
		
		for listing in &station.listings {
			if listing.is_buy() {
				ret.push( listing );
			}
		}
		
		ret
	}
	
	pub fn sells_from_systems( &'a self, systems: Vec<&'a System> ) -> SellOptions {
		let mut ret = SellOptions::default();
		
		for system in systems {
			for station in &*system.stations {
				for listing in &station.listings {
					if listing.is_sell() {
						ret.push( listing );
					}
				}
			}
		}
		
		ret
	}
	
	pub fn sells_from_system( &'a self, system: &'a System ) -> SellOptions {
		let mut ret = SellOptions::default();
		
		for station in &*system.stations {
			for listing in &station.listings {
				if listing.is_sell() {
					ret.push( listing );
				}
			}
		}
		
		ret
	}
	
	pub fn sells_from_station( &'a self, station: &'a Station ) -> SellOptions {
		let mut ret = SellOptions::default();
		
		for listing in &station.listings {
			if listing.supply > 0 {
				if listing.is_sell() {
					ret.push( listing );
				}
			}
		}
		
		ret
	}
}

fn get_octree<'a>( systems: Vec<System> ) -> Octree<f64, System> {
	let volume : Volume<f64> = Volume::new( [-2700f64,-1000f64,-1400f64], [1600f64,1200f64,4000f64] );
	let mut octree = Octree::new(volume);
	
	for system in systems {
		octree.insert( system );
	}
	
	octree
}
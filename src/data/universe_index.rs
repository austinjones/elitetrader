extern crate time;

use std::collections::HashMap;

use spatial::octree::Index;
use spatial::octree::Octree;
use spatial::octree::Volume;

use data::trader::*;

use util::map_list::MapList;

#[derive(Clone)]
struct OctreeSystemPosition {
	pub loc: [f64; 3],
	pub index: usize
}

impl Index<f64> for OctreeSystemPosition {
	fn octree_index(&self) -> [f64; 3] {
		self.loc
	}
}

pub struct UniverseIndex {
	// lookups for high-performance access
	systems: HashMap<u32, usize>,
	systems_by_name: HashMap<String, usize>,
	
	// 3D spatial lookup.  SO COOL.
	octree: Octree<f64, OctreeSystemPosition>,
	
	// lookups for user interaction
	stations: HashMap<u32, (usize, usize)>,
	stations_by_name: MapList<String, (usize, usize)>,
	
	listings: HashMap<(u32, u16), (usize, usize, usize)>
}

#[allow(dead_code)]
impl<'a> UniverseIndex {
	pub fn calculate( systems: &Vec<System> ) -> UniverseIndex {
		
//		println!( "Found {} systems.  Generating indexes...", universe.systems.len() );
		
		let mut systems_map = HashMap::new();
		let mut systems_by_name = HashMap::new();
		let mut listings = HashMap::new();
		
		for (index, system) in systems.iter().enumerate() {
			systems_by_name.insert( system.system_name.to_lowercase(), index );
			systems_map.insert( system.to_id(), index );
		}	
			
	//	let mut system_name_map = HashMap::new();
	//	for system in systems.clone() {
	//		system_name_map.insert( system.system_name.to_lowercase(), system );
	//	}	
		
	//	let mut listings_by_system = MapList::new();
	//	for mut system in systems.clone() {
	//		let system_id = system.to_id();
	//		for mut station in system.stations.drain(..) {
	//			for listing in station.listings.drain(..) {
	//				listings_by_system.insert( system_id.clone(), listing );
	//			}
	//		}
	//	}
		
		let mut stations_map = HashMap::new();
		let mut stations_by_name = MapList::new();
		for (system_index, system) in systems.iter().enumerate() {
			for (station_index, station) in system.stations.iter().enumerate() {
				stations_by_name.insert( station.station_name.to_lowercase(), (system_index, station_index) );
				stations_map.insert( station.to_id(), (system_index, station_index) );
				
				for (listing_index, listing) in station.listings.iter().enumerate() {
					listings.insert( (station.to_id(), listing.commodity.commodity_id), (system_index, station_index, listing_index) );
				}
			}
		}
		
	//	let mut listings_by_station = MapList::new();
	//	for mut system in systems.clone() {
	//		for mut station in system.stations.drain(..) {
	//			let station_id = station.to_id();
	//			for listing in station.listings.drain(..) {
	//				listings_by_station.insert( station_id.clone(), listing );
	//			}
	//		}
	//	}
		
		let octree = get_octree( &systems );
		
		UniverseIndex {
			systems: systems_map,
			systems_by_name: systems_by_name,
			stations: stations_map,
			stations_by_name: stations_by_name,
			listings: listings,
	//		listings_by_system: listings_by_system,
	//		listings_by_station: listings_by_station,
			octree: octree
		}
	}
	
	pub fn get_index_system( &self, id: u32 ) -> Option<usize> {
		self.systems.get( &id ).map(|e| *e)
	}
	
	pub fn get_index_system_by_name( &self, system_name: &String ) -> Option<usize> {
		self.systems_by_name.get( &system_name.to_lowercase() ).map(|e| *e)
	}
	
	pub fn get_index_station( &self, id: u32 ) -> Option<(usize, usize)> {
		self.stations.get( &id ).map(|e| *e)
	}
	
	pub fn get_index_station_by_name( &self, station_name: &String ) -> Vec<(usize, usize)> {
		self.stations_by_name.get( &station_name.to_lowercase() ).unwrap_or(&Vec::new()).clone()
	}
	
//	pub fn get_listings_in_system( &self, id: &u16 ) -> Option<&Vec<Listing>> {
//		self.listings_by_system.get( &id )
//	}
//	
//	pub fn get_listings_in_station( &self, id: &u32 ) -> Option<&Vec<Listing>> {
//		self.listings_by_station.get( &id )
//	}
	
	pub fn get_index_systems_in_range( &'a self, system: &System, range: f64 ) -> Vec<usize> {
		self.octree.get_in_radius( system.octree_index(), range ).into_iter().map(|e| e.index).collect()
	}
	
	pub fn get_index_listing( &self, station_id: u32, commodity_id: u16 ) -> Option<(usize, usize, usize)> {
		self.listings.get( &(station_id, commodity_id) ).map(|e| *e)
	}
}

fn get_octree( systems: &Vec<System> ) -> Octree<f64, OctreeSystemPosition> {
	let volume : Volume<f64> = Volume::new( [-2700f64,-1000f64,-1400f64], [1600f64,1200f64,4000f64] );
	let mut octree = Octree::new(volume);
	
	for (index, system) in systems.iter().enumerate() {
		octree.insert( OctreeSystemPosition {
			loc: [system.x, system.y, system.z],
			index: index
		} );
	}
	
	octree
}
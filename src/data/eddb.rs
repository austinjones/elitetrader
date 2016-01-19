use std::collections::HashMap;
use data::trader::Identified;

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct SystemJson {
	pub id: u32,
	pub name: String,
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub needs_permit: Option<u8>,
	pub updated_at: u64
//	faction: Option<String>,
//	population: Option<u64>,
//	government: Option<String>,
//	allegiance: Option<String>,
//	state: Option<String>,
//	security: Option<String>,
//	primary_economy: Option<String>,
}

impl Identified<u32> for SystemJson { 
	fn to_id(&self) -> u32 {
		self.id
	}
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct StationJson {
	pub id: u32,
	pub system_id: u32,
	pub name: String,
	pub max_landing_pad_size: Option<String>,
	pub distance_to_star: Option<u32>,
	pub prohibited_commodities: Vec<String>,
//	faction: Option<String>,
//	government: Option<String>,
//	allegiance: Option<String>,
//	state: Option<String>,
//	has_blackmarket: Option<u8>,
//	has_market: Option<u8>,
//	has_refuel: Option<u8>,
//	has_rearm: Option<u8>,
//	has_shipyard: Option<u8>,
//	import_commodities: Box<Vec<String>>,
//	export_commodities: Box<Vec<String>>,
//	economies: Box<Vec<String>>,
	pub updated_at: u64,
	pub market_updated_at: Option<u64>,
	pub is_planetary: Option<u8>,
}

impl Identified<u32> for StationJson { 
	fn to_id(&self) -> u32 {
		self.id
	}
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct StationCommodityListingJson {
	pub id: u32,
	pub station_id: u32,
	pub commodity_id: u16,
	pub supply: i64,
	pub buy_price: i32,
	pub sell_price: i32,
	pub collected_at: u64,
	pub demand: i64,
	pub update_count: u32
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct CommodityJson {
	pub id: u16,
	pub name: String,
	pub category_id: u8,
//	pub average_price: Option<u32>,
	pub category: CommodityCategoryJson
}

impl Identified<u16> for CommodityJson { 
	fn to_id(&self) -> u16 {
		self.id
	}
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct CommodityCategoryJson {
//	pub id: u8,
	pub name: String
}

pub fn get_stations_by_system( stations: &Vec<StationJson> ) -> HashMap<u32, Vec<&StationJson>> {
	let mut result = HashMap::new();
	
	for s in stations.iter() {		
		match result.get_mut( &s.system_id ) {
			None => { result.insert( s.system_id, Vec::new() ); },
			_ => {}
		};
						
		let mut vec = match result.get_mut( &s.system_id ) {
			Some(vbox) => vbox,
			None => panic!( "Should have been inserted above" )
		};
		
		vec.push( s );
	}
	
	result
}

pub fn get_listings_by_station( listings: &Vec<StationCommodityListingJson> ) -> HashMap<u32, Vec<&StationCommodityListingJson>> {
	let mut result = HashMap::new();
	
	for s in listings.iter() {		
		match result.get_mut( &s.station_id ) {
			None => { result.insert( s.station_id, Vec::new() ); },
			_ => {}
		};
						
		let mut vec = match result.get_mut( &s.station_id ) {
			Some(vbox) => vbox,
			None => panic!( "Should have been inserted above" )
		};
		
		vec.push( s );
	}
	
	result
}
//
//pub fn get_octree( systems: Vec<System> ) -> Octree<f32, System> {
//	let volume : Volume<f32> = Volume::new( [-200f32,-200f32,-200f32], [200f32,200f32,200f32] );
//	let mut octree = Octree::new(volume);
//	
//	for system in systems {
//		octree.insert( system );
//	}
//	
//	octree
//}

//pub fn minify() {
//	let commodities : Vec<Commodity> = read_json(&Path::new("src/data/commodities.json"));
//	write_json(&Path::new("src/min/commodities.json"), &commodities);
//	
//	let stations : Vec<Station> = read_json(&Path::new("src/data/stations.json"));
//	write_json(&Path::new("src/min/stations.json"), &stations);
//	
//	let systems : Vec<System> = read_json(&Path::new("src/data/systems.json"));
//	write_json(&Path::new("src/min/systems.json"), &systems);
//}
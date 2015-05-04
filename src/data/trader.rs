use spatial::octree::Index;
use std::str::FromStr;

pub trait Identified<K> {
	fn to_id(&self) -> K;
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct System {
	pub system_id: u16,
	pub system_name: String,
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub stations: Vec<Station>
//	faction: Option<String>,
//	population: Option<u64>,
//	government: Option<String>,
//	allegiance: Option<String>,
//	state: Option<String>,
//	security: Option<String>,
//	primary_economy: Option<String>,
//	needs_permit: Option<u8>,
//	updated_at: u64
}

impl Index<f64> for System {
	fn octree_index(&self) -> [f64; 3] {
		[self.x, self.y, self.z]
	}
}

impl Identified<u16> for System { 
	fn to_id(&self) -> u16 {
		self.system_id
	}
}

impl System {
	pub fn distance( &self, other: &System ) -> f64 {
		((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
	}
	
	#[allow(dead_code)]
	pub fn get_station_by_name( &self, name: String ) -> Option<&Station> {
		let name_lowercase = name.to_lowercase();
		
		for station in &*self.stations {
			if station.station_name.to_lowercase() == name_lowercase {
				return Some(station)
			}
		}
		
		return None;
	}
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Station {
//	pub id: u32,
	pub system_id: u16,
	pub station_id: u32,
	pub station_name: String,
	pub max_landing_pad_size: ShipSize,
	pub distance_to_star: Option<u32>,
	pub listings: Vec<Listing>,
	pub prohibited_commodities: Vec<u8>
//	faction: Option<String>,
//	government: Option<String>,
//	allegiance: Option<String>,
//	state: Option<String>,
//	has_blackmarket: Option<u8>,
//	has_commodities: Option<u8>,
//	has_refuel: Option<u8>,
//	has_rearm: Option<u8>,
//	has_shipyard: Option<u8>,
//	import_commodities: Box<Vec<String>>,
//	export_commodities: Box<Vec<String>>,
//	economies: Box<Vec<String>>,
//	updated_at: u32,
}

impl Identified<u32> for Station { 
	fn to_id(&self) -> u32 {
		self.station_id
	}
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Listing {
	pub system_id: u16,
	pub station_id: u32,
	pub commodity: Commodity,
	pub supply: u32,
	pub buy_price: u16,
	pub sell_price: u16,
//	pub demand: u32,
//	collected_at: u32,
//	update_count: u16
}

impl Listing {
	pub fn is_buy( &self ) -> bool {
		self.supply > 0
	}
	
	pub fn is_sell( &self ) -> bool {
		self.sell_price > 0
	}
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Commodity {
	pub commodity_id: u8,
	pub commodity_name: String,
	pub category: String
}

impl PartialEq for Commodity {
	fn eq(&self, other: &Commodity) -> bool {
		self.commodity_id == other.commodity_id
	}
}

impl Identified<u8> for Commodity { 
	fn to_id(&self) -> u8 {
		self.commodity_id
	}
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
pub enum ShipSize {
	Small,
	Medium,
	Large
}

impl FromStr for ShipSize {
    type Err = String;

    fn from_str(s: &str) -> Result<ShipSize, String> {
        match s.to_lowercase().as_str() {
            "small" => Ok(ShipSize::Small),
            "medium" => Ok(ShipSize::Medium),
            "large" => Ok(ShipSize::Large),
            "med" => Ok(ShipSize::Medium),
            "s" => Ok(ShipSize::Small),
            "m" => Ok(ShipSize::Medium),
            "l" => Ok(ShipSize::Large),
            _ => Err( format!("Unknown enum variant '{}'", s) ),
        }
    }
}

//pub fn write_json<T: rustc_serialize::Encodable>( path: &Path, data: &T ) {
//	let mut file = File::create( path ).unwrap();
//	println!("Encoding file {}", path.to_str().unwrap() );
//	
//	let string = json::encode( data ).unwrap();
//	let bytes : &[u8] = string.as_bytes();
//	
//	println!("Writing file {}", path.to_str().unwrap() );
//	file.write_all(bytes);
//}

//pub fn get_stations_by_system( stations: &mut Vec<Station> ) -> HashMap<u16, Vec<Station>> {
//	let mut result = HashMap::new();
//	
//	for s in stations.drain() {		
//		match result.get_mut( &s.system_id ) {
//			None => { result.insert( s.system_id, Vec::new() ); },
//			_ => {}
//		};
//						
//		let mut vec = match result.get_mut( &s.system_id ) {
//			Some(vbox) => vbox,
//			None => panic!( "Should have been inserted above" )
//		};
//		
//		vec.push( s );
//	}
//	
//	result
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
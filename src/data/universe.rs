extern crate time;

use std::collections::HashMap;

use std::env::home_dir;
use std::path::Path;
use std::fs::PathExt;
use std::fs::File;

use time::now;
use time::Duration;

use persist::*;
use data::update::PriceUpdate;
use data::trader::*;
use data::eddb::*;

use std::str::FromStr;

use CACHE_FILENAME;

fn get_cachefile_loc() -> String {
	match home_dir() {
		Some(pathbuf) => match pathbuf.to_str() {
			Some( path ) => path.to_string() + "/" + &CACHE_FILENAME.to_string(),
			None => CACHE_FILENAME.to_string()
		},
		None => CACHE_FILENAME.to_string()
	}
}

pub struct Universe {
	pub systems: Vec<System>
}

impl Universe {
	pub fn load(ship_size: &ShipSize) -> Universe {
		let cachefile_loc = get_cachefile_loc();
		let cachefile_path = Path::new( &cachefile_loc );
		
		let mut systems = match cachefile_path.exists() {
			true => {
				let file = match File::open(&cachefile_loc) {
					Ok(f) => f,
					Err(reason) => panic!("Failed to open cachefile ({}), but the path exists.  The path may be a directory: {}", 
						reason, cachefile_loc)
				};
				
				let modtime = match file.metadata() {
					Ok(meta) => meta.modified() / 1000,
					Err(reason) => panic!("Failed to load file metadata ({}) for path: {}", 
						reason, cachefile_loc)
				};
				
				let now = time::now().to_timespec().sec;
				
				let age = Duration::milliseconds( now as i64 - modtime as i64 );
				let threshold = Duration::days(1);
				
				if age < threshold {
					println!("Loading cached file from {} ...", cachefile_loc );
					read_json( cachefile_path )
				} else {
					println!("File was modified {} hours ago - refreshing", age.num_hours() );
					Universe::recalculate_systems( cachefile_path )
				}
			},
			false => {
				Universe::recalculate_systems( cachefile_path )
			}
		};
		
		Universe::filter_systems( &mut systems, ship_size );
		
		Universe {
			systems: systems
		}
	}

	fn recalculate_systems( path: &Path ) -> Vec<System> {
		println!("The cached data file is stale, or did not exist.  Reloading data from eddb.io ...");
		
		println!("Loading commodities.json...");
		let commodities_json : Vec<CommodityJson> = http_read_json(&"http://eddb.io/archive/v3/commodities.json".to_string());
		
		println!("Loading system.json...");
		let systems_json : Vec<SystemJson> = http_read_json(&"http://eddb.io/archive/v3/systems.json".to_string());
		
		println!("Loading stations.json...");
		let stations_json : Vec<StationJson> = http_read_json(&"http://eddb.io/archive/v3/stations.json".to_string());
		
		println!("Loads complete.  Converting to internal format...");
		
	//	println!("Grouping stations by system");
		let stations_by_system : HashMap<u16, Vec<&StationJson>> = get_stations_by_system( &stations_json );
		
		let mut commodities_by_id = HashMap::new();
		let mut commodities_by_name = HashMap::new();
		
	//	println!("Calculating commodity lookups");
		for commodity_json in commodities_json {
			let commodity = Box::new(Commodity {
				commodity_id: commodity_json.id,
				commodity_name: commodity_json.name,
				category: commodity_json.category.name
			});
			
			commodities_by_id.insert( commodity_json.id, *commodity.clone() );
			// this is the lowercase string-name value
			commodities_by_name.insert( commodity.commodity_name.to_lowercase(), *commodity );
		}
	//	println!("Calculating systems");
		//let mut systems_map = HashMap::new();
		let mut systems = Vec::with_capacity(systems_json.len());
		
		//let mut stations_map = HashMap::new();
		
		for system_json in systems_json {
			let system_id = system_json.id;
			let mut system = Box::new(System {
				system_id: system_json.id,
				system_name: system_json.name.clone(),
				x: system_json.x,
				y: system_json.y,
				z: system_json.z,
				stations: Vec::new()
			});
			
			match stations_by_system.get( &system_json.id ) {
				Some( stations_jsons ) => {
					for station_json in stations_jsons {
						let station_id = station_json.id;
						
						let mut prohibited_commodities = Vec::new();
						for commodity_name in &station_json.prohibited_commodities {
							let commodity = match commodities_by_name.get( &commodity_name.to_lowercase() ) {
								Some(c) => c,
								None => panic!("Unknown commodity '{}' in prohibited listing for station '{}'",
									commodity_name, station_json.name )
							};
							
							prohibited_commodities.push( commodity.commodity_id );
						}
						
						let ship_size_in = station_json.max_landing_pad_size.clone().unwrap_or("S".to_string());
						let ship_size = match ShipSize::from_str(ship_size_in.as_str()) {
							Ok(v) => v,
							Err(reason) => panic!("Unknown ship size '{}' for station '{}': {}", 
								ship_size_in, station_json.name, reason )
						};
						
						let mut station = Box::new(Station {
							system_id: system_id,
							station_id: station_id,
							station_name: station_json.name.clone(),
							max_landing_pad_size: ship_size,
							distance_to_star: station_json.distance_to_star,
							listings: Vec::new(),
							prohibited_commodities: prohibited_commodities
						});
						
						for listing_json in &station_json.listings {
							let commodity = commodities_by_id.get( &listing_json.commodity_id ).unwrap().clone();
							
							let listing = Listing {
								system_id: system_id,
								station_id: station_id,
								commodity: commodity,
								supply: match listing_json.supply > 0 
											{ true => listing_json.supply as u32, _ => 0 },
											
								buy_price: match listing_json.buy_price > 0 
											{ true => listing_json.buy_price as u16 , _ => 0 },
											
								sell_price:  match listing_json.sell_price > 0 
											{ true => listing_json.sell_price as u16 , _ => 0 }
							};
							
							station.listings.push( listing );
						}
						
						system.stations.push( *station );
						//stations_map.insert( station.to_id(), *station );
					}
				},
				None => {}
			};
			
			systems.push( *system );
		}
		
		println!("Saving cachefile to {} ...", path.to_str().unwrap() );
		write_json( path, &systems );
		
		return systems;
	}
	
	fn filter_systems( systems: &mut Vec<System>, ship_size: &ShipSize ) {
		for mut system in systems {
			let new_stations = system.stations.drain()
				.filter(|e| e.max_landing_pad_size >= *ship_size )
				.collect();
			
			system.stations = new_stations;
		}
	}
	
	pub fn apply_updates( &mut self, updates: Vec<PriceUpdate> ) {
		for update in updates {
			for system in self.systems.iter_mut() {
				if update.system_id != system.system_id {
					continue;
				}
				
				for station in system.stations.iter_mut() {
					if update.station_id != station.station_id {
						continue;
					}
					
					for listing in station.listings.iter_mut() {
						if update.commodity_id != listing.commodity.commodity_id {
							continue;
						}
						
						match update.buy_price {
							Some(v) => {listing.buy_price = v},
							None => {}
						};
						
						match update.supply {
							Some(v) => {listing.supply = v},
							None => {}
						};
						
						match update.sell_price {
							Some(v) => {listing.sell_price = v},
							None => {}
						};
					}
				}
			}
			
		}
	}
	
	pub fn apply_update( &mut self, update: PriceUpdate ) {
		self.apply_updates( vec!( update ) );
	}
}
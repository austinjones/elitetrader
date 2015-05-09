extern crate time;

use std::collections::HashMap;

use std::path::Path;
use std::path::PathBuf;
use std::fs::PathExt;
use std::fs::File;

use time::now;
use time::Duration;

use persist::*;
use data::price_adjustment::PriceAdjustment;
use data::time_adjustment::TimeAdjustment;
use data::trader::*;
use data::eddb::*;
use search::time_estimate::TimeEstimate;

use util::scored_buf::ScoredCircularBuffer;
use util::scored_buf::Sort;
use std::str::FromStr;



use CACHE_FILENAME;

fn get_cachefile_loc() -> PathBuf {
	get_base_directory().join(CACHE_FILENAME).to_path_buf()
}

pub struct Universe {
	pub systems: Vec<System>,
//	pub time_adjustments: HashMap<u32, ScoredCircularBuffer<u64, TimeAdjustment>>
	pub time_adjustments: ScoredCircularBuffer<u64, TimeAdjustment>
}

impl Universe {
	pub fn load(ship_size: &ShipSize) -> Universe {
		let cachefile_path = get_cachefile_loc();
		let cachefile_str = cachefile_path.to_str().unwrap_or("<unknown>");
		
		let systems = match cachefile_path.exists() {
			true => {
				let file = match File::open(&cachefile_path) {
					Ok(f) => f,
					Err(reason) => panic!("Failed to open cachefile ({}), but the path exists.  The path may be a directory: {}", 
						reason, cachefile_str)
				};
				
				let modtime = match file.metadata() {
					Ok(meta) => meta.modified() / 1000,
					Err(reason) => panic!("Failed to load file metadata ({}) for path: {}", 
						reason, cachefile_str)
				};
				
				let now = time::now().to_timespec().sec;
				
				let age = Duration::milliseconds( now as i64 - modtime as i64 );
				let threshold = Duration::days(1);
				
				if age < threshold {
					println!("Loading cached file from {} ...", cachefile_str );
					read_json( &cachefile_path.as_path() )
				} else {
					println!("File was modified {} hours ago - refreshing", age.num_hours() );
					Universe::recalculate_systems( &cachefile_path.as_path() )
				}
			},
			false => {
				Universe::recalculate_systems( &cachefile_path.as_path() )
			}
		};
		
		let systems = Universe::filter_systems( systems, ship_size );
		
		let mut universe = Universe {
			systems: systems,
			time_adjustments: ScoredCircularBuffer::new(20, Sort::Descending)
		};
		
		universe.apply_price_adjustments( PriceAdjustment::load_all() );
		universe.apply_time_adjustments( TimeAdjustment::load_all() );
		
		universe
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
				updated_at: system_json.updated_at,
				needs_permit: system_json.needs_permit.map(|e| e != 0).unwrap_or(false),
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
//							time_to_station: 0f64,
							updated_at: station_json.updated_at,
							listings: Vec::new(),
							prohibited_commodities: prohibited_commodities
						});
						
						for listing_json in &station_json.listings {
							let commodity = commodities_by_id.get( &listing_json.commodity_id ).unwrap().clone();
							
							let listing = Listing {
								system_id: system_id,
								station_id: station_id,
								commodity: commodity,
								collected_at: listing_json.collected_at,
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
	
	fn filter_systems( mut systems: Vec<System>, ship_size: &ShipSize ) -> Vec<System> {
		let illegal_categories = ["drugs", "weapons", "slavery"];
		let mut systems : Vec<System> = systems.drain()
			.filter(|e| !e.needs_permit)
			.collect();
		
		for mut system in &mut systems {
			let mut new_stations : Vec<Station> = system.stations.drain()
				.filter(|e| e.max_landing_pad_size >= *ship_size )
				.collect();

			for mut station in new_stations.iter_mut() {
				if station.prohibited_commodities.len() == 0 {
					let new_listings : Vec<Listing> = station.listings.drain()
						.filter(|e| !illegal_categories.contains( &e.commodity.category.to_lowercase().as_str() ) )
						.collect();
					station.listings = new_listings;
				} else {
					let prohibited_commodities = &station.prohibited_commodities;
					let new_listings : Vec<Listing> = station.listings.drain()
						.filter(|e| !prohibited_commodities.contains( &e.commodity.commodity_id ) )
						.collect();
					station.listings = new_listings;
				}
			}
			
			system.stations = new_stations;
		}
		
		systems
	}
	
	pub fn apply_price_adjustments( &mut self, prices: Vec<PriceAdjustment> ) {
		for price in prices {
			self.apply_price_adjustment( &price );
		}
	}
	
	pub fn apply_time_adjustments( &mut self, times: Vec<TimeAdjustment> ) {
		for time in &times {
			self.apply_time_adjustment( time.clone() );
		}
		
//		for time in &times {
//			self.cache_time_adjustment( time.clone() );
//		}
		
//		let raw_adjustment_factor = self.get_raw_adjustment_factor();
//		
//		for time in &times {
//			self.apply_time_to_station( time.sell_system_id, time.sell_station_id, raw_adjustment_factor );
//		}
	}
	
	pub fn apply_price_adjustment( &mut self, price: &PriceAdjustment ) {
		for system in self.systems.iter_mut() {
			if price.system_id != system.system_id {
				continue;
			}
			
			for station in system.stations.iter_mut() {
				if price.station_id != station.station_id {
					continue;
				}
				
				for listing in station.listings.iter_mut() {
					if price.commodity_id != listing.commodity.commodity_id {
						continue;
					}
					
					// only overwrite if the timestamp of the adjustment is newer than the date from eddb
					// this prevents old user-entered values from becoming stale.
					
					if listing.collected_at < price.timestamp  {
						match price.buy_price {
							Some(v) => {listing.buy_price = v},
							None => {}
						};
						
						match price.supply {
							Some(v) => {listing.supply = v},
							None => {}
						};
						
						match price.sell_price {
							Some(v) => {listing.sell_price = v},
							None => {}
						};
					}
				}
			}
		}
	}
	
	pub fn apply_time_adjustment( &mut self, time: TimeAdjustment ) {
		let timestamp = time.timestamp;
		self.time_adjustments.push( time, timestamp );
	}
	
//	pub fn apply_time_adjustment( &mut self, time: TimeAdjustment ) {
//		self.cache_time_adjustment( time.clone() );
//		
//		let adjustment_factor = self.get_raw_adjustment_factor();
//		self.apply_time_to_station( time.sell_system_id, time.sell_station_id, adjustment_factor );
//	}
//	
//	fn cache_time_adjustment( &mut self, time: TimeAdjustment ) {
//		match self.time_adjustments.get( &time.sell_station_id ) {
//			Some(_) => {},
//			None => {
//				let buf_new = ScoredCircularBuffer::new( 10, Sort::Descending );
//				self.time_adjustments.insert( time.sell_station_id,  buf_new );
//			}
//		};
//		
//		let mut buf = match self.time_adjustments.get_mut( &time.sell_station_id ) {
//			Some(v) => v,
//			None => panic!("Should have inserted Time Adjustment into buffer")
//		};
//		
//		let timestamp = time.timestamp;
//		
//		buf.push( time, timestamp );
//	}
//
//	
//	fn apply_time_to_station( &mut self, system_id: u16, station_id: u32, raw_adjustment_factor: f64 ) {
//		let buf = match self.time_adjustments.get( &station_id ) {
//			Some(v) => v,
//			None => { return; }
//		};
//		
//		let mut sum : f64 = buf.iter().map(|e| e.value.actual_time.time_to_station ).sum();
//		let mut n = buf.len();
//		
//		if n == 0 {
//			return;
//		}
//		
//		'system: for system in self.systems.iter_mut() {
//			if system_id != system.system_id {
//				continue;
//			}
//			
//			'station: for station in system.stations.iter_mut() {
//				if station_id != station.station_id {
//					continue;
//				}
//				
//				let distance_to_station = station.distance_to_star.unwrap_or( DEFAULT_STATION_DISTANCE );
//				let estimate = TimeEstimate::raw_time_to_station( distance_to_station as f64 ) * raw_adjustment_factor;
//				
//				while n < 10 {
//					sum += estimate;
//					n += 1;
//				}
//				
//				station.time_to_station = sum / n as f64;
//				break 'system;
//			}
//		}
//	}
//	
//	pub fn apply_time_adjustment( &mut self, time: TimeAdjustment ) {
//		self.cache_time_adjustment( time.clone() );
//		
//		let adjustment_factor = self.get_raw_adjustment_factor();
//		self.apply_time_to_station( time.sell_system_id, time.sell_station_id, adjustment_factor );
//	}
	
	pub fn get_raw_adjustment_factor( &self ) -> f64 {
		let mut n = self.time_adjustments.len();
		let mut ratios : f64 = self.time_adjustments.iter().map( |e| e.value.actual_time.time_to_station / e.value.raw_estimate.time_to_station ).sum();
		
		// make sure the adjustment factor moves slowly so we don't get scared away from trades with long station distances
		while n < 10 {
			n += 1;
			ratios += 1f64;
		}
			
//		for scored_buf in self.time_adjustments.values() {
//			let actual : f64 = scored_buf.iter().map( |e| e.value.actual_time.time_to_station ).sum();
//			let normalized : f64 = scored_buf.iter().map( |e| e.value.raw_estimate.time_to_station ).sum();
//			sum_actual += actual;
//			sum_normalized += normalized;
//		}
		
		let factor = ratios / n as f64;
		
		println!("Got adjustment factor {:.2}", factor);
		factor
	}
}
extern crate time;

use std::collections::HashMap;

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use time::Duration;
use time::{PrimitiveDateTime, Time};

use crate::data::edce::EdceData;
use crate::data::eddb::*;
use crate::data::price_adjustment::PriceAdjustment;
use crate::data::time_adjustment::TimeAdjustment;
use crate::data::trader::*;
use crate::data::universe_index::UniverseIndex;
use crate::persist::*;

use crate::search::FullTrade;

use crate::search::SearchCache;
use crate::util::scored_buf::ScoredCircularBuffer;
use crate::util::scored_buf::Sort;

use crate::CACHE_FILENAME;
use filetime::FileTime;
use statistical;
use std::str::FromStr;

fn get_cachefile_loc() -> PathBuf {
    get_base_directory().join(CACHE_FILENAME).to_path_buf()
}

pub struct Universe {
    pub systems: Vec<System>,
    //	pub time_adjustments: HashMap<u32, ScoredCircularBuffer<u64, TimeAdjustment>>
    pub time_adjustments: ScoredCircularBuffer<u64, TimeAdjustment>,

    pub index: UniverseIndex,
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
                    Ok(meta) => FileTime::from_last_modification_time(&meta),
                    Err(reason) => panic!(
                        "Failed to load file metadata ({}) for path: {}",
                        reason, cachefile_str
                    ),
                };

                let now = PrimitiveDateTime::now().timestamp();

                let age = Duration::seconds(now as i64 - modtime.unix_seconds() as i64);
                let num_hours = age.num_seconds() as f64 / 3600f64;
                let threshold = Duration::days(1);

                if age < threshold {
                    println!(
                        "Loading cached file from {} ... file cached {:.1} hours ago",
                        cachefile_str, num_hours
                    );
                    read_json(&cachefile_path.as_path())
                } else {
                    println!("File was modified {} hours ago - refreshing", num_hours);
                    Universe::recalculate_systems(&cachefile_path.as_path())
                }
            }
            false => Universe::recalculate_systems(&cachefile_path.as_path()),
        };

        let systems = Universe::filter_systems(systems, ship_size);
        let index = UniverseIndex::calculate(&systems);

        let mut universe = Universe {
            systems: systems,
            time_adjustments: ScoredCircularBuffer::new(20, Sort::Descending),
            index: index,
        };

        universe.apply_price_adjustments(PriceAdjustment::load_all());
        universe.apply_time_adjustments(TimeAdjustment::load_all());

        universe
    }

    pub fn snapshot(&self) -> Universe {
        let systems_new = self.systems.clone();
        let index = UniverseIndex::calculate(&systems_new);

        Universe {
            systems: systems_new,
            time_adjustments: ScoredCircularBuffer::new(20, Sort::Descending),
            index: index,
        }
    }

    pub fn apply_trade(&mut self, trade: &FullTrade, cache: &SearchCache) {
        if let Some(mut station) = self.get_station_mut(trade.unit.buy.station_id) {
            for listing in station.listings.iter_mut() {
                if listing.commodity.commodity_id == trade.unit.commodity_id {
                    listing.supply = listing.supply - trade.used_cargo;
                    break;
                }
            }
        }

        cache.invalidate_station(trade.unit.buy_station.station_id);
    }

    fn recalculate_systems(path: &Path) -> Vec<System> {
        println!(
            "The cached data file is stale, or did not exist.  Reloading data from eddb.io ..."
        );

        println!("Loading commodities.json...");
        let commodities_json: Vec<CommodityJson> =
            http_read_json(&"https://eddb.io/archive/v6/commodities.json".to_string());

        println!("Loading system.json...");
        let systems_json: Vec<SystemJson> =
            http_read_json(&"https://eddb.io/archive/v6/systems_populated.json".to_string());

        println!("Loading stations.json...");
        let stations_json: Vec<StationJson> =
            http_read_json(&"https://eddb.io/archive/v6/stations.json".to_string());

        println!("Loading listings.csv...");
        let listings_csv: Vec<StationCommodityListingJson> =
            http_read_csv(&"https://eddb.io/archive/v6/listings.csv".to_string());

        println!("Listing 0: {:?}", listings_csv[0]);
        println!("Loads complete.  Converting to internal format...");

        //	println!("Grouping stations by system");
        let stations_by_system = get_stations_by_system(&stations_json);
        let listings_by_station = get_listings_by_station(&listings_csv);

        let mut commodities_by_id = HashMap::new();
        let mut commodities_by_name = HashMap::new();

        //	println!("Calculating commodity lookups");
        for commodity_json in commodities_json {
            let commodity = Box::new(Commodity {
                commodity_id: commodity_json.id,
                commodity_name: commodity_json.name,
                category: commodity_json.category.name,
            });

            commodities_by_id.insert(commodity_json.id, *commodity.clone());
            // this is the lowercase string-name value
            commodities_by_name.insert(commodity.commodity_name.to_lowercase(), *commodity);
        }
        //	println!("Calculating systems");
        //let mut systems_map = HashMap::new();
        let mut systems = Vec::with_capacity(systems_json.len());

        //let mut stations_map = HashMap::new();

        let now = PrimitiveDateTime::now().timestamp();

        for system_json in systems_json {
            let system_id = system_json.id;
            let mut system = Box::new(System {
                system_id: system_json.id,
                system_name: system_json.name.clone(),
                x: system_json.x,
                y: system_json.y,
                z: system_json.z,
                updated_at: system_json.updated_at,
                needs_permit: system_json.needs_permit.unwrap_or(false),
                stations: Vec::new(),
            });

            match stations_by_system.get(&system_json.id) {
                Some(stations_jsons) => {
                    for station_json in stations_jsons {
                        let station_id = station_json.id;

                        // todo: exclude stations in a way that allows players to start trading from excluded stations

                        if station_json.is_planetary.unwrap_or(false) {
                            // todo: allow configuration of planetary stations
                            continue;
                        }

                        // Exclude fleet carriers
                        if station_json.government == Some("Private Ownership".to_string()) {
                            continue;
                        }

                        if let Some(market_updated_at) = station_json.market_updated_at {
                            // based on lots of work in Mathematica, station sell prices significantly change,
                            //   about 22 days after they collected

                            // let's filter out any stations that haven't been updated in this time
                            // this should prevent the player from being sent to stations for a loss

                            // this technically should be recalculated on every run,
                            //  but we recalculate the cachefile every day or two anyway

                            //todo: dynamically scale buy/sell prices based on confidence during trade calculations

                            let age = Duration::seconds(now as i64 - market_updated_at as i64);
                            let threshold = Duration::days(22);

                            if age > threshold {
                                //println!("Rejecting {} as it was updated {} hours ago", station_json.name, age.num_hours());
                                continue;
                            }
                        }

                        let mut prohibited_commodities = Vec::new();
                        for commodity_name in &station_json.prohibited_commodities {
                            let commodity =
                                match commodities_by_name.get(&commodity_name.to_lowercase()) {
                                    Some(c) => c,
                                    None => panic!(
                                    "Unknown commodity '{}' in prohibited listing for station '{}'",
                                    commodity_name, station_json.name
                                ),
                                };

                            prohibited_commodities.push(commodity.commodity_id);
                        }

                        let ship_size_in = station_json
                            .max_landing_pad_size
                            .clone()
                            .unwrap_or("S".to_string());

                        if ship_size_in == "None" || ship_size_in.trim().len() == 0 {
                            continue;
                        }

                        let ship_size = match ShipSize::from_str(&ship_size_in[..]) {
                            Ok(v) => v,
                            Err(reason) => panic!(
                                "Unknown ship size '{}' for station '{}': {}",
                                ship_size_in, station_json.name, reason
                            ),
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
                            prohibited_commodities: prohibited_commodities,
                            market_updated_at: station_json.market_updated_at,
                            is_planetary: station_json.is_planetary.unwrap_or(false),
                        });

                        for listing_json in
                            listings_by_station.get(&station_id).unwrap_or(&Vec::new())
                        {
                            let commodity = commodities_by_id
                                .get(&listing_json.commodity_id)
                                .unwrap()
                                .clone();

                            let listing = Listing {
                                system_id: system_id,
                                station_id: station_id,
                                commodity: commodity,
                                collected_at: listing_json.collected_at,
                                supply: match listing_json.supply > 0 {
                                    true => listing_json.supply as u32,
                                    _ => 0,
                                },

                                buy_price: match listing_json.buy_price > 0 {
                                    true => listing_json.buy_price as u32,
                                    _ => 0,
                                },

                                sell_price: match listing_json.sell_price > 0 {
                                    true => listing_json.sell_price as u32,
                                    _ => 0,
                                },
                            };

                            station.listings.push(listing);
                        }

                        system.stations.push(*station);
                        //stations_map.insert( station.to_id(), *station );
                    }
                }
                None => {}
            };

            systems.push(*system);
        }

        println!("Saving cachefile to {} ...", path.to_str().unwrap());
        write_json(path, &systems);

        return systems;
    }

    fn filter_systems(mut systems: Vec<System>, ship_size: &ShipSize) -> Vec<System> {
        let illegal_categories = ["drugs", "weapons", "slavery"];
        let mut systems: Vec<System> = systems.drain(..).filter(|e| !e.needs_permit).collect();

        for mut system in &mut systems {
            let mut new_stations: Vec<Station> = system
                .stations
                .drain(..)
                .filter(|e| e.max_landing_pad_size >= *ship_size)
                .collect();

            for mut station in new_stations.iter_mut() {
                if station.prohibited_commodities.len() == 0 {
                    let new_listings: Vec<Listing> = station
                        .listings
                        .drain(..)
                        //						.filter(|e| !illegal_categories.contains( &&e.commodity.category.to_lowercase()[..] ) )
                        .collect();
                    station.listings = new_listings;
                } else {
                    let prohibited_commodities = &station.prohibited_commodities;
                    let new_listings: Vec<Listing> = station
                        .listings
                        .drain(..)
                        .filter(|e| !prohibited_commodities.contains(&e.commodity.commodity_id))
                        .collect();
                    station.listings = new_listings;
                }
            }

            system.stations = new_stations;
        }

        systems
    }

    pub fn apply_price_adjustments(&mut self, prices: Vec<PriceAdjustment>) {
        for price in prices {
            self.apply_price_adjustment(&price);
        }
    }

    pub fn apply_time_adjustments(&mut self, times: Vec<TimeAdjustment>) {
        for time in &times {
            self.apply_time_adjustment(time.clone());
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

    pub fn apply_price_adjustment(&mut self, price: &PriceAdjustment) {
        if let Some(mut station) = self.get_station_mut(price.station_id) {
            for listing in station.listings.iter_mut() {
                if price.commodity_id != listing.commodity.commodity_id {
                    continue;
                }

                // only overwrite if the timestamp of the adjustment is newer than the date from eddb
                // this prevents old user-entered values from becoming stale.
                // the adjustment has to be 10 minutes before the EDDB entry,
                // in case we created the update using EDCE

                if listing.collected_at < price.timestamp - 600 {
                    match price.buy_price {
                        Some(v) => listing.buy_price = v,
                        None => {}
                    };

                    match price.supply {
                        Some(v) => listing.supply = v,
                        None => {}
                    };

                    match price.sell_price {
                        Some(v) => listing.sell_price = v,
                        None => {}
                    };
                }
            }
        }
    }

    pub fn apply_time_adjustment(&mut self, time: TimeAdjustment) {
        let timestamp = time.timestamp;
        self.time_adjustments.push(time, timestamp);
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

    pub fn get_raw_adjustment_factor(&self) -> f64 {
        let time_adjustments = self.time_adjustments.sort();

        //		for (i,time) in time_adjustments.iter().enumerate() {
        //			println!("Time Adjustment {} => {:.1} / {:.1} = {:.2}, {:.1} / {:.1} = {:.2}", i,
        //				 time.actual_time.time_to_station,
        //				 time.raw_estimate.time_to_station,
        //				 time.actual_time.time_to_station / time.raw_estimate.time_to_station,
        //				 time.actual_time.time_to_station,
        //				 time.adjusted_estimate.time_to_station,
        //				 time.actual_time.time_to_station / time.adjusted_estimate.time_to_station );
        //		}

        let mut vals: Vec<f64> = time_adjustments
            .iter()
            .map(|e| e.actual_time.time_to_station / e.raw_estimate.time_to_station)
            .collect();

        // make sure the adjustment factor moves slowly so we don't get scared away from trades with long station distances
        // if the players first few trades are slow
        while vals.len() < 10 {
            vals.push(1f64);
        }

        let factor = statistical::median(&vals[..]);

        //		println!("Summary adjustment factor {:.2}", factor);
        factor
    }

    pub fn get_index(&self) -> &UniverseIndex {
        &self.index
    }

    pub fn get_system_by_index<'a>(&'a self, index: Option<usize>) -> Option<&'a System> {
        index.map(|i| &self.systems[i])
    }

    pub fn get_system_by_index_mut<'a>(
        &'a mut self,
        index: Option<usize>,
    ) -> Option<&'a mut System> {
        index.map(move |i| &mut self.systems[i])
    }

    pub fn get_systems_by_index<'a>(&'a self, indeces: Vec<usize>) -> Vec<&'a System> {
        indeces.iter().map(|&index| &self.systems[index]).collect()
    }

    pub fn get_station_by_index<'a>(
        &'a self,
        index: Option<(usize, usize)>,
    ) -> Option<&'a Station> {
        index.map(|(sys, stat)| &self.systems[sys].stations[stat])
    }

    pub fn get_station_by_index_mut<'a>(
        &'a mut self,
        index: Option<(usize, usize)>,
    ) -> Option<&'a mut Station> {
        index.map(move |(sys, stat)| &mut self.systems[sys].stations[stat])
    }

    pub fn get_stations_by_index<'a>(&'a self, indeces: Vec<(usize, usize)>) -> Vec<&'a Station> {
        indeces
            .iter()
            .map(|&(sys, stat)| &self.systems[sys].stations[stat])
            .collect()
    }

    pub fn get_listing_by_index<'a>(
        &'a self,
        index: Option<(usize, usize, usize)>,
    ) -> Option<&'a Listing> {
        index.map(move |(sys, stat, list)| &self.systems[sys].stations[stat].listings[list])
    }

    pub fn get_listing_by_index_mut<'a>(
        &'a mut self,
        index: Option<(usize, usize, usize)>,
    ) -> Option<&'a mut Listing> {
        index.map(move |(sys, stat, list)| &mut self.systems[sys].stations[stat].listings[list])
    }

    pub fn get_system(&self, id: u32) -> Option<&System> {
        self.get_system_by_index(self.index.get_index_system(id))
    }

    fn get_system_mut(&mut self, id: u32) -> Option<&mut System> {
        let index = self.index.get_index_system(id);
        self.get_system_by_index_mut(index)
    }

    pub fn get_system_by_name(&self, system_name: &String) -> Option<&System> {
        self.get_system_by_index(self.index.get_index_system_by_name(system_name))
    }

    pub fn get_station(&self, id: u32) -> Option<&Station> {
        self.get_station_by_index(self.index.get_index_station(id))
    }

    fn get_station_mut(&mut self, id: u32) -> Option<&mut Station> {
        let index = self.index.get_index_station(id);
        self.get_station_by_index_mut(index)
    }

    pub fn get_station_by_name(
        &self,
        system_name: &String,
        station_name: &String,
    ) -> Option<&Station> {
        match self.get_system_by_index(self.index.get_index_system_by_name(system_name)) {
            Some(system) => system
                .stations
                .iter()
                .filter(|e| &e.station_name == station_name)
                .next(),
            _ => None,
        }
    }

    pub fn get_station_by_name_mut(
        &mut self,
        system_name: &String,
        station_name: &String,
    ) -> Option<&mut Station> {
        let index = self.index.get_index_system_by_name(system_name);
        match self.get_system_by_index_mut(index) {
            Some(mut system) => system
                .stations
                .iter_mut()
                .filter(|e| &e.station_name == station_name)
                .next(),
            _ => None,
        }
    }

    pub fn get_stations_by_name(&self, station_name: &String) -> Vec<&Station> {
        self.get_stations_by_index(self.index.get_index_station_by_name(station_name))
    }

    //	pub fn get_listings_in_system( &self, id: &u16 ) -> Option<&Vec<Listing>> {
    //		self.listings_by_system.get( &id )
    //	}
    //
    //	pub fn get_listings_in_station( &self, id: &u32 ) -> Option<&Vec<Listing>> {
    //		self.listings_by_station.get( &id )
    //	}

    pub fn get_systems_in_range(&self, system: &System, range: f64) -> Vec<&System> {
        self.get_systems_by_index(self.index.get_index_systems_in_range(system, range))
    }
}

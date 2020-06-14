use crate::persist;
use std::fs::File;

use crate::data::price_adjustment::PriceAdjustment;
use crate::data::trader::*;
use crate::data::universe::Universe;
use serde::Deserialize;
use serde::Serialize;
use std::process::Command;
use time;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdceData {
    pub commander: EdceCommander,
    pub lastSystem: EdceSystem,
    pub lastStarport: EdceStarport,
    pub ship: EdceShip,
}

impl EdceData {
    pub fn generate_opt(basePath: &Option<String>) -> Option<EdceData> {
        match basePath {
            &Some(ref path) => Self::generate(&path),
            &None => None,
        }
    }

    pub fn generate(basePath: &String) -> Option<EdceData> {
        match Command::new("python.exe")
            .arg(String::new() + basePath + "\\edce_client.py")
            .current_dir(&basePath)
            .output()
        {
            Ok(_) => EdceData::read(&basePath),
            Err(text) => {
                println!("EDCE Error: Could not execute command path: {}", text);
                None
            }
        }
    }

    fn read(basePath: &String) -> Option<EdceData> {
        let jsonPath = String::new() + basePath + "/last.json";
        //		let timePath = String::new() + basePath + "/last.time";

        //		match File::open(&timePath) {
        //			Ok(mut f) => {
        //				let text = persist::read_text_from_file( &mut f );
        //				match text.parse::<i64>() {
        //					Ok(epoch) => {
        //						let now = time::now().to_timespec().sec;
        //						let diff = now - epoch;
        //						println!("EDCE data loaded {} seconds ago", diff);
        //					},
        //					_ => {}
        //				}
        //			},
        //			_ => {}
        //		};

        match File::open(&jsonPath) {
            Ok(mut f) => persist::read_json_from_file(&mut f),
            Err(text) => {
                println!(
                    "EDCE Error: Could not open last.json ({}): {}",
                    text, &jsonPath
                );
                None
            }
        }
    }

    pub fn apply_edce_adjustments<'a>(
        &'a self,
        universe: &'a mut Universe,
    ) -> Option<EdceStationUpdate> {
        let mut updates = Vec::new();

        let system_name = self.lastSystem.name.to_lowercase();
        let station_name = self.lastStarport.name.to_lowercase();
        let commodities = &self.lastStarport.commodities;

        if let Some(mut station) =
            universe.get_station_by_name_mut(&self.lastSystem.name, &self.lastStarport.name)
        {
            'listing: for listing in station.listings.iter_mut() {
                'commodity: for commodity in commodities {
                    let universe_name = listing.commodity.commodity_name.to_lowercase();
                    let commodity_name = commodity.get_eddb_commodity_name().to_lowercase();
                    if universe_name != commodity_name {
                        continue;
                    }

                    let buy_price_delta = listing.buy_price as i64 - commodity.buyPrice as i64;
                    let sell_price_delta = listing.sell_price as i64 - commodity.sellPrice as i64;
                    let supply_delta = listing.supply as i64 - commodity.stock as i64;

                    if buy_price_delta.abs() > 0
                        || sell_price_delta.abs() > 0
                        || supply_delta.abs() > 0
                    {
                        //println!("Created adjustment from EDCE data: {}, buy: {}, sell: {}, suppy: {}",
                        //	listing.commodity.commodity_name,
                        //	buy_price_delta, sell_price_delta, supply_delta );

                        let listing_old = listing.clone();

                        listing.buy_price = commodity.buyPrice;
                        listing.sell_price = commodity.sellPrice;
                        listing.supply = commodity.stock as u32;

                        let commodity_update = EdceListingUpdate {
                            old_listing: listing_old,
                            new_listing: listing.clone(),
                        };
                        updates.push(commodity_update);

                        let adjustment = PriceAdjustment::new(
                            commodity.stock as u32,
                            commodity.buyPrice,
                            commodity.sellPrice,
                            listing,
                        );
                        adjustment.save();

                        break 'commodity;
                    }
                }
            }

            Some(EdceStationUpdate {
                station: station,
                changes: updates,
            })
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdceCommander {
    pub credits: usize,
    pub currentShipId: usize,
    pub docked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdceSystem {
    pub id: usize,
    pub name: String,
    pub faction: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdceStarport {
    pub id: usize,
    pub name: String,
    pub faction: String,
    pub commodities: Vec<EdceCommodityListing>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdceCommodityListing {
    pub name: String,
    pub buyPrice: u32,
    pub sellPrice: u32,
    pub stock: f64,
    //	pub categoryname: String
}

impl EdceCommodityListing {
    pub fn get_eddb_commodity_name(&self) -> String {
        match &self.name[..] {
            "Agricultural Medicines" => "Agri-Medicines".to_string(),
            "Atmospheric Extractors" => "Atmospheric Processors".to_string(),
            "Auto Fabricators" => "Auto-Fabricators".to_string(),
            "Basic Narcotics" => "Narcotics".to_string(),
            "Bio Reducing Lichen" => "Bioreducing Lichen".to_string(),
            "Hazardous Environment Suits" => "H.E. Suits".to_string(),
            "Heliostatic Furnaces" => "Microbial Furnaces".to_string(),
            "Marine Supplies" => "Marine Equipment".to_string(),
            "Non Lethal Weapons" => "Non-lethal Weapons".to_string(),
            "S A P8 Core Container" => "SAP 8 Core Container".to_string(),
            "Terrain Enrichment Systems" => "Land Enrichment Systems".to_string(),
            _ => self.name.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdceShip {
    pub name: String,
    pub cargo: EdceShipCargo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdceShipCargo {
    pub capacity: u16,
    pub qty: u16,
}

pub struct EdceStationUpdate<'a> {
    pub station: &'a Station,
    pub changes: Vec<EdceListingUpdate>,
}

pub struct EdceListingUpdate {
    pub old_listing: Listing,
    pub new_listing: Listing,
}

impl EdceListingUpdate {
    pub fn get_commodity_id(&self) -> u16 {
        self.old_listing.commodity.commodity_id
    }
}

use crate::data::trader::ShipSize;
use crate::data::EdceData;
use crate::search::SearchQuality;
use crate::user_input::prompt_value;
use crate::util::num_unit::*;
use getopts::Matches;
use std::str::FromStr;

pub struct Arguments {
    pub system: Option<String>,
    pub station: String,
    pub cargo: u32,
    pub credit_balance: u32,
    pub minimum_balance: u32,
    pub jump_range: f64,
    pub ship_size: ShipSize,
    pub search_quality: SearchQuality,
    pub edce_path: Option<String>,
    pub auto_accept: bool,
}

impl Arguments {
    pub fn collect(config: &Matches, edce_data: &Option<EdceData>) -> Arguments {
        if edce_data.is_some() {
            println!("EDCE data was successfully retrieved.  Some arguments can be automatically extracted.");
        }

        let system_in = match config.opt_str("s") {
            Some(t) => Some(t),
            None => match edce_data {
                &Some(ref edce) => {
                    let system = edce.lastSystem.name.clone();
                    println!("Loaded EDCE data - system location:\t{}", system);
                    Some(system)
                }
                &None => None,
            },
        };

        let station_in = match config.opt_str("t") {
            Some(t) => t,
            None => match edce_data {
                &Some(ref edce) => {
                    let starport = edce.lastStarport.name.clone();
                    println!("Loaded EDCE data - starport location:\t{}", starport);
                    starport
                }
                &None => prompt_value("t", "current station name (e.g. Git Hub)"),
            },
        };

        let balance_in = match config.opt_str("b") {
            Some(v) => v,
            None => match edce_data {
                &Some(ref edce) => {
                    let credits = edce.commander.credits;
                    println!(
                        "Loaded EDCE data - credit balance:\t{}",
                        NumericUnit::new_string(credits, &"cr".to_string())
                    );
                    credits.to_string()
                }
                &None => prompt_value("b", "current credit balance (e.g. 525.4k or 525412)"),
            },
        };
        let balance = match NumericUnit::from_str(balance_in.as_ref()) {
            Ok(v) => v.to_num(),
            Err(reason) => panic!("Invalid balance '{}' - {}", balance_in, reason),
        };

        let cargo_in = match config.opt_str("c") {
            Some(v) => v,
            None => match edce_data {
                &Some(ref edce) => {
                    let cargo = edce.ship.cargo.capacity.to_string();
                    println!("Loaded EDCE data - cargo capcity:\t{} tons", cargo);
                    cargo
                }
                &None => prompt_value("c", "current cargo capacity in tons (e.g. 216)"),
            },
        };
        let cargo_capacity = match NumericUnit::from_str(cargo_in.as_ref()) {
            Ok(v) => v.to_num(),
            Err(reason) => panic!("Invalid cargo capacity '{}' - {}", cargo_in, reason),
        };

        let minimum_balance_in = match config.opt_str("m") {
            Some(v) => v,
            None => prompt_value(
                "m",
                "minimum credit balance - saftey net for rebuy (e.g. 3.2m)",
            ),
        };
        let minimum_balance = match NumericUnit::from_str(minimum_balance_in.as_ref()) {
            Ok(v) => v.to_num(),
            Err(reason) => panic!(
                "Invalid minimum balance '{}' - {}",
                minimum_balance_in, reason
            ),
        };

        let jump_range_in = match config.opt_str("r") {
            Some(v) => v,
            None => prompt_value("r", "current laden jump range in light years"),
        };
        let jump_range = match NumericUnit::from_str(jump_range_in.as_ref()) {
            Ok(v) => v.to_num(),
            Err(reason) => panic!("Invalid jump range '{}' - {}", jump_range_in, reason),
        };

        let ship_size_in = match config.opt_str("p") {
            Some(v) => v,
            None => match Self::get_ship_size(edce_data) {
                Some(v) => {
                    println!("Loaded EDCE data - ship size:\t\t{} ", v);
                    v
                }
                None => prompt_value("p", "current ship size [small|med|large], or [s|m|l]"),
            },
        };
        let ship_size = match ShipSize::from_str(ship_size_in.as_ref()) {
            Ok(v) => v,
            Err(reason) => panic!("Invalid ship size '{}' - {}", ship_size_in, reason),
        };

        let quality_in = config.opt_str("q").unwrap_or("ultra".to_string());
        let quality: SearchQuality = match SearchQuality::from_str(&quality_in[..]) {
            Ok(v) => v,
            Err(reason) => panic!("Invalid search quality '{}' - {}", quality_in, reason),
        };

        Arguments {
            system: system_in,
            station: station_in,
            cargo: cargo_capacity,
            credit_balance: balance,
            jump_range: jump_range,
            minimum_balance: minimum_balance,
            ship_size: ship_size,
            search_quality: quality,
            edce_path: config.opt_str("C").map(|e| e.replace("\"", "")),
            auto_accept: config.opt_present("A"),
        }
    }

    fn get_ship_size(edce_data: &Option<EdceData>) -> Option<String> {
        match edce_data {
            &Some(ref edce) => match &edce.ship.name[..] {
                "Adder" => Some("small"),
                "Anaconda" => Some("large"),
                "Asp Explorer" => Some("medium"),
                "Asp Scout" => Some("medium"),
                "Cobra Mk III" => Some("small"),
                "Cobra Mk IV" => Some("small"),
                "Diamondback Explorer" => Some("small"),
                "Diamondback Scout" => Some("small"),
                "Eagle" => Some("small"),
                "Federal Assault Ship" => Some("medium"),
                "Federal Corvette" => Some("large"),
                "Federal Dropship" => Some("medium"),
                "Federal Gunship" => Some("medium"),
                "Fer-de-Lance" => Some("medium"),
                "Hauler" => Some("small"),
                "Imperial Clipper" => Some("large"),
                "Imperial Courier" => Some("small"),
                "Imperial Cutter" => Some("large"),
                "Imperial Eagle" => Some("small"),
                "Keelback" => Some("medium"),
                "Orca" => Some("large"),
                "Python" => Some("medium"),
                "Sidewinder" => Some("small"),
                "Type-6 Transporter" => Some("medium"),
                "Type-7 Transporter" => Some("large"),
                "Type-9 Heavy" => Some("large"),
                "Viper" => Some("small"),
                "Viper Mk IV" => Some("small"),
                "Vulture" => Some("small"),
                _ => None,
            }
            .map(|e| e.to_string()),
            &None => None,
        }
    }
}

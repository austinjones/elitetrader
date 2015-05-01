#![feature(custom_derive)]
#![feature(collections)]
#![feature(core)]
#![feature(convert)]
#![feature(path_ext)]
#![feature(std_misc)]
#![feature(fs_time)]

extern crate rustc_serialize;
extern crate spatial;
extern crate core;
extern crate rand;
extern crate hyper;
extern crate flate2;
extern crate time;
extern crate getopts;

mod data;
mod universe;
mod analysis;
mod options;
mod io;
mod conversion;

mod scored_buf;
mod map_list;

use getopts::{Options, Matches};
use std::str::FromStr;

use analysis::Analyzer;

pub const SEPARATOR : &'static str = "------------------------------------------------------------";
pub const CACHE_FILENAME : &'static str = ".elite_universe_min.json";

fn options() -> Options {
	let mut opts = Options::new();
//	opts.optopt("s", "system", "set current system name", "LTT 826");
	opts.optopt("t", "station", "set current station name", "GitHub");
	opts.optopt("c", "cargo", "maximum cargo capacity in tons. find this in your right cockpit panel's Cargo tab.", "216");
	opts.optopt("r", "range", "maximum laden jump range in light years.  find this in your outfitting menu.", "18.52");
	opts.optopt("q", "quality", "search quality setting [low|med|high]", "med");
	
	opts.optflag("h", "help", "prints this help menu");
	opts
}

fn prompt_value( description: &'static str ) -> String {
	println!( "You can set these options in the command line arguments.  Try the --help flag." );
	print!( "Please enter the {}: ", description );
	
	let mut val = String::new();
	match std::io::stdin().read_line(&mut val) {
		Err(reason) => panic!("Failed to read line: {}", reason ),
		_ => {}
	};
	
	val
}

enum SearchQuality {
	Low,
	Med,
	High
}

impl FromStr for SearchQuality {
    type Err = String;

    fn from_str(s: &str) -> Result<SearchQuality, String> {
        match s {
            "Low" => Ok(SearchQuality::Low),
            "Med" => Ok(SearchQuality::Med),
            "High" => Ok(SearchQuality::High),
            _ => Err( format!("Unknown enum variant '{}'", s) ),
        }
    }
}

struct Arguments {
//	pub system: String,
	pub station: String,
	pub cargo: u64,
	pub jump_range: f64,
	pub search_quality: SearchQuality
}

impl Arguments {
	pub fn collect( config: &Matches ) -> Arguments {
//		let system_in = config.opt_str("s")
//			.unwrap_or( prompt_value( "current system name" ) );
			
		let station_in = config.opt_str("t")
			.unwrap_or( prompt_value( "current station name" ) );
			
		let cargo_in = config.opt_str("c")
			.unwrap_or( prompt_value( "current cargo capacity in tons" ) );
		let cargo = u64::from_str( cargo_in.as_str() ).unwrap();
		
		let jump_range_in = config.opt_str("r")
			.unwrap_or( prompt_value( "current laden jump range in light years" ) );
		let jump_range = f64::from_str( jump_range_in.as_str() ).unwrap();
		
		let quality_in = config.opt_str("t")
			.unwrap_or("med".to_string());
		let quality : SearchQuality = SearchQuality::from_str(quality_in.as_str()).unwrap();
		
		Arguments {
//			system: system_in,
			station: station_in,
			cargo: cargo,
			jump_range: jump_range,
			search_quality: quality
		}
	}
}

fn main() {
	println!("{}", SEPARATOR );
	println!("Welcome to Austin's Elite Trading calculator.");
	println!("See https://github.com/austinjones/elitetrader/ for usage instructions.");
	println!("");
	
	println!("Thank you to to Paul Heisig and the maintainers of http://eddb.io/ for hosting the data used by this tool!");
	println!("");
	
	println!("This software is available under the GNU General Public License:");
	println!("https://www.gnu.org/copyleft/gpl.html");
	
	println!("{}", SEPARATOR );
	
	let opts = options();
	let opt_vals = match opts.parse( std::env::args() ) {
		Ok(opts) => opts,
		Err(reason) => { panic!( "Failed to parse command line arguments: {}", reason ) }
	};
	
	if opt_vals.opt_present( "h" ) {
		println!( "{}", opts.usage( "Usage: {} [options]\nMissing or invalid properties will be interactively prompted." ) );
		return;
	}
	
	let arguments = Arguments::collect( &opt_vals );
	
	println!("Loading Elite Dangerous universe data.");
	println!("");
	
	let universe = universe::load_universe();
	println!("Universe loaded!");
	
	println!("{}", SEPARATOR );
	
//	let mut system_name = arguments.system;
//	let mut system = None;
//	while !system.is_some() {
//		system = universe.get_system_by_name( &system_name );
//		
//		if !system.is_some() {
//			println!( "The system '{}' was not found.  Please try again.", system_name );
//			system_name = prompt_value( "corrected station name" );
//		}
//	}
//	
//	let mut system = system.unwrap().clone();
	
	let mut station_name = arguments.station;
	let mut station = None;
	while !station.is_some() {
		station = universe.get_station_by_name( &station_name );
		
		if !station.is_some() {
			println!( "The station '{}' was not found.", station_name );
			station_name = prompt_value( "corrected station name" );
		}
	}
	
	let mut station = station.unwrap().clone();
	let system = universe.get_system( &station.system_id ).unwrap();
	
	let mut analyzer = Analyzer {
		jump_range : 18.52,
		money : 20000,
		cargo_capacity: 216,
		universe: &universe
	};
	
	println!("");
	
	println!("Initalization complete. Starting route search from {}.{}", system.system_name, station.station_name );
	
	// depth 0
	for i in 1..5 {
		let profit = match analyzer.best_next_trade(&station, 60f64, 6, 5).first() {
			Some(trade) => {
				let profit_per_min = trade.profit_per_min.unwrap_or(0f64) as usize;
				let duration_minutes = trade.distance_in_seconds as f64 / 60f64;
				let new_balance = analyzer.money + trade.profit_total;
				
				println!("{}", SEPARATOR);
				println!("hop {}:\t{} [{}]", i,
					trade.buy_system.system_name,
					trade.buy_station.station_name);
				
				println!("");
				
				println!("buy:\t{}x {} [in {}]",
					trade.used_cargo,
					trade.commodity_name,
					trade.sell.commodity.category );
				
				println!("sell:\t{} [{}]",
					trade.sell_system.system_name,
					trade.sell_station.station_name
				);
				
				println!("\t{} profit for balance {}",
					format_credits( trade.profit_total as f64 ),
					format_credits( new_balance as f64 ) );
				
				println!("");
				
				println!("stats:\t{} profit/min over {:.1} mins", 
					format_credits( profit_per_min as f64 ),
					duration_minutes);
								
				println!("\t{} profit/ton for {} tons", 
					format_credits( trade.profit_per_ton as f64 ),
					trade.used_cargo);
				
				println!("\t{:.1} ly to system, {} ls to station, {:.1} min total",
					trade.distance_to_system,
					trade.distance_to_station,
					duration_minutes
				);
				
				station = trade.sell_station.clone();
				
				trade.profit_total
			},
			None => { println!("No trade found"); break; }
		};
		
		analyzer.money += profit;
	}
	
	fn format_credits( credits: f64 ) -> String {
		let qualifiers = ["cr", "Kcr", "Mcr", "Bcr", "Tcr"];
		
		let float = credits as f64;
		let log_factor = 1000f64;
		
		let base = float.log(log_factor).floor();
		let significand = float / log_factor.powf(base);
		
		let index = base as usize;
		let precision = std::cmp::min( index, 3 );
		
		format!("{:.*} {}", precision, significand, qualifiers[index] )
	}
}

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

mod analysis;
mod conversion;
mod data;
mod io;
mod messages;
mod options;
mod universe;

mod scored_buf;
mod map_list;

use std::thread;
use std::str::FromStr;
use time::PreciseTime;
use getopts::{Options, Matches};

use messages::*;
use analysis::{Analyzer};

pub const SEPARATOR : &'static str = "-------------------------------------------------------------------";
pub const CACHE_FILENAME : &'static str = ".elite_universe_min.json";

fn options() -> Options {
	let mut opts = Options::new();
//	opts.optopt("s", "system", "set current system name", "LTT 826");
	opts.optopt("t", "station", "set current station name", "GitHub");
	opts.optopt("c", "cargo", "maximum cargo capacity in tons. find this in your right cockpit panel's Cargo tab.", "216");
	opts.optopt("r", "range", "maximum laden jump range in light years.  find this in your outfitting menu.", "18.52");
	opts.optopt("b", "balance", "current credit balance", "18.52");
	opts.optopt("q", "quality", "search quality setting [low|med|high]", "med");
	
	opts.optflag("h", "help", "prints this help menu");
	opts
}

fn prompt_value( flag: &'static str, description: &'static str ) -> String {
	println!( "Please provide the flag -{}, or enter the {} now: ", flag, description );
	
	let mut val = String::new();
	match std::io::stdin().read_line(&mut val) {
		Err(reason) => panic!("Failed to read line: {}", reason ),
		_ => {}
	};
	
	val.trim().to_string()
}

enum SearchQuality {
	Low,
	Med,
	High
}

impl FromStr for SearchQuality {
    type Err = String;

    fn from_str(s: &str) -> Result<SearchQuality, String> {
        match s.to_lowercase().as_str() {
            "low" => Ok(SearchQuality::Low),
            "med" => Ok(SearchQuality::Med),
            "high" => Ok(SearchQuality::High),
            _ => Err( format!("Unknown enum variant '{}'", s) ),
        }
    }
}

struct Arguments {
//	pub system: String,
	pub station: String,
	pub cargo: u32,
	pub credit_balance: u32,
	pub jump_range: f64,
	pub search_quality: SearchQuality
}

impl Arguments {
	pub fn collect( config: &Matches ) -> Arguments {
//		let system_in = config.opt_str("s")
//			.unwrap_or( prompt_value( "current system name" ) );
			
		let station_in = match config.opt_str("t") {
			Some(t) => t,
			None => prompt_value( "t", "current station name" )
		};
		
		let cargo_in = match config.opt_str("c") {
			Some(v) => v,
			None => prompt_value( "c", "current cargo capacity in tons" )
		};
		let cargo_capacity = match u32::from_str( cargo_in.as_str() ) {
			Ok(v) => v,
			Err(reason) => panic!("Invalid cargo capacity '{}' - {}", cargo_in, reason)
		};
		
		let balance_in = match config.opt_str("b") {
			Some(v) => v,
			None => prompt_value( "b", "current credit balance" )
		};
		let balance = match u32::from_str( balance_in.as_str() ) {
			Ok(v) => v,
			Err(reason) => panic!("Invalid balance '{}' - {}", balance_in, reason)
		};
		
		let jump_range_in = match config.opt_str("r") {
			Some(v) => v,
			None => prompt_value( "r", "current laden jump range in light years" )
		};
		let jump_range = match f64::from_str( jump_range_in.as_str() ) {
			Ok(v) => v,
			Err(reason) => panic!("Invalid jump range '{}' - {}", jump_range_in, reason)
		};
		
		let quality_in = config.opt_str("q").unwrap_or( "med".to_string() );
		let quality : SearchQuality = match SearchQuality::from_str(quality_in.as_str()) {
			Ok(v) => v,
			Err(reason) => panic!("Invalid search quality '{}' - {}", quality_in, reason)
		};
		
		Arguments {
//			system: system_in,
			station: station_in,
			cargo: cargo_capacity,
			credit_balance: balance,
			jump_range: jump_range,
			search_quality: quality
		}
	}
}

fn main() {
	println!("{}", SEPARATOR );
	println!("Welcome to Austin's Elite Dangerous trading calculator.");
	println!("Use the -h or --help flags for instructions,\n visit https://github.com/austinjones/elitetrader/");
	println!("");
	
	println!("Thank you to to Paul Heisig and the maintainers of\n http://eddb.io/ for hosting the data used by this tool!");
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
		print!( "{}", HELP_MESSAGE_BEFORE_OPTS );
		println!( "{}", opts.usage("") );
		println!( "{}", HELP_MESSAGE_AFTER_OPTS );
		return;
	}
	
	let arguments = Arguments::collect( &opt_vals );
	
	println!("Loading Elite Dangerous universe data...");
	println!("");
	
	let universe = universe::load_universe();
	println!("");
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
			station_name = prompt_value( "t", "corrected station name" );
		}
	}
	
	let mut station = station.unwrap().clone();
	let system = universe.get_system( &station.system_id ).unwrap();
	
	let mut analyzer = Analyzer {
		jump_range : arguments.jump_range,
		credit_balance : arguments.credit_balance,
		cargo_capacity: arguments.cargo,
		universe: &universe
	};
	
	
	println!("Starting route search from {} [{}] ...", system.system_name, station.station_name );
	
	let mut i = 0;
	let mut last_trade : Option<(u32, f64, PreciseTime)> = None;
	
	// depth 0
	loop {
		i += 1;
		let (width, depth) = match arguments.search_quality {
			SearchQuality::High => (7, 5),
			SearchQuality::Med => (6, 5),
			SearchQuality::Low => (3, 5)
		};
		
		let profit = match analyzer.best_next_trade(&station, 60f64, width, depth).first() {
			Some(trade) => {
				let expected_profit_per_min = trade.profit_per_min.unwrap_or(0f64);
				let expected_minutes = trade.distance_in_seconds as f64 / 60f64;
				let new_balance = analyzer.credit_balance + trade.profit_total;
				
				// the first trade is from the station the user is docked at
				// so calculate it automatically
				if last_trade.is_some() {
					println!("");
					println!("wait:\tpress <enter> once trade is complete.");
					
					let mut str = String::new();
					match std::io::stdin().read_line(&mut str) {
						Err(reason) => panic!("Failed to read line: {}", reason ),
						_ => {}
					};
				}
				
				match last_trade {
					Some((total_profit, expected_min, start_time)) => {
						let span = start_time.to( PreciseTime::now() );
						let minutes = span.num_milliseconds() as f64 / 60000f64;
						
						let profit_per_min = total_profit as f64 / minutes;
						let ratio = minutes / expected_min;
						
						println!("actual:\t{:.1} per min over {:.1} minutes",
							profit_per_min, minutes );
						
						let compare = match ratio {
							0f64...1f64 => 100f64 * (1f64-ratio),
							_ => 100f64 * (ratio-1f64)
						};
						println!("\t{:.2}% faster than expected", compare);
						println!("{}", &SEPARATOR.to_string());
						
						// this makes the timer result more visible,
						// and tricks the user into thinking 
						// trade calculations take 1 second
						thread::sleep_ms( 1000 );
					}, 
					_ => {}
				};
				
				println!("hop {}:\t{} [{}]", i,
					trade.buy_system.system_name,
					trade.buy_station.station_name);
				
				println!("");
				
				println!("buy:\t{}x {} [{} category]",
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
				println!("expect:\t{} profit/min over {:.1} mins", 
						format_credits( expected_profit_per_min as f64 ),
						expected_minutes);

								
				println!("\t{} profit/ton for {} tons",
					format_credits( trade.profit_per_ton as f64 ),
					trade.used_cargo);
				
				println!("\t{:.1} ly to system, {} ls to station, {:.1} min total",
					trade.distance_to_system,
					trade.distance_to_station,
					expected_minutes
				);
				
				last_trade = Some((trade.profit_total, expected_minutes, PreciseTime::now()));
				station = trade.sell_station.clone();
				
				trade.profit_total
			},
			None => { println!("No trade found"); break; }
		};
		
		analyzer.credit_balance += profit;
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

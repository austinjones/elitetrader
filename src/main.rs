#![feature(custom_derive)]
#![feature(collections)]
#![feature(core)]
#![feature(convert)]
#![feature(path_ext)]
#![feature(std_misc)]
#![feature(fs_time)]
#![feature(scoped)]

extern crate rustc_serialize;
extern crate spatial;
extern crate core;
extern crate rand;
extern crate hyper;
extern crate flate2;
extern crate time;
extern crate getopts;
extern crate num;

mod data;
mod io;
mod messages;
mod search;
mod util;


use data::Universe;
use std::thread;
use std::str::FromStr;
use time::PreciseTime;
use getopts::{Options, Matches};

use search::SearchStation;
use search::PlayerState;
use search::SearchQuality;
use util::num_unit::*;
use messages::*;
use data::trader::ShipSize;
use data::PriceUpdate;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const SEPARATOR : &'static str = "-------------------------------------------------------------------";
pub const CACHE_FILENAME : &'static str = concat!(".elite_universe_", env!("CARGO_PKG_VERSION"), ".min.json");

fn options() -> Options {
	let mut opts = Options::new();
//	opts.optopt("s", "system", "set current system name", "LTT 826");
	opts.optopt("t", "station", "current station name", "GitHub");
	opts.optopt("c", "cargo", "maximum cargo capacity in tons. find this in your right cockpit panel's Cargo tab.", "216");
	opts.optopt("r", "range", "maximum laden jump range in light years.  find this in your outfitting menu.", "18.52");
	opts.optopt("b", "balance", "current credit balance", "525.4k");
	opts.optopt("m", "minbalance", "minimum credit balance - safety net for rebuy", "3.5m");
	opts.optopt("q", "quality", "search quality setting [low|med|high]", "med");
	opts.optopt("p", "shipsize", "current ship size (small|med|large)", "large");
	opts.optopt("d", "debug", "searches to the given hop length and prints stats", "12");
	
	opts.optflag("h", "help", "prints this help menu");
	opts
}

fn prompt_value( flag: &'static str, description: &'static str ) -> String {
	println!( "Please provide the flag -{}, or enter the {} now: ", flag, description );
	
	let val = read_line();
	
	println!("");
	
	val
}

struct Arguments {
//	pub system: String,
	pub station: String,
	pub cargo: u32,
	pub credit_balance: u32,
	pub minimum_balance: u32,
	pub jump_range: f64,
	pub ship_size: ShipSize,
	pub search_quality: SearchQuality
}

fn read_line() -> String {
	let mut str = String::new();
	match std::io::stdin().read_line(&mut str) {
		Err(reason) => panic!("Failed to read line: {}", reason ),
		_ => {}
	};
	str.trim().to_string()
}

impl Arguments {
	pub fn collect( config: &Matches ) -> Arguments {
//		let system_in = config.opt_str("s")
//			.unwrap_or( prompt_value( "current system name" ) );
		
		let station_in = match config.opt_str("t") {
			Some(t) => t,
			None => prompt_value( "t", "current station name (e.g. Git Hub)" )
		};
		
		
		let cargo_in = match config.opt_str("c") {
			Some(v) => v,
			None => prompt_value( "c", "current cargo capacity in tons (e.g. 216)" )
		};
		let cargo_capacity = match NumericUnit::from_str( cargo_in.as_str() ) {
			Ok(v) => v.to_num(),
			Err(reason) => panic!("Invalid cargo capacity '{}' - {}", cargo_in, reason)
		};
		
		
		let balance_in = match config.opt_str("b") {
			Some(v) => v,
			None => prompt_value( "b", "current credit balance (e.g. 525.4k or 525412)" )
		};
		let balance = match NumericUnit::from_str( balance_in.as_str() ) {
			Ok(v) => v.to_num(),
			Err(reason) => panic!("Invalid balance '{}' - {}", balance_in, reason)
		};
		
		
		let minimum_balance_in = match config.opt_str("m") {
			Some(v) => v,
			None => prompt_value( "b", "minimum credit balance - saftey net for rebuy (e.g. 3.2m)" )
		};
		let minimum_balance = match NumericUnit::from_str( minimum_balance_in.as_str() ) {
			Ok(v) => v.to_num(),
			Err(reason) => panic!("Invalid minimum balance '{}' - {}", minimum_balance_in, reason)
		};
		
		
		let jump_range_in = match config.opt_str("r") {
			Some(v) => v,
			None => prompt_value( "r", "current laden jump range in light years" )
		};
		let jump_range = match NumericUnit::from_str( jump_range_in.as_str() ) {
			Ok(v) => v.to_num(),
			Err(reason) => panic!("Invalid jump range '{}' - {}", jump_range_in, reason)
		};
		
		
		let ship_size_in = match config.opt_str("p") {
			Some(v) => v,
			None => prompt_value( "r", "current ship size [small|med|large], or [s|m|l]" )
		};
		let ship_size = match ShipSize::from_str( ship_size_in.as_str() ) {
			Ok(v) => v,
			Err(reason) => panic!("Invalid ship size '{}' - {}", ship_size_in, reason)
		};
		
		
		let quality_in = config.opt_str("q").unwrap_or( "high".to_string() );
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
			minimum_balance: minimum_balance,
			ship_size: ship_size,
			search_quality: quality
		}
	}
}

fn main() {
	println!("{}", SEPARATOR );
	println!("Welcome to Austin's Elite Dangerous trading calculator v{}", VERSION);
	println!("Use the -h or --help flags for instructions,\n visit https://github.com/austinjones/elitetrader/");
	println!("");
	
	println!("Thank you to to Paul Heisig and the maintainers of\n http://eddb.io/ for hosting the data used by this tool!");
	println!("");
	
	println!("This software is distributed under the GNU General Public License:");
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
	
	println!("Loading Elite Dangerous universe data...");
	println!("");
	
	let arguments = Arguments::collect( &opt_vals );
	
	let universe = Universe::load(&arguments.ship_size);
	
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
	
	let station = station.unwrap();
	
	let state = PlayerState {
		universe: &universe,
		station: &station,
		
		credit_balance: arguments.credit_balance,
		minimum_balance: arguments.minimum_balance,
		
		jump_range: arguments.jump_range,
		cargo_capacity: arguments.cargo
	};
	
	println!("");
	println!("Universe loaded!");
	match opt_vals.opt_str("d") {
		Some(str) => {
			let depth = match usize::from_str( str.as_str() ) {
				Ok(v) => v,
				Err(reason) => panic!("Invalid debug depth '{}': {}", str, reason)
			};
			
			run_debug( &state, arguments.search_quality, depth );
		},
		None => {
			run_search( &state, arguments.search_quality );
		}
	}
}
	
fn run_debug( state: &PlayerState, search_quality: SearchQuality, hops: usize ) {
	println!("{}", SEPARATOR );
	
	let hop_width = search_quality.get_hop_width();
	let depth = search_quality.get_depth();
	let total = hop_width.pow( depth as u32 );
	
	println!("Enumerating {} trades per station to a depth of {} hops ...", hop_width, depth );
	println!("Total stations: {}", total);
	
	println!("{}", SEPARATOR );
	let mut search = SearchStation::new( state.clone(), search_quality );
	
	let mut profit_total = 0;
	let mut cost_in_seconds = 0f64;
	println!("hop\tprofit\tly\tminutes\tprofit/min\tcargo\tcmdy.\tsystem\tstation");
	
	for i in 0..hops {
		match search.next_trade() {
			Some(search_trade) => {
				search = search_trade.sell_station;
				let trade = search_trade.trade;
				
				profit_total += trade.profit_total;
				cost_in_seconds += trade.distance_in_seconds;
				
				let minutes = trade.distance_in_seconds / 60f64;
				let profit_per_min = trade.profit_total as f64 / minutes;
				
				println!("{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{}\t{}\t{}\t{}",
					i, 
					trade.profit_total,
					trade.distance_to_system,
					minutes,
					profit_per_min,
					trade.used_cargo,
					trade.commodity_name,
					trade.sell_system.system_name,
					trade.sell_station.station_name,
				);
			},
			None => { println!("No trade found"); break; }
		};
	}
	
	let minutes = cost_in_seconds / 60f64;
	let profit_per_min = match minutes {
		0f64 => 0f64,
		_ => profit_total as f64 / minutes
	};
	
	println!("{}", SEPARATOR );
	println!("hops\tprofit\tminutes\tprofit/min");
	println!("{}\t{}\t{:.3}\t{:.3}", hops, profit_total, minutes, profit_per_min);
	println!("{}", SEPARATOR );
	println!("{} hops", hops );
	println!("{} total profit", NumericUnit::new_string( profit_total, &"cr".to_string() ) );
	println!("{:.1} minutes", minutes );
	println!("{} profit/min", NumericUnit::new_string( profit_per_min, &"cr".to_string() ));
}

fn read_price_update() -> u16 {
	println!("Enter the sell price per ton:");
	
	let line = read_line();
	let val = match u16::from_str( line.as_str() ) {
		Ok(price) => price,
		Err(reason) => {
			println!("Failed to parse answer '{}' ({}).  Please try again.", line, reason);
			read_price_update()
		}
	};
	
	println!("");
	val
}

fn run_search( state: &PlayerState, search_quality: SearchQuality ) {
	let hop_width = search_quality.get_hop_width();
	let depth = search_quality.get_depth();
	
	let station = state.station;
	let system = state.universe.get_system( &state.station.system_id ).unwrap();
	
	println!("");
	println!("Starting route search from {} [{}] ...", system.system_name, station.station_name );
	println!("Enumerating {} trades per station to a depth of {} hops ...", hop_width, depth );
	println!("{}", SEPARATOR );
	
	let mut i = 0;
	let mut last_trade : Option<(u32, f64, PreciseTime)> = None;
	
	let mut sum_profit = 0;
	let mut sum_minutes = 0f64;
	
	let mut search = SearchStation::new( state.clone(), search_quality );
//	let mut price_updates = Vec::new();
	
	loop {
		i += 1;
		
		match search.next_trade() {
			Some(search_trade) => {
				search = search_trade.sell_station;
				let trade = search_trade.trade;
				
				let state = trade.state_after_trade();
				let expected_profit_per_min = trade.profit_per_min.unwrap_or(0f64);
				let expected_minutes = trade.distance_in_seconds as f64 / 60f64;
				let new_balance = state.credit_balance + trade.profit_total;
				
				// the first trade is from the station the user is docked at
				// so calculate it automatically
				let quit = if last_trade.is_some() {
					println!("");
					println!("wait:\tonce trade is complete, press <enter> for next, or q to quit");
					
					let str = read_line();
					match str.as_str() {
//						"u" | "update" => {
//							let new_price = read_price_update();
//							price_updates.push( PriceUpdate::new_sell_update( new_price, trade.sell ) );
//						},
						"q" | "quit"  => true,
						_ => false
					}
				} else {
					false
				};
				
				match last_trade {
					Some((total_profit, expected_min, start_time)) => {
						let span = start_time.to( PreciseTime::now() );
						let minutes = span.num_milliseconds() as f64 / 60000f64;
						
						let profit_per_min = total_profit as f64 / minutes;
						let ratio = minutes / expected_min;
						
						println!("actual:\t{} per min over {:.1} minutes",
							NumericUnit::new_string( profit_per_min, &"cr".to_string()),
							minutes );
						
						let compare = match ratio {
							0f64...1f64 => 100f64 * (1f64-ratio),
							_ => 100f64 * (ratio-1f64)
						};
						println!("\t{:.2}% faster than expected", compare);
						println!("{}", &SEPARATOR.to_string());
						
						sum_profit += total_profit;
						sum_minutes += minutes;
						
						// this makes the timer result more visible,
						// and tricks the user into thinking 
						// trade calculations take 1 second
						// the real caulcation happens when the player
						// is flying the trade route
						if !quit {
							thread::sleep_ms( 1000 );
						}
					}, 
					_ => {}
				};
				
				if quit {
					break;
				}
				
				println!("hop {}:\t{} [{}]", i,
					trade.buy_system.system_name,
					trade.buy_station.station_name);
				
				println!("");
				
				println!("buy:\t{}x {} [{}]",
					trade.used_cargo,
					trade.commodity_name,
					trade.sell.commodity.category );
				
				println!("sell:\t{} [{}]",
					trade.sell_system.system_name,
					trade.sell_station.station_name
				);
				
				println!("\t{} profit for balance {}",
					NumericUnit::new_string( trade.profit_total, &"cr".to_string()),
					NumericUnit::new_string( new_balance, &"cr".to_string()) );
				
				println!("");
				println!("expect:\t{} profit/min over {:.1} mins", 
						NumericUnit::new_string( expected_profit_per_min, &"cr".to_string()),
						expected_minutes);

								
				println!("\t{} profit/ton for {} tons",
					NumericUnit::new_string( trade.profit_per_ton, &"cr".to_string()),
					trade.used_cargo);
				
				println!("\t{:.1} ly to system, {} ls to station",
					trade.distance_to_system,
					trade.distance_to_station
				);
				
				last_trade = Some((trade.profit_total, expected_minutes, PreciseTime::now()));
			},
			None => { println!("No trade found"); break; }
		};
	}
	
	let profit_per_min = match sum_minutes {
		0f64 => 0f64,
		_ => sum_profit as f64 / sum_minutes
	};
	
	println!("Trade Summary!");
	println!("\t{} profit/min over {:.1} mins", 
		NumericUnit::new_string( profit_per_min, &"cr".to_string() ),
		sum_minutes );
	
	println!("\t{} total profit", 
		NumericUnit::new_string( sum_profit, &"cr".to_string() ) );
		
	println!("\tstart balance {} -> end balance {}", 
		NumericUnit::new_string( state.credit_balance, &"cr".to_string() ),
		NumericUnit::new_string( state.credit_balance + sum_profit, &"cr".to_string() ) );
	
	println!("Done!");
	// print overall stats
	// save price updates
}

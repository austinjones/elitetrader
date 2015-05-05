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
extern crate num;

mod arguments;
mod data;
mod messages;
mod persist;
mod search;
mod util;
mod user_input;


use std::thread;
use time::PreciseTime;
use std::str::FromStr;
use getopts::{Options};

use arguments::Arguments;
use search::SearchStation;
use search::PlayerState;
use search::SearchQuality;
use util::num_unit::*;
use messages::*;
use data::PriceUpdate;
use data::Universe;
use data::IndexedUniverse;

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
	let mut universe = Universe::load(&arguments.ship_size);
	let indexed_universe = IndexedUniverse::calculate( &universe );
	let player_state = PlayerState::new( &arguments, &indexed_universe );
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
	
	println!("");
	println!("Universe loaded!");
	println!("{}", SEPARATOR);
	
	match opt_vals.opt_str("d") {
		Some(str) => {
			let depth = match usize::from_str( str.as_str() ) {
				Ok(v) => v,
				Err(reason) => panic!("Invalid debug depth '{}': {}", str, reason)
			};
			
			run_debug( &indexed_universe, &player_state, arguments.search_quality, depth );
		},
		None => {
			run_search( &mut universe, indexed_universe, &player_state, arguments.search_quality );
		}
	}
}
	
fn run_debug( iuniverse: &IndexedUniverse, state_in: &PlayerState, search_quality: SearchQuality, hops: usize ) {
	let hop_width = search_quality.get_hop_width();
	let depth = search_quality.get_depth();
	let total = hop_width.pow( depth as u32 );
	
	println!("Enumerating {} trades per station to a depth of {} hops ...", hop_width, depth );
	println!("Total stations: {}", total);
	
	println!("{}", SEPARATOR );
	
	let mut profit_total = 0;
	let mut cost_in_seconds = 0f64;
	println!("hop\tprofit\tly\tls\tminutes\tprofit/min\tcargo\tcmdy.\tsystem\tstation");
	
	let mut state = state_in.clone();
	
	for i in 0..hops {
		let mut search = SearchStation::new( state.clone(), search_quality.clone() );
		match search.next_trades(iuniverse).iter().next() {
			Some(trade) => {
				profit_total += trade.profit_total;
				cost_in_seconds += trade.unit.distance_in_seconds;
				
				let minutes = trade.unit.distance_in_seconds / 60f64;
				let profit_per_min = trade.profit_total as f64 / minutes;
				
				println!("{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{}\t{}\t{}\t{}",
					i, 
					trade.profit_total,
					trade.unit.distance_to_system,
					trade.unit.distance_to_station,
					minutes,
					profit_per_min,
					trade.used_cargo,
					trade.unit.commodity_name,
					trade.unit.sell_system.system_name,
					trade.unit.sell_station.station_name,
				);
				
				state = trade.state_after_trade();
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

fn run_search( universe: &mut Universe, mut iuniverse: IndexedUniverse, 
				state_in: &PlayerState, search_quality: SearchQuality ) {
	let hop_width = search_quality.get_hop_width();
	let depth = search_quality.get_depth();
	
	let station = iuniverse.get_station( &state_in.station_id ).unwrap().clone();
	let system = iuniverse.get_system( &state_in.system_id ).unwrap().clone();
	
	println!("Starting route search from {} [{}] ...", system.system_name, station.station_name );
	println!("Enumerating {} trades per station to a depth of {} hops ...", hop_width, depth );
	println!("{}", SEPARATOR );
	
	let mut i = 0;
//	let mut last_trade : Option<(u32, f64, PreciseTime)> = None;
	
	let mut sum_profit = 0;
	let mut sum_minutes = 0f64;
	
	let mut player_state = state_in.clone();
	
//	let mut price_updates = Vec::new();
	let mut quit = false;
	let mut reindex = false;
	
	'hop: while !quit {
		if reindex {
			println!("Recalculating index due to price update...");
			iuniverse = IndexedUniverse::calculate( universe );
			println!("{}", SEPARATOR );
			reindex = false;
		}
		
		let search_quality = match i {
			0 => SearchQuality::Low,
			_ => search_quality
		};
		
		let mut search = SearchStation::new( player_state.clone(), search_quality );
		let mut trades = search.next_trades(&iuniverse);
		
		let mut accepted_trade = None;
		
		'trade: for trade in trades.drain() {
			let trade_state = trade.state_after_trade();
			let expected_profit_per_min = trade.profit_per_min.unwrap_or(0f64);
			let expected_minutes = trade.unit.distance_in_seconds as f64 / 60f64;
			
			println!("hop {}:\t{} [{}]", i,
				trade.unit.buy_system.system_name,
				trade.unit.buy_station.station_name);
			
			println!("");
			
			println!("buy:\t{}x {} [{}]",
				trade.used_cargo,
				trade.unit.commodity_name,
				trade.unit.sell.commodity.category );
			
			println!("sell:\t{} [{}]",
				trade.unit.sell_system.system_name,
				trade.unit.sell_station.station_name
			);
			
			println!("\t{} profit for balance {}",
				NumericUnit::new_string( trade.profit_total, &"cr".to_string()),
				NumericUnit::new_string( trade_state.credit_balance, &"cr".to_string()) );
			
			println!("");
			println!("expect:\t{} profit/min over {:.1} mins", 
					NumericUnit::new_string( expected_profit_per_min, &"cr".to_string()),
					expected_minutes);

							
			println!("\t{} profit/ton for {} tons",
				NumericUnit::new_string( trade.unit.profit_per_ton, &"cr".to_string()),
				trade.used_cargo);
			
			println!("\t{:.1} ly to system, {} ls to station",
				trade.unit.distance_to_system,
				trade.unit.distance_to_station
			);
			
			println!("");
			println!("start:\t<enter> to accept trade, u to update buy price, or n for new trade");
			
			// the first trade is from the station the user is docked at
			// so calculate it automatically
			let str = user_input::read_line();
			match str.as_str() {
//					"u" | "update" => {
//						let new_price = read_price_update();
//						price_updates.push( PriceUpdate::new_sell_update( new_price, trade.sell ) );
//					},
				"u" | "update"  => {
					let buy_price = user_input::read_price_update("buy price");
					let supply = user_input::read_price_update("supply");
					
					let update = PriceUpdate::new_buy_update(buy_price, supply, trade.unit.buy);
					universe.apply_update( update );
					
					reindex = true;
					println!("{}", SEPARATOR);
					continue 'hop;
				},
				"n" | "new"  => { continue; },
				_ => { accepted_trade = Some(trade); break }
			}
		};
		
		if !accepted_trade.is_some() {
			println!("No trade found"); 
			break;
		}
		
		i += 1;
		let mut trade = accepted_trade.unwrap();
		
		let start_time = PreciseTime::now();
		println!("end:\t<enter> to complete trade, u to update sell price, or q to complete route");
		
		// the first trade is from the station the user is docked at
		// so calculate it automatically
		let str = user_input::read_line();
		match str.as_str() {
			"u" | "update"  => {
					let sell_price = user_input::read_price_update("sell price");
					
					let update = PriceUpdate::new_sell_update(sell_price, trade.unit.sell);
					universe.apply_update( update );
					
					reindex = true;
					
					trade = trade.with_sell_price( sell_price );
			},
			"q" | "quit"  => { quit = true; },
			_ => {}
		}
		
		let trade_state = trade.state_after_trade();
		
		let span = start_time.to( PreciseTime::now() );
		let minutes = span.num_milliseconds() as f64 / 60000f64;
		let expected_minutes = trade.unit.distance_in_seconds as f64 / 60f64;
		
		
		let profit_per_min = trade.profit_total as f64 / minutes;
		let ratio = minutes / expected_minutes;
		
		println!("actual:\t{} per min over {:.1} minutes",
			NumericUnit::new_string( profit_per_min, &"cr".to_string()),
			minutes );
		
		let (compare, text) = match ratio {
			0f64...1f64 => (100f64 * (1f64-ratio), "faster"),
			_ => (100f64 * (ratio-1f64), "slower")
		};
		
		println!("\t{:.2}% {} than expected", compare, text);
		println!("{}", &SEPARATOR.to_string());
		
		sum_profit += trade.profit_total;
		sum_minutes += minutes;
		
		player_state = trade_state;
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
		NumericUnit::new_string( player_state.credit_balance, &"cr".to_string() ),
		NumericUnit::new_string( player_state.credit_balance + sum_profit, &"cr".to_string() ) );
	
	println!("Done!");
	// print overall stats
	// save price updates
}

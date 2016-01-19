// todo: delete old universe cachefiles
//#![feature(custom_derive)]
//#![feature(collections)]
//#![feature(core)]
//#![feature(convert)]
//#![feature(path_ext)]
//#![feature(std_misc)]
//#![feature(fs_time)]

extern crate rustc_serialize;
extern crate csv;
extern crate spatial;
extern crate core;
extern crate rand;
extern crate hyper;
extern crate flate2;
extern crate time;
extern crate getopts;
extern crate num;
extern crate filetime;
extern crate crossbeam;
extern crate num_cpus;
extern crate statistical;

mod arguments;
mod data;
mod messages;
mod persist;
mod search;
mod util;
mod user_input;
mod processor;

use time::PreciseTime;
use std::str::FromStr;
use getopts::{Options, Matches};

use arguments::Arguments;
use search::SearchStation;
use search::PlayerState;
use search::SearchQuality;
use search::SearchCache;
use util::num_unit::*;
use messages::*;
use data::TimeAdjustment;
use data::PriceAdjustment;
use data::Universe;
use data::EdceData;

use user_input::prompt_value;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const SEPARATOR : &'static str = "-------------------------------------------------------------------";
pub const CACHE_FILENAME : &'static str = concat!("elite_universe_", env!("CARGO_PKG_VERSION"), ".min.json");

fn options() -> Options {
	let mut opts = Options::new();
	opts.optopt("s", "system", "set current system name", "LTT 826");
	opts.optopt("t", "station", "current station name", "GitHub");
	opts.optopt("c", "cargo", "maximum cargo capacity in tons. find this in your right cockpit panel's Cargo tab.", "216");
	opts.optopt("r", "range", "maximum laden jump range in light years.  find this in your outfitting menu.", "18.52");
	opts.optopt("b", "balance", "current credit balance", "525.4k");
	opts.optopt("m", "minbalance", "minimum credit balance - safety net for rebuy", "3.5m");
	opts.optopt("q", "quality", "search quality setting [med|high|ultra]", "med");
	opts.optopt("p", "shipsize", "current ship size (small|med|large)", "large");
	opts.optopt("d", "debug", "searches to the given hop length and prints stats", "12");
	opts.optopt("C", "edce", "enables Elite Dangerous Companion Emulator integration - \
		automatically sets all user state (except for jump range) when enabled", 
		"<full path to emulator directory>");
	
	opts.optflag("i", "timetables", "prints time tables");
	opts.optflag("h", "help", "prints this help menu");
	opts.optflag("A", "autoaccept", "automatically accepts trade options");
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
	
	if opt_vals.opt_present( "i" ) {
		run_timetables( &opt_vals );
		return;
	}
	
	println!("Loading Elite Dangerous universe data...");
	println!("");
	
	let edce_data = EdceData::generate_opt( &opt_vals.opt_str("C") );
	let arguments = Arguments::collect( &opt_vals, &edce_data );

	let mut universe = Universe::load(&arguments.ship_size);
	let player_state = PlayerState::new( &arguments, &universe );
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
			let depth = match usize::from_str( &str[..] ) {
				Ok(v) => v,
				Err(reason) => panic!("Invalid debug depth '{}': {}", str, reason)
			};
			
			if depth > 0 {
				run_debug( &mut universe, &player_state, arguments.search_quality, depth );
			} else {
				run_diagnostic( &mut universe, &player_state, arguments.search_quality );
			}
		},
		None => {
			run_search( &mut universe, &arguments, &player_state, arguments.search_quality );
		}
	}
}

fn run_timetables( config: &Matches ) {
	let jump_range_in = match config.opt_str("r") {
		Some(v) => v,
		None => prompt_value( "r", "current laden jump range in light years" )
	};
	let jump_range = match NumericUnit::from_str( &jump_range_in[..] ) {
		Ok(v) => v.to_num(),
		Err(reason) => panic!("Invalid jump range '{}' - {}", jump_range_in, reason)
	};
	
	println!("ls\tseconds");
	for i in 1..100 {
		let ls = (i as f64).powf(1.8f64).floor();
		let time = search::time_estimate::TimeEstimate::supercruise_time(ls);
		println!("{:.0}\t{:.2}", ls, time);
	}
	
	println!("{}", SEPARATOR );
	
	println!("ly\tseconds\tjumps");
	for ly in 1..120 {
		let jumps = search::time_estimate::TimeEstimate::jump_count(ly as f64, jump_range);
		let time = search::time_estimate::TimeEstimate::jump_time(ly as f64, jump_range);
		println!("{:.2}\t{:.2}\t{:.1}", ly, time, jumps);
	}
}

fn run_diagnostic( universe: &mut Universe, state_in: &PlayerState, search_quality: SearchQuality ) {
	let hop_width = search_quality.get_hop_width();
	let depth = search_quality.get_depth();
	let total_routes = hop_width.pow( depth as u32 );
	
	println!("Enumerating {} trades per station to a depth of {} hops ...", hop_width, depth );
	println!("Total routes to examine: {}", total_routes);
	
	println!("{}", SEPARATOR );
	
	println!("\troute:\t\t\ttrade:");
	println!("option\tpft/min\tmins\tprofit\tpft/min\tmins\tprofit\tcmdy.\tplanetary\tsystem\tstation");
	
	let mut search_cache = SearchCache::new();
	let universe_snapshot = universe.snapshot();
	let mut search = SearchStation::new( state_in.clone(), search_quality.clone() );
	let trades = search.next_trades(&universe_snapshot, &mut search_cache);
	
	for (i, result) in trades.iter().enumerate() {
		let minutes = result.time_total / 60f64;
		let profit_per_min = result.profit_per_min();
		println!("{}\t{:.0}\t{:.2}\t{}\t{:.0}\t{:.2}\t{}\t{}\t{}\t{}\t{}",
			i,
			profit_per_min,
			minutes,
			result.profit_total,
			result.trade.profit_per_min,
			result.trade.unit.adjusted_time.time_total/60f64,
			result.trade.profit_total,
			result.trade.unit.commodity_name,
			if result.trade.unit.sell_station.is_planetary { "planetary" } else { "station" }, 
			result.trade.unit.sell_system.system_name,
			result.trade.unit.sell_station.station_name,
		);
	}
}

fn run_debug( universe: &mut Universe, state_in: &PlayerState, search_quality: SearchQuality, hops: usize ) {
	let hop_width = search_quality.get_hop_width();
	let depth = search_quality.get_depth();
	let total_routes = hop_width.pow( depth as u32 );
	
	println!("Enumerating {} trades per station to a depth of {} hops ...", hop_width, depth );
	println!("Total routes to examine: {}", total_routes);
	
	println!("{}", SEPARATOR );
	
	let mut profit_total = 0;
	let mut time_total = 0f64;
	println!("hop\tms\tcache\tmins\tpft/min\tprofit\ttrips\tly\tls\tcargo\tcmdy.\tsystem\tstation");
	
	let mut state = state_in.clone();
	let search_cache = SearchCache::new();
		
	for i in 0..hops {
		let universe_snapshot = universe.snapshot();
		let mut search = SearchStation::new( state.clone(), search_quality.clone() );
			let process_start = time::precise_time_s();
		let trades = search.next_trades(&universe_snapshot, &search_cache);		
			let process_end = time::precise_time_s();
			
		let process_time_ms = 1000f64 * (process_end - process_start) ;
//		for (index, result) in trades.iter().enumerate() {
//			println!("r{}: {:?}", index, result );
//		}
		match trades.iter().next() {
			Some(result) => {
//				println!("SearchCache has {} entries", search_cache.trade_cache.len() );
				let trade = &result.trade;
				profit_total += trade.profit_total;
				time_total += trade.unit.normalized_time.time_total;
				
				let minutes = trade.unit.normalized_time.time_total / 60f64;
				let profit_per_min = trade.profit_total as f64 / minutes;
				println!("{}\t{:.0}\t{}\t{:.2}\t{:.0}\t{}\t{:.1}\t{:.2}\t{}\t{}\t{}\t{}\t{}",
					i,
					process_time_ms,
					search_cache.len(),
					minutes,
					profit_per_min,
					trade.profit_total,
					trade.unit.credit_potential() as f64 / trade.profit_total as f64,
					trade.unit.normalized_time.distance_to_system,
					trade.unit.normalized_time.distance_to_station,
					trade.used_cargo,
					trade.unit.commodity_name,
//					if trade.unit.sell_station.is_planetary { "planetary" } else { "station" }, 
					trade.unit.sell_system.system_name,
					trade.unit.sell_station.station_name,
				);
				
				state = trade.state_after_trade();
				universe.apply_trade(trade, &search_cache);
			},
			None => { println!("No trade found"); break; }
		};
	}
	
	let minutes = time_total / 60f64;
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

fn run_search( universe: &mut Universe, args: &Arguments, state_in: &PlayerState, search_quality: SearchQuality ) {
	let hop_width = search_quality.get_hop_width();
	let depth = search_quality.get_depth();
	let total_routes = hop_width.pow( depth as u32 );
	
	println!("Enumerating {} trades per station to a depth of {} hops ...", hop_width, depth );
	println!("Total routes to examine: {}", NumericUnit::new_string(total_routes, &"".to_string()));
	println!("{}", SEPARATOR );
	
	let mut i = 0;
//	let mut last_trade : Option<(u32, f64, PreciseTime)> = None;
	
	let mut sum_profit = 0;
	let mut sum_minutes = 0f64;
	
	let start_state = state_in.clone();
	let mut player_state = state_in.clone();
	
	let is_edce = args.edce_path.is_some();
	
	let search_cache = SearchCache::new();
	
//	let mut price_updates = Vec::new();
	let mut quit = false;
	'route: while !quit {
//		let search_quality = match i {
//			0 => SearchQuality::Medium,
//			_ => search_quality
//		};

		println!("wait:\tcalculating ...");
		
		let universe_snapshot = universe.snapshot();
		
		let mut search = SearchStation::new( player_state.clone(), search_quality );
		let mut results = search.next_trades(&universe_snapshot, &search_cache);
//		for (index, result) in results.iter().enumerate() {
//			println!("r{}: {:?}", index, result );
//		}
		
		let mut accepted_trade = None;
		
		'trade: for result in results.drain(..) {
			let trade = result.trade.clone();
			let trade_state = trade.state_after_trade();
			let expected_profit_per_min = trade.profit_per_min;
			let expected_minutes = trade.unit.adjusted_time.time_total / 60f64;
			
			println!("{}", SEPARATOR);
			
			let now = time::now();
			println!("hop {}:\t{}, estimated {} profit/min over next {:.0} minutes", 
				i,
				now.strftime("%l:%M%p").unwrap().to_string().trim(),
				NumericUnit::new_string( result.profit_per_min(), &"cr".to_string()),
				result.time_total / 60f64
			 );
			
			println!("");
			
			println!("buy:\t{} [{}]",
				trade.unit.buy_system.system_name,
				trade.unit.buy_station.station_name,
			);
			
			println!("\t{} [{}] at {} x {}",
				trade.unit.sell.commodity.category,
				trade.unit.commodity_name,
				NumericUnit::new_string( trade.unit.buy.buy_price, &"cr".to_string() ),
				trade.used_cargo );
				
			println!("supply:\t{} [{} over {:.2} hours]",
				NumericUnit::new_string( trade.unit.buy.supply, &"tn".to_string() ),
				NumericUnit::new_string( trade.unit.credit_potential(), &"cr".to_string() ),
				trade.max_runs() * trade.unit.adjusted_time.time_total / 3600f64
			);
			
			println!("");
			
			println!("sell:\t{} [{}] at {}{}",
				trade.unit.sell_system.system_name,
				trade.unit.sell_station.station_name,
				NumericUnit::new_string( trade.unit.sell.sell_price, &"cr".to_string() ),
				if trade.unit.is_prohibited { ", Illegal Cargo!" } else { "" }
			);
			
			println!("\t{} profit for balance {}",
				NumericUnit::new_string( trade.profit_total, &"cr".to_string()),
				NumericUnit::new_string( trade_state.credit_balance, &"cr".to_string()) );
			
			println!("");
			println!("expect:\t{} profit/min from trade over {:.1} mins", 
					NumericUnit::new_string( expected_profit_per_min, &"cr".to_string()),
					expected_minutes);
							
			println!("\t{} profit/ton for {} tons",
				NumericUnit::new_string( trade.unit.profit_per_ton, &"cr".to_string()),
				trade.used_cargo);
			
			println!("\t{:.0} ly to system [{:.1} mins]",
				trade.unit.adjusted_time.distance_to_system,
				trade.unit.adjusted_time.time_to_system / 60f64
			);
			
			println!("\t{:.0} ls to station [{:.1} mins]",
				trade.unit.adjusted_time.distance_to_station,
				trade.unit.adjusted_time.time_to_station / 60f64
			);
			
			if args.auto_accept {
				accepted_trade = Some(trade); 
				println!("");
				break 'trade;
			} else {
				println!("");
				println!("start:\tenter) to accept trade" );
				if !is_edce {
					println!("\tu) to update buy price ({})", trade.unit.buy_price);
				}
				
				println!("\tn) for new trade");
				println!("\tq) to quit");
				// the first trade is from the station the user is docked at
				// so calculate it automatically
				let str = user_input::read_line();
				match &str[..] {
//					"u" | "update" => {
//						let new_price = read_price_update();
//						price_updates.push( PriceUpdate::new_sell_update( new_price, trade.sell ) );
//					},
					"u" | "update"  => {
						let buy_price = user_input::read_price_update("buy price");
						let supply = user_input::read_price_update("supply");
						
						let update = PriceAdjustment::from_buy(buy_price, supply, trade.unit.buy);
						universe.apply_price_adjustment( &update );
						update.save();
						
						search_cache.invalidate_station( trade.unit.buy_station.station_id );
						
						println!("{}", SEPARATOR);
						continue 'route;
					},
					"n" | "new"  => { 
						println!("{}", SEPARATOR);
						continue; 
					},
					"q" | "quit" => {
						// it's technically not needed to set this,
						// but just in case the code changes in the future,
						// let's set it anyway.
						quit = true;
						
						break 'route;
					}
					_ => { accepted_trade = Some(trade); break 'trade; }
				}
			}
		};
		
		if !accepted_trade.is_some() {
			println!("No trade found"); 
			break;
		}
		
		i += 1;
		let mut trade = accepted_trade.unwrap();
		let trade_snapshot = trade.clone();
		
		let start_time = PreciseTime::now();
		println!("end:\tenter) to complete trade" );
		if !is_edce {
			println!("\tu) to update sell price ({})", trade.unit.sell_price );
		}
		println!("\tq) to complete route" );
		
		// the first trade is from the station the user is docked at
		// so calculate it automatically
		let str = user_input::read_line();
		
		if let Some(edce_data) = EdceData::generate_opt( &args.edce_path ) {
			if let Some(price_update) = edce_data.apply_edce_adjustments(universe) {
				if price_update.station.station_id == trade.unit.sell_station.station_id {
					let updated_active = price_update.changes.iter()
						.filter(|change| change.get_commodity_id() == trade.unit.commodity_id)
						.next().is_some();
					
					if updated_active {
						println!("edce:\tupdated trade commodity and {} others",
							std::cmp::max(price_update.changes.len() - 1, 0)
						);
					} else {
						println!("edce:\tupdated {} commodities",
							price_update.changes.len()
						);
					}
					println!("");
				} else if price_update.station.station_id == trade.unit.buy_station.station_id {
					println!("edce:\tdata received was for original buy station - {}", price_update.station.station_name );
					println!("\tplease wait a few seconds after docking before completing trade" );
				} else if price_update.station.station_id == trade.unit.buy_station.station_id {
					println!("edce:\tdata received was for unknown station - {}", price_update.station.station_name );
					println!("\tyou appear to have docked at the wrong station!" );
				}
			}
			
		}
		
		match &str[..] {
			"u" | "update"  => {
					let sell_price = user_input::read_price_update("sell price");
					
					let update = PriceAdjustment::from_sell(sell_price, trade.unit.sell);
					universe.apply_price_adjustment( &update );
					update.save();
					
					trade = trade.with_sell_price( sell_price );
			},
			"q" | "quit"  => { quit = true; },
			_ => {}
		}
		
		let trade_state = trade.state_after_trade();
		universe.apply_trade( &trade, &search_cache );
		
		let span = start_time.to( PreciseTime::now() );
		let seconds = span.num_milliseconds() as f64 / 1000f64;
		let minutes = span.num_milliseconds() as f64 / 60000f64;
		
		let profit_per_min = trade.profit_total as f64 / minutes;
		
		println!("actual:\t{:.1}% of expected - {} profit/min from trade",
			100f64 * profit_per_min / trade_snapshot.profit_per_min,
			NumericUnit::new_string( profit_per_min, &"cr".to_string()) );
		
		println!("\t{:.1}% of expected - {} profit per ton",
			100f64 * trade.unit.profit_per_ton as f64 / trade_snapshot.unit.profit_per_ton as f64,
			NumericUnit::new_string( trade.unit.profit_per_ton, &"cr".to_string()) );
		
		println!("\t{:.1}% of expected - {:.2} minutes",
			100f64 * minutes / (trade_snapshot.unit.adjusted_time.time_total / 60f64),
			minutes );
		// 159% of expected - 70.2 Kcr profit/min from trade
		// 129% of expected - 1.4 Kcr profit per ton
		// 75% of expected - 7.2 mins travel time
		
		match TimeAdjustment::new( &trade, seconds ) {
			Some(adjustment) => {
				adjustment.save();
				universe.apply_time_adjustment( adjustment.clone() );
			},
			None => {}
		};
		
		sum_profit += trade.profit_total;
		sum_minutes += minutes;
		
		player_state = trade_state.refresh_time_adjustment( universe );
		println!("{}", SEPARATOR);
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
		NumericUnit::new_string( start_state.credit_balance, &"cr".to_string() ),
		NumericUnit::new_string( player_state.credit_balance, &"cr".to_string() ) );
	
	println!("Done!");
	// print overall stats
	// save price updates
}

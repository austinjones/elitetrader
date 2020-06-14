//TODO: optimize route cycles?

use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

use num_cpus;

use rand::{thread_rng, Rng};
use std::collections::HashSet;

use crate::data::trader::*;
use crate::data::Universe;

use crate::search::full_trade::FullTrade;
use crate::search::player_state::PlayerState;
use crate::search::search_cache::SearchCache;
use crate::search::search_cycle::{SearchCycle, SearchCycleTracker};
use crate::search::search_quality::SearchQuality;
use crate::search::unit_trade::UnitTrade;

use crate::util::num_unit::NumericUnit;
use crate::util::scored_buf::*;

use crossbeam::thread::ScopedJoinHandle;
use std::cmp::*;

#[derive(Clone)]
pub struct SearchResult<'a> {
    pub trade: FullTrade<'a>,
    pub profit_total: u32,
    pub time_total: f64,
}

impl<'a> SearchResult<'a> {
    pub fn new(trade: FullTrade<'a>) -> SearchResult<'a> {
        let profit_total = trade.profit_total;
        let time_total = trade.unit.adjusted_time.time_total;

        SearchResult {
            trade: trade,
            profit_total: profit_total,
            time_total: time_total,
        }
    }

    pub fn with_trade(&self, trade: &FullTrade<'a>) -> SearchResult<'a> {
        let profit_total = self.profit_total + trade.profit_total;
        let distance_in_seconds = self.time_total + trade.unit.adjusted_time.time_total;

        SearchResult {
            trade: trade.clone(),
            profit_total: profit_total,
            time_total: distance_in_seconds,
        }
    }

    pub fn with_cycle(&self, cycle: &SearchCycle) -> SearchResult<'a> {
        let profit_total = self.profit_total + cycle.profit_total;
        let distance_in_seconds = self.time_total + cycle.time_total;

        SearchResult {
            trade: self.trade.clone(),
            profit_total: profit_total,
            time_total: distance_in_seconds,
        }
    }

    pub fn with_score(&self, other: &SearchResult<'a>) -> SearchResult<'a> {
        let profit_total = self.profit_total + other.profit_total;
        let distance_in_seconds = self.time_total + other.time_total;

        SearchResult {
            trade: self.trade.clone(),
            profit_total: profit_total,
            time_total: distance_in_seconds,
        }
    }

    fn fudge(val: f64, fudge_factor: f64) -> f64 {
        val * thread_rng().gen_range(1f64 - fudge_factor, 1f64 + fudge_factor)
    }

    pub fn profit_per_min(&self) -> f64 {
        60f64 * self.profit_total as f64 / self.time_total
    }
}

impl<'a> Scored<f64> for SearchResult<'a> {
    fn score(&self) -> f64 {
        let val = match self.time_total {
            0f64 => panic!("Cannot score result with 0 distance_in_seconds"),
            _ => self.profit_total as f64 / self.time_total,
        };

        val
        //		SearchResult::fudge( val, 0.02 )
    }
}

impl<'a> Debug for SearchResult<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let str = format!(
            "{} profit in {:.0} minutes, {} profit/min -- {:?}",
            NumericUnit::new_string(self.profit_total, &"cr".to_string()),
            self.time_total / 60f64,
            NumericUnit::new_string(self.profit_per_min(), &"cr".to_string()),
            self.trade.unit
        );

        formatter.write_str(&str)
    }
}

pub struct SearchTrade<'a> {
    pub trade: FullTrade<'a>,
    pub sell_station: SearchStation,
}

pub struct SearchStation {
    pub state: PlayerState,
    pub search_quality: SearchQuality,
}

impl<'a> SearchStation {
    pub fn new(state: PlayerState, search_quality: SearchQuality) -> SearchStation {
        SearchStation {
            state: state,
            search_quality: search_quality,
        }
    }

    pub fn next_trades(
        &mut self,
        universe: &'a Universe,
        search_cache: &SearchCache,
    ) -> Vec<SearchResult<'a>> {
        let max_depth = self.search_quality.get_depth();

        let trades = self.next_trades_recurse(
            &SearchCycleTracker::new(&self.search_quality),
            universe,
            search_cache,
            0,
            max_depth,
        );
        match trades {
            Some(mut buffer) => buffer.sort_mut(),
            None => Vec::new(),
        }
    }

    fn new_search_result(
        &self,
        cycle_tracker: &SearchCycleTracker,
        unit_trade: UnitTrade<'a>,
        universe: &'a Universe,
        cache: &SearchCache,
        depth: usize,
        max_depth: usize,
    ) -> Option<SearchResult<'a>> {
        let search_trade_1 = self.new_trade(unit_trade);
        if !search_trade_1.trade.is_valid {
            println!(
                "Invalid trade: {}tons - {:?}",
                search_trade_1.trade.used_cargo, search_trade_1.trade.unit
            );
            return None;
        }

        let full_trade_1 = search_trade_1.trade;
        let result_1 = SearchResult::new(full_trade_1);

        match cycle_tracker.find_cycle(&result_1.trade, max_depth - depth) {
            Some(cycle) => return Some(result_1.with_cycle(&cycle)),
            None => {}
        }

        let cycle_tracker = cycle_tracker.push(&result_1.trade);

        let mut results_2 = match search_trade_1.sell_station.next_trades_recurse(
            &cycle_tracker,
            universe,
            cache,
            depth + 1,
            max_depth,
        ) {
            Some(r) => r,
            None => {
                return Some(result_1);
            }
        };

        let mut best_score_with_2_buffer = ScoredCircularBuffer::new(1usize, Sort::Descending);
        for result_2 in results_2.drain().map(|e| e.value) {
            let result_1_with_2 = result_1.with_score(&result_2);
            best_score_with_2_buffer.push_scored(result_1_with_2);
        }

        let mut best_1 = best_score_with_2_buffer.sort_mut();
        let best1_val = best_1.drain(..).next();
        match best1_val {
            Some(_) => best1_val,
            None => Some(result_1),
        }
    }

    fn next_trades_recurse(
        &self,
        cycles: &SearchCycleTracker,
        universe: &'a Universe,
        cache: &SearchCache,
        depth: usize,
        max_depth: usize,
    ) -> Option<ScoredCircularBuffer<f64, SearchResult<'a>>> {
        if depth >= max_depth {
            return None;
        }

        let hop_width = self.search_quality.get_hop_width();
        // we only need the top result
        let mut route_buffer = ScoredCircularBuffer::new(hop_width, Sort::Descending);
        // this method is complicated, so the number postfixes are the 'depth'
        // depth 1 is the next trade, and depth 2 is the trade after that...
        // we are looking for the best depth 1 trades with the highest score,
        // including the best depth 2, best depth 3, ... best depth N trades.
        //		let mut trades = cache.get_1hop_trades( universe, &self );
        //
        //		if depth == 0 {
        //			for _ in 0..self.search_quality.get_random_hops() {
        //				// take the worst trade off the buffer
        //				trades.pop();
        //
        //				// add a random one
        //				match SearchCache::random_1hop_trade(universe, &self.state) {
        //					Some(v) => trades.push(v),
        //					_ => {}
        //				}
        //			}
        //
        //		}

        let mut trades_1hop = cache.get_1hop_trades(universe, &self);
        if depth == 0 {
            // split the trades into as many chunks are there are cpus
            let trade_len = trades_1hop.len();
            let cpus = num_cpus::get();
            let chunk_size = max(trade_len / cpus, 0) + 1;
            //println!("Grouping into {} sized chunks ({}/{})", chunk_size, trade_len, cpus);
            let trades_1hop_parts = trades_1hop.chunks(chunk_size);

            // create a crossbeam scope
            let mut options: Vec<SearchResult<'a>> = crossbeam::scope(|scope| {
                let mut handles = Vec::new();

                // for each chunk, create a thread
                for trade_slice in trades_1hop_parts {
                    let search_handle: ScopedJoinHandle<Vec<SearchResult<'a>>> =
                        scope.spawn(move |scope| {
                            trade_slice
                                .iter()
                                .map(|trade| {
                                    self.new_search_result(
                                        &cycles,
                                        trade.clone(),
                                        universe,
                                        cache,
                                        depth,
                                        max_depth,
                                    )
                                })
                                .filter_map(|e| e)
                                .collect()
                        });

                    // save the handle to the list, so we can extract the results later
                    handles.push(search_handle);
                }

                // join all the chunk-handles together
                handles
                    .into_iter()
                    // join the scoped thread handle
                    .map(|h| h.join())
                    // flatten all the sub-iterators, since each handle is for a chunk, not a single trade
                    .flat_map(|e| e.unwrap())
                    // collect it all!
                    .collect()
            })
            .unwrap();

            for option in options {
                route_buffer.push_scored(option);
            }
        } else {
            let options: Vec<SearchResult<'a>> = trades_1hop
                .drain(..)
                .map(|trade| {
                    self.new_search_result(&cycles, trade, universe, cache, depth, max_depth)
                })
                .filter_map(|e| e)
                .collect();

            for option in options {
                route_buffer.push_scored(option);
            }
        };

        if route_buffer.len() > 0 {
            Some(route_buffer)
        } else {
            None
        }
    }

    fn new_trade(&self, unit: UnitTrade<'a>) -> SearchTrade<'a> {
        if unit.buy_station.to_id() != self.state.station_id {
            panic!("Cannot create trade that originates from a different station");
        }

        let trade = FullTrade::new(&self.state, unit);

        let sell_state = self.state.with_trade(&trade);
        let sell_station = SearchStation::new(sell_state, self.search_quality);

        SearchTrade {
            trade: trade,
            sell_station: sell_station,
        }
    }
}

use crate::data::adjustments::*;
use crate::search::time_estimate::TimeEstimate;
use crate::search::FullTrade;
use serde::Deserialize;
use serde::Serialize;
use time::OffsetDateTime;
use time::Time;

// adjustment behavior:
// compare expected time (not factoring system estimate) to actual time
//   add this factor to global adjustment
// if a recorded adjustment exists (for local star -> station time),
//  use it as the system estimate

#[derive(Serialize, Deserialize, Clone)]
pub struct TimeAdjustment {
    pub buy_system_id: u32,
    pub buy_station_id: u32,

    pub sell_system_id: u32,
    pub sell_station_id: u32,

    pub raw_estimate: TimeEstimate,
    pub adjusted_estimate: TimeEstimate,
    pub actual_time: TimeEstimate,

    pub timestamp: u64,
}

impl TimeAdjustment {
    pub fn new(trade: &FullTrade, num_seconds: f64) -> Option<TimeAdjustment> {
        // it's possible the user just pressed enter to jump through the systems,
        //   or forgot to press enter at the 'accept trade' prompt.
        // the jump time estimates are pretty solid.  let's ignore the value
        if num_seconds < trade.unit.adjusted_time.time_to_system {
            return None;
        }

        let timestamp =
            OffsetDateTime::now().timestamp() - OffsetDateTime::unix_epoch().timestamp();
        Some(TimeAdjustment {
            buy_system_id: trade.unit.buy_system.system_id,
            buy_station_id: trade.unit.buy_station.station_id,

            sell_system_id: trade.unit.sell_system.system_id,
            sell_station_id: trade.unit.sell_station.station_id,

            raw_estimate: trade.unit.normalized_time.clone(),
            adjusted_estimate: trade.unit.adjusted_time.clone(),
            actual_time: trade.unit.normalized_time.to_aboslute(num_seconds),

            // I'm pretty sure the current timestamp isn't going to be before Jan 1st 1970...
            // lets cast it to u64
            timestamp: timestamp as u64,
        })
    }

    pub fn load_all() -> Vec<TimeAdjustment> {
        load_adjustments("time")
    }

    pub fn save(&self) {
        save_adjustment("time", self);
    }
}

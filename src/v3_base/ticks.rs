use alloy::primitives::aliases::I24;

use crate::v3_base::states::{Tick, TradeState};

pub struct Ticks {
    ticks: Vec<Tick>,
}
impl Ticks {
    pub fn new(mut ticks: Vec<Tick>) -> Ticks {
        ticks.sort_by_key(|x| x.tick);
        ticks.dedup_by_key(|x| x.tick);

        Ticks {
            ticks,
        }
    }
    pub fn get_tick_index(&self, tick: I24) -> Result<usize, usize> {
        let result = self.ticks.binary_search_by_key(&tick, |t| t.tick);
        result
    }

    pub fn get_tick(&self, tick: I24) -> Result<Tick, usize> {
        match self.get_tick_index(tick) {
            Ok(tick) => Ok(self.ticks[tick]),
            Err(idx) => Err(idx),
        }
    }

    pub fn insert_ticks(&mut self, mut ticks: Vec<Tick>) {
        if ticks.len() == 0 {
            return;
        }

        ticks.sort_by_key(|x| x.tick);
        ticks.dedup_by_key(|x| x.tick);

        let mut all_ticks = Vec::<Tick>::with_capacity(self.ticks.len() + ticks.len());
        let mut new_idx = 0;
        let mut self_idx = 0;

        while new_idx < ticks.len() && self_idx < self.ticks.len() {
            let stick = self.ticks[self_idx].tick;
            let ntick = ticks[new_idx].tick;

            if stick > ntick {
                all_ticks.push(self.ticks[self_idx]);
                self_idx += 1;
            } else if stick < ntick {
                all_ticks.push(ticks[new_idx]);
                new_idx += 1;
            } else {
                all_ticks.push(ticks[new_idx]);
                self_idx += 1;
                new_idx += 1;
            }
        }
        if new_idx >= ticks.len() {
            all_ticks.extend_from_slice(&self.ticks.split_at(self_idx).1);
        } else {
            all_ticks.extend_from_slice(ticks.split_at(new_idx).1);
        }
        self.ticks = all_ticks;
    }
    pub fn len(&self) -> usize {
        self.ticks.len()
    }
    pub fn get(&self, index: usize) -> Option<&Tick> {
        self.ticks.get(index)
    }
}

pub fn trade_step(trade_state: TradeState) {}

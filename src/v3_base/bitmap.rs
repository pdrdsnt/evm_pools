use alloy::primitives::{U256, aliases::I24};

use crate::{
    err::WordError,
    v3_base::bitmap_math::{normalize_tick, word_index},
};

pub struct BitMap {
    bitmap: Vec<Result<U256, WordError>>,
    tick_spacing: I24,
}

impl BitMap {
    pub fn new(tick_spacing: I24, words: Vec<(i16, U256)>) -> Self {
        println!("tick spacing {}:", tick_spacing);
        let total_words = (65536_i64 + tick_spacing.as_i64())
            .checked_div(tick_spacing.as_i64())
            .expect("problem dividing full bitmap range by tick spacing ");

        let mut bm = Vec::with_capacity(total_words as usize);

        for _ in 0..total_words {
            bm.push(Err(WordError::NotTried));
        }
        for (pos, word) in words {
            let idx = (pos + i16::MAX) as usize;
            bm[idx] = std::result::Result::Ok(word);
        }

        BitMap {
            tick_spacing: tick_spacing,
            bitmap: bm,
        }
    }
    pub fn get_word_from_pos(&self, word_pos: i16) -> Option<&Result<U256, WordError>> {
        let index = Self::pos_to_idx(word_pos, self.tick_spacing);

        self.bitmap.get(index)
    }
    pub fn get_word_from_tick(
        &self,
        tick: I24,
        tick_spacing: I24,
    ) -> Option<&Result<alloy::primitives::Uint<256, 4>, WordError>> {
        let normalized_tick = normalize_tick(tick, tick_spacing);
        let word_pos = word_index(normalized_tick);
        self.get_word_from_pos(word_pos)
    }
    pub fn insert(&mut self, pos: i16, word: U256) {
        self.bitmap[Self::pos_to_idx(pos, self.tick_spacing)] = Ok(word);
    }
    pub fn pos_to_idx(word_pos: i16, tick_spacing: I24) -> usize {
        println!("word pos: {} , tick spacing: {}", word_pos, tick_spacing);
        (word_pos as isize
            + (i16::max_value() as isize) as isize / tick_spacing.as_isize())
            as usize
    }
}

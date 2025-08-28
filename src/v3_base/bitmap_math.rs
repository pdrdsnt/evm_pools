use alloy::primitives::{aliases::I24, U256};

/// Normalize a tick by tick spacing (division towards zero)
pub fn normalize_tick(current_tick: I24, tick_spacing: I24) -> I24 {
    current_tick.div_euclid(tick_spacing)
}

pub fn word_index(normalized_tick: I24) -> i16 {
    let divisor = I24::try_from(256).unwrap(); // Infallible for 256
    let quotient = normalized_tick.div_euclid(divisor);
    quotient.as_i16() // Safe: quotient ∈ [-32,768, 32,767]
}

/// Extract initialized tick values from a single bitmap word
pub fn extract_ticks_from_bitmap(
    bitmap: U256,
    word_idx: i16,
    tick_spacing: I24,
) -> Vec<I24> {
    let mut ticks = Vec::new();
    if bitmap.is_zero() {
        return ticks;
    }
    for bit in 0..256_i16 {
        if bitmap.bit(bit as usize) {
            let normalized = (word_idx as i32 * 256) + bit as i32;
            ticks.push(I24::try_from(normalized).unwrap() * tick_spacing);
        }
    }
    ticks
}
pub fn get_pos_from_tick(tick: I24, tick_spacing: I24) -> i16 {
    let normalized_tick = normalize_tick(tick, tick_spacing);
    word_index(normalized_tick)
}
pub fn next_left(word: &U256, start: &i16) -> Option<usize> {
    // clamp start to valid range 0..=255
    let mut idx = *start.max(&0_i16).min(&255_i16) as usize;
    // scan forward until we find a set bit or run out of bits
    while idx > 0 {
        idx -= 1;
        if word.bit(idx) {
            return Some(idx);
        }
    }
    None
}

pub fn next_right(word: &U256, start: &i16) -> Option<usize> {
    // clamp start to valid range 0..=255
    let mut idx = *start.max(&0_i16).min(&255_i16) as usize;
    // scan forward until we find a set bit or run out of bits
    while idx < 255 {
        idx += 1;
        if word.bit(idx) {
            return Some(idx);
        }
    }
    None
}

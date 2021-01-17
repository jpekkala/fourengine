use crate::bitboard;
use crate::bitboard::BoardInteger;
use crate::score::*;

type Entry = bitboard::BoardInteger;

#[derive(Copy, Clone)]
struct Slot {
    expensive: Entry,
    recent: Entry,
}

/// A hash table for connect-4 positions. This table is two-level which means that each slot has
/// room for two positions. If more than two positions need to be stored in the same slot, the
/// replacement scheme TwoBig1 (Breuker et al. 1994) is used. The replacement scheme keeps the most
/// expensive entry and the most recent entry.
pub struct TransTable {
    /// How many slots the table has. The table size also acts as a hash function so preferably it
    /// should be a prime
    table_size: usize,
    slots: Vec<Slot>,
    /// How many entries are saved. For diagnostics only
    stored_count: usize,

    /// Each position is divided by table_size so that the remainder is an index into entries. The
    /// quotient (=key) is then saved inside the entry so that we can reconstruct what position the
    /// saved entry is.
    ///
    /// The number of bits needed for the key depends on the table_size.
    key_bits: u32,
    key_score_bits: u32,

    key_mask: Entry,
    score_mask: Entry,
    #[allow(dead_code)]
    work_mask: Entry,
}

/// The number of bits needed to encode a score
const SCORE_BITS: u32 = 3;

impl TransTable {
    pub fn new(table_size: usize) -> TransTable {
        let slots: Vec<Slot> = vec![
            Slot {
                expensive: 0,
                recent: 0
            };
            table_size
        ];
        let largest_possible_position: BoardInteger = (1 << bitboard::POSITION_BITS) - 1;
        let key_size = closest_power_of_two(largest_possible_position / table_size as BoardInteger);
        let key_score_size = key_size + SCORE_BITS;

        let key_mask = (1 << key_size) - 1;
        let score_mask = ((1 << key_score_size) - 1) ^ key_mask;
        let work_mask = !0 ^ score_mask ^ key_mask;

        TransTable {
            table_size,
            slots,
            stored_count: 0,
            key_bits: key_size,
            key_score_bits: key_score_size,

            key_mask,
            score_mask,
            work_mask,
        }
    }

    pub fn reset(&mut self) {
        self.stored_count = 0;
        for slot in &mut self.slots {
            slot.expensive = 0;
            slot.recent = 0;
        }
    }

    pub fn store(&mut self, position_code: BoardInteger, score: Score, work: u32) {
        let index: usize = (position_code % self.table_size as Entry) as usize;
        let key: Entry = position_code / self.table_size as Entry;

        let new_entry: Entry =
            key | ((score as Entry) << self.key_bits) | ((work as Entry) << self.key_score_bits);

        let mut slot = self.slots[index];
        let expensive_entry = slot.expensive;
        let recent_entry = slot.recent;

        if expensive_entry == 0 {
            self.stored_count += 1;
            slot.expensive = new_entry;
        } else if (expensive_entry & self.key_mask) == key {
            slot.expensive = new_entry;
        } else if work >= (expensive_entry >> self.key_score_bits) as u32 {
            if recent_entry == 0 {
                self.stored_count += 1;
            }
            slot.expensive = new_entry;
            slot.recent = expensive_entry;
        } else {
            if recent_entry == 0 {
                self.stored_count += 1;
            }
            slot.recent = new_entry;
        }
        self.slots[index] = slot;
    }

    pub fn fetch(&self, position_code: BoardInteger) -> Score {
        let index: usize = (position_code % self.table_size as Entry) as usize;
        let key: Entry = position_code / self.table_size as Entry;

        let slot = self.slots[index];

        let mut found_entry = None;
        let expensive_entry = slot.expensive;
        if (expensive_entry & self.key_mask) == key {
            found_entry = Some(expensive_entry);
        } else {
            let recent_entry = slot.recent;
            if (recent_entry & self.key_mask) == key {
                found_entry = Some(recent_entry);
            }
        }

        if let Some(entry) = found_entry {
            let score = (entry & self.score_mask) >> self.key_bits;
            Score::from_u64_fast(score)
        } else {
            Score::Unknown
        }
    }
}

/// log_2 rounded upwards
fn closest_power_of_two(number: BoardInteger) -> u32 {
    let mut remaining = number;
    let mut bit_count = 0;
    while remaining > 0 {
        bit_count += 1;
        remaining = remaining >> 1;
    }

    bit_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::BoardInteger;

    #[test]
    fn validate_masks() {
        let tt = TransTable::new(1021);
        // the union of masks should have all bits set
        assert_eq!(tt.key_mask | tt.score_mask | tt.work_mask, !0);
        // none of the masks should overlap
        assert_eq!(tt.key_mask & tt.score_mask, 0);
        assert_eq!(tt.key_mask & tt.work_mask, 0);
        assert_eq!(tt.score_mask & tt.work_mask, 0);
    }

    #[test]
    fn remember_stored_value() {
        let mut tt = TransTable::new(1021);

        let position = Position::from_variation("4444");
        tt.store(position.to_position_code(), Score::Win, 0);
        assert_eq!(tt.stored_count, 1);
        assert_eq!(tt.fetch(position.to_position_code()), Score::Win);
    }

    #[test]
    fn keep_expensive_and_recent_entries() {
        let table_size = 1021;
        let mut tt = TransTable::new(table_size);

        let offset = Position::empty().to_position_code();
        let pos1 = Position::from_position_code(offset + table_size as BoardInteger);
        let pos2 = Position::from_position_code(offset + 2 * table_size as BoardInteger);
        let pos3 = Position::from_position_code(offset + 3 * table_size as BoardInteger);
        let pos4 = Position::from_position_code(offset + 4 * table_size as BoardInteger);

        tt.store(pos1.to_position_code(), Score::Win, 300);
        tt.store(pos2.to_position_code(), Score::Win, 600);
        tt.store(pos3.to_position_code(), Score::Win, 500);
        tt.store(pos4.to_position_code(), Score::Win, 400);

        assert_eq!(tt.fetch(pos1.to_position_code()), Score::Unknown);
        assert_eq!(tt.fetch(pos2.to_position_code()), Score::Win);
        assert_eq!(tt.fetch(pos3.to_position_code()), Score::Unknown);
        assert_eq!(tt.fetch(pos4.to_position_code()), Score::Win);
    }
}

use num_traits::FromPrimitive;

use crate::bitboard;
use crate::bitboard::Position;
use crate::constants::*;

type Entry = bitboard::BoardInteger;

/// A hash table for connect-4 positions. This table is two-level which means that each slot has
/// room for two positions. If more than two positions need to be stored in the same slot, the
/// replacement scheme TwoBig1 (Breuker et al. 1994) is used. The replacement scheme keeps the most
/// expensive entry and the most recent entry.
struct TransTable {
    /// How many slots the table has. The table size also acts as a hash function so preferably it
    /// should be a prime. Note that the entries array is twice of table_size because each slot can
    /// fit two positions.
    table_size: usize,
    entries: Vec<Entry>,
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
    work_mask: Entry,
}

/// The number of bits needed to encode a score
const SCORE_BITS: u32 = 3;

impl TransTable {
    pub fn new(table_size: usize) -> TransTable {
        let entries: Vec<Entry> = vec![0; table_size * 2];
        let largest_possible_position = (1 << POSITION_BITS) - 1;
        let key_size = closest_power_of_two(largest_possible_position / table_size);
        let key_score_size = key_size + SCORE_BITS;

        let key_mask = (1 << key_size) - 1;
        let score_mask = ((1 << key_score_size) - 1) ^ key_mask;
        let work_mask = !0 ^ score_mask ^ key_mask;

        TransTable {
            table_size,
            entries,
            stored_count: 0,
            key_bits: key_size,
            key_score_bits: key_score_size,

            key_mask,
            score_mask,
            work_mask,
        }
    }

    pub fn store(&mut self, position: Position, score: Score, work: u32) {
        let position_integer = position.to_integer();
        let index: usize = ((position_integer % self.table_size as u64) * 2) as usize;
        let key: Entry = position_integer / self.table_size as Entry;

        let new_entry: Entry =
            key | ((score as Entry) << self.key_bits) | ((work as Entry) << self.key_score_bits);
        let expensive_entry = self.entries[index];
        let recent_entry = self.entries[index + 1];

        if expensive_entry == 0 {
            self.stored_count += 1;
            self.entries[index] = new_entry
        } else if (expensive_entry & self.key_mask) == key {
            self.entries[index] = new_entry;
        } else if work >= (expensive_entry >> self.key_score_bits) as u32 {
            if recent_entry == 0 {
                self.stored_count += 1;
            }
            self.entries[index] = new_entry;
            self.entries[index + 1] = expensive_entry;
        } else {
            if recent_entry == 0 {
                self.stored_count += 1;
            }
            self.entries[index + 1] = new_entry;
        }
    }

    pub fn fetch(&self, position: Position) -> Score {
        let position_integer = position.to_integer();
        let index: usize = ((position_integer % self.table_size as u64) * 2) as usize;
        let key: Entry = position_integer / self.table_size as Entry;

        let mut found_entry = None;
        let expensive_entry = self.entries[index];
        if (expensive_entry & self.key_mask) == key {
            found_entry = Some(expensive_entry);
        } else {
            let recent_entry = self.entries[index + 1];
            if (recent_entry & self.key_mask) == key {
                found_entry = Some(recent_entry);
            }
        }

        match found_entry {
            Some(entry) => Score::from_u64((entry & self.score_mask) >> self.key_bits)
                .expect("Invalid score"),
            None => Score::Unknown,
        }
    }
}

/// log_2 rounded upwards
fn closest_power_of_two(number: usize) -> u32 {
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

        let position = Position::from_integer(1000);
        tt.store(position, Score::Win, 0);
        assert_eq!(tt.stored_count, 1);
        assert_eq!(tt.fetch(position), Score::Win);
    }

    #[test]
    fn keep_expensive_and_recent_entries() {
        let table_size = 1021;
        let mut tt = TransTable::new(table_size);

        let pos1 = Position::from_integer(table_size as BoardInteger);
        let pos2 = Position::from_integer(2 * table_size as BoardInteger);
        let pos3 = Position::from_integer(3 * table_size as BoardInteger);
        let pos4 = Position::from_integer(4 * table_size as BoardInteger);

        tt.store(pos1, Score::Win, 300);
        tt.store(pos2, Score::Win, 600);
        tt.store(pos3, Score::Win, 500);
        tt.store(pos4, Score::Win, 400);

        assert_eq!(tt.fetch(pos1), Score::Unknown);
        assert_eq!(tt.fetch(pos2), Score::Win);
        assert_eq!(tt.fetch(pos3), Score::Unknown);
        assert_eq!(tt.fetch(pos4), Score::Win);
    }
}

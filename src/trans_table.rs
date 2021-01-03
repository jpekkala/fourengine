use crate::bitboard;
use crate::bitboard::Bitboard;

type Entry = u64;

struct TransTable {
    // the table size also acts as a hash function so preferably it should be a prime
    table_size: usize,
    entries: Vec<Entry>,

    // how many bits the key needs
    key_bits: u32,
    key_score_bits: u32,

    key_mask: Entry,
    score_mask: Entry,
    work_mask: Entry,
}

// The number of bits needed to encode a position
const POSITION_BITS: u32 = bitboard::BIT_HEIGHT * bitboard::WIDTH;
// The number of bits needed to encode a score
const SCORE_BITS: u32 = 2;

impl TransTable {

    pub fn new(table_size: usize) -> TransTable {
        let entries: Vec<Entry> = vec![0; table_size * 2];
        let key_bits = POSITION_BITS - calc_bits_required(table_size);
        let key_score_bits = key_bits + SCORE_BITS;

        let key_mask = (1 << key_bits) - 1;
        let score_mask = ((1 << key_score_bits) - 1) ^ key_mask;
        let work_mask = !0 ^ score_mask ^ key_mask;

        TransTable {
            table_size,
            entries,
            key_bits,
            key_score_bits,

            key_mask,
            score_mask,
            work_mask,
        }
    }

    pub fn store(&mut self, position: Bitboard, score: u32, work: u32) {
        let index: usize = ((position % self.table_size as u64) * 2) as usize;
        let key: Entry = position >> self.key_bits;

        let new_entry: Entry = key | ((score as Entry) << self.key_bits) | ((work as Entry) << self.key_score_bits);
        let expensive_entry = self.entries[index];

        if (expensive_entry & self.key_mask) == key {
            self.entries[index] = new_entry;
        } else if work >= (expensive_entry >> self.key_score_bits) as u32 {
            self.entries[index] = new_entry;
            self.entries[index + 1] = expensive_entry;
        } else {
            self.entries[index + 1] = new_entry;
        }
    }

    pub fn fetch(&self, position: Bitboard) -> u32 {
        let index: usize = ((position % self.table_size as u64) * 2) as usize;
        let key: Entry = position >> self.key_bits;

        let expensive_entry = self.entries[index];
        if (expensive_entry & self.key_mask) == key {
            return ((expensive_entry & self.score_mask) >> self.key_bits) as u32;
        }

        let recent_entry = self.entries[index + 1];
        if (recent_entry & self.key_mask) == key {
            return ((recent_entry & self.score_mask) >> self.key_bits) as u32
        }

        0
    }
}

fn calc_bits_required(number: usize) -> u32 {
    let mut remaining = number;
    let mut bit_count = 0;
    while remaining > 0 {
        bit_count += 1;
        remaining = remaining >> 1;
    }

    bit_count
}

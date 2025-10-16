// Copyright Anysphere Inc.
// Hash table implementation for LZ77 pattern matching

use crate::types::{Hash, WINDOW_SIZE, WINDOW_MASK, MIN_MATCH};

const HASH_SHIFT: i32 = 5;
const HASH_MASK: i32 = 32767;

/// Update the sliding hash value with the given byte. All calls to this function
/// must be made on consecutive input characters. Since the hash value exists out
/// of multiple input bytes, a few warmups with this function are needed initially.
#[inline]
fn update_hash_value(h: &mut Hash, c: u8) {
    h.val = (((h.val) << HASH_SHIFT) ^ (c as i32)) & HASH_MASK;
}

/// Prepopulates hash:
/// Fills in the initial values in the hash, before update_hash can be used correctly.
pub fn warmup_hash(array: &[u8], pos: usize, end: usize, h: &mut Hash) {
    update_hash_value(h, array[pos]);
    if pos + 1 < end {
        update_hash_value(h, array[pos + 1]);
    }
}

/// Updates the hash values based on the current position in the array. All calls
/// to this must be made for consecutive bytes.
pub fn update_hash(array: &[u8], pos: usize, end: usize, h: &mut Hash) {
    let hpos = (pos & WINDOW_MASK) as usize;
    let mut amount: usize = 0;
    
    let next_char = if pos + MIN_MATCH <= end {
        array[pos + MIN_MATCH - 1]
    } else {
        0
    };
    update_hash_value(h, next_char);
    
    h.hashval[hpos] = h.val;
    if h.head[h.val as usize] != -1 && h.hashval[h.head[h.val as usize] as usize] == h.val {
        h.prev[hpos] = h.head[h.val as usize] as u16;
    } else {
        h.prev[hpos] = hpos as u16;
    }
    h.head[h.val as usize] = hpos as i32;
    
    // Update "same"
    if h.same[((pos.wrapping_sub(1)) & WINDOW_MASK) as usize] > 1 {
        amount = h.same[((pos.wrapping_sub(1)) & WINDOW_MASK) as usize] as usize - 1;
    }
    while pos + amount + 1 < end 
        && array[pos] == array[pos + amount + 1] 
        && amount < u16::MAX as usize {
        amount += 1;
    }
    h.same[hpos] = amount as u16;
    
    h.val2 = (((h.same[hpos] as i32 - MIN_MATCH as i32) & 255) ^ h.val) as i32;
    h.hashval2[hpos] = h.val2;
    if h.head2[h.val2 as usize] != -1 && h.hashval2[h.head2[h.val2 as usize] as usize] == h.val2 {
        h.prev2[hpos] = h.head2[h.val2 as usize] as u16;
    } else {
        h.prev2[hpos] = hpos as u16;
    }
    h.head2[h.val2 as usize] = hpos as i32;
}

/// Resets all fields of Hash.
pub fn reset_hash(h: &mut Hash) {
    h.val = 0;
    h.val2 = 0;
    
    // Reset arrays
    for i in 0..65536 {
        h.head[i] = -1;
        h.head2[i] = -1;
    }
    
    for i in 0..h.prev.len() {
        h.prev[i] = 0;
        h.hashval[i] = -1;
        h.prev2[i] = 0;
        h.hashval2[i] = -1;
        h.same[i] = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_creation() {
        let hash = Hash::new(WINDOW_SIZE);
        assert_eq!(hash.head.len(), 65536);
        assert_eq!(hash.prev.len(), WINDOW_SIZE);
        assert_eq!(hash.same.len(), WINDOW_SIZE);
        assert_eq!(hash.val, 0);
        assert_eq!(hash.val2, 0);
    }
    
    #[test]
    fn test_reset_hash() {
        let mut hash = Hash::new(WINDOW_SIZE);
        hash.val = 123;
        hash.val2 = 456;
        hash.head[0] = 100;
        hash.head2[0] = 200;
        
        reset_hash(&mut hash);
        
        assert_eq!(hash.val, 0);
        assert_eq!(hash.val2, 0);
        assert_eq!(hash.head[0], -1);
        assert_eq!(hash.head2[0], -1);
    }
    
    #[test]
    fn test_warmup_hash() {
        let data = b"hello world";
        let mut hash = Hash::new(WINDOW_SIZE);
        reset_hash(&mut hash);
        
        warmup_hash(data, 0, data.len(), &mut hash);
        
        // After warmup, hash value should be non-zero
        assert_ne!(hash.val, 0);
    }
    
    #[test]
    fn test_update_hash() {
        let data = b"abcdefghijklmnopqrstuvwxyz";
        let mut hash = Hash::new(WINDOW_SIZE);
        reset_hash(&mut hash);
        
        warmup_hash(data, 0, data.len(), &mut hash);
        
        for i in 0..data.len() - MIN_MATCH {
            update_hash(data, i, data.len(), &mut hash);
        }
        
        // Verify hash chains are set up
        assert!(hash.head[hash.val as usize] >= 0);
    }
    
    #[test]
    fn test_hash_with_repeated_pattern() {
        let data = b"aaaaaaaaaa";
        let mut hash = Hash::new(WINDOW_SIZE);
        reset_hash(&mut hash);
        
        warmup_hash(data, 0, data.len(), &mut hash);
        
        for i in 0..data.len() - MIN_MATCH {
            update_hash(data, i, data.len(), &mut hash);
            // With repeated bytes, 'same' should be high
            let hpos = (i & WINDOW_MASK) as usize;
            if i > 0 {
                assert!(hash.same[hpos] > 0);
            }
        }
    }
}


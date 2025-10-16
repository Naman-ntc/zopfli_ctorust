// Copyright Anysphere Inc.
// LZ77 compression implementation

use crate::types::{LZ77Store, BlockState, Hash, MIN_MATCH, MAX_MATCH, WINDOW_SIZE, WINDOW_MASK};
use crate::symbols::{get_length_symbol, get_dist_symbol};
use crate::hash::{update_hash, warmup_hash, reset_hash};
use crate::cache::{try_get_from_longest_match_cache, store_in_longest_match_cache};

const MAX_CHAIN_HITS: usize = 8192;

/// Verifies if length and dist are indeed valid, only used for assertion.
pub fn verify_len_dist(data: &[u8], datasize: usize, pos: usize, dist: u16, length: u16) {
    debug_assert!(pos + length as usize <= datasize);
    for i in 0..length as usize {
        debug_assert_eq!(
            data[pos.wrapping_sub(dist as usize) + i],
            data[pos + i],
            "Mismatch at position {} with dist={}, length={}",
            i, dist, length
        );
    }
}

/// Finds how long the match of scan and match is.
fn get_match(
    array: &[u8],
    pos: usize,
    match_pos: usize,
    end: usize,
) -> usize {
    let mut scan_pos = pos;
    let mut match_idx = match_pos;
    
    // Compare 8 bytes at a time when possible
    while scan_pos + 8 <= end {
        if array.len() >= match_idx + 8 && array.len() >= scan_pos + 8 {
            // Safe to compare 8 bytes
            let scan_chunk = &array[scan_pos..scan_pos + 8];
            let match_chunk = &array[match_idx..match_idx + 8];
            
            if scan_chunk == match_chunk {
                scan_pos += 8;
                match_idx += 8;
                continue;
            }
        }
        break;
    }
    
    // Compare remaining bytes one by one
    while scan_pos < end && array[scan_pos] == array[match_idx] {
        scan_pos += 1;
        match_idx += 1;
    }
    
    scan_pos - pos
}

/// Finds the longest match (length and corresponding distance) for LZ77 compression.
pub fn find_longest_match(
    s: &mut BlockState,
    h: &Hash,
    array: &[u8],
    pos: usize,
    size: usize,
    mut limit: usize,
    mut sublen: Option<&mut [u16]>,
    distance: &mut u16,
    length: &mut u16,
) {
    // Try to get from cache first
    let sublen_ref = sublen.as_deref_mut();
    if try_get_from_longest_match_cache(s, pos, &mut limit, sublen_ref, distance, length) {
        debug_assert!(pos + *length as usize <= size);
        return;
    }
    
    debug_assert!(limit <= MAX_MATCH);
    debug_assert!(limit >= MIN_MATCH);
    debug_assert!(pos < size);
    
    if size - pos < MIN_MATCH {
        *length = 0;
        *distance = 0;
        return;
    }
    
    if pos + limit > size {
        limit = size - pos;
    }
    
    let hpos = (pos & WINDOW_MASK) as u16;
    let mut bestdist = 0u16;
    let mut bestlength = 1u16;
    
    let mut hhead = &h.head;
    let mut hprev = &h.prev;
    let mut hhashval = &h.hashval;
    let mut hval = h.val;
    
    debug_assert!((hval as usize) < 65536);
    
    let mut pp = hhead[hval as usize];
    if pp < 0 {
        *length = bestlength;
        *distance = bestdist;
        return;
    }
    
    let mut p = hprev[pp as usize];
    let mut dist = if (p as u16) < hpos {
        hpos - p as u16
    } else {
        (WINDOW_SIZE as u16 - p as u16) + hpos
    };
    
    let mut chain_counter = MAX_CHAIN_HITS;
    
    // Go through all distances
    while (dist as usize) < WINDOW_SIZE {
        debug_assert!((p as usize) < WINDOW_SIZE);
        debug_assert_eq!(p, hprev[pp as usize] as u16);
        debug_assert_eq!(hhashval[p as usize], hval);
        
        if dist > 0 {
            debug_assert!(pos < size);
            debug_assert!((dist as usize) <= pos);
            
            let scan_pos = pos;
            let match_pos = pos - dist as usize;
            
            // Testing the byte at position bestlength first, goes slightly faster
            let mut currentlength = 0usize;
            if pos + bestlength as usize >= size || 
               array[scan_pos + bestlength as usize] == array[match_pos + bestlength as usize] {
                
                let same0 = h.same[(pos & WINDOW_MASK) as usize];
                if same0 > 2 && array[scan_pos] == array[match_pos] {
                    let same1 = h.same[((pos - dist as usize) & WINDOW_MASK) as usize];
                    let same = if same0 < same1 { same0 } else { same1 };
                    let same = if same as usize > limit { limit as u16 } else { same };
                    
                    currentlength = same as usize;
                }
                
                let remaining = get_match(array, scan_pos + currentlength, match_pos + currentlength, scan_pos + limit);
                currentlength += remaining;
            }
            
            if currentlength > bestlength as usize {
                if let Some(ref mut sublen_arr) = sublen {
                    for j in (bestlength as usize + 1)..=currentlength {
                        sublen_arr[j] = dist;
                    }
                }
                bestdist = dist;
                bestlength = currentlength as u16;
                if currentlength >= limit {
                    break;
                }
            }
        }
        
        // Switch to the other hash once this will be more efficient
        if hhead as *const _ != &h.head2 as *const _ && bestlength >= h.same[hpos as usize] &&
           h.val2 == h.hashval2[p as usize] {
            hhead = &h.head2;
            hprev = &h.prev2;
            hhashval = &h.hashval2;
            hval = h.val2;
        }
        
        pp = p as i32;
        p = hprev[p as usize];
        if p == pp as u16 {
            break; // Uninited prev value
        }
        
        let new_dist = if (p as u16) < (pp as u16) {
            (pp as u16) - (p as u16)
        } else {
            (WINDOW_SIZE as u16 - p as u16) + (pp as u16)
        };
        dist = dist + new_dist;
        
        chain_counter -= 1;
        if chain_counter == 0 {
            break;
        }
    }
    
    store_in_longest_match_cache(s, pos, limit, sublen.map(|s| &s[..]), bestdist, bestlength);
    
    debug_assert!(bestlength as usize <= limit);
    *distance = bestdist;
    *length = bestlength;
    debug_assert!(pos + *length as usize <= size);
}

/// Appends the length and distance to the LZ77 arrays of the LZ77Store.
pub fn store_lit_len_dist(length: u16, dist: u16, pos: usize, store: &mut LZ77Store) {
    use crate::types::{NUM_LL, NUM_D};
    
    let origsize = store.size();
    let llstart = NUM_LL * (origsize / NUM_LL);
    let dstart = NUM_D * (origsize / NUM_D);
    
    // Everytime the index wraps around, a new cumulative histogram is made
    if origsize % NUM_LL == 0 {
        for i in 0..NUM_LL {
            let val = if origsize == 0 {
                0
            } else {
                store.ll_counts[origsize - NUM_LL + i]
            };
            store.ll_counts.push(val);
        }
    }
    if origsize % NUM_D == 0 {
        for i in 0..NUM_D {
            let val = if origsize == 0 {
                0
            } else {
                store.d_counts[origsize - NUM_D + i]
            };
            store.d_counts.push(val);
        }
    }
    
    store.litlens.push(length);
    store.dists.push(dist);
    store.pos.push(pos);
    debug_assert!(length < 259);
    
    if dist == 0 {
        store.ll_symbol.push(length);
        store.d_symbol.push(0);
        store.ll_counts[llstart + length as usize] += 1;
    } else {
        let ll_sym = get_length_symbol(length as usize) as u16;
        let d_sym = get_dist_symbol(dist as usize) as u16;
        store.ll_symbol.push(ll_sym);
        store.d_symbol.push(d_sym);
        store.ll_counts[llstart + ll_sym as usize] += 1;
        store.d_counts[dstart + d_sym as usize] += 1;
    }
}

/// Gets length score for greedy algorithm
fn get_length_score(length: u16, dist: u16) -> i32 {
    // Typically, longer matches are better, but if the distance is very large,
    // they might not be worth it
    if length < MIN_MATCH as u16 {
        return 0;
    }
    
    // Simple scoring: prefer longer matches, penalize longer distances
    let length_score = length as i32 * 1024;
    let dist_penalty = if dist > 1024 {
        (dist as i32 - 1024) / 32
    } else {
        0
    };
    
    length_score - dist_penalty
}

/// Does LZ77 using an algorithm similar to gzip, with lazy matching.
pub fn lz77_greedy(
    s: &mut BlockState,
    input: &[u8],
    instart: usize,
    inend: usize,
    store: &mut LZ77Store,
    h: &mut Hash,
) {
    if instart == inend {
        return;
    }
    
    let windowstart = if instart > WINDOW_SIZE {
        instart - WINDOW_SIZE
    } else {
        0
    };
    
    let mut dummysublen = [0u16; 259];
    
    // Lazy matching variables
    let mut prev_length = 0u16;
    let mut prev_match = 0u16;
    let mut match_available = false;
    
    reset_hash(h);
    warmup_hash(input, windowstart, inend, h);
    
    for i in windowstart..instart {
        update_hash(input, i, inend, h);
    }
    
    let mut i = instart;
    while i < inend {
        update_hash(input, i, inend, h);
        
        let mut leng = 0u16;
        let mut dist = 0u16;
        find_longest_match(s, h, input, i, inend, MAX_MATCH, Some(&mut dummysublen), &mut dist, &mut leng);
        
        let lengthscore = get_length_score(leng, dist);
        let prevlengthscore = get_length_score(prev_length, prev_match);
        
        if match_available {
            match_available = false;
            if lengthscore > prevlengthscore + 1 {
                store_lit_len_dist(input[i - 1] as u16, 0, i - 1, store);
                if lengthscore >= MIN_MATCH as i32 && (leng as usize) < MAX_MATCH {
                    match_available = true;
                    prev_length = leng;
                    prev_match = dist;
                    i += 1;
                    continue;
                }
            } else {
                // Add previous to output
                leng = prev_length;
                dist = prev_match;
                
                verify_len_dist(input, inend, i - 1, dist, leng);
                store_lit_len_dist(leng, dist, i - 1, store);
                
                for _ in 2..leng {
                    debug_assert!(i < inend);
                    i += 1;
                    update_hash(input, i, inend, h);
                }
                i += 1;
                continue;
            }
        } else if lengthscore >= MIN_MATCH as i32 && (leng as usize) < MAX_MATCH {
            match_available = true;
            prev_length = leng;
            prev_match = dist;
            i += 1;
            continue;
        }
        
        // Add to output
        if lengthscore >= MIN_MATCH as i32 {
            verify_len_dist(input, inend, i, dist, leng);
            store_lit_len_dist(leng, dist, i, store);
        } else {
            leng = 1;
            store_lit_len_dist(input[i] as u16, 0, i, store);
        }
        
        for _ in 1..leng {
            debug_assert!(i < inend);
            i += 1;
            update_hash(input, i, inend, h);
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Options;
    
    #[test]
    fn test_get_match() {
        let data = b"hello world hello";
        let pos = 12;
        let match_pos = 0;
        let end = 17;
        
        let len = get_match(data, pos, match_pos, end);
        assert_eq!(len, 5); // "hello" matches
    }
    
    #[test]
    fn test_store_lit_len_dist_literal() {
        let data = b"test";
        let mut store = LZ77Store::new(data);
        
        store_lit_len_dist(b't' as u16, 0, 0, &mut store);
        
        assert_eq!(store.size(), 1);
        assert_eq!(store.litlens[0], b't' as u16);
        assert_eq!(store.dists[0], 0);
    }
    
    #[test]
    fn test_store_lit_len_dist_match() {
        let data = b"test";
        let mut store = LZ77Store::new(data);
        
        store_lit_len_dist(4, 10, 0, &mut store);
        
        assert_eq!(store.size(), 1);
        assert_eq!(store.litlens[0], 4);
        assert_eq!(store.dists[0], 10);
    }
    
    #[test]
    fn test_lz77_greedy_simple() {
        let data = b"aaaaaa";
        let opts = Options::default();
        let mut state = BlockState::new(&opts, 0, data.len(), true);
        let mut store = LZ77Store::new(data);
        let mut hash = Hash::new(WINDOW_SIZE);
        
        lz77_greedy(&mut state, data, 0, data.len(), &mut store, &mut hash);
        
        // Should have compressed the repeated 'a's
        assert!(store.size() > 0);
        assert!(store.size() < data.len()); // Some compression should happen
    }
    
    #[test]
    fn test_lz77_greedy_with_pattern() {
        let data = b"hello worldhello";
        let opts = Options::default();
        let mut state = BlockState::new(&opts, 0, data.len(), true);
        let mut store = LZ77Store::new(data);
        let mut hash = Hash::new(WINDOW_SIZE);
        
        lz77_greedy(&mut state, data, 0, data.len(), &mut store, &mut hash);
        
        // Should find the repeated "hello"
        assert!(store.size() > 0);
        
        // Check that we found at least one backreference
        let has_backreference = store.dists.iter().any(|&d| d > 0);
        assert!(has_backreference, "Should find repeated 'hello'");
    }
}

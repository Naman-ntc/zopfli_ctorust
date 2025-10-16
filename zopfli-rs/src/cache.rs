// Copyright Anysphere Inc.
// Longest match cache implementation

use crate::types::{LongestMatchCache, BlockState, CACHE_LENGTH, MIN_MATCH, MAX_MATCH};

/// Returns the length up to which could be stored in the cache.
pub fn max_cached_sublen(lmc: &LongestMatchCache, pos: usize, _length: usize) -> usize {
    let cache_start = CACHE_LENGTH * pos * 3;
    if lmc.sublen[cache_start + 1] == 0 && lmc.sublen[cache_start + 2] == 0 {
        return 0; // No sublen cached
    }
    lmc.sublen[(CACHE_LENGTH - 1) * 3 + cache_start] as usize + 3
}

/// Stores sublen array in the cache.
pub fn sublen_to_cache(sublen: &[u16], pos: usize, length: usize, lmc: &mut LongestMatchCache) {
    let mut j = 0;
    let mut bestlength = 0;
    let cache_start = CACHE_LENGTH * pos * 3;
    
    if length < 3 {
        return;
    }
    
    for i in 3..=length {
        if i == length || sublen[i] != sublen[i + 1] {
            lmc.sublen[cache_start + j * 3] = (i - 3) as u8;
            lmc.sublen[cache_start + j * 3 + 1] = (sublen[i] % 256) as u8;
            lmc.sublen[cache_start + j * 3 + 2] = ((sublen[i] >> 8) % 256) as u8;
            bestlength = i;
            j += 1;
            if j >= CACHE_LENGTH {
                break;
            }
        }
    }
    
    if j < CACHE_LENGTH {
        debug_assert_eq!(bestlength, length);
        lmc.sublen[cache_start + (CACHE_LENGTH - 1) * 3] = (bestlength - 3) as u8;
    } else {
        debug_assert!(bestlength <= length);
    }
    debug_assert_eq!(bestlength, max_cached_sublen(lmc, pos, length));
}

/// Extracts sublen array from the cache.
pub fn cache_to_sublen(lmc: &LongestMatchCache, pos: usize, length: usize, sublen: &mut [u16]) {
    let maxlength = max_cached_sublen(lmc, pos, length);
    let mut prevlength = 0;
    let cache_start = CACHE_LENGTH * pos * 3;
    
    if length < 3 {
        return;
    }
    
    for j in 0..CACHE_LENGTH {
        let len = lmc.sublen[cache_start + j * 3] as usize + 3;
        let dist = lmc.sublen[cache_start + j * 3 + 1] as u16 
                   + 256 * lmc.sublen[cache_start + j * 3 + 2] as u16;
        
        for i in prevlength..=len {
            sublen[i] = dist;
        }
        
        if len == maxlength {
            break;
        }
        prevlength = len + 1;
    }
}

/// Stores the found sublen, distance and length in the longest match cache, if possible.
pub fn store_in_longest_match_cache(
    s: &mut BlockState,
    pos: usize,
    limit: usize,
    sublen: Option<&[u16]>,
    distance: u16,
    length: u16,
) {
    if let Some(lmc) = s.lmc.as_mut() {
        // The LMC cache starts at the beginning of the block rather than the
        // beginning of the whole array.
        let lmcpos = pos - s.blockstart;
        
        // Length > 0 and dist 0 is invalid combination, which indicates on purpose
        // that this cache value is not filled in yet.
        let cache_available = lmc.length[lmcpos] == 0 || lmc.dist[lmcpos] != 0;
        
        if limit == MAX_MATCH && sublen.is_some() && !cache_available {
            debug_assert_eq!(lmc.length[lmcpos], 1);
            debug_assert_eq!(lmc.dist[lmcpos], 0);
            
            lmc.dist[lmcpos] = if length < MIN_MATCH as u16 { 0 } else { distance };
            lmc.length[lmcpos] = if length < MIN_MATCH as u16 { 0 } else { length };
            
            debug_assert!(!(lmc.length[lmcpos] == 1 && lmc.dist[lmcpos] == 0));
            
            if let Some(sublen_arr) = sublen {
                sublen_to_cache(sublen_arr, lmcpos, length as usize, lmc);
            }
        }
    }
}

/// Gets distance, length and sublen values from the cache if possible.
/// Returns true if it got the values from the cache, false if not.
/// Updates the limit value to a smaller one if possible with more limited
/// information from the cache.
pub fn try_get_from_longest_match_cache(
    s: &BlockState,
    pos: usize,
    limit: &mut usize,
    sublen: Option<&mut [u16]>,
    distance: &mut u16,
    length: &mut u16,
) -> bool {
    if let Some(lmc) = &s.lmc {
        // The LMC cache starts at the beginning of the block rather than the
        // beginning of the whole array.
        let lmcpos = pos - s.blockstart;
        
        // Length > 0 and dist 0 is invalid combination, which indicates on purpose
        // that this cache value is not filled in yet.
        let cache_available = lmc.length[lmcpos] == 0 || lmc.dist[lmcpos] != 0;
        
        let limit_ok_for_cache = cache_available &&
            (*limit == MAX_MATCH || 
             lmc.length[lmcpos] as usize <= *limit ||
             (sublen.is_some() && 
              max_cached_sublen(lmc, lmcpos, lmc.length[lmcpos] as usize) >= *limit));
        
        if limit_ok_for_cache && cache_available {
            if sublen.is_none() || 
               lmc.length[lmcpos] as usize <= max_cached_sublen(lmc, lmcpos, lmc.length[lmcpos] as usize) {
                *length = lmc.length[lmcpos];
                if *length as usize > *limit {
                    *length = *limit as u16;
                }
                
                if let Some(sublen_arr) = sublen {
                    cache_to_sublen(lmc, lmcpos, *length as usize, sublen_arr);
                    *distance = sublen_arr[*length as usize];
                    
                    if *limit == MAX_MATCH && *length >= MIN_MATCH as u16 {
                        debug_assert_eq!(sublen_arr[*length as usize], lmc.dist[lmcpos]);
                    }
                } else {
                    *distance = lmc.dist[lmcpos];
                }
                return true;
            }
            // Can't use much of the cache, since the "sublens" need to be calculated,
            // but at least we already know when to stop.
            *limit = lmc.length[lmcpos] as usize;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Options;
    
    #[test]
    fn test_cache_creation() {
        let cache = LongestMatchCache::new(1000);
        assert_eq!(cache.length.len(), 1000);
        assert_eq!(cache.dist.len(), 1000);
        assert_eq!(cache.sublen.len(), CACHE_LENGTH * 1000 * 3);
    }
    
    #[test]
    fn test_max_cached_sublen_empty() {
        let cache = LongestMatchCache::new(100);
        let max_len = max_cached_sublen(&cache, 0, 10);
        assert_eq!(max_len, 0); // Empty cache
    }
    
    #[test]
    fn test_sublen_to_cache_and_back() {
        let mut cache = LongestMatchCache::new(100);
        let mut sublen = vec![0u16; 259];
        
        // Fill sublen with some test data
        for i in 3..=10 {
            sublen[i] = (i * 10) as u16;
        }
        
        // Store in cache
        sublen_to_cache(&sublen, 5, 10, &mut cache);
        
        // Retrieve from cache
        let mut retrieved = vec![0u16; 259];
        cache_to_sublen(&cache, 5, 10, &mut retrieved);
        
        // Verify
        for i in 3..=10 {
            assert_eq!(retrieved[i], sublen[i]);
        }
    }
    
    #[test]
    fn test_cache_with_block_state() {
        let opts = Options::default();
        let mut state = BlockState::new(&opts, 0, 100, true);
        
        let sublen = vec![5u16; 259];
        store_in_longest_match_cache(&mut state, 10, MAX_MATCH, Some(&sublen), 42, 15);
        
        if let Some(lmc) = &state.lmc {
            assert_eq!(lmc.dist[10], 42);
            assert_eq!(lmc.length[10], 15);
        }
    }
    
    #[test]
    fn test_try_get_from_cache() {
        let opts = Options::default();
        let mut state = BlockState::new(&opts, 0, 100, true);
        
        // Store something in cache
        let sublen = vec![10u16; 259];
        store_in_longest_match_cache(&mut state, 10, MAX_MATCH, Some(&sublen), 42, 15);
        
        // Try to retrieve
        let mut limit = MAX_MATCH;
        let mut distance = 0;
        let mut length = 0;
        
        let found = try_get_from_longest_match_cache(
            &state,
            10,
            &mut limit,
            None,
            &mut distance,
            &mut length,
        );
        
        assert!(found);
        assert_eq!(distance, 42);
        assert_eq!(length, 15);
    }
}


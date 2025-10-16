// Copyright Anysphere Inc.
// Block operations for calculating block sizes and histograms

use crate::types::{LZ77Store, NUM_LL, NUM_D};
use crate::huffman::{calculate_bit_lengths, optimize_huffman_for_rle};
use crate::symbols::{get_length_symbol, get_dist_symbol, get_length_symbol_extra_bits, get_dist_symbol_extra_bits};

/// Gets the histogram of lit/len and dist symbols in the given range at a specific position.
fn lz77_get_histogram_at(lz77: &LZ77Store, lpos: usize, ll_counts: &mut [usize], d_counts: &mut [usize]) {
    // The real histogram is created by using the histogram for this chunk, but
    // all superfluous values of this chunk subtracted.
    let llpos = NUM_LL * (lpos / NUM_LL);
    let dpos = NUM_D * (lpos / NUM_D);
    
    for i in 0..NUM_LL {
        ll_counts[i] = lz77.ll_counts[llpos + i];
    }
    
    for i in (lpos + 1)..(llpos + NUM_LL).min(lz77.size()) {
        ll_counts[lz77.ll_symbol[i] as usize] = ll_counts[lz77.ll_symbol[i] as usize].saturating_sub(1);
    }
    
    for i in 0..NUM_D {
        d_counts[i] = lz77.d_counts[dpos + i];
    }
    
    for i in (lpos + 1)..(dpos + NUM_D).min(lz77.size()) {
        if lz77.dists[i] != 0 {
            d_counts[lz77.d_symbol[i] as usize] = d_counts[lz77.d_symbol[i] as usize].saturating_sub(1);
        }
    }
}

/// Gets the histogram of lit/len and dist symbols in the given range, using the
/// cumulative histograms, so faster than adding one by one for large range. Does
/// not add the one end symbol of value 256.
pub fn lz77_get_histogram(lz77: &LZ77Store, lstart: usize, lend: usize, ll_counts: &mut [usize], d_counts: &mut [usize]) {
    if lstart + NUM_LL * 3 > lend {
        // Small range, calculate directly
        ll_counts.fill(0);
        d_counts.fill(0);
        
        for i in lstart..lend {
            ll_counts[lz77.ll_symbol[i] as usize] += 1;
            if lz77.dists[i] != 0 {
                d_counts[lz77.d_symbol[i] as usize] += 1;
            }
        }
    } else {
        // Subtract the cumulative histograms at the end and the start to get the
        // histogram for this range.
        lz77_get_histogram_at(lz77, lend - 1, ll_counts, d_counts);
        
        if lstart > 0 {
            let mut ll_counts2 = vec![0usize; NUM_LL];
            let mut d_counts2 = vec![0usize; NUM_D];
            lz77_get_histogram_at(lz77, lstart - 1, &mut ll_counts2, &mut d_counts2);
            
            for i in 0..NUM_LL {
                ll_counts[i] = ll_counts[i].saturating_sub(ll_counts2[i]);
            }
            for i in 0..NUM_D {
                d_counts[i] = d_counts[i].saturating_sub(d_counts2[i]);
            }
        }
    }
}

/// Gets the amount of raw bytes that this range of LZ77 symbols spans.
pub fn lz77_get_byte_range(lz77: &LZ77Store, lstart: usize, lend: usize) -> usize {
    if lstart == lend {
        return 0;
    }
    let l = lend - 1;
    let end_pos = lz77.pos[l] + if lz77.dists[l] == 0 { 1 } else { lz77.litlens[l] as usize };
    end_pos - lz77.pos[lstart]
}

/// Gets the fixed tree for DEFLATE fixed blocks.
pub fn get_fixed_tree(ll_lengths: &mut [u32], d_lengths: &mut [u32]) {
    for i in 0..144 {
        ll_lengths[i] = 8;
    }
    for i in 144..256 {
        ll_lengths[i] = 9;
    }
    for i in 256..280 {
        ll_lengths[i] = 7;
    }
    for i in 280..288 {
        ll_lengths[i] = 8;
    }
    for i in 0..32 {
        d_lengths[i] = 5;
    }
}

/// Ensures there are at least 2 distance codes to support buggy decoders.
pub fn patch_distance_codes_for_buggy_decoders(d_lengths: &mut [u32]) {
    let mut num_dist_codes = 0;
    
    for i in 0..30 {
        if d_lengths[i] > 0 {
            num_dist_codes += 1;
        }
        if num_dist_codes >= 2 {
            return;
        }
    }
    
    if num_dist_codes == 0 {
        d_lengths[0] = 1;
        d_lengths[1] = 1;
    } else if num_dist_codes == 1 {
        if d_lengths[0] > 0 {
            d_lengths[1] = 1;
        } else {
            d_lengths[0] = 1;
        }
    }
}

/// Calculates size of the part after the header and tree of an LZ77 block, in bits (small version).
fn calculate_block_symbol_size_small(
    ll_lengths: &[u32],
    d_lengths: &[u32],
    lz77: &LZ77Store,
    lstart: usize,
    lend: usize,
) -> usize {
    let mut result = 0;
    
    for i in lstart..lend {
        debug_assert!(i < lz77.size());
        debug_assert!(lz77.litlens[i] < 259);
        
        if lz77.dists[i] == 0 {
            result += ll_lengths[lz77.litlens[i] as usize] as usize;
        } else {
            let ll_symbol = get_length_symbol(lz77.litlens[i] as usize);
            let d_symbol = get_dist_symbol(lz77.dists[i] as usize);
            result += ll_lengths[ll_symbol] as usize;
            result += d_lengths[d_symbol] as usize;
            result += get_length_symbol_extra_bits(ll_symbol);
            result += get_dist_symbol_extra_bits(d_symbol);
        }
    }
    
    result += ll_lengths[256] as usize; // end symbol
    result
}

/// Calculates size with given counts.
fn calculate_block_symbol_size_given_counts(
    ll_counts: &[usize],
    d_counts: &[usize],
    ll_lengths: &[u32],
    d_lengths: &[u32],
    lz77: &LZ77Store,
    lstart: usize,
    lend: usize,
) -> usize {
    if lstart + NUM_LL * 3 > lend {
        return calculate_block_symbol_size_small(ll_lengths, d_lengths, lz77, lstart, lend);
    }
    
    let mut result = 0;
    
    for i in 0..256 {
        result += ll_lengths[i] as usize * ll_counts[i];
    }
    
    for i in 257..286 {
        result += ll_lengths[i] as usize * ll_counts[i];
        result += get_length_symbol_extra_bits(i) * ll_counts[i];
    }
    
    for i in 0..30 {
        result += d_lengths[i] as usize * d_counts[i];
        result += get_dist_symbol_extra_bits(i) * d_counts[i];
    }
    
    result += ll_lengths[256] as usize; // end symbol
    result
}

/// Calculates size of the part after the header and tree of an LZ77 block, in bits.
fn calculate_block_symbol_size(
    ll_lengths: &[u32],
    d_lengths: &[u32],
    lz77: &LZ77Store,
    lstart: usize,
    lend: usize,
) -> usize {
    if lstart + NUM_LL * 3 > lend {
        calculate_block_symbol_size_small(ll_lengths, d_lengths, lz77, lstart, lend)
    } else {
        let mut ll_counts = vec![0usize; NUM_LL];
        let mut d_counts = vec![0usize; NUM_D];
        lz77_get_histogram(lz77, lstart, lend, &mut ll_counts, &mut d_counts);
        calculate_block_symbol_size_given_counts(&ll_counts, &d_counts, ll_lengths, d_lengths, lz77, lstart, lend)
    }
}

/// Stub function for tree size calculation (simplified for now).
fn calculate_tree_size(_ll_lengths: &[u32], _d_lengths: &[u32]) -> usize {
    // Simplified: assume average tree size
    // Real implementation would call EncodeTree with all 8 combinations
    500 // Approximate tree size in bits
}

/// Tries to optimize Huffman for RLE and returns size.
fn try_optimize_huffman_for_rle(
    lz77: &LZ77Store,
    lstart: usize,
    lend: usize,
    ll_counts: &[usize],
    d_counts: &[usize],
    ll_lengths: &mut [u32],
    d_lengths: &mut [u32],
) -> f64 {
    let mut ll_counts2 = ll_counts.to_vec();
    let mut d_counts2 = d_counts.to_vec();
    let mut ll_lengths2 = vec![0u32; NUM_LL];
    let mut d_lengths2 = vec![0u32; NUM_D];
    
    let treesize = calculate_tree_size(ll_lengths, d_lengths);
    let datasize = calculate_block_symbol_size_given_counts(ll_counts, d_counts, ll_lengths, d_lengths, lz77, lstart, lend);
    
    optimize_huffman_for_rle(NUM_LL, &mut ll_counts2);
    optimize_huffman_for_rle(NUM_D, &mut d_counts2);
    calculate_bit_lengths(&ll_counts2, NUM_LL, 15, &mut ll_lengths2);
    calculate_bit_lengths(&d_counts2, NUM_D, 15, &mut d_lengths2);
    patch_distance_codes_for_buggy_decoders(&mut d_lengths2);
    
    let treesize2 = calculate_tree_size(&ll_lengths2, &d_lengths2);
    let datasize2 = calculate_block_symbol_size_given_counts(ll_counts, d_counts, &ll_lengths2, &d_lengths2, lz77, lstart, lend);
    
    if treesize2 + datasize2 < treesize + datasize {
        ll_lengths.copy_from_slice(&ll_lengths2);
        d_lengths.copy_from_slice(&d_lengths2);
        return (treesize2 + datasize2) as f64;
    }
    
    (treesize + datasize) as f64
}

/// Calculates the bit lengths for the symbols for dynamic blocks.
fn get_dynamic_lengths(
    lz77: &LZ77Store,
    lstart: usize,
    lend: usize,
    ll_lengths: &mut [u32],
    d_lengths: &mut [u32],
) -> f64 {
    let mut ll_counts = vec![0usize; NUM_LL];
    let mut d_counts = vec![0usize; NUM_D];
    
    lz77_get_histogram(lz77, lstart, lend, &mut ll_counts, &mut d_counts);
    ll_counts[256] = 1; // End symbol
    
    calculate_bit_lengths(&ll_counts, NUM_LL, 15, ll_lengths);
    calculate_bit_lengths(&d_counts, NUM_D, 15, d_lengths);
    patch_distance_codes_for_buggy_decoders(d_lengths);
    
    try_optimize_huffman_for_rle(lz77, lstart, lend, &ll_counts, &d_counts, ll_lengths, d_lengths)
}

/// Calculates block size in bits.
pub fn calculate_block_size(lz77: &LZ77Store, lstart: usize, lend: usize, btype: i32) -> f64 {
    let mut ll_lengths = vec![0u32; NUM_LL];
    let mut d_lengths = vec![0u32; NUM_D];
    
    let mut result = 3.0; // bfinal and btype bits
    
    if btype == 0 {
        // Uncompressed
        let length = lz77_get_byte_range(lz77, lstart, lend);
        let rem = length % 65535;
        let blocks = length / 65535 + if rem > 0 { 1 } else { 0 };
        return (blocks * 5 * 8 + length * 8) as f64;
    } else if btype == 1 {
        // Fixed tree
        get_fixed_tree(&mut ll_lengths, &mut d_lengths);
        result += calculate_block_symbol_size(&ll_lengths, &d_lengths, lz77, lstart, lend) as f64;
    } else {
        // Dynamic tree
        result += get_dynamic_lengths(lz77, lstart, lend, &mut ll_lengths, &mut d_lengths);
    }
    
    result
}

/// Calculates block size in bits, automatically using the best btype.
pub fn calculate_block_size_auto_type(lz77: &LZ77Store, lstart: usize, lend: usize) -> f64 {
    let uncompressedcost = calculate_block_size(lz77, lstart, lend, 0);
    
    // Don't do the expensive fixed cost calculation for larger blocks that are
    // unlikely to use it.
    let fixedcost = if lz77.size() > 1000 {
        uncompressedcost
    } else {
        calculate_block_size(lz77, lstart, lend, 1)
    };
    
    let dyncost = calculate_block_size(lz77, lstart, lend, 2);
    
    if uncompressedcost < fixedcost && uncompressedcost < dyncost {
        uncompressedcost
    } else if fixedcost < dyncost {
        fixedcost
    } else {
        dyncost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_fixed_tree() {
        let mut ll_lengths = vec![0u32; NUM_LL];
        let mut d_lengths = vec![0u32; NUM_D];
        
        get_fixed_tree(&mut ll_lengths, &mut d_lengths);
        
        assert_eq!(ll_lengths[0], 8);
        assert_eq!(ll_lengths[143], 8);
        assert_eq!(ll_lengths[144], 9);
        assert_eq!(ll_lengths[255], 9);
        assert_eq!(ll_lengths[256], 7);
        assert_eq!(ll_lengths[279], 7);
        assert_eq!(ll_lengths[280], 8);
        assert_eq!(ll_lengths[287], 8);
        
        for i in 0..32 {
            assert_eq!(d_lengths[i], 5);
        }
    }
    
    #[test]
    fn test_patch_distance_codes() {
        let mut d_lengths = vec![0u32; 32];
        patch_distance_codes_for_buggy_decoders(&mut d_lengths);
        
        // Should have at least 2 codes
        let count = d_lengths.iter().filter(|&&x| x > 0).count();
        assert!(count >= 2);
    }
    
    #[test]
    fn test_lz77_get_byte_range() {
        let data = b"hello world";
        let mut store = LZ77Store::new(data);
        
        // Add some literal symbols
        for i in 0..data.len() {
            store.litlens.push(data[i] as u16);
            store.dists.push(0);
            store.pos.push(i);
            store.ll_symbol.push(data[i] as u16);
            store.d_symbol.push(0);
        }
        
        let range = lz77_get_byte_range(&store, 0, data.len());
        assert_eq!(range, data.len());
    }
}

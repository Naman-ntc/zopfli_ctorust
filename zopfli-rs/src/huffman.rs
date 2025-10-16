// Copyright Anysphere Inc.
// Huffman encoding implementation

use crate::types::{Node, NodePool};
use crate::util::abs_diff;
use std::cmp::Ordering;

/// Changes the population counts in a way that the consequent Huffman tree
/// compression, especially its rle-part, will be more likely to compress this data
/// more efficiently. length contains the size of the histogram.
pub fn optimize_huffman_for_rle(length: usize, counts: &mut [usize]) {
    if length == 0 {
        return;
    }
    
    // 1) We don't want to touch the trailing zeros. We may break the
    // rules of the format by adding more data in the distance codes.
    let mut length = length;
    while length > 0 {
        if counts[length - 1] != 0 {
            // Now counts[0..length - 1] does not have trailing zeros.
            break;
        }
        length -= 1;
    }
    
    if length == 0 {
        return;
    }
    
    // 2) Let's mark all population counts that already can be encoded with an rle code.
    let mut good_for_rle = vec![false; length];
    
    // Let's not spoil any of the existing good rle codes.
    // Mark any seq of 0's that is longer than 5 as a good_for_rle.
    // Mark any seq of non-0's that is longer than 7 as a good_for_rle.
    let mut symbol = counts[0];
    let mut stride = 0;
    for i in 0..=length {
        if i == length || counts[i] != symbol {
            if (symbol == 0 && stride >= 5) || (symbol != 0 && stride >= 7) {
                for k in 0..stride {
                    good_for_rle[i - k - 1] = true;
                }
            }
            stride = 1;
            if i != length {
                symbol = counts[i];
            }
        } else {
            stride += 1;
        }
    }
    
    // 3) Let's replace those population counts that lead to more rle codes.
    stride = 0;
    let mut limit = counts[0];
    let mut sum = 0;
    for i in 0..=length {
        if i == length || good_for_rle[i] ||
            // Heuristic for selecting the stride ranges to collapse.
            abs_diff(counts[i], limit) >= 4 {
            if stride >= 4 || (stride >= 3 && sum == 0) {
                // The stride must end, collapse what we have, if we have enough (4).
                let mut count = (sum + stride / 2) / stride;
                if count < 1 {
                    count = 1;
                }
                if sum == 0 {
                    // Don't make an all zeros stride to be upgraded to ones.
                    count = 0;
                }
                for k in 0..stride {
                    // We don't want to change value at counts[i],
                    // that is already belonging to the next stride. Thus - 1.
                    counts[i - k - 1] = count;
                }
            }
            stride = 0;
            sum = 0;
            if i < length.saturating_sub(3) {
                // All interesting strides have a count of at least 4,
                // at least when non-zeros.
                limit = (counts[i] + counts[i + 1] + counts[i + 2] + counts[i + 3] + 2) / 4;
            } else if i < length {
                limit = counts[i];
            } else {
                limit = 0;
            }
        }
        stride += 1;
        if i != length {
            sum += counts[i];
        }
    }
}

/// Comparator for sorting leaves by weight
fn leaf_comparator(a: &Node, b: &Node) -> Ordering {
    a.weight.cmp(&b.weight)
}

/// Initializes a chain node with the given values
fn init_node(weight: usize, count: i32, tail: usize, node: &mut Node) {
    node.weight = weight;
    node.count = count;
    node.tail = tail;
}

/// Initializes each list with as lookahead chains the two leaves with lowest weights
fn init_lists(pool: &mut NodePool, leaves: &[Node], maxbits: usize, lists: &mut [[usize; 2]]) {
    let node0_idx = pool.allocate();
    let node1_idx = pool.allocate();
    
    init_node(leaves[0].weight, 1, usize::MAX, pool.get_mut(node0_idx));
    init_node(leaves[1].weight, 2, usize::MAX, pool.get_mut(node1_idx));
    
    for i in 0..maxbits {
        lists[i][0] = node0_idx;
        lists[i][1] = node1_idx;
    }
}

/// Performs final boundary package-merge step
fn boundary_pm_final(
    lists: &mut [[usize; 2]],
    leaves: &[Node],
    numsymbols: usize,
    pool: &mut NodePool,
    index: usize,
) {
    let lastcount = pool.get(lists[index][1]).count as usize;
    
    let sum = pool.get(lists[index - 1][0]).weight + pool.get(lists[index - 1][1]).weight;
    
    if lastcount < numsymbols && sum > leaves[lastcount].weight {
        let newchain_idx = pool.next_index;
        let oldchain_tail = pool.get(lists[index][1]).tail;
        
        lists[index][1] = newchain_idx;
        init_node(0, (lastcount + 1) as i32, oldchain_tail, pool.get_mut(newchain_idx));
    } else {
        let prev_list_1 = lists[index - 1][1];
        pool.get_mut(lists[index][1]).tail = prev_list_1;
    }
}

/// Performs a Boundary Package-Merge step
fn boundary_pm(
    lists: &mut [[usize; 2]],
    leaves: &[Node],
    numsymbols: usize,
    pool: &mut NodePool,
    index: usize,
) {
    let lastcount = pool.get(lists[index][1]).count as usize;
    
    if index == 0 && lastcount >= numsymbols {
        return;
    }
    
    let newchain_idx = pool.allocate();
    let oldchain_idx = lists[index][1];
    
    // These are set up before the recursive calls below, so that there is a list
    // pointing to the new node, to let the garbage collection know it's in use.
    lists[index][0] = oldchain_idx;
    lists[index][1] = newchain_idx;
    
    if index == 0 {
        // New leaf node in list 0.
        let leaf_weight = leaves[lastcount].weight;
        init_node(leaf_weight, (lastcount + 1) as i32, usize::MAX, pool.get_mut(newchain_idx));
    } else {
        let sum = pool.get(lists[index - 1][0]).weight + pool.get(lists[index - 1][1]).weight;
        if lastcount < numsymbols && sum > leaves[lastcount].weight {
            // New leaf inserted in list, so count is incremented.
            let leaf_weight = leaves[lastcount].weight;
            let oldchain_tail = pool.get(oldchain_idx).tail;
            init_node(leaf_weight, (lastcount + 1) as i32, oldchain_tail, pool.get_mut(newchain_idx));
        } else {
            let prev_list_1 = lists[index - 1][1];
            init_node(sum, lastcount as i32, prev_list_1, pool.get_mut(newchain_idx));
            // Two lookahead chains of previous list used up, create new ones.
            boundary_pm(lists, leaves, numsymbols, pool, index - 1);
            boundary_pm(lists, leaves, numsymbols, pool, index - 1);
        }
    }
}

/// Converts result of boundary package-merge to the bitlengths
fn extract_bit_lengths(chain_idx: usize, leaves: &[Node], pool: &NodePool, bitlengths: &mut [u32]) {
    let mut counts = [0i32; 16];
    let mut end = 16usize;
    let mut ptr = 15usize;
    let mut value = 1u32;
    
    // Traverse chain to fill counts
    let mut node_idx = chain_idx;
    while node_idx != usize::MAX {
        let node = pool.get(node_idx);
        end -= 1;
        counts[end] = node.count;
        node_idx = node.tail;
    }
    
    let mut val = counts[15];
    while ptr >= end {
        while val > counts[ptr.wrapping_sub(1)] {
            bitlengths[leaves[(val - 1) as usize].count as usize] = value;
            val -= 1;
        }
        if ptr == 0 {
            break;
        }
        ptr -= 1;
        value += 1;
    }
}

/// Outputs minimum-redundancy length-limited code bitlengths for symbols with the
/// given counts. The bitlengths are limited by maxbits.
///
/// The output is tailored for DEFLATE: symbols that never occur, get a bit length
/// of 0, and if only a single symbol occurs at least once, its bitlength will be 1,
/// and not 0 as would theoretically be needed for a single symbol.
///
/// Returns Ok(()) on success, Err on error.
pub fn length_limited_code_lengths(
    frequencies: &[usize],
    n: usize,
    maxbits: usize,
    bitlengths: &mut [u32],
) -> Result<(), &'static str> {
    // Initialize all bitlengths at 0
    for i in 0..n {
        bitlengths[i] = 0;
    }
    
    // Count used symbols and place them in the leaves
    let mut leaves = Vec::new();
    for i in 0..n {
        if frequencies[i] > 0 {
            leaves.push(Node {
                weight: frequencies[i],
                count: i as i32,
                tail: usize::MAX,
            });
        }
    }
    let numsymbols = leaves.len();
    
    // Check special cases and error conditions
    if (1 << maxbits) < numsymbols {
        return Err("Too few maxbits to represent symbols");
    }
    if numsymbols == 0 {
        return Ok(()); // No symbols at all. OK.
    }
    if numsymbols == 1 {
        bitlengths[leaves[0].count as usize] = 1;
        return Ok(()); // Only one symbol, give it bitlength 1, not 0. OK.
    }
    if numsymbols == 2 {
        bitlengths[leaves[0].count as usize] += 1;
        bitlengths[leaves[1].count as usize] += 1;
        return Ok(());
    }
    
    // Sort the leaves from lightest to heaviest. Add count into the same
    // variable for stable sorting.
    const CHAR_BIT: usize = 8;
    for leaf in &mut leaves {
        if leaf.weight >= (1usize << (std::mem::size_of::<usize>() * CHAR_BIT - 9)) {
            return Err("Weight too large, need 9 bits for count");
        }
        leaf.weight = (leaf.weight << 9) | (leaf.count as usize);
    }
    leaves.sort_by(leaf_comparator);
    for leaf in &mut leaves {
        leaf.weight >>= 9;
    }
    
    let maxbits = if numsymbols - 1 < maxbits {
        numsymbols - 1
    } else {
        maxbits
    };
    
    // Initialize node memory pool
    let pool_size = maxbits * 2 * numsymbols;
    let mut pool = NodePool::new(pool_size);
    
    let mut lists = vec![[0usize; 2]; maxbits];
    init_lists(&mut pool, &leaves, maxbits, &mut lists);
    
    // In the last list, 2 * numsymbols - 2 active chains need to be created. Two
    // are already created in the initialization. Each BoundaryPM run creates one.
    let num_boundary_pm_runs = 2 * numsymbols - 4;
    for _ in 0..num_boundary_pm_runs - 1 {
        boundary_pm(&mut lists, &leaves, numsymbols, &mut pool, maxbits - 1);
    }
    boundary_pm_final(&mut lists, &leaves, numsymbols, &mut pool, maxbits - 1);
    
    extract_bit_lengths(lists[maxbits - 1][1], &leaves, &pool, bitlengths);
    
    Ok(())
}

/// Calculates the bitlengths for the Huffman tree, based on the counts of each symbol.
pub fn calculate_bit_lengths(count: &[usize], n: usize, maxbits: usize, bitlengths: &mut [u32]) {
    let result = length_limited_code_lengths(count, n, maxbits, bitlengths);
    debug_assert!(result.is_ok());
}

/// Converts a series of Huffman tree bitlengths, to the bit values of the symbols.
pub fn lengths_to_symbols(lengths: &[u32], n: usize, maxbits: u32, symbols: &mut [u32]) {
    let mut bl_count = vec![0usize; maxbits as usize + 1];
    let mut next_code = vec![0usize; maxbits as usize + 1];
    
    // Initialize symbols
    for i in 0..n {
        symbols[i] = 0;
    }
    
    // 1) Count the number of codes for each code length
    for bits in 0..=maxbits as usize {
        bl_count[bits] = 0;
    }
    for i in 0..n {
        debug_assert!(lengths[i] <= maxbits);
        bl_count[lengths[i] as usize] += 1;
    }
    
    // 2) Find the numerical value of the smallest code for each code length
    let mut code = 0usize;
    bl_count[0] = 0;
    for bits in 1..=maxbits as usize {
        code = (code + bl_count[bits - 1]) << 1;
        next_code[bits] = code;
    }
    
    // 3) Assign numerical values to all codes
    for i in 0..n {
        let len = lengths[i] as usize;
        if len != 0 {
            symbols[i] = next_code[len] as u32;
            next_code[len] += 1;
        }
    }
}

/// Calculates the entropy of each symbol, based on the counts of each symbol.
pub fn calculate_entropy(count: &[usize], n: usize, bitlengths: &mut [f64]) {
    let mut sum = 0usize;
    for i in 0..n {
        sum += count[i];
    }
    
    let log2sum = (sum as f64).log2();
    for i in 0..n {
        // Cannot take log(0)
        if count[i] == 0 {
            bitlengths[i] = 0.0;
        } else {
            bitlengths[i] = log2sum - (count[i] as f64).log2();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimize_huffman_for_rle_empty() {
        let mut counts = vec![0; 10];
        optimize_huffman_for_rle(0, &mut counts);
        // Should handle empty gracefully
    }
    
    #[test]
    fn test_optimize_huffman_for_rle() {
        let mut counts = vec![5, 5, 5, 5, 0, 0, 0, 0, 0, 0, 3, 3, 3];
        optimize_huffman_for_rle(counts.len(), &mut counts);
        // Should optimize for RLE encoding
    }
    
    #[test]
    fn test_length_limited_code_lengths_simple() {
        let frequencies = vec![5, 7, 10, 15];
        let mut bitlengths = vec![0; 4];
        
        let result = length_limited_code_lengths(&frequencies, 4, 15, &mut bitlengths);
        assert!(result.is_ok());
        
        // All should have non-zero bit lengths
        for &bl in &bitlengths {
            assert!(bl > 0);
        }
        
        // More frequent symbols should have shorter or equal codes
        assert!(bitlengths[3] <= bitlengths[2]);
        assert!(bitlengths[2] <= bitlengths[1]);
    }
    
    #[test]
    fn test_length_limited_code_lengths_single_symbol() {
        let frequencies = vec![10, 0, 0, 0];
        let mut bitlengths = vec![0; 4];
        
        let result = length_limited_code_lengths(&frequencies, 4, 15, &mut bitlengths);
        assert!(result.is_ok());
        assert_eq!(bitlengths[0], 1); // Single symbol gets bitlength 1
        assert_eq!(bitlengths[1], 0);
        assert_eq!(bitlengths[2], 0);
        assert_eq!(bitlengths[3], 0);
    }
    
    #[test]
    fn test_length_limited_code_lengths_two_symbols() {
        let frequencies = vec![10, 5, 0, 0];
        let mut bitlengths = vec![0; 4];
        
        let result = length_limited_code_lengths(&frequencies, 4, 15, &mut bitlengths);
        assert!(result.is_ok());
        assert_eq!(bitlengths[0], 1);
        assert_eq!(bitlengths[1], 1);
        assert_eq!(bitlengths[2], 0);
        assert_eq!(bitlengths[3], 0);
    }
    
    #[test]
    fn test_lengths_to_symbols() {
        let lengths = vec![3, 3, 3, 3, 2, 4];
        let mut symbols = vec![0; 6];
        
        lengths_to_symbols(&lengths, 6, 15, &mut symbols);
        
        // Verify all symbols are different
        let mut seen = std::collections::HashSet::new();
        for i in 0..6 {
            if lengths[i] > 0 {
                assert!(seen.insert(symbols[i]));
            }
        }
    }
    
    #[test]
    fn test_calculate_entropy() {
        let counts = vec![10, 20, 30, 40];
        let mut bitlengths = vec![0.0; 4];
        
        calculate_entropy(&counts, 4, &mut bitlengths);
        
        // More frequent symbols should have lower entropy (fewer bits needed)
        assert!(bitlengths[3] < bitlengths[2]);
        assert!(bitlengths[2] < bitlengths[1]);
        assert!(bitlengths[1] < bitlengths[0]);
    }
    
    #[test]
    fn test_calculate_entropy_with_zero() {
        let counts = vec![10, 0, 30];
        let mut bitlengths = vec![0.0; 3];
        
        calculate_entropy(&counts, 3, &mut bitlengths);
        
        assert_eq!(bitlengths[1], 0.0); // Zero count should give 0 bits
    }
}

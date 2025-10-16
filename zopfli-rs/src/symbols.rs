// Copyright Anysphere Inc.
// Symbol and bit manipulation functions for DEFLATE format

/// Gets the symbol for the given length, as per the DEFLATE spec.
/// Returns the symbol in the range [257-285] (inclusive)
pub fn get_length_symbol(length: usize) -> usize {
    const TABLE: [usize; 259] = [
        0, 0, 0, 257, 258, 259, 260, 261, 262, 263, 264,
        265, 265, 266, 266, 267, 267, 268, 268,
        269, 269, 269, 269, 270, 270, 270, 270,
        271, 271, 271, 271, 272, 272, 272, 272,
        273, 273, 273, 273, 273, 273, 273, 273,
        274, 274, 274, 274, 274, 274, 274, 274,
        275, 275, 275, 275, 275, 275, 275, 275,
        276, 276, 276, 276, 276, 276, 276, 276,
        277, 277, 277, 277, 277, 277, 277, 277,
        277, 277, 277, 277, 277, 277, 277, 277,
        278, 278, 278, 278, 278, 278, 278, 278,
        278, 278, 278, 278, 278, 278, 278, 278,
        279, 279, 279, 279, 279, 279, 279, 279,
        279, 279, 279, 279, 279, 279, 279, 279,
        280, 280, 280, 280, 280, 280, 280, 280,
        280, 280, 280, 280, 280, 280, 280, 280,
        281, 281, 281, 281, 281, 281, 281, 281,
        281, 281, 281, 281, 281, 281, 281, 281,
        281, 281, 281, 281, 281, 281, 281, 281,
        281, 281, 281, 281, 281, 281, 281, 281,
        282, 282, 282, 282, 282, 282, 282, 282,
        282, 282, 282, 282, 282, 282, 282, 282,
        282, 282, 282, 282, 282, 282, 282, 282,
        282, 282, 282, 282, 282, 282, 282, 282,
        283, 283, 283, 283, 283, 283, 283, 283,
        283, 283, 283, 283, 283, 283, 283, 283,
        283, 283, 283, 283, 283, 283, 283, 283,
        283, 283, 283, 283, 283, 283, 283, 283,
        284, 284, 284, 284, 284, 284, 284, 284,
        284, 284, 284, 284, 284, 284, 284, 284,
        284, 284, 284, 284, 284, 284, 284, 284,
        284, 284, 284, 284, 284, 284, 284, 285,
    ];
    TABLE[length]
}

/// Gets the symbol for the given dist, as per the DEFLATE spec.
pub fn get_dist_symbol(dist: usize) -> usize {
    if dist < 5 {
        dist - 1
    } else {
        // log2(dist - 1): equivalent to 31 ^ __builtin_clz(dist - 1) in C
        // __builtin_clz returns number of leading zero bits in a 32-bit int
        let l = (31 ^ ((dist - 1) as u32).leading_zeros()) as usize;
        let r = ((dist - 1) >> (l - 1)) & 1;
        l * 2 + r
    }
}

/// Gets the amount of extra bits for the given distance symbol.
pub fn get_dist_symbol_extra_bits(symbol: usize) -> usize {
    const TABLE: [usize; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8,
        9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
    ];
    TABLE[symbol]
}

/// Gets the amount of extra bits for the given length symbol.
pub fn get_length_symbol_extra_bits(symbol: usize) -> usize {
    const TABLE: [usize; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2,
        3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    TABLE[symbol - 257]
}

/// Gets the amount of extra bits for the given length, as per the DEFLATE spec.
pub fn get_length_extra_bits(length: usize) -> usize {
    const TABLE: [usize; 259] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0,
    ];
    TABLE[length]
}

/// Gets the amount of extra bits for the given dist, as per the DEFLATE spec.
pub fn get_dist_extra_bits(dist: usize) -> usize {
    if dist < 5 {
        0
    } else {
        (31 ^ ((dist - 1) as u32).leading_zeros()) as usize - 1 // log2(dist - 1) - 1
    }
}

/// Gets value of the extra bits for the given length, as per the DEFLATE spec.
pub fn get_length_extra_bits_value(length: usize) -> usize {
    const TABLE: [usize; 259] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 2, 3, 0,
        1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5,
        6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6,
        7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
        13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2,
        3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
        10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
        29, 30, 31, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
        18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 0, 1, 2, 3, 4, 5, 6,
        7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
        27, 28, 29, 30, 31, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 0,
    ];
    TABLE[length]
}

/// Gets value of the extra bits for the given dist, as per the DEFLATE spec.
pub fn get_dist_extra_bits_value(dist: usize) -> usize {
    if dist < 5 {
        0
    } else {
        let l = (31 ^ ((dist - 1) as u32).leading_zeros()) as usize; // log2(dist - 1)
        (dist - (1 + (1 << l))) & ((1 << (l - 1)) - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MIN_MATCH, MAX_MATCH};
    
    #[test]
    fn test_length_symbol() {
        // Test boundary cases
        assert_eq!(get_length_symbol(3), 257);   // MIN_MATCH
        assert_eq!(get_length_symbol(4), 258);
        assert_eq!(get_length_symbol(5), 259);
        assert_eq!(get_length_symbol(10), 264);
        assert_eq!(get_length_symbol(258), 285);  // MAX_MATCH
        
        // Test ranges
        assert_eq!(get_length_symbol(11), 265);
        assert_eq!(get_length_symbol(12), 265);
        assert_eq!(get_length_symbol(18), 268);
    }
    
    #[test]
    fn test_dist_symbol() {
        assert_eq!(get_dist_symbol(1), 0);
        assert_eq!(get_dist_symbol(2), 1);
        assert_eq!(get_dist_symbol(3), 2);
        assert_eq!(get_dist_symbol(4), 3);
        assert_eq!(get_dist_symbol(5), 4);
        assert_eq!(get_dist_symbol(6), 4);
        assert_eq!(get_dist_symbol(7), 5);
        assert_eq!(get_dist_symbol(8), 5);
        assert_eq!(get_dist_symbol(9), 6);
        assert_eq!(get_dist_symbol(10), 6);
    }
    
    #[test]
    fn test_length_extra_bits() {
        assert_eq!(get_length_extra_bits(3), 0);
        assert_eq!(get_length_extra_bits(10), 0);
        assert_eq!(get_length_extra_bits(11), 1);
        assert_eq!(get_length_extra_bits(18), 1);
        assert_eq!(get_length_extra_bits(19), 2);
        assert_eq!(get_length_extra_bits(258), 0);  // Special case
    }
    
    #[test]
    fn test_dist_extra_bits() {
        assert_eq!(get_dist_extra_bits(1), 0);
        assert_eq!(get_dist_extra_bits(4), 0);
        assert_eq!(get_dist_extra_bits(5), 1);
        assert_eq!(get_dist_extra_bits(6), 1);
        assert_eq!(get_dist_extra_bits(7), 1);
        assert_eq!(get_dist_extra_bits(8), 1);
        assert_eq!(get_dist_extra_bits(9), 2);
        assert_eq!(get_dist_extra_bits(12), 2);
        assert_eq!(get_dist_extra_bits(17), 3);
    }
    
    #[test]
    fn test_length_extra_bits_value() {
        assert_eq!(get_length_extra_bits_value(3), 0);
        assert_eq!(get_length_extra_bits_value(11), 0);
        assert_eq!(get_length_extra_bits_value(12), 1);
        assert_eq!(get_length_extra_bits_value(13), 0);
        assert_eq!(get_length_extra_bits_value(14), 1);
    }
    
    #[test]
    fn test_dist_extra_bits_value() {
        assert_eq!(get_dist_extra_bits_value(1), 0);
        assert_eq!(get_dist_extra_bits_value(4), 0);
        assert_eq!(get_dist_extra_bits_value(5), 0);
        assert_eq!(get_dist_extra_bits_value(6), 1);
        assert_eq!(get_dist_extra_bits_value(7), 0);
        assert_eq!(get_dist_extra_bits_value(8), 1);
        assert_eq!(get_dist_extra_bits_value(9), 0);
        assert_eq!(get_dist_extra_bits_value(10), 1);
        assert_eq!(get_dist_extra_bits_value(11), 2);
        assert_eq!(get_dist_extra_bits_value(12), 3);
    }
    
    #[test]
    fn test_symbol_roundtrip() {
        // Test that symbol functions are consistent
        for len in MIN_MATCH..=MAX_MATCH {
            let symbol = get_length_symbol(len);
            assert!(symbol >= 257 && symbol <= 285);
            
            let extra_bits = get_length_extra_bits(len);
            let extra_value = get_length_extra_bits_value(len);
            
            // Verify extra value fits in extra bits
            if extra_bits > 0 {
                assert!(extra_value < (1 << extra_bits));
            } else {
                assert_eq!(extra_value, 0);
            }
        }
    }
    
    #[test]
    fn test_dist_symbol_range() {
        // Test a range of distances
        for dist in 1..=1024 {
            let symbol = get_dist_symbol(dist);
            assert!(symbol < 30);  // Max dist symbol is 29
            
            let extra_bits = get_dist_extra_bits(dist);
            let extra_value = get_dist_extra_bits_value(dist);
            
            // Verify extra value fits in extra bits
            if extra_bits > 0 {
                assert!(extra_value < (1 << extra_bits));
            } else {
                assert_eq!(extra_value, 0);
            }
        }
    }
}


// Copyright Anysphere Inc.
// Utility functions

/// Returns absolute difference between two values
#[inline]
pub fn abs_diff(x: usize, y: usize) -> usize {
    if x > y {
        x - y
    } else {
        y - x
    }
}

/// Returns minimum of two values
#[inline]
pub fn zopfli_min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

/// Ceiling division
#[inline]
pub fn ceil_div(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_abs_diff() {
        assert_eq!(abs_diff(5, 3), 2);
        assert_eq!(abs_diff(3, 5), 2);
        assert_eq!(abs_diff(5, 5), 0);
        assert_eq!(abs_diff(0, 10), 10);
        assert_eq!(abs_diff(10, 0), 10);
    }
    
    #[test]
    fn test_zopfli_min() {
        assert_eq!(zopfli_min(5, 3), 3);
        assert_eq!(zopfli_min(3, 5), 3);
        assert_eq!(zopfli_min(5, 5), 5);
        assert_eq!(zopfli_min(0, 10), 0);
    }
    
    #[test]
    fn test_ceil_div() {
        assert_eq!(ceil_div(10, 3), 4);
        assert_eq!(ceil_div(9, 3), 3);
        assert_eq!(ceil_div(1, 1), 1);
        assert_eq!(ceil_div(0, 1), 0);
        assert_eq!(ceil_div(7, 2), 4);
        assert_eq!(ceil_div(8, 2), 4);
    }
}


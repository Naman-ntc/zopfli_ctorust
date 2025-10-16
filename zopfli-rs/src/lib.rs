// Copyright Anysphere Inc.
// Safe Rust implementation of Zopfli compression library

pub mod types;
pub mod symbols;
pub mod util;
pub mod huffman;
pub mod hash;
pub mod cache;
pub mod lz77;
pub mod block;
pub mod split;
pub mod deflate;

pub use types::{Options, LZ77Store, BlockState};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_import() {
        let _ = Options::default();
    }
}

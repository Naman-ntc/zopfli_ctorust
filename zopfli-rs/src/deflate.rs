// Copyright Anysphere Inc.
// DEFLATE output generation

use crate::types::{LZ77Store, Options, NUM_LL, NUM_D};
use crate::block::get_fixed_tree;
use crate::huffman::lengths_to_symbols;
use crate::symbols::{get_length_symbol, get_dist_symbol, get_length_extra_bits, get_length_extra_bits_value, get_dist_extra_bits, get_dist_extra_bits_value};

pub struct BitWriter {
    pub out: Vec<u8>,
    pub bp: u8, // bit position 0..7
}

impl BitWriter {
    pub fn new() -> Self {
        Self { out: Vec::new(), bp: 0 }
    }
    
    pub fn add_bit(&mut self, bit: u8) {
        if self.bp == 0 { 
            self.out.push(0); 
        }
        let last = self.out.len() - 1;
        self.out[last] |= (bit & 1) << self.bp;
        self.bp = (self.bp + 1) & 7;
    }
    
    pub fn add_bits_le(&mut self, mut bits: u32, n: u8) {
        for _ in 0..n {
            let b = (bits & 1) as u8;
            bits >>= 1;
            self.add_bit(b);
        }
    }
    
    pub fn add_huff(&mut self, symbol: u32, length: u32) {
        // Huffman codes are reversed bit order
        let mut sym = symbol;
        for _ in 0..length {
            self.add_bit((sym >> (length - 1)) as u8);
            sym = (sym << 1) & ((1 << length) - 1);
        }
    }
}

fn add_lz77_data(
    lz77: &LZ77Store,
    lstart: usize,
    lend: usize,
    ll_symbols: &[u32],
    ll_lengths: &[u32],
    d_symbols: &[u32],
    d_lengths: &[u32],
    bw: &mut BitWriter,
) {
    for i in lstart..lend {
        let dist = lz77.dists[i] as usize;
        let litlen = lz77.litlens[i] as usize;
        
        if dist == 0 {
            // Literal
            bw.add_huff(ll_symbols[litlen], ll_lengths[litlen]);
        } else {
            // Match
            let ls = get_length_symbol(litlen);
            bw.add_huff(ll_symbols[ls], ll_lengths[ls]);
            let lbits = get_length_extra_bits(litlen) as u8;
            let lval = get_length_extra_bits_value(litlen) as u32;
            if lbits > 0 { 
                bw.add_bits_le(lval, lbits); 
            }
            
            let ds = get_dist_symbol(dist);
            bw.add_huff(d_symbols[ds], d_lengths[ds]);
            let dbits = get_dist_extra_bits(dist) as u8;
            let dval = get_dist_extra_bits_value(dist) as u32;
            if dbits > 0 { 
                bw.add_bits_le(dval, dbits); 
            }
        }
    }
    
    // End symbol 256
    bw.add_huff(ll_symbols[256], ll_lengths[256]);
}

pub fn deflate_fixed_block(lz77: &LZ77Store, lstart: usize, lend: usize, final_block: bool) -> Vec<u8> {
    let mut bw = BitWriter::new();
    
    // BFINAL bit
    bw.add_bit(if final_block {1} else {0});
    
    // BTYPE = 01 (fixed huffman)
    bw.add_bit(1);
    bw.add_bit(0);
    
    let mut ll_lengths = vec![0u32; NUM_LL];
    let mut d_lengths = vec![0u32; NUM_D];
    get_fixed_tree(&mut ll_lengths, &mut d_lengths);
    
    let mut ll_syms = vec![0u32; NUM_LL];
    let mut d_syms = vec![0u32; NUM_D];
    lengths_to_symbols(&ll_lengths, NUM_LL, 15, &mut ll_syms);
    lengths_to_symbols(&d_lengths, NUM_D, 15, &mut d_syms);
    
    add_lz77_data(lz77, lstart, lend, &ll_syms, &ll_lengths, &d_syms, &d_lengths, &mut bw);
    
    bw.out
}

pub fn deflate_greedy_fixed(input: &[u8]) -> Vec<u8> {
    use crate::lz77::lz77_greedy;
    use crate::types::{Hash, BlockState};
    
    let opts = Options::default();
    let mut state = BlockState::new(&opts, 0, input.len(), true);
    let mut store = LZ77Store::new(input);
    let mut hash = Hash::new(crate::types::WINDOW_SIZE);
    
    lz77_greedy(&mut state, input, 0, input.len(), &mut store, &mut hash);
    deflate_fixed_block(&store, 0, store.size(), true)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bit_writer() {
        let mut bw = BitWriter::new();
        bw.add_bit(1);
        bw.add_bit(0);
        bw.add_bit(1);
        
        assert_eq!(bw.out[0] & 0x07, 0b101);
    }
    
    #[test]
    fn test_deflate_simple() {
        let data = b"aaaa";
        let output = deflate_greedy_fixed(data);
        
        // Should produce valid DEFLATE output
        assert!(output.len() > 0);
        println!("Compressed {} bytes to {} bytes", data.len(), output.len());
    }
}

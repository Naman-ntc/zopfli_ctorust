// Copyright Anysphere Inc.
// Core type definitions for Zopfli compression

/// Number of distinct literal/length symbols in DEFLATE
pub const NUM_LL: usize = 288;

/// Number of distinct distance symbols in DEFLATE
pub const NUM_D: usize = 32;

/// Minimum match length
pub const MIN_MATCH: usize = 3;

/// Maximum match length
pub const MAX_MATCH: usize = 258;

/// Window size for DEFLATE (must be power of 2)
pub const WINDOW_SIZE: usize = 32768;

/// Window mask for wrapping indices
pub const WINDOW_MASK: usize = WINDOW_SIZE - 1;

/// Maximum hash chain hits
pub const MAX_CHAIN_HITS: usize = 8192;

/// Cache length multiplier
pub const CACHE_LENGTH: usize = 8;

/// Large float value for initialization
pub const LARGE_FLOAT: f64 = 1e30;

/// Master block size for huge files
pub const MASTER_BLOCK_SIZE: usize = 1000000;

/// Options used throughout the program
#[derive(Debug, Clone)]
pub struct Options {
    /// Whether to print output
    pub verbose: bool,
    
    /// Whether to print more detailed output
    pub verbose_more: bool,
    
    /// Maximum amount of times to rerun forward and backward pass to optimize LZ77
    /// compression cost. Good values: 10, 15 for small files, 5 for files over
    /// several MB in size or it will be too slow.
    pub numiterations: i32,
    
    /// If true, splits the data in multiple deflate blocks with optimal choice
    /// for the block boundaries. Block splitting gives better compression.
    pub blocksplitting: bool,
    
    /// No longer used, left for compatibility.
    pub blocksplittinglast: bool,
    
    /// Maximum amount of blocks to split into (0 for unlimited, but this can give
    /// extreme results that hurt compression on some files). Default value: 15.
    pub blocksplittingmax: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            verbose: false,
            verbose_more: false,
            numiterations: 15,
            blocksplitting: true,
            blocksplittinglast: false,
            blocksplittingmax: 15,
        }
    }
}

/// Stores lit/length and dist pairs for LZ77.
/// The memory is managed by init_lz77_store to initialize it,
/// clean_lz77_store to destroy it, and store_lit_len_dist to append values.
#[derive(Debug, Clone)]
pub struct LZ77Store {
    /// Literal or length values
    pub litlens: Vec<u16>,
    
    /// If 0: indicates literal in corresponding litlens,
    /// if > 0: length in corresponding litlens, this is the distance.
    pub dists: Vec<u16>,
    
    /// Original data reference
    pub data: Vec<u8>,
    
    /// Position in data where this LZ77 command begins
    pub pos: Vec<usize>,
    
    /// Literal/length symbols
    pub ll_symbol: Vec<u16>,
    
    /// Distance symbols
    pub d_symbol: Vec<u16>,
    
    /// Cumulative histograms wrapping around per chunk
    pub ll_counts: Vec<usize>,
    
    /// Cumulative distance histograms
    pub d_counts: Vec<usize>,
}

impl LZ77Store {
    pub fn new(data: &[u8]) -> Self {
        LZ77Store {
            litlens: Vec::new(),
            dists: Vec::new(),
            data: data.to_vec(),
            pos: Vec::new(),
            ll_symbol: Vec::new(),
            d_symbol: Vec::new(),
            ll_counts: Vec::new(),
            d_counts: Vec::new(),
        }
    }
    
    pub fn size(&self) -> usize {
        self.litlens.len()
    }
}

/// Symbol statistics for Huffman encoding
#[derive(Debug, Clone)]
pub struct SymbolStats {
    /// The literal and length symbol counts
    pub litlens: [usize; NUM_LL],
    
    /// The 32 unique dist symbol counts
    pub dists: [usize; NUM_D],
    
    /// Length of each lit/len symbol in bits
    pub ll_symbols: [f64; NUM_LL],
    
    /// Length of each dist symbol in bits
    pub d_symbols: [f64; NUM_D],
}

impl Default for SymbolStats {
    fn default() -> Self {
        SymbolStats {
            litlens: [0; NUM_LL],
            dists: [0; NUM_D],
            ll_symbols: [0.0; NUM_LL],
            d_symbols: [0.0; NUM_D],
        }
    }
}

/// Random state for optimization randomization
#[derive(Debug, Clone, Copy)]
pub struct RanState {
    pub m_w: u32,
    pub m_z: u32,
}

impl Default for RanState {
    fn default() -> Self {
        RanState { m_w: 1, m_z: 2 }
    }
}

/// Hash table for LZ77 pattern matching
#[derive(Debug)]
pub struct Hash {
    /// Hash value to index of its most recent occurrence
    pub head: Vec<i32>,
    
    /// Index to index of prev occurrence of same hash
    pub prev: Vec<u16>,
    
    /// Index to hash value at this index
    pub hashval: Vec<i32>,
    
    /// Current hash value
    pub val: i32,
    
    /// Second hash table
    pub head2: Vec<i32>,
    pub prev2: Vec<u16>,
    pub hashval2: Vec<i32>,
    pub val2: i32,
    
    /// Amount of repetitions of same byte after this
    pub same: Vec<u16>,
}

impl Hash {
    pub fn new(window_size: usize) -> Self {
        Hash {
            head: vec![-1; 65536],
            prev: vec![0; window_size],
            hashval: vec![-1; window_size],
            val: 0,
            head2: vec![-1; 65536],
            prev2: vec![0; window_size],
            hashval2: vec![-1; window_size],
            val2: 0,
            same: vec![0; window_size],
        }
    }
}

/// Node for Huffman tree construction
#[derive(Debug, Clone, Copy)]
pub struct Node {
    /// Total weight (symbol count) of this chain
    pub weight: usize,
    
    /// Index of tail node, or usize::MAX if none
    pub tail: usize,
    
    /// Leaf symbol index, or number of leaves before this chain
    pub count: i32,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            weight: 0,
            tail: usize::MAX,
            count: 0,
        }
    }
}

/// Memory pool for nodes (using indices instead of pointers)
#[derive(Debug)]
pub struct NodePool {
    /// All nodes in the pool
    pub nodes: Vec<Node>,
    
    /// Next available node index
    pub next_index: usize,
}

impl NodePool {
    pub fn new(capacity: usize) -> Self {
        NodePool {
            nodes: vec![Node::default(); capacity],
            next_index: 0,
        }
    }
    
    pub fn allocate(&mut self) -> usize {
        let index = self.next_index;
        self.next_index += 1;
        index
    }
    
    pub fn get(&self, index: usize) -> &Node {
        &self.nodes[index]
    }
    
    pub fn get_mut(&mut self, index: usize) -> &mut Node {
        &mut self.nodes[index]
    }
}

/// Cache used by find_longest_match to remember previously found length/dist values
#[derive(Debug)]
pub struct LongestMatchCache {
    /// Length for each position
    pub length: Vec<u16>,
    
    /// Distance for each position
    pub dist: Vec<u16>,
    
    /// Sublen array (uses large amounts of memory)
    pub sublen: Vec<u8>,
}

impl LongestMatchCache {
    pub fn new(blocksize: usize) -> Self {
        LongestMatchCache {
            length: vec![1; blocksize],
            dist: vec![0; blocksize],
            sublen: vec![0; CACHE_LENGTH * blocksize * 3],
        }
    }
}

/// Some state information for compressing a block
#[derive(Debug)]
pub struct BlockState<'a> {
    /// Reference to global options
    pub options: &'a Options,
    
    /// Cache for length/distance pairs found so far
    pub lmc: Option<LongestMatchCache>,
    
    /// The start (inclusive) and end (not inclusive) of the current block
    pub blockstart: usize,
    pub blockend: usize,
}

impl<'a> BlockState<'a> {
    pub fn new(
        options: &'a Options,
        blockstart: usize,
        blockend: usize,
        add_lmc: bool,
    ) -> Self {
        let blocksize = blockend - blockstart;
        BlockState {
            options,
            lmc: if add_lmc {
                Some(LongestMatchCache::new(blocksize))
            } else {
                None
            },
            blockstart,
            blockend,
        }
    }
}

/// Context for split cost calculation
#[derive(Debug)]
pub struct SplitCostContext<'a> {
    pub lz77: &'a LZ77Store,
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constants() {
        assert_eq!(NUM_LL, 288);
        assert_eq!(NUM_D, 32);
        assert_eq!(MIN_MATCH, 3);
        assert_eq!(MAX_MATCH, 258);
        assert_eq!(WINDOW_SIZE, 32768);
        assert_eq!(WINDOW_MASK, 32767);
        assert!(WINDOW_SIZE.is_power_of_two());
    }
    
    #[test]
    fn test_options_default() {
        let opts = Options::default();
        assert_eq!(opts.numiterations, 15);
        assert!(opts.blocksplitting);
        assert_eq!(opts.blocksplittingmax, 15);
    }
    
    #[test]
    fn test_lz77_store_new() {
        let data = vec![1, 2, 3, 4, 5];
        let store = LZ77Store::new(&data);
        assert_eq!(store.size(), 0);
        assert_eq!(store.data.len(), 5);
    }
    
    #[test]
    fn test_hash_new() {
        let hash = Hash::new(WINDOW_SIZE);
        assert_eq!(hash.head.len(), 65536);
        assert_eq!(hash.prev.len(), WINDOW_SIZE);
        assert_eq!(hash.same.len(), WINDOW_SIZE);
    }
    
    #[test]
    fn test_node_pool() {
        let mut pool = NodePool::new(100);
        let idx1 = pool.allocate();
        let idx2 = pool.allocate();
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        
        pool.get_mut(idx1).weight = 42;
        assert_eq!(pool.get(idx1).weight, 42);
    }
    
    #[test]
    fn test_longest_match_cache() {
        let cache = LongestMatchCache::new(100);
        assert_eq!(cache.length.len(), 100);
        assert_eq!(cache.dist.len(), 100);
        assert_eq!(cache.sublen.len(), CACHE_LENGTH * 100 * 3);
    }
    
    #[test]
    fn test_block_state() {
        let opts = Options::default();
        let state = BlockState::new(&opts, 0, 100, true);
        assert_eq!(state.blockstart, 0);
        assert_eq!(state.blockend, 100);
        assert!(state.lmc.is_some());
        
        let state2 = BlockState::new(&opts, 0, 100, false);
        assert!(state2.lmc.is_none());
    }
}


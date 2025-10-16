# Zopfli C to Safe Rust Translation Plan

## Overview
This document outlines the systematic translation of the Zopfli compression library from C to safe Rust. The C codebase consists of approximately 3,564 lines containing DEFLATE compression implementation.

## Project Structure

### Source Files
- `c_code/zopfli.h` (529 lines) - Header with all type definitions and function declarations
- `c_code/zopfli.c` (3,564 lines) - Complete implementation

### Rust Target Structure
```
zopfli-rs/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Main library interface
│   ├── types.rs                # All struct definitions
│   ├── huffman.rs              # Huffman encoding algorithms
│   ├── lz77.rs                 # LZ77 compression
│   ├── hash.rs                 # Hash table for pattern matching
│   ├── cache.rs                # Longest match cache
│   ├── block.rs                # Block operations
│   ├── deflate.rs              # DEFLATE format output
│   ├── split.rs                # Block splitting
│   ├── symbols.rs              # Symbol and bit manipulation
│   └── util.rs                 # Utility functions
├── tests/
│   ├── unit_tests.rs           # Unit tests for each module
│   ├── equivalence_tests.rs    # Equivalence tests against C
│   └── integration_tests.rs    # Full compression tests
└── benches/
    └── compression_bench.rs    # Performance benchmarks
```

## Key Translation Challenges

### 1. Memory Management
- **C**: Manual malloc/free with raw pointers
- **Rust**: Use Vec<T>, Box<T>, and ownership system
- **Strategy**: Replace all malloc/free with Vec::new() or Vec::with_capacity()

### 2. Function Pointers
- **C**: `typedef double (*CostModelFun)(unsigned litlen, unsigned dist, void *context);`
- **Rust**: Use trait objects or generic trait bounds
- **Strategy**: Define traits like `CostModel` and use generics or `Box<dyn Trait>`

### 3. Void Pointers
- **C**: `void *context` for generic data
- **Rust**: Use generics with trait bounds
- **Strategy**: Replace with specific types or generic parameters

### 4. Macros
- **C**: `ZOPFLI_APPEND_DATA` macro for dynamic array growth
- **Rust**: Use Vec::push() which handles growth automatically
- **Strategy**: Replace macro with standard Vec operations

### 5. Mutable Global State
- **C**: None present (good!)
- **Rust**: N/A

### 6. Arrays and Pointer Arithmetic
- **C**: `unsigned char *array`, pointer arithmetic
- **Rust**: Slices `&[u8]`, safe indexing
- **Strategy**: Use slices with bounds checking

## Module Breakdown

### Module 1: types.rs (Foundation)
All struct definitions from the header:
- `ZopfliOptions` - Configuration options
- `ZopfliLZ77Store` - LZ77 data storage
- `ZopfliHash` - Hash table for pattern matching
- `ZopfliLongestMatchCache` - Cache for match finding
- `ZopfliBlockState` - Block compression state
- `SymbolStats` - Symbol statistics
- `RanState` - Random number generator state
- `Node` - Huffman tree node
- `NodePool` - Memory pool for nodes
- `SplitCostContext` - Context for split cost calculation

**Testing**: Ensure struct sizes are reasonable and Debug/Clone traits work

### Module 2: symbols.rs (Symbol Utilities)
Functions for DEFLATE symbol handling:
- `get_length_symbol()` - Maps length to DEFLATE symbol (16-52)
- `get_dist_symbol()` - Maps distance to DEFLATE symbol (55-67)
- `get_length_extra_bits()` - Extra bits for length (203-209)
- `get_length_extra_bits_value()` - Value of extra bits (1286-1302)
- `get_dist_extra_bits()` - Extra bits for distance (1165-1170)
- `get_dist_extra_bits_value()` - Value of extra bits (1305-1316)
- `get_length_symbol_extra_bits()` - Symbol extra bits (194-200)
- `get_dist_symbol_extra_bits()` - Symbol extra bits (203-209)

**Testing**: Unit tests with known DEFLATE symbol mappings

### Module 3: util.rs (Utility Functions)
Basic utility functions:
- `abs_diff()` - Absolute difference (69-75)
- `zopfli_min()` - Minimum of two values (2124-2129)
- `ceil_div()` - Ceiling division (2805-2808)

**Testing**: Simple unit tests with edge cases

### Module 4: huffman.rs (Huffman Encoding)
Huffman tree construction and encoding:
- `optimize_huffman_for_rle()` - RLE optimization (82-191)
- `length_limited_code_lengths()` - Package-merge algorithm (439-542)
- `calculate_bit_lengths()` - Wrapper function (548-554)
- `lengths_to_symbols()` - Convert lengths to symbols (594-640)
- `calculate_entropy()` - Calculate entropy (2687-2717)
- `leaf_comparator()` - For sorting (285-288)
- `extract_bit_lengths()` - Extract from tree (295-319)
- `init_node()` - Initialize node (324-329)
- `init_lists()` - Initialize lists (335-348)
- `boundary_pm()` - Boundary package-merge (383-423)
- `boundary_pm_final()` - Final boundary PM (350-370)

**Testing**: Test with various frequency distributions, verify against known Huffman codes

### Module 5: hash.rs (Hash Table)
Hash table for LZ77 pattern matching:
- `alloc_hash()` - Allocate hash table (2335-2356)
- `reset_hash()` - Reset hash table (1999-2029)
- `clean_hash()` - Free hash table (2322-2333)
- `update_hash_value()` - Update hash value (1877-1885)
- `warmup_hash()` - Initialize hash (1887-1897)
- `update_hash()` - Update hash for position (1960-1997)

**Testing**: Test hash collisions, verify pattern matching works

### Module 6: cache.rs (Longest Match Cache)
Caching for longest match queries:
- `init_cache()` - Initialize cache (2410-2433)
- `clean_cache()` - Free cache (2394-2399)
- `max_cached_sublen()` - Get max cached length (1485-1494)
- `sublen_to_cache()` - Store in cache (1496-1535)
- `cache_to_sublen()` - Retrieve from cache (1561-1590)
- `store_in_longest_match_cache()` - Helper (1537-1559)
- `try_get_from_longest_match_cache()` - Try retrieve (1592-1646)

**Testing**: Verify cache hit/miss logic, test performance improvement

### Module 7: lz77.rs (LZ77 Compression)
Core LZ77 compression algorithm:
- `init_lz77_store()` - Initialize store (2464-2475)
- `clean_lz77_store()` - Free store (2453-2462)
- `copy_lz77_store()` - Copy store (2810-2856)
- `append_lz77_store()` - Append stores (3292-3310)
- `store_lit_len_dist()` - Store LZ77 data (1899-1958)
- `verify_len_dist()` - Verify match (1850-1875)
- `find_longest_match()` - Find longest match (1708-1848)
- `get_match()` - Helper for matching (1648-1706)
- `lz77_greedy()` - Greedy algorithm (2580-2685)
- `lz77_optimal()` - Optimal algorithm (2885-2986)
- `lz77_optimal_fixed()` - Optimal with fixed tree (2358-2392)
- `get_best_lengths()` - Dynamic programming (2187-2302)
- `lz77_optimal_run()` - Single run (2304-2320)
- `follow_path()` - Trace path (2031-2096)
- `trace_backwards()` - Trace backwards (2098-2122)
- `get_length_score()` - Score for greedy (2564-2578)

**Testing**: Test with simple patterns, verify match finding, compare output with C

### Module 8: block.rs (Block Operations)
Block size calculation and operations:
- `lz77_get_histogram()` - Get histogram (950-987)
- `lz77_get_histogram_at()` - Get at position (923-948)
- `lz77_get_byte_range()` - Get byte range (1027-1034)
- `calculate_block_size()` - Calculate size (1066-1096)
- `calculate_block_size_auto_type()` - Auto type (1101-1112)
- `calculate_block_symbol_size()` - Symbol size (1039-1057)
- `calculate_block_symbol_size_small()` - Small blocks (215-242)
- `calculate_block_symbol_size_given_counts()` - With counts (247-280)
- `calculate_tree_size()` - Tree size (824-840)
- `get_dynamic_lengths()` - Dynamic tree lengths (996-1010)
- `get_fixed_tree()` - Fixed tree (1012-1025)
- `patch_distance_codes_for_buggy_decoders()` - Workaround (854-874)
- `try_optimize_huffman_for_rle()` - Try RLE optimization (881-918)
- `init_block_state()` - Initialize state (2435-2451)
- `clean_block_state()` - Clean state (2401-2408)

**Testing**: Verify block size calculations match C implementation

### Module 9: split.rs (Block Splitting)
Block splitting for optimal compression:
- `block_split()` - Split uncompressed data (3240-3290)
- `block_split_lz77()` - Split LZ77 data (3154-3238)
- `find_minimum()` - Find minimum cost split (3082-3152)
- `split_cost()` - Cost function (1135-1139)
- `estimate_cost()` - Estimate block cost (1124-1128)
- `find_largest_splittable_block()` - Find block (2988-3012)
- `print_block_split_points()` - Debug print (3014-3053)
- `add_sorted()` - Add to sorted array (3055-3080)

**Testing**: Test split point selection, verify cost improvements

### Module 10: deflate.rs (DEFLATE Output)
DEFLATE format encoding and output:
- `add_bit()` - Add single bit (1229-1236)
- `add_bits()` - Add multiple bits (576-589)
- `add_huffman_bits()` - Add Huffman coded bits (560-574)
- `encode_tree()` - Encode Huffman tree (646-819)
- `add_dynamic_tree()` - Add dynamic tree (1366-1390)
- `add_lz77_data()` - Add LZ77 encoded data (1323-1364)
- `add_lz77_block()` - Add LZ77 block (1409-1480)
- `add_lz77_block_auto_type()` - Auto type (2477-2562)
- `add_non_compressed_block()` - Uncompressed (1240-1283)
- `deflate_part()` - Compress part (3312-3442)
- `deflate()` - Main deflate function (3444-3466)

**Testing**: Verify bit-level output matches C, test with various input sizes

### Module 11: Symbol Statistics
Symbol statistics for optimization:
- `init_stats()` - Initialize (2871-2883)
- `copy_stats()` - Copy (2858-2869)
- `get_statistics()` - Get from store (2726-2744)
- `calculate_statistics()` - Calculate (2719-2724)
- `clear_stat_freqs()` - Clear frequencies (2746-2754)
- `randomize_stat_freqs()` - Randomize (2773-2778)
- `randomize_freqs()` - Helper (2763-2771)
- `add_weighed_stat_freqs()` - Add weighted (2787-2803)
- `init_ran_state()` - Initialize RNG (2780-2785)
- `ran()` - Random number (2756-2761)

**Testing**: Verify statistics calculation

### Module 12: Cost Models
Cost models for optimization:
- `get_cost_fixed()` - Fixed tree cost (1197-1220)
- `get_cost_stat()` - Statistical cost (1176-1191)
- `get_cost_model_min_cost()` - Min cost (2133-2185)

**Testing**: Test cost calculations

## Translation Strategy

### Phase 1: Project Setup (Step 1)
1. Create Rust project with `cargo new --lib zopfli-rs`
2. Set up Cargo.toml with dependencies
3. Create module structure
4. Set up test framework

### Phase 2: Foundation Types (Steps 2-3)
1. Translate all struct definitions to types.rs
2. Implement basic traits (Debug, Clone, Default where appropriate)
3. Write basic instantiation tests

### Phase 3: Utility & Symbol Functions (Steps 4-6)
1. Translate symbols.rs (all symbol lookup functions)
2. Translate util.rs (utility functions)
3. Write comprehensive unit tests
4. Create test vectors from C implementation

### Phase 4: Core Algorithms (Steps 7-12)
1. Translate huffman.rs (Huffman encoding)
2. Translate hash.rs (hash table)
3. Translate cache.rs (match cache)
4. Write unit tests for each module
5. Create integration tests

### Phase 5: LZ77 Compression (Steps 13-15)
1. Translate lz77.rs (core compression)
2. This is the most complex module
3. Extensive testing with known patterns
4. Verify match finding correctness

### Phase 6: Block Operations (Steps 16-18)
1. Translate block.rs (block calculations)
2. Translate split.rs (block splitting)
3. Integration tests with previous modules

### Phase 7: DEFLATE Output (Steps 19-21)
1. Translate deflate.rs (output encoding)
2. Bit-level testing
3. Format validation

### Phase 8: Integration & Testing (Steps 22-25)
1. End-to-end compression tests
2. Equivalence tests against C implementation
3. Performance benchmarks
4. Fuzzing tests

## Safety Considerations

### Memory Safety
1. **No unsafe code** - Use only safe Rust
2. **Bounds checking** - All array accesses checked
3. **No raw pointers** - Use references and slices
4. **No manual memory management** - Let Rust handle it

### Integer Safety
1. **Use checked arithmetic** where overflow is possible
2. **Use usize for indices** consistently
3. **Validate casts** between integer types
4. **Handle edge cases** in bit manipulation

### Array Access
1. **Use .get()** for optional access
2. **Use pattern matching** on Option results
3. **Validate indices** before access
4. **Use iterators** where possible

## Testing Strategy

### Unit Tests
- Test each function individually
- Cover edge cases (0, max values, etc.)
- Test error conditions

### Equivalence Tests
- Generate test vectors from C implementation
- Compare output byte-for-byte
- Test with various input sizes and patterns
- Test all compression levels and options

### Integration Tests
- Full compression pipeline
- Various file types
- Large files (>1MB)
- Empty input
- Single byte input
- Repetitive patterns
- Random data

### Property-Based Tests
- Use proptest for random input testing
- Verify decompression produces original data
- Verify output is valid DEFLATE format

### Performance Tests
- Benchmark against C implementation
- Ensure no major performance regression
- Profile hot paths

## Test Data Sources
1. Empty data
2. Single byte patterns
3. Repeated patterns (aaa...a, abab...ab)
4. English text
5. Random data
6. Already compressed data
7. Real-world files (HTML, JSON, images)

## Validation Criteria
1. **Correctness**: Output decompresses to original input
2. **Equivalence**: Byte-identical output to C version (with same RNG seed)
3. **Safety**: No unsafe code, no panics on valid input
4. **Performance**: Within 2x of C implementation
5. **Compatibility**: Passes existing Zopfli test suite
6. **Code Quality**: Passes clippy with no warnings

## Implementation Order

### Critical Path Functions (Must be perfect)
1. Symbol lookup functions (foundational)
2. Hash table operations (performance critical)
3. Find longest match (core algorithm)
4. LZ77 optimal (compression quality)
5. Huffman encoding (correctness critical)
6. DEFLATE output (format compliance)

### Less Critical (Can iterate)
1. Block splitting optimization
2. RLE optimization
3. Random state for optimization
4. Verbose output

## Known C Patterns to Translate

### Pattern 1: Dynamic Array Growth
**C**:
```c
ZOPFLI_APPEND_DATA(value, &data, &size)
// Macro that doubles allocation when size is power of 2
```
**Rust**:
```rust
data.push(value);  // Vec handles growth automatically
```

### Pattern 2: Manual Memory Management
**C**:
```c
int *data = (int*)malloc(n * sizeof(int));
// ... use data ...
free(data);
```
**Rust**:
```rust
let data = vec![0; n];  // Automatically freed when dropped
```

### Pattern 3: Function Pointers with Context
**C**:
```c
typedef double (*CostModelFun)(unsigned litlen, unsigned dist, void *context);
double cost = costfun(litlen, dist, context);
```
**Rust** (Option 1 - Trait):
```rust
trait CostModel {
    fn cost(&self, litlen: u16, dist: u16) -> f64;
}
let cost = model.cost(litlen, dist);
```
**Rust** (Option 2 - Closure):
```rust
let costfun: Box<dyn Fn(u16, u16) -> f64> = ...;
let cost = costfun(litlen, dist);
```

### Pattern 4: Two-way Iteration with Pointers
**C**:
```c
const unsigned char *scan = array + pos;
const unsigned char *match = array + pos - dist;
while (*scan == *match) { scan++; match++; }
```
**Rust**:
```rust
let scan = &array[pos..];
let match_slice = &array[pos - dist..];
let len = scan.iter().zip(match_slice.iter())
    .take_while(|(a, b)| a == b)
    .count();
```

### Pattern 5: Output Array Growth
**C**:
```c
unsigned char **out;  // Pointer to pointer for resizable array
ZOPFLI_APPEND_DATA(byte, out, outsize);
```
**Rust**:
```rust
let mut out = Vec::new();
out.push(byte);
```

## Module Dependencies

```
types.rs (no dependencies)
    ↓
util.rs, symbols.rs (depend on types)
    ↓
huffman.rs, hash.rs (depend on types, symbols, util)
    ↓
cache.rs (depends on types, hash)
    ↓
lz77.rs (depends on types, hash, cache, symbols)
    ↓
block.rs (depends on types, lz77, huffman, symbols)
    ↓
split.rs (depends on types, lz77, block)
    ↓
deflate.rs (depends on all above)
    ↓
lib.rs (public API)
```

## Incremental Testing Approach

After each module translation:
1. Run unit tests
2. Run integration tests up to that point
3. Compare intermediate results with C version
4. Fix any discrepancies before moving on

## Risk Areas

### High Risk (Complex, Easy to Get Wrong)
1. **LZ77 optimal path finding** - Complex DP algorithm
2. **Huffman package-merge** - Intricate tree construction
3. **Hash table updates** - Off-by-one errors possible
4. **Bit-level output** - Easy to mess up bit ordering
5. **Cache management** - Complex indexing

### Medium Risk
1. **Block splitting** - Optimization, not correctness critical
2. **Symbol calculations** - Many lookup tables
3. **Statistics calculations** - Floating point precision

### Low Risk
1. **Utility functions** - Simple, easy to verify
2. **Options struct** - Just data
3. **Initialization/cleanup** - Rust handles automatically

## Progress Tracking

We'll create a checklist as we go:

- [ ] Phase 1: Project Setup
- [ ] Phase 2: Foundation Types
- [ ] Phase 3: Utility & Symbol Functions
- [ ] Phase 4: Core Algorithms
- [ ] Phase 5: LZ77 Compression
- [ ] Phase 6: Block Operations  
- [ ] Phase 7: DEFLATE Output
- [ ] Phase 8: Integration & Testing

## Next Steps

1. Create Cargo project
2. Set up module structure
3. Begin with types.rs
4. Implement symbols.rs with tests
5. Continue module by module, testing each

This plan ensures:
- **Correctness**: Extensive testing at each step
- **Safety**: Only safe Rust, no unsafe blocks
- **Equivalence**: Byte-for-byte comparison with C
- **Maintainability**: Clean, idiomatic Rust code


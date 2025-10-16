// Copyright Anysphere Inc.
// Roundtrip tests - compress with our code, decompress with flate2

use flate2::read::DeflateDecoder;
use std::io::Read;
use zopfli_rs::deflate::deflate_greedy_fixed;

fn decompress_deflate(compressed: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

#[test]
fn test_roundtrip_hello_world() {
    let original = b"hello world";
    let compressed = deflate_greedy_fixed(original);
    
    println!("Original: {} bytes", original.len());
    println!("Compressed: {} bytes", compressed.len());
    println!("Compressed data: {:02X?}", &compressed[..compressed.len().min(50)]);
    
    match decompress_deflate(&compressed) {
        Ok(decompressed) => {
            assert_eq!(decompressed, original, "Decompressed data doesn't match original!");
            println!("✅ Roundtrip successful!");
        }
        Err(e) => {
            panic!("Failed to decompress: {}", e);
        }
    }
}

#[test]
fn test_roundtrip_repeated_pattern() {
    let original = b"aaaaaaaaaa";
    let compressed = deflate_greedy_fixed(original);
    
    println!("Original: {} bytes ({})", original.len(), std::str::from_utf8(original).unwrap());
    println!("Compressed: {} bytes", compressed.len());
    
    match decompress_deflate(&compressed) {
        Ok(decompressed) => {
            assert_eq!(decompressed, original, "Decompressed data doesn't match original!");
            println!("✅ Roundtrip successful for repeated pattern!");
        }
        Err(e) => {
            panic!("Failed to decompress: {}", e);
        }
    }
}

#[test]
fn test_roundtrip_hhhheeeeellllloooooo() {
    let original = b"hhhheeeeellllloooooo";
    let compressed = deflate_greedy_fixed(original);
    
    println!("Original: {} bytes ({})", original.len(), std::str::from_utf8(original).unwrap());
    println!("Compressed: {} bytes", compressed.len());
    
    match decompress_deflate(&compressed) {
        Ok(decompressed) => {
            assert_eq!(decompressed, original, "Decompressed data doesn't match original!");
            println!("✅ Roundtrip successful for hhhheeeeellllloooooo!");
        }
        Err(e) => {
            panic!("Failed to decompress: {}", e);
        }
    }
}

#[test]
fn test_roundtrip_hello_worldaaaaaaaaa() {
    let original = b"hello worldaaaaaaaaa";
    let compressed = deflate_greedy_fixed(original);
    
    println!("Original: {} bytes ({})", original.len(), std::str::from_utf8(original).unwrap());
    println!("Compressed: {} bytes", compressed.len());
    
    match decompress_deflate(&compressed) {
        Ok(decompressed) => {
            assert_eq!(decompressed, original, "Decompressed data doesn't match original!");
            println!("✅ Roundtrip successful for 'hello worldaaaaaaaaa'!");
        }
        Err(e) => {
            panic!("Failed to decompress: {}", e);
        }
    }
}

#[test]
fn test_roundtrip_empty() {
    let original = b"";
    let compressed = deflate_greedy_fixed(original);
    
    match decompress_deflate(&compressed) {
        Ok(decompressed) => {
            assert_eq!(decompressed, original);
            println!("✅ Roundtrip successful for empty input!");
        }
        Err(e) => {
            panic!("Failed to decompress empty: {}", e);
        }
    }
}

#[test]
fn test_roundtrip_single_byte() {
    let original = b"a";
    let compressed = deflate_greedy_fixed(original);
    
    match decompress_deflate(&compressed) {
        Ok(decompressed) => {
            assert_eq!(decompressed, original);
            println!("✅ Roundtrip successful for single byte!");
        }
        Err(e) => {
            panic!("Failed to decompress single byte: {}", e);
        }
    }
}

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_roundtrip_property(s in "\\PC*") {
        let original = s.as_bytes();
        if original.len() > 1000 {
            return Ok(()); // Skip very large inputs
        }
        
        let compressed = deflate_greedy_fixed(original);
        
        match decompress_deflate(&compressed) {
            Ok(decompressed) => {
                prop_assert_eq!(&decompressed[..], original, 
                    "Roundtrip failed for input: {:?}", 
                    std::str::from_utf8(original).unwrap_or("<binary>"));
            }
            Err(e) => {
                return Err(proptest::test_runner::TestCaseError::fail(
                    format!("Failed to decompress: {}", e)
                ));
            }
        }
    }
    
    #[test]
    fn test_roundtrip_property_bytes(data in prop::collection::vec(any::<u8>(), 0..100)) {
        let original = &data[..];
        let compressed = deflate_greedy_fixed(original);
        
        match decompress_deflate(&compressed) {
            Ok(decompressed) => {
                prop_assert_eq!(&decompressed[..], original, 
                    "Roundtrip failed for binary data");
            }
            Err(e) => {
                return Err(proptest::test_runner::TestCaseError::fail(
                    format!("Failed to decompress: {}", e)
                ));
            }
        }
    }
}


// Copyright Anysphere Inc.
// Direct byte-for-byte C comparison tests

use zopfli_rs::deflate::deflate_greedy_fixed;
use std::process::Command;

fn get_c_output(input: &str) -> Vec<u8> {
    let output = Command::new("/home/ubuntu/Repos/Syzygy_Zopfli/zopfliii/c_test_single")
        .arg(input)
        .output()
        .expect("Failed to run C test binary");
    
    if !output.status.success() {
        eprintln!("C test failed with status: {}", output.status);
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("C compression failed");
    }
    
    output.stdout
}

fn compare_with_c(input: &str, test_name: &str) {
    let rust_output = deflate_greedy_fixed(input.as_bytes());
    let c_output = get_c_output(input);
    
    println!("\n=== {} ===", test_name);
    println!("Input: \"{}\" ({} bytes)", input, input.len());
    println!("Rust: {} bytes - {:02X?}", rust_output.len(), &rust_output[..rust_output.len().min(20)]);
    println!("C:    {} bytes - {:02X?}", c_output.len(), &c_output[..c_output.len().min(20)]);
    
    assert_eq!(
        rust_output, c_output,
        "\n❌ BYTE MISMATCH for: \"{}\"\nRust ({} bytes): {:02X?}\nC    ({} bytes): {:02X?}",
        input, rust_output.len(), rust_output, c_output.len(), c_output
    );
    
    println!("✅ PERFECT MATCH!");
}

#[test]
fn equiv_hello_world() {
    compare_with_c("hello world", "hello world");
}

#[test]
fn equiv_aaaaaaaaaa() {
    compare_with_c("aaaaaaaaaa", "10 a's");
}

#[test]
fn equiv_hhhheeeeellllloooooo() {
    compare_with_c("hhhheeeeellllloooooo", "hhhheeeeellllloooooo");
}

#[test]
fn equiv_hello_worldaaaaaaaaa() {
    compare_with_c("hello worldaaaaaaaaa", "hello worldaaaaaaaaa");
}

#[test]
fn equiv_helllloooo_world() {
    compare_with_c("helllloooo world", "helllloooo world");
}

#[test]
fn equiv_testaaaaaaaaaa() {
    compare_with_c("testaaaaaaaaaa", "test + 10 a's");
}

#[test]
fn equiv_aaaaaaaaatest() {
    compare_with_c("aaaaaaaaatest", "9 a's + test");
}

#[test]
fn equiv_aaabbbcccddd() {
    compare_with_c("aaabbbcccddd", "aaabbbcccddd");
}

#[test]
fn equiv_long_repeat() {
    compare_with_c("aaaaaaaaaaaaaaaaaaaabbbbbbbbbbbbbbbbbbbb", "20 a's + 20 b's");
}

#[test]
fn equiv_alphabet() {
    compare_with_c("abcdefghijklmnopqrstuvwxyz", "alphabet");
}

#[test]
fn equiv_numbers() {
    compare_with_c("0123456789012345678901234567890123456789", "repeated digits");
}

#[test]
fn equiv_punctuation() {
    compare_with_c("!!!!!!......??????", "punctuation repeats");
}

#[test]
fn equiv_sentence() {
    compare_with_c("The quick brown fox jumps over the lazy dog", "pangram");
}

#[test]
fn equiv_repeated_words() {
    compare_with_c("test test test test", "repeated test");
}

#[test]
fn equiv_pattern_abc() {
    compare_with_c("abcabcabcabcabcabcabc", "repeated abc pattern");
}

#[test]
fn equiv_pattern_xyz() {
    compare_with_c("xyzxyzxyzxyzxyzxyzxyz", "repeated xyz pattern");
}

#[test]
fn equiv_hello_variant1() {
    compare_with_c("hhhheeeelllllllooooo", "hello variant 1");
}

#[test]
fn equiv_hello_variant2() {
    compare_with_c("hheelllloo", "hello variant 2");
}

#[test]
fn equiv_hello_variant3() {
    compare_with_c("hhhhhheeeeeeelllllllloooooooo", "hello variant 3");
}

#[test]
fn equiv_world_variant() {
    compare_with_c("wwwwoooorrrrlllldddd", "world variant");
}

#[test]
fn equiv_alternating() {
    compare_with_c("ababababababababab", "alternating ab");
}

#[test]
fn equiv_increasing() {
    compare_with_c("abbcccddddeeeeeffffff", "increasing repeats");
}

#[test]
fn equiv_empty() {
    compare_with_c("", "empty string");
}

#[test]
fn equiv_single_char() {
    compare_with_c("a", "single character");
}

#[test]
fn equiv_two_chars() {
    compare_with_c("ab", "two characters");
}

#[test]
fn equiv_three_chars() {
    compare_with_c("abc", "three characters");
}

#[test]
fn equiv_100_as() {
    let input = "a".repeat(100);
    compare_with_c(&input, "100 a's");
}

#[test]
fn equiv_50_pattern() {
    let input = "ab".repeat(50);
    compare_with_c(&input, "50x 'ab' pattern");
}

#[test]
fn equiv_json_like() {
    compare_with_c(r#"{"key":"value","key2":"value2"}"#, "JSON-like");
}

#[test]
fn equiv_html_like() {
    compare_with_c("<html><body><p>test</p></body></html>", "HTML-like");
}

#[test]
fn equiv_url_like() {
    compare_with_c("https://example.com/path/to/resource?param=value", "URL-like");
}


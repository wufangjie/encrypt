extern crate encrypt;
use base64 as base64b;
use criterion::{criterion_group, criterion_main, Criterion};
use encrypt::base64 as base64m;
use encrypt::conv;
use openssl::base64 as base64o;
// use num_bigint::{BigInt, Sign};
// use num_traits::{One, Zero};
use std::fs;
use std::io;
use std::path::Path;

fn read_string(filename: impl AsRef<Path>) -> Result<String, io::Error> {
    fs::read_to_string(filename)
}

fn test_my_base64() {
    let m = read_string("src/aes.rs").unwrap_or_else(|_| "".to_string());
    let c = base64m::encode(m.as_bytes());
    let d = base64m::decode(&c);
    assert_eq!(m, conv::bytes_to_string(&d.unwrap()));
}

fn test_base64() {
    let m = read_string("src/aes.rs").unwrap_or_else(|_| "".to_string());
    let c = base64b::encode(m.as_bytes());
    let d = base64b::decode(c.as_bytes());
    assert_eq!(m, conv::bytes_to_string(&d.unwrap()));
}

fn test_openssl_base64() {
    let m = read_string("src/aes.rs").unwrap_or_else(|_| "".to_string());
    let c = base64o::encode_block(m.as_bytes());
    let d = base64o::decode_block(&c);
    assert_eq!(m, conv::bytes_to_string(&d.unwrap()));
}

// fn test_mine_crate() {
//     let m = read_string("src/aes.rs").unwrap_or_else(|_| "".to_string());
//     let c = Base64::encode(m.as_bytes());
//     let d = base64c::decode(&c);
//     assert_eq!(m, conv::bytes_to_string(&d.unwrap()));
// }

// fn test_crate_mine() {
//     let m = read_string("src/aes.rs").unwrap_or_else(|_| "".to_string());
//     let c = base64c::encode(m.as_bytes());
//     let d = Base64::decode(c.as_bytes());
//     assert_eq!(m, conv::bytes_to_string(&d.unwrap()));
// }

pub fn criterion_benchmark_my_base64(c: &mut Criterion) {
    c.bench_function("mine", |b| b.iter(test_my_base64));
}

pub fn criterion_benchmark_base64(c: &mut Criterion) {
    c.bench_function("crate", |b| b.iter(test_base64));
}

pub fn criterion_benchmark_openssl_base64(c: &mut Criterion) {
    c.bench_function("crate", |b| b.iter(test_openssl_base64));
}

// pub fn criterion_benchmark_mine_crate(c: &mut Criterion) {
//     c.bench_function("mine + crate", |b| b.iter(test_mine_crate));
// }

// pub fn criterion_benchmark_crate_mine(c: &mut Criterion) {
//     c.bench_function("crate + mine", |b| b.iter(test_crate_mine));
// }

criterion_group!(
    benches,
    // criterion_benchmark_my_base64,
    criterion_benchmark_base64,
    criterion_benchmark_openssl_base64,
    // criterion_benchmark_mine_crate,
    // criterion_benchmark_crate_mine,
);
criterion_main!(benches);

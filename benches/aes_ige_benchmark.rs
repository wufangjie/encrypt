extern crate encrypt;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use encrypt::aes::{ByteSquare, AES};
use encrypt::conv::hex_to_bytes;
use openssl::aes::{aes_ige, AesKey};
use openssl::symm::Mode;
use std::fs;
use std::io;
use std::path::Path;

fn read_string(filename: impl AsRef<Path>) -> Result<String, io::Error> {
    fs::read_to_string(filename)
}

fn my_aes_ige() {
    let mut s = read_string("src/aes.rs").unwrap_or_else(|_| "".to_string());
    for _ in 0..8 {
        s.push_str(&s.clone());
    }
    let blocks = s.as_bytes();

    let iv1 = ByteSquare::from_col(
        &hex_to_bytes("6D656E74 6174696F 6E206F66 20494745".replace(' ', "")).unwrap(),
    );
    let iv2 = ByteSquare::from_col(
        &hex_to_bytes("206D6F64 6520666F 72204F70 656E5353".replace(' ', "")).unwrap(),
    );

    let key = [
        0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x6E, 0x20, 0x69, 0x6D, 0x70, 0x6C,
        0x65,
    ];

    let a = AES::new(&key);
    let cipher = a.encode_ige(blocks, iv1, iv2);
    let origin = a.decode_ige(&cipher, iv1, iv2);
    assert_eq!(origin, blocks);
}

fn openssl_aes_ige() {
    let mut s = read_string("src/aes.rs").unwrap_or_else(|_| "".to_string());
    for _ in 0..8 {
        s.push_str(&s.clone());
    }
    let blocks = s.as_bytes();

    let mut iv_all = hex_to_bytes(
        "6D656E74 6174696F 6E206F66 20494745 206D6F64 6520666F 72204F70 656E5353".replace(' ', ""),
    )
    .unwrap();

    let key = [
        0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x6E, 0x20, 0x69, 0x6D, 0x70, 0x6C,
        0x65,
    ];

    let aes_key = AesKey::new_encrypt(&key).unwrap();
    let n = blocks.len();
    let mut encrypted = vec![0; n];
    aes_ige(blocks, &mut encrypted, &aes_key, &mut iv_all, Mode::Encrypt);

    let aes_key = AesKey::new_decrypt(&key).unwrap();
    let mut iv_all = hex_to_bytes(
        "6D656E74 6174696F 6E206F66 20494745 206D6F64 6520666F 72204F70 656E5353".replace(' ', ""),
    )
    .unwrap(); // NOTE: need
    let mut decrypted = vec![0; n];
    aes_ige(
        &encrypted,
        &mut decrypted,
        &aes_key,
        &mut iv_all,
        Mode::Decrypt,
    );

    assert_eq!(decrypted, blocks);
}

pub fn criterion_benchmark_mine(c: &mut Criterion) {
    c.bench_function("mine", |b| b.iter(my_aes_ige));
}

pub fn criterion_benchmark_openssl(c: &mut Criterion) {
    c.bench_function("openssl", |b| b.iter(openssl_aes_ige));
}

criterion_group!(
    benches,
    criterion_benchmark_mine,
    criterion_benchmark_openssl,
);
criterion_main!(benches);

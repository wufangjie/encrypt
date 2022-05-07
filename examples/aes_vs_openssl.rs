extern crate encrypt;
use encrypt::aes::{ByteSquare, AES};
use encrypt::conv::hex_to_bytes;
use std::path::Path;
use std::io;
use std::fs;
use openssl::aes::{AesKey, aes_ige};
use openssl::symm::Mode;

fn read_string(filename: impl AsRef<Path>) -> Result<String, io::Error> {
    fs::read_to_string(filename)
}

fn main() {
    let mut s = read_string("src/aes.rs").unwrap_or("".to_string());
    let mut n = s.len();
    while n % 16 != 0 {
	s.push('!');
	n += 1;
    }

    // dbg!(s.as_bytes().len());
    let blocks = s.as_bytes();

    let iv1 = hex_to_bytes("6D656E74 6174696F 6E206F66 20494745".replace(' ', "")).unwrap();
    let iv2 = hex_to_bytes("206D6F64 6520666F 72204F70 656E5353".replace(' ', "")).unwrap();

    let mut iv_all = hex_to_bytes("6D656E74 6174696F 6E206F66 20494745 206D6F64 6520666F 72204F70 656E5353".replace(' ', "")).unwrap();

    let iv1 = ByteSquare::from_col(&iv1);
    let iv2 = ByteSquare::from_col(&iv2);

    let key = [0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x6E, 0x20, 0x69, 0x6D, 0x70, 0x6C, 0x65];


    let a = AES::new(&key);

    let timer = std::time::Instant::now();
    let cipher = a.encode_ige(blocks, (iv1, iv2));
    println!("my encode cost: {:?}", timer.elapsed());


    let aes_key = AesKey::new_encrypt(&key).unwrap();

    let timer = std::time::Instant::now();
    let mut encrypted = vec![0; n];
    aes_ige(&blocks, &mut encrypted, &aes_key, &mut iv_all, Mode::Encrypt);
    println!("openssl encode cost: {:?}", timer.elapsed());


    assert_eq!(cipher.to_vec(), encrypted);

    let timer = std::time::Instant::now();
    let origin = a.decode_ige(&encrypted, (iv1, iv2));
    println!("my decode cost: {:?}", timer.elapsed());

    let mut iv_all = hex_to_bytes("6D656E74 6174696F 6E206F66 20494745 206D6F64 6520666F 72204F70 656E5353".replace(' ', "")).unwrap();
    let timer = std::time::Instant::now();
    let mut decrypted = vec![0; n];
    aes_ige(&encrypted, &mut decrypted, &aes_key, &mut iv_all, Mode::Decrypt);
    println!("openssl cost: {:?}", timer.elapsed());

    assert_eq!(origin.to_vec(), blocks);
    dbg!(&decrypted[..10]);
    dbg!(&blocks[..10]);
}

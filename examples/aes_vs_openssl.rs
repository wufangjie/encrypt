extern crate encrypt;
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

fn main() {
    let timer_all = std::time::Instant::now();
    let mut s = read_string("src/aes.rs").unwrap_or("".to_string());
    for _ in 0..8 {
        s.push_str(&s.clone());
    }

    let n = s.len();
    dbg!(n);

    // dbg!(s.as_bytes().len());
    let blocks = s.as_bytes();

    let iv1 = ByteSquare::from_col(
        &hex_to_bytes("6D656E74 6174696F 6E206F66 20494745".replace(' ', "")).unwrap(),
    );
    let iv2 = ByteSquare::from_col(
        &hex_to_bytes("206D6F64 6520666F 72204F70 656E5353".replace(' ', "")).unwrap(),
    );

    let mut iv_all = hex_to_bytes(
        "6D656E74 6174696F 6E206F66 20494745 206D6F64 6520666F 72204F70 656E5353".replace(' ', ""),
    )
    .unwrap();

    let key = [
        0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x6E, 0x20, 0x69, 0x6D, 0x70, 0x6C,
        0x65,
    ];

    let a = AES::new(&key);
    let aes_key = AesKey::new_encrypt(&key).unwrap();

    let timer = std::time::Instant::now();
    let mut encrypted = vec![0; n];
    aes_ige(
        &blocks,
        &mut encrypted,
        &aes_key,
        &mut iv_all,
        Mode::Encrypt,
    );
    let t1 = timer.elapsed();

    let timer = std::time::Instant::now();
    let cipher = a.encode_ige(blocks, iv1, iv2);
    let t0 = timer.elapsed();

    println!("mine encode, cost {:?}", t0);
    println!("openssl encode, cost {:?}", t1);
    println!("mine / openssl, {:.2}", t0.as_secs_f32() / t1.as_secs_f32());

    assert_eq!(cipher.to_vec(), encrypted);

    let aes_key = AesKey::new_decrypt(&key).unwrap(); // NOTE: decrypt
    let mut iv_all = hex_to_bytes(
        "6D656E74 6174696F 6E206F66 20494745 206D6F64 6520666F 72204F70 656E5353".replace(' ', ""),
    )
    .unwrap();
    let timer = std::time::Instant::now();
    let mut decrypted = vec![0; n];
    aes_ige(
        &encrypted,
        &mut decrypted,
        &aes_key,
        &mut iv_all,
        Mode::Decrypt,
    );
    let t1 = timer.elapsed();

    let timer = std::time::Instant::now();
    let origin = a.decode_ige(&encrypted, iv1, iv2);
    let t0 = timer.elapsed();

    println!("mine decode cost: {:?}", t0);
    println!("openssl cost: {:?}", t1);
    println!("mine / openssl, {:.2}", t0.as_secs_f32() / t1.as_secs_f32());

    assert_eq!(origin.to_vec(), decrypted);
    // dbg!(&decrypted[..10]);
    // dbg!(&blocks[..10]);

    // dbg!(timer_all.elapsed());
}

// fn main() {
//     let mut s = read_string("src/aes.rs").unwrap_or("".to_string());
//     let mut n = s.len();
//     while n % 16 != 0 {
//         s.push('!');
//         n += 1;
//     }

//     // dbg!(s.as_bytes().len());
//     let blocks = s.as_bytes();

//     let iv1 = hex_to_bytes("6D656E74 6174696F 6E206F66 20494745".replace(' ', "")).unwrap();
//     let iv2 = hex_to_bytes("206D6F64 6520666F 72204F70 656E5353".replace(' ', "")).unwrap();

//     let mut iv_all = hex_to_bytes(
//         "6D656E74 6174696F 6E206F66 20494745 206D6F64 6520666F 72204F70 656E5353".replace(' ', ""),
//     )
//     .unwrap();

//     let key = [
//         0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x6E, 0x20, 0x69, 0x6D, 0x70, 0x6C,
//         0x65,
//     ];

//     let a = AES::new(&key);

//     let timer = std::time::Instant::now();
//     let cipher = a.encode_ige(blocks, &mut iv1.clone(), &mut iv2.clone());
//     println!("my encode cost: {:?}", timer.elapsed());

//     let aes_key = AesKey::new_encrypt(&key).unwrap();

//     let timer = std::time::Instant::now();
//     let mut encrypted = vec![0; n];
//     aes_ige(
//         &blocks,
//         &mut encrypted,
//         &aes_key,
//         &mut iv_all,
//         Mode::Encrypt,
//     );
//     println!("openssl encode cost: {:?}", timer.elapsed());

//     assert_eq!(cipher.to_vec(), encrypted);

//     let iv1 = hex_to_bytes("6D656E74 6174696F 6E206F66 20494745".replace(' ', "")).unwrap();
//     let iv2 = hex_to_bytes("206D6F64 6520666F 72204F70 656E5353".replace(' ', "")).unwrap();

//     let timer = std::time::Instant::now();
//     let origin = a.decode_ige(&encrypted, &mut iv1.clone(), &mut iv2.clone());
//     println!("my decode cost: {:?}", timer.elapsed());

//     let mut iv_all = hex_to_bytes(
//         "6D656E74 6174696F 6E206F66 20494745 206D6F64 6520666F 72204F70 656E5353".replace(' ', ""),
//     )
//     .unwrap();
//     let timer = std::time::Instant::now();
//     let mut decrypted = vec![0; n];
//     aes_ige(
//         &encrypted,
//         &mut decrypted,
//         &aes_key,
//         &mut iv_all,
//         Mode::Decrypt,
//     );
//     println!("openssl cost: {:?}", timer.elapsed());

//     assert_eq!(origin.to_vec(), blocks);
//     dbg!(&decrypted[..10]);
//     dbg!(&blocks[..10]);
// }

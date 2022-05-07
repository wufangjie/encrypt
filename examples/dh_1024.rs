extern crate encrypt;

use encrypt::conv::bytes_to_hex_lower;
use num_bigint::BigUint;
use num_primes::{Generator, Verification};
use num_traits::{One, ToPrimitive}; // , Zero

fn main() {
    // let a = Generator::new_prime(2048);
    // let b = Generator::new_uint(2048);
    // let p = Generator::safe_prime(2048);
    // // 2048 位算不出来, 用 1024 代替

    let p = gen_dh_prime(); // dh_prime
    dbg!(Verification::is_prime(&p));
    dbg!();

    let one: BigUint = One::one();

    // re-keying a, b after about 100 messages (forward secrecy)
    let a = Generator::new_uint(1024).modpow(&one, &p);
    let b = Generator::new_uint(1024).modpow(&one, &p);

    // for python
    dbg!(bytes_to_hex_lower(&p.to_bytes_be()));
    dbg!(bytes_to_hex_lower(&a.to_bytes_be()));
    dbg!(bytes_to_hex_lower(&b.to_bytes_be()));

    for g in get_possible_generator(&p) {
        println!("g = {}", g);
        let g = BigUint::from(g);

        let timer = std::time::Instant::now();
        let ab = g.modpow(&a, &p).modpow(&b, &p);
        println!("cost: {:?}", timer.elapsed());

        assert_eq!(ab, g.modpow(&b, &p).modpow(&a, &p));
    }
}

fn get_possible_generator(p: &BigUint) -> Vec<u8> {
    let one: BigUint = One::one();
    let eight: BigUint = &one << 3usize;
    let three: BigUint = &one + &one + &one;
    let five: BigUint = (&one << 2) + &one;
    let seven: BigUint = &five + (&one << 1);

    // check if 2, 3, 5, 6, 7 is a generator
    let mut res = vec![];
    if p.modpow(&one, &eight) == BigUint::from(7u8) {
        res.push(2);
    }
    if p.modpow(&one, &three) == BigUint::from(2u8) {
        res.push(3);
    }
    res.push(4);
    match p.modpow(&one, &five).to_u8().unwrap() {
        1u8 | 4u8 => res.push(5),
        _ => (),
    }
    match p.modpow(&one, &(&eight + &eight + &eight)).to_u8().unwrap() {
        19u8 | 23u8 => res.push(6),
        _ => (),
    }
    match p.modpow(&one, &seven).to_u8().unwrap() {
        3u8 | 5u8 | 6u8 => res.push(7),
        _ => (),
    }
    res
}

fn gen_dh_prime() -> BigUint {
    let p: Vec<u32> = vec![
        1264733995, 3077687274, 356252747, 1503923254, 1009902259, 1171092640, 3801468486,
        2851061519, 3194537618, 222468113, 2460969622, 4048386986, 4160824348, 125984898,
        3284192299, 2425033075, 2505177613, 3476823095, 4058689276, 1388940990, 1059342871,
        1205621453, 2781718971, 366659147, 1499645104, 2565302923, 3239117366, 3020727608,
        2993889032, 2148249226, 515284369, 4021485001,
    ];
    let mut bytes: Vec<u8> = vec![];

    // 整体的顺序是反的, 但 u32 的顺序是正的
    for v in p.into_iter().rev() {
        bytes.extend(v.to_be_bytes());
    }
    BigUint::from_bytes_be(&bytes)
}

// padding with 12 to 1024 random padding bytes to make its length divisible by 16 bytes. (In the older MTProto 1.0 encryption, only 0 to 15 padding bytes were used.)
// msg_key, is computed as the 128 middle bits of the SHA256

// key 就是 dh 最后算出的结果
// msg_key_large = SHA256 (substr (key, 88+x, 32) + plaintext + random_padding);
// msg_key = substr (msg_key_large, 8, 16);
// sha256_a = SHA256 (msg_key + substr (key, x, 36));
// sha256_b = SHA256 (substr (key, 40+x, 36) + msg_key);
// aes_key = substr (sha256_a, 0, 8) + substr (sha256_b, 8, 16) + substr (sha256_a, 24, 8);
// aes_iv = substr (sha256_b, 0, 8) + substr (sha256_a, 8, 16) + substr (sha256_b, 24, 8);

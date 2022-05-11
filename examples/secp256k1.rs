extern crate encrypt;
use encrypt::ecc::{Ec, Point, i_from_hex4, u_from_hex4};
use num_bigint::{BigInt, ToBigInt}; // BigUint, Sign,
use num_primes::Generator;
//use num_integer::{ExtendedGcd}; // It's slow
use num_traits::{Zero}; // One,

fn main() {
    let timer = std::time::Instant::now();
    let p_str = "FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE FFFFFC2F";
    let x_str = "79BE667E F9DCBBAC 55A06295 CE870B07 029BFCDB 2DCE28D9 59F2815B 16F81798";
    let y_str = "483ADA77 26A3C465 5DA4FBFC 0E1108A8 FD17B448 A6855419 9C47D08F FB10D4B8";
    let n_str = "FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141"; // the order
    let g = Point::new(i_from_hex4(x_str), i_from_hex4(y_str));
    let p = u_from_hex4(p_str);
    let n = u_from_hex4(n_str);
    let ec = Ec::new(p, BigInt::zero(), BigInt::from(7u8), n, g);
    println!("gen checked secp256k1 () cost: {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    let ec = Ec::secp256k1();
    println!("gen unchecked secp256k1 cost: {:?}", timer.elapsed());


    let (pri_key, pub_key) = ec.gen_key();
    let hash_m = Generator::new_uint(256).to_bigint().unwrap();

    let timer = std::time::Instant::now();
    let (s, pr) = ec.sig_gen(&hash_m, &pri_key);
    println!("sig_gen() cost: {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    dbg!(ec.sig_ver(&hash_m, &pub_key, s, pr));
    println!("sig_ver() cost: {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    let (pri_key1, pub_key1) = ec.gen_key();
    let (pri_key2, pub_key2) = ec.gen_key();
    assert_eq!(ec.mul(&pri_key1, &pub_key2), ec.mul(&pri_key2, &pub_key1));
    println!("twice ecdh cost: {:?}", timer.elapsed());
}

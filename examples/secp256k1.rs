extern crate encrypt;
use encrypt::conv::hex_to_bytes;
use encrypt::ecc_bigint::{Point, ECC};
use num_bigint::BigUint;
use num_traits::{One, Zero};

fn from_format_hex4(s: &str) -> BigUint {
    BigUint::from_bytes_be(&hex_to_bytes(s.replace(' ', "")).unwrap())
}

/// secp256k1 is the bitcoin
fn main() {
    // https://en.bitcoin.it/wiki/Secp256k1
    let p_str = "FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE FFFFFC2F";
    let x_str = "79BE667E F9DCBBAC 55A06295 CE870B07 029BFCDB 2DCE28D9 59F2815B 16F81798";
    let y_str = "483ADA77 26A3C465 5DA4FBFC 0E1108A8 FD17B448 A6855419 9C47D08F FB10D4B8";
    let n_str = "FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141"; // order

    let p = from_format_hex4(p_str);
    let ec = ECC::new(p, BigUint::zero(), BigUint::from(7u8));

    let g = Point::new(from_format_hex4(x_str), from_format_hex4(y_str));
    assert!(ec.contains(&g));

    let n = from_format_hex4(n_str);

    let timer = std::time::Instant::now();
    assert!(ec.mul(&n, &g).is_zero()); // 188ms on macbook air m1
    println!("cost {:?}", timer.elapsed());

    let pri = from_format_hex4("1E99423A4ED27608A15A2616A2B0E9E52CED330AC530EDCC32C8FFC6A526AEDD");
    let pub_x =
        from_format_hex4("F028892BAD7ED57D2FB57BF33081D5CFCF6F9ED3D3D7F159C2E2FFF579DC341A");
    let pub_y =
        from_format_hex4("07CF33DA18BD734C600B96A72BBC4749D5141C90EC8AC328AE52DDFE2E505BDB");

    let timer = std::time::Instant::now();
    let key = ec.mul(&pri, &g); // 133ms on macbook air m1
    println!("cost {:?}", timer.elapsed());

    assert_eq!(key, Point::new(pub_x, pub_y));
}

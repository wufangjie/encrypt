use crate::conv::hex_to_bytes;
use num_bigint::{BigInt, BigUint, Sign, ToBigInt};
use num_integer::Integer;
use num_primes::{Generator, Verification};
use num_traits::{One, Zero}; // ,
use std::ops::Deref;

/// 有限域上的点, 负数表示无穷远点
#[derive(PartialEq, Clone, Debug)]
pub struct Point {
    x: BigInt,
    y: BigInt,
}

impl Point {
    pub fn new(x: BigInt, y: BigInt) -> Self {
        Self { x, y }
    }

    pub fn new_i32(x: i32, y: i32) -> Self {
        Self::new(BigInt::from(x), BigInt::from(y))
    }

    /// generate zero (identity) element
    pub fn zero() -> Self {
        Self {
            x: BigInt::from(-1i8),
            y: BigInt::from(-1i8),
        }
    }

    /// is identity element or not
    pub fn is_zero(&self) -> bool {
        // 零元 (无穷远点), 用负数来表示
        self.x.sign() == Sign::Minus || self.y.sign() == Sign::Minus
    }
}

#[inline]
pub fn mod_p(mut x: BigInt, p: &BigInt) -> BigInt {
    // NOTE: we always need p > 0
    x %= p;
    if let Sign::Minus = x.sign() {
        x += p;
    }
    x
}

pub fn calc_inv(mut x: BigInt, p: &BigInt) -> Option<BigInt> {
    // NOTE: we always need p > 0
    // In most cases, we will never use x again
    match x.sign() {
        Sign::NoSign => return None,
        Sign::Plus => x %= p,
        Sign::Minus => {
            x %= p;
            x += p
        }
    };
    let mut y = p.clone();
    let mut k1 = BigInt::zero();
    let mut k2 = BigInt::one();
    loop {
        if x.is_zero() {
            return None;
        } else if x.is_one() {
            return Some(mod_p(k2, p));
        } else {
            let k = &y / &x;
            std::mem::swap(&mut x, &mut y);
            x -= &k * &y; // non-negative
            std::mem::swap(&mut k1, &mut k2);
            k2 -= &k * &k1;
            // NOTE: keep k2 non-negative is slower than only do it when return
        }
    }
}

pub struct EcBase {
    p: BigInt,
    a: BigInt,
    b: BigInt,
}

impl EcBase {
    pub fn new(p: BigUint, a: BigInt, b: BigInt) -> Self {
        // assert p is prime
        assert!(Verification::is_prime(&p));
        Self {
            p: p.to_bigint().unwrap(),
            a,
            b,
        }
    }

    pub fn new_unchecked(p: BigInt, a: BigInt, b: BigInt) -> Self {
        Self { p, a, b }
    }

    pub fn calc_inv(&self, x: BigInt) -> Option<BigInt> {
        calc_inv(x, &self.p)
    }

    fn calc_lambda(&self, p1: &Point, p2: &Point) -> Option<BigInt> {
        if p1 != p2 {
            if p1.x == p2.x {
                None
            } else {
                Some((&p2.y - &p1.y) * calc_inv(&p2.x - &p1.x, &self.p).unwrap())
            }
        } else if p1.y.is_zero() {
            None
        } else {
            Some((3 * &p1.x * &p1.x + &self.a) * calc_inv(&p1.y + &p1.y, &self.p).unwrap())
        }
    }

    pub fn add(&self, p1: &Point, p2: &Point) -> Point {
        if p1.is_zero() {
            p2.clone()
        } else if p2.is_zero() {
            p1.clone()
        } else {
            match self.calc_lambda(p1, p2) {
                None => Point::zero(),
                Some(lambda) => {
                    // NOTE: x * x % p >> x.modpow(2, p) >>> logn +
                    let x3 = mod_p(&lambda * &lambda - &p1.x - &p2.x, &self.p);
                    let y3 = mod_p(&lambda * (&p1.x - &x3) - &p1.y, &self.p);
                    Point::new(x3, y3)
                }
            }
        }
    }

    pub fn sub(&self, p1: &Point, p2: &Point) -> Point {
        if p2.is_zero() {
            p1.clone()
        } else {
            let p3 = Point::new(p2.x.clone(), &self.p - &p2.y);
            if p1.is_zero() {
                p3
            } else {
                self.add(p1, &p3)
            }
        }
    }

    pub fn mul(&self, k: &BigInt, p: &Point) -> Point {
        // assert!(k.sign() != Sign::Minus);
        if k.is_zero() {
            Point::zero()
        } else {
            let mut k = k.clone();
            let mut p = p.clone();
            let mut res = Point::zero();
            loop {
                if k.is_one() {
                    return self.add(&res, &p);
                } else if k.is_odd() {
                    k -= 1u8;
                    res = self.add(&res, &p);
                } else {
                    k >>= 1;
                    p = self.add(&p, &p);
                }
            }
        }
    }

    pub fn contains(&self, p: &Point) -> bool {
        p.is_zero()
            || ((&p.x * &p.x * &p.x + &self.a * &p.x + &self.b - &p.y * &p.y) % (&self.p)).is_zero()
    }
}

pub fn i_from_hex4(s: &str) -> BigInt {
    BigInt::from_bytes_be(Sign::Plus, &hex_to_bytes(s.replace(' ', "")).unwrap())
}

pub fn u_from_hex4(s: &str) -> BigUint {
    BigUint::from_bytes_be(&hex_to_bytes(s.replace(' ', "")).unwrap())
}

pub struct Ec {
    // p: BigInt,
    // a: BigInt,
    // b: BigInt,
    ecb: EcBase,
    n: BigInt, // order
    g: Point,
}

impl Ec {
    pub fn new(p: BigUint, a: BigInt, b: BigInt, n: BigUint, g: Point) -> Self {
        assert!(Verification::is_prime(&n));
        let ecb = EcBase::new(p, a, b);
        assert!(ecb.contains(&g));
        let n = n.to_bigint().unwrap();
        assert!(ecb.mul(&n, &g).is_zero());
        Self { ecb, n, g }
    }

    pub fn new_unchecked(p: BigInt, a: BigInt, b: BigInt, n: BigInt, g: Point) -> Self {
        let ecb = EcBase::new_unchecked(p, a, b);
        Self { ecb, n, g }
    }

    pub fn gen_pri_key(&self) -> BigInt {
        loop {
            let pri_key = Generator::new_uint(self.n.bits()).to_bigint().unwrap() % &self.n;
            if !pri_key.is_zero() {
                return pri_key;
            }
        }
    }

    pub fn gen_key(&self) -> (BigInt, Point) {
        let pri_key = self.gen_pri_key();
        let pub_key = self.mul(&pri_key, &self.g);
        (pri_key, pub_key)
    }

    pub fn sig_gen(&self, hash_m: &BigInt, pri_key: &BigInt) -> (BigInt, Point) {
        loop {
            let r = self.gen_pri_key();
            let p = self.mul(&r, &self.g);
            let xr = &p.x % &self.n;
            if xr.is_zero() {
                continue;
            }
            let s = calc_inv(r, &self.n).unwrap() * (hash_m + xr * pri_key);
            if s.is_zero() {
                continue;
            }
            return (s, p);
        }
    }

    pub fn sig_ver(&self, hash_m: &BigInt, pub_key: &Point, s: BigInt, pr: Point) -> bool {
        let s_inv = calc_inv(s, &self.n).unwrap();
        let pv = self.add(
            &self.mul(&(hash_m * &s_inv), &self.g),
            &self.mul(&(&pr.x * s_inv), pub_key),
        );
        pv == pr
    }
}

impl Ec {
    pub fn secp256k1() -> Self {
        let p_str = "FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE FFFFFC2F";
        let x_str = "79BE667E F9DCBBAC 55A06295 CE870B07 029BFCDB 2DCE28D9 59F2815B 16F81798";
        let y_str = "483ADA77 26A3C465 5DA4FBFC 0E1108A8 FD17B448 A6855419 9C47D08F FB10D4B8";
        let n_str = "FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141"; // the order
        let g = Point::new(i_from_hex4(x_str), i_from_hex4(y_str));

        // let p = u_from_hex4(p_str);
        // let n = u_from_hex4(n_str);
        // Ec::new(p, BigInt::zero(), BigInt::from(7u8), n, g);

        let p = i_from_hex4(p_str);
        let n = i_from_hex4(n_str);
        Ec::new_unchecked(p, BigInt::zero(), BigInt::from(7u8), n, g)
    }
}

impl Deref for Ec {
    type Target = EcBase;

    fn deref(&self) -> &Self::Target {
        &self.ecb
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_inv() {
        // dbg!(BigInt::from(-8i8) % BigInt::from(23u8));
        // rust's normal rem % -8

        dbg!(BigInt::from(0u8).sign());

        let p = BigInt::from(23u8);
        assert_eq!(
            BigInt::from(15u8),
            calc_inv(BigInt::from(20u8), &p).unwrap()
        );
        assert_eq!(
            BigInt::from(17u8),
            calc_inv(BigInt::from(19u8), &p).unwrap()
        );
    }

    #[test]
    fn test_pub_key_gen() {
        let ec = Ec::secp256k1();
        let pri_key =
            i_from_hex4("1E99423A4ED27608A15A2616A2B0E9E52CED330AC530EDCC32C8FFC6A526AEDD");
        let pub_x = i_from_hex4("F028892BAD7ED57D2FB57BF33081D5CFCF6F9ED3D3D7F159C2E2FFF579DC341A");
        let pub_y = i_from_hex4("07CF33DA18BD734C600B96A72BBC4749D5141C90EC8AC328AE52DDFE2E505BDB");
        let pub_key = ec.mul(&pri_key, &ec.g);
        assert_eq!(pub_key, Point::new(pub_x, pub_y));
    }

    #[test]
    fn test_ecdsa() {
        let ec = Ec::secp256k1();
        let (pri_key, pub_key) = ec.gen_key();
        let hash_m = Generator::new_uint(256).to_bigint().unwrap();
        let (s, pr) = ec.sig_gen(&hash_m, &pri_key);
        dbg!(ec.sig_ver(&hash_m, &pub_key, s, pr));
    }

    #[test]
    fn test_ecdh() {
        let ec = Ec::secp256k1();
        let (pri_key1, pub_key1) = ec.gen_key();
        let (pri_key2, pub_key2) = ec.gen_key();
        assert_eq!(ec.mul(&pri_key1, &pub_key2), ec.mul(&pri_key2, &pub_key1));
    }
}

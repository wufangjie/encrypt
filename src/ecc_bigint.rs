use num_bigint::BigUint;
use num_integer::Integer;
use num_primes::Verification;
use num_traits::{One, Zero}; // , ToPrimitive

#[derive(PartialEq, Clone, Debug)]
pub struct Point {
    x: BigUint,
    y: BigUint,
}

impl Point {
    pub fn new(x: BigUint, y: BigUint) -> Self {
        Self { x, y }
    }

    pub fn new_u32(x: u32, y: u32) -> Self {
        // TODO: find a more elegant way
        Self {
            x: BigUint::from_bytes_be(&x.to_be_bytes()),
            y: BigUint::from_bytes_be(&y.to_be_bytes()),
        }
    }

    pub fn zero() -> Self {
        // TODO: FIXME: 25519 它的 (0, 0) 是曲线上的一点
        Self {
            x: Zero::zero(),
            y: Zero::zero(),
        }
    }

    pub fn is_zero(&self) -> bool {
        // 零元 (无穷远点)
        self.x.is_zero() && self.y.is_zero()
    }
}

/// galois field (prime)
struct GFP {
    p: BigUint,
}

impl GFP {
    pub fn new(p: BigUint) -> Self {
        assert!(Verification::is_prime(&p));
        Self { p }
    }

    pub fn add(&self, x: &BigUint, y: &BigUint) -> BigUint {
        (x + y) % (&self.p)
    }

    pub fn sub(&self, x: &BigUint, y: &BigUint) -> BigUint {
        self.add(x, &self.calc_neg(y))
    }

    pub fn mul(&self, x: &BigUint, y: &BigUint) -> BigUint {
        if x.is_zero() || y.is_zero() {
            Zero::zero()
        } else {
            let mut x = x.clone();
            let mut y = y.clone();
            let mut res = Zero::zero();
            loop {
                if x.is_one() {
                    return self.add(&res, &y);
                } else if x.is_odd() {
                    x -= 1u8;
                    res = self.add(&res, &y);
                } else {
                    x >>= 1;
                    y = self.add(&y, &y);
                }
            }
        }
    }

    pub fn div(&self, x: &BigUint, y: &BigUint) -> BigUint {
        self.mul(x, &self.calc_inv(y))
    }

    pub fn calc_neg(&self, x: &BigUint) -> BigUint {
        // &self.p - x % (&self.p)
        &self.p - x // 0 <= x < p
    }

    pub fn calc_inv(&self, x: &BigUint) -> BigUint {
        // NOTE: x != 0

        let mut k1 = BigUint::zero();
        let mut k2 = BigUint::one();
        let mut x = (*x).clone();
        let mut y = self.p.clone();
        loop {
            if x.is_zero() {
                panic!("No inverse!")
            } else if x.is_one() {
                return k2;
            } else {
                let k = &y / &x;
                // let x_bak = x.clone();
                // x = y - &k * x;
                // y = x_bak;

                // let k2_bak = k2.clone();
                // k2 = self.sub(&k1, &self.mul(&k, &k2));
                // k1 = k2_bak;

                (x, y) = (y - &k * &x, x); // no mod(p) is needed
                (k2, k1) = (self.sub(&k1, &self.mul(&k, &k2)), k2);
            }
        }
    }
}

pub struct ECC {
    fp: GFP,
    a: BigUint,
    b: BigUint,
}

impl ECC {
    pub fn new(p: BigUint, a: BigUint, b: BigUint) -> Self {
        let fp = GFP::new(p);
        ECC { fp, a, b }
    }

    fn calc_lambda(&self, p1: &Point, p2: &Point) -> Option<BigUint> {
        if p1 != p2 {
            if p1.x == p2.x {
                None
            } else {
                Some(
                    self.fp
                        .div(&self.fp.sub(&p2.y, &p1.y), &self.fp.sub(&p2.x, &p1.x)),
                )
            }
        } else {
            // NOTE: here we always have p1.y != 0, if p1.y = 0, then p1 == p2
            if p1.y.is_zero() {
                None
            } else {
                Some(self.fp.div(
                    &self.fp.add(
                        &self.fp.mul(&self.fp.mul(&BigUint::from(3u8), &p1.x), &p1.x),
                        &self.a,
                    ),
                    &self.fp.mul(&BigUint::from(2u8), &p1.y),
                ))
            }
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
                    let x3 = self
                        .fp
                        .sub(&self.fp.sub(&self.fp.mul(&lambda, &lambda), &p1.x), &p2.x);
                    let y3 = self
                        .fp
                        .sub(&self.fp.mul(&lambda, &self.fp.sub(&p1.x, &x3)), &p1.y);
                    Point::new(x3, y3)
                }
            }
        }
    }

    pub fn sub(&self, p1: &Point, p2: &Point) -> Point {
        let mut p3 = p2.clone();
        p3.y = self.fp.calc_neg(&p3.y);
        self.add(p1, &p3)
    }

    pub fn mul(&self, k: &BigUint, p: &Point) -> Point {
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
            || self
                .fp
                .sub(
                    &self.fp.sub(
                        &self.fp.sub(
                            &self.fp.mul(&p.y, &p.y),
                            &self.fp.mul(&self.fp.mul(&p.x, &p.x), &p.x),
                        ),
                        &self.fp.mul(&self.a, &p.x),
                    ),
                    &self.b,
                )
                .is_zero()
    }
}

#[test]
fn test_gfp() {
    let fp = GFP::new(BigUint::from(23u8));
    assert_eq!(BigUint::from(15u8), fp.calc_inv(&BigUint::from(20u8)));
    assert_eq!(BigUint::from(17u8), fp.calc_inv(&BigUint::from(19u8)));
}

#[test]
fn test_ecc() {
    let ec = ECC::new(BigUint::from(23u8), BigUint::from(1u8), BigUint::from(1u8));

    assert_eq!(
        Point::new_u32(17, 20),
        ec.add(&Point::new_u32(3, 10), &Point::new_u32(9, 7))
    );

    for i in 2..10u8 {
        let i = BigUint::from_bytes_be(&i.to_be_bytes());
        let p = ec.mul(&i, &Point::new_u32(3, 10));
        assert!(ec.contains(&p));
        assert_eq!(p, ec.mul(&i, &Point::new_u32(3, 10)));
    }

    assert_eq!(
        Point::new_u32(4, 0),
        ec.mul(&BigUint::from(14u8), &Point::new_u32(3, 10))
    );
    assert_eq!(
        Point::new_u32(3, 13),
        ec.mul(&BigUint::from(27u8), &Point::new_u32(3, 10))
    );
    assert!(ec
        .mul(&BigUint::from(28u8), &Point::new_u32(3, 10))
        .is_zero());
    assert_eq!(
        Point::new_u32(3, 10),
        ec.mul(&BigUint::from(29u8), &Point::new_u32(3, 10))
    );
    assert!(ec
        .sub(&Point::new_u32(3, 10), &Point::new_u32(3, 10))
        .is_zero());
    assert_eq!(
        Point::new_u32(7, 12),
        ec.sub(&Point::new_u32(3, 10), &Point::new_u32(3, 13))
    );

    // the order of EC(23, 1, 1) is 28, no matter the G is
    for i in 1..28u8 {
        let i = BigUint::from_bytes_be(&i.to_be_bytes());
        let ord = BigUint::from(28u8);
        assert!(ec.mul(&ord, &ec.mul(&i, &Point::new_u32(3, 10))).is_zero());
    }
}

#[test]
fn test_elgamal() {}

#[test]
fn test_ecdsa() {}

// TODO: add bigint test example (test speed)

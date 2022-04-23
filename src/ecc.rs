use std::fmt;

type T = i64;

#[derive(PartialEq, Clone, Debug)]
pub struct Point {
    x: T,
    y: T,
}

impl Point {
    pub fn new(x: T, y: T) -> Self {
        Point { x, y }
    }

    pub fn is_unit(&self) -> bool {
        // NOTE: use (0, 0) to present inf point
        self.x == 0 && self.y == 0
    }

    pub fn copy(&self) -> Self {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    // pub fn get_negative(&self) -> Self {
    //     Point {
    //         x: self.x,
    //         y: -self.y,
    //     }
    // }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Point({}, {})", self.x, self.y)
    }
}

pub struct ECC {
    p: T, // prime
    a: T,
    b: T,
}

impl ECC {
    pub fn new(p: T, a: T, b: T) -> Self {
        if p < 2 {
            panic!("p must be a prime!");
        }
        ECC { p, a, b }
    }

    pub fn add_p1_p2(&self, p1: &Point, p2: &Point) -> Point {
        if p1.is_unit() {
            p2.copy()
        } else if p2.is_unit() {
            p1.copy()
        } else {
            match self.calc_lambda(p1, p2) {
                None => Point::new(0, 0),
                Some(lam) => {
                    let x3 = self.modulo(lam * lam - p1.x - p2.x);
                    let y3 = self.modulo(lam * (p1.x - x3) - p1.y);
                    Point::new(x3, y3)
                }
            }
        }
    }

    pub fn sub_p1_p2(&self, p1: &Point, p2: &Point) -> Point {
        self.add_p1_p2(p1, &self.get_negative_point(p2))
    }

    pub fn mul_k_p_1by1(&self, k: T, p1: &Point) -> Point {
        let mut p2 = (*p1).clone();
        for _ in 1..k {
            p2 = self.add_p1_p2(&p1, &p2);
        }
        p2
    }

    pub fn mul_k_p_logn(&self, k: T, p1: &Point) -> Point {
        if k < 0 {
            self.mul_k_p_logn(-k, &self.get_negative_point(p1))
        } else if k == 1 {
            (*p1).clone()
        } else if k & 1 == 1 {
            self.add_p1_p2(&p1, &self.mul_k_p_logn(k - 1, &p1))
        } else {
            let p2 = self.mul_k_p_logn(k >> 1, &p1);
            self.add_p1_p2(&p2, &p2)
        }
    }

    pub fn contains(&self, p: &Point) -> bool {
        p.is_unit() || (p.x * p.x * p.x + self.a * p.x + self.b - p.y * p.y) % self.p == 0
    }

    pub fn find_order(&self, p: &Point) -> T {
        if self.contains(p) {
            let mut cur = Point::new(0, 0);
            let mut count = 0;
            loop {
                count += 1;
                cur = self.add_p1_p2(p, &cur);
                if cur.is_unit() {
                    return count;
                }
            }
        }
        0
    }

    fn get_negative_point(&self, p1: &Point) -> Point {
        Point::new(p1.x, self.calc_negative(p1.y))
    }

    fn calc_lambda(&self, p1: &Point, p2: &Point) -> Option<T> {
        // NOTE: p1 != -p2
        let (num, den) = if p1 != p2 {
            if p1.x == p2.x {
                return None;
                //panic!("{} == -{}", p1, p2);
            }
            (p2.y - p1.y, p2.x - p1.x)
        } else {
            if p1.y == 0 {
                return None; // NOTE: unreachable
                //panic!("{} == -{}", p1, p1);
            }
            (3 * p1.x * p1.x + self.a, 2 * p1.y)
        };
        Some(self.modulo(self.modulo(num) * self.calc_inverse_gcd_tail(self.modulo(den))))
    }

    fn calc_inverse(&self, x: T) -> T {
        if x != 0 {
            let mut y = x;
            for i in 1..self.p {
                if y % self.p == 1 {
                    return i;
                }
                y += x
            }
        }
        panic!("No inverse!");
    }

    fn calc_inverse_gcd(&self, x: T) -> T {
        fn rec(y: T, x: T, k: T) -> (T, T) {
            if x == 1 {
                (0, 1) //(1, -k)
            } else {
                let k = y / x;
                let (a, b) = rec(x, y - k * x, k);
                (b, a - k * b)
            }
        }
        self.modulo(rec(x, self.p, 1).0)
    }

    fn calc_inverse_gcd_tail(&self, x: T) -> T {
        fn rec(y: T, x: T, a: T, b: T) -> T {
            // one pre = how many x, one cur = how many x (tail recursive)
            if x == 1 {
                b
            } else {
                let k = y / x;
                rec(x, y - k * x, b, a - k * b)
            }
        }
        self.modulo(rec(x, self.p, 1, 0))
    }

    fn calc_negative(&self, x: T) -> T {
        self.modulo(-x)
    }

    fn modulo(&self, x: T) -> T {
        match x % self.p {
            y if y >= 0 => y,
            y => y + self.p, // NOTE: we have self.p > 0
        }
    }

    fn gcd(x: T, y: T) -> T {
        if y == 0 {
            x
        } else {
            Self::gcd(y, x % y)
        }
    }
}

#[test]
fn test_ecc() {
    let ec = ECC::new(23, 1, 1);
    assert_eq!(7, ECC::gcd(49, 14));
    assert_eq!(1, ECC::gcd(49, 13));
    assert_eq!(7, ECC::gcd(49, 7));

    assert_eq!(1, ec.modulo(47));
    assert_eq!(22, ec.modulo(-47));
    assert_eq!(1, ec.calc_negative(-47));
    assert_eq!(15, ec.calc_inverse(20));

    assert_eq!(
        Point::new(17, 20),
        ec.add_p1_p2(&Point::new(3, 10), &Point::new(9, 7))
    );

    for i in 2..10 {
        let p = ec.mul_k_p_1by1(i, &Point::new(3, 10));
        assert!(ec.contains(&p));
        assert_eq!(p, ec.mul_k_p_logn(i, &Point::new(3, 10)));
    }

    assert_eq!(Point::new(4, 0), ec.mul_k_p_logn(14, &Point::new(3, 10)));
    assert_eq!(Point::new(3, 13), ec.mul_k_p_logn(27, &Point::new(3, 10)));
    assert!(ec.mul_k_p_logn(28, &Point::new(3, 10)).is_unit());
    assert_eq!(Point::new(3, 10), ec.mul_k_p_logn(29, &Point::new(3, 10)));
    assert!(ec
        .sub_p1_p2(&Point::new(3, 10), &Point::new(3, 10))
        .is_unit());
    assert_eq!(
        Point::new(7, 12),
        ec.sub_p1_p2(&Point::new(3, 10), &Point::new(3, 13))
    );

    // the order of EC(23, 1, 1) is 28, no matter the G is
    for i in 1..28 {
        assert!(ec
            .mul_k_p_logn(28, &ec.mul_k_p_logn(i, &Point::new(3, 10)))
            .is_unit());
    }
}

#[test]
#[ignore]
fn test_mod_and_rem() {
    let f = |x: i32, y: i32| match x % y {
        z if z > 0 => z,
        z => z + y.abs(),
    };
    dbg!(f(-1, -7));
    dbg!(f(-13, -7));
    dbg!(f(1, -7));
    dbg!(f(13, -7));
    dbg!(f(-1, 7));
    dbg!(f(-13, 7));
    dbg!(f(1, 7));
    dbg!(f(13, 7));

    dbg!((-1) % (-7));
    dbg!((-13) % (-7));
    dbg!((1) % (-7));
    dbg!((13) % (-7));
    dbg!((-1) % (7));
    dbg!((-13) % (7));
    dbg!((1) % (7));
    dbg!((13) % (7));
}

#[test]
fn test_elgamal_small() {
    let ec = ECC::new(11, 1, 6);
    let k = 7;
    let g = Point::new(2, 7);
    let p = ec.mul_k_p_logn(k, &g);

    assert_ne!(p, ec.mul_k_p_logn(18, &g)); // NOTE: we can not modulo k

    // -k * G = -1 * k * G = k * (-1) * G
    dbg!(ec.mul_k_p_logn(3, &Point::new(2, 7)));
    dbg!(ec.mul_k_p_logn(3, &Point::new(2, 4)));

    let m = Point::new(10, 9);
    let k2 = 3;
    let c1 = ec.mul_k_p_logn(k2, &g);
    let c2 = ec.add_p1_p2(&ec.mul_k_p_logn(k2, &p), &m);
    assert_eq!(c1, Point::new(8, 3));
    assert_eq!(c2, Point::new(10, 2));
    assert_eq!(m, ec.sub_p1_p2(&c2, &ec.mul_k_p_logn(k, &c1)));
}

#[test]
fn test_elgamal() {
    // finished in 17.33s (brute force) vs 0.29s (get_inverse_gcd_tail)
    let ec = ECC::new(10000019, 0, 225);
    let g = Point::new(720114, 611085363);
    assert!(ec.contains(&g));
    let k = 2323532;
    let p = ec.mul_k_p_logn(k, &g);
    let m = Point::new(3243213, 3231234); // NOTE: m is not a point on curve!
    let k2 = 1111111;
    let c1 = ec.mul_k_p_logn(k2, &g);
    let c2 = ec.add_p1_p2(&ec.mul_k_p_logn(k2, &p), &m);
    assert_eq!(m, ec.sub_p1_p2(&c2, &ec.mul_k_p_logn(k, &c1)));
}

#[test]
fn test_ecdsa() {
    // let ec = ECC::new(23, 1, 1);
    // let k = 17;
    // let g = Point::new(3, 10);
    // let p = ec.mul_k_p_logn(k, &g);
    // let m = 7;
    // let r = 3;
    // let pr = ec.mul_k_p_logn(r, &g);

    let ec = ECC::new(11, 1, 6);
    let k = 10;
    let g = Point::new(2, 7);
    let p = ec.mul_k_p_logn(k, &g);
    let m = 6; // 7
    let r = 4; // 2
    let pr = ec.mul_k_p_logn(r, &g);

    let ec2 = ECC::new(ec.find_order(&p), 1, 1);
    let s = ec2.modulo(ec2.calc_inverse_gcd_tail(r) * (m + pr.x * k));

    //dbg!(&s); // need pr.x != 0, s != 0
    let s_1 = ec2.calc_inverse_gcd_tail(s);
    let v = ec.add_p1_p2(
        &ec.mul_k_p_logn(m * s_1, &g),
        &ec.mul_k_p_logn(pr.x * s_1, &p),
    );
    assert_eq!(pr, v);

    let s2 = r - m * k;
    assert_eq!(
        pr,
        ec.add_p1_p2(&ec.mul_k_p_logn(s2, &g), &ec.mul_k_p_logn(m, &p))
    );
}

#[test]
#[ignore]
fn find_a_good_ec() {
    'main: for i in 1u64..300u64 {
        for j in 100000u64..2000000u64 {
            let rhs = j * j * j + i;
            let y = (rhs as f64).sqrt() as u64;
            if y * y == rhs {
                println!("(i, j, y) = {}, {}, {}", i, j, y);
                break 'main;
            }
        }
    }
    //225, 720114, 611085363
}

#[test]
fn test_inverse_fast() {
    let ec = ECC::new(102, 1, 1);
    assert_eq!(53, ec.calc_inverse_gcd(77));
    assert_eq!(53, ec.calc_inverse_gcd_tail(77));

    assert_eq!(ec.calc_inverse_gcd(7), ec.calc_inverse_gcd_tail(7));
}

#[test]
fn test_find_order() {
    dbg!(ECC::new(23, 1, 1).find_order(&Point::new(3, 10)));
    dbg!(ECC::new(11, 1, 6).find_order(&Point::new(2, 7)));
}

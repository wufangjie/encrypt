extern crate encrypt;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use encrypt::conv::hex_to_bytes;
use num_bigint::{BigInt, Sign};
use num_traits::{One, Zero};

fn make_data() -> (BigInt, BigInt) {
    let p_str = "FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE FFFFFC2F";
    let x_str = "79BE667E F9DCBBAC 55A06295 CE870B07 029BFCDB 2DCE28D9 59F2815B 16F81798";
    let x = BigInt::from_bytes_be(Sign::Plus, &hex_to_bytes(x_str.replace(' ', "")).unwrap());
    let p = BigInt::from_bytes_be(Sign::Plus, &hex_to_bytes(p_str.replace(' ', "")).unwrap());
    (x, p)
}

fn test_mod_owned() {
    let (mut x, mut p) = make_data();
    loop {
        if x.is_one() || x.is_zero() {
            break;
        } else {
            p %= &x;
            std::mem::swap(&mut p, &mut x);
        }
    }
}

fn test_mod_ref() {
    let (mut x, mut p) = make_data();
    loop {
        if x.is_one() || x.is_zero() {
            break;
        } else {
            p = &p % &x;
            std::mem::swap(&mut p, &mut x);
        }
    }
}

fn test_mod_clone() {
    let (mut x, mut p) = make_data();
    loop {
        if x.is_one() || x.is_zero() {
            break;
        } else {
            p = p.clone() % x.clone();
            std::mem::swap(&mut p, &mut x);
        }
    }
}

pub fn criterion_benchmark_owned(c: &mut Criterion) {
    c.bench_function("owned", |b| b.iter(test_mod_owned));
}

pub fn criterion_benchmark_clone(c: &mut Criterion) {
    c.bench_function("clone", |b| b.iter(test_mod_clone));
}

pub fn criterion_benchmark_ref(c: &mut Criterion) {
    c.bench_function("ref", |b| b.iter(test_mod_ref));
}

criterion_group!(
    benches,
    criterion_benchmark_owned,
    criterion_benchmark_clone,
    criterion_benchmark_ref,
);
criterion_main!(benches);

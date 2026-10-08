//! Задание 8 (Rust): Python-модуль на Rust, собирается Maturin и импортируется в Python.
//!
//! Вычислительная логика вынесена в обычные Rust-функции (их проверяет `cargo test`),
//! а `#[pyfunction]`-обёртки только переводят типы и ошибки в Python.
//! Тяжёлые функции отпускают GIL, чтобы не блокировать другие потоки Python.

use pyo3::exceptions::{PyOverflowError, PyValueError};
use pyo3::prelude::*;

/// Верхняя граница для решета: 1e9 байт памяти под массив — разумный максимум.
pub const MAX_SIEVE_LIMIT: usize = 1_000_000_000;

/// Сумма квадратов с проверкой переполнения i64.
pub fn sum_squares_checked(numbers: &[i64]) -> Option<i64> {
    numbers
        .iter()
        .try_fold(0i64, |acc, &x| x.checked_mul(x).and_then(|sq| acc.checked_add(sq)))
}

/// Решето Эратосфена: флаги простоты для чисел 0..=limit.
fn sieve(limit: usize) -> Vec<bool> {
    let mut is_prime = vec![true; limit + 1];
    is_prime[0] = false;
    if limit >= 1 {
        is_prime[1] = false;
    }
    let mut i = 2;
    while i * i <= limit {
        if is_prime[i] {
            let mut j = i * i;
            while j <= limit {
                is_prime[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    is_prime
}

pub fn count_primes_upto(limit: usize) -> usize {
    sieve(limit).into_iter().filter(|&p| p).count()
}

pub fn primes_upto(limit: usize) -> Vec<u64> {
    sieve(limit)
        .into_iter()
        .enumerate()
        .filter_map(|(n, p)| p.then_some(n as u64))
        .collect()
}

/// Детерминированный тест Миллера — Рабина, точен для всех u64.
pub fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    const BASES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    for &p in &BASES {
        if n % p == 0 {
            return n == p;
        }
    }
    let mut d = n - 1;
    let mut s = 0;
    while d % 2 == 0 {
        d /= 2;
        s += 1;
    }
    'witness: for &a in &BASES {
        let mut x = pow_mod(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 1..s {
            x = mul_mod(x, x, n);
            if x == n - 1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

fn pow_mod(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut result = 1;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 {
            result = mul_mod(result, base, m);
        }
        base = mul_mod(base, base, m);
        exp >>= 1;
    }
    result
}

fn check_limit(limit: usize) -> PyResult<()> {
    if limit > MAX_SIEVE_LIMIT {
        return Err(PyValueError::new_err(format!(
            "limit слишком большой (максимум {MAX_SIEVE_LIMIT})"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------- Python API

/// Сумма квадратов списка целых чисел.
#[pyfunction]
fn sum_squares(numbers: Vec<i64>) -> PyResult<i64> {
    sum_squares_checked(&numbers)
        .ok_or_else(|| PyOverflowError::new_err("переполнение int64 при вычислении суммы квадратов"))
}

/// Количество простых чисел, не превосходящих limit.
#[pyfunction]
fn count_primes(py: Python<'_>, limit: usize) -> PyResult<usize> {
    check_limit(limit)?;
    Ok(py.allow_threads(|| count_primes_upto(limit)))
}

/// Список простых чисел, не превосходящих limit.
#[pyfunction]
fn primes(py: Python<'_>, limit: usize) -> PyResult<Vec<u64>> {
    check_limit(limit)?;
    Ok(py.allow_threads(|| primes_upto(limit)))
}

/// Проверка числа на простоту (тест Миллера — Рабина).
#[pyfunction]
fn is_prime(n: u64) -> bool {
    is_prime_u64(n)
}

#[pymodule]
fn fastmath(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_squares, m)?)?;
    m.add_function(wrap_pyfunction!(count_primes, m)?)?;
    m.add_function(wrap_pyfunction!(primes, m)?)?;
    m.add_function(wrap_pyfunction!(is_prime, m)?)?;
    m.add("MAX_SIEVE_LIMIT", MAX_SIEVE_LIMIT)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_squares_works() {
        assert_eq!(sum_squares_checked(&[1, 2, 3, 4, 5]), Some(55));
        assert_eq!(sum_squares_checked(&[]), Some(0));
        assert_eq!(sum_squares_checked(&[-3]), Some(9));
        assert_eq!(sum_squares_checked(&[i64::MAX]), None);
        assert_eq!(sum_squares_checked(&[3_000_000_000, 3_000_000_000]), None);
    }

    #[test]
    fn prime_counts() {
        for (limit, want) in [(0, 0), (1, 0), (2, 1), (10, 4), (100, 25), (1_000_000, 78_498)] {
            assert_eq!(count_primes_upto(limit), want, "limit = {limit}");
        }
        assert_eq!(primes_upto(20), vec![2, 3, 5, 7, 11, 13, 17, 19]);
    }

    #[test]
    fn miller_rabin_matches_sieve() {
        let flags = sieve(10_000);
        for n in 0..=10_000u64 {
            assert_eq!(is_prime_u64(n), flags[n as usize], "n = {n}");
        }
        assert!(is_prime_u64(1_000_000_007));
        assert!(is_prime_u64(18_446_744_073_709_551_557)); // наибольшее простое < 2^64
        assert!(!is_prime_u64(3_215_031_751)); // сильное псевдопростое по основаниям 2, 3, 5, 7
    }
}

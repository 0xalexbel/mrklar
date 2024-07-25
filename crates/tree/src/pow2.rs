
#[inline]
pub fn two_pow_n(power: u8) -> u64 {
    assert!(power <= 63);
    1u64 << power
}

#[allow(dead_code)]
pub fn next_power_of_two(num: u64) -> u8 {
    let log2 = f64::log2(num as f64);
    if log2 > u8::MAX as f64 {
        panic!("too big number");
    }

    let ulog2 = log2 as u8;

    let p0 = two_pow_n(ulog2);
    assert!(p0 <= num);
    if num == p0 {
        return ulog2;
    }

    let p1 = two_pow_n(ulog2 + 1);
    assert!(p1 >= num);
    ulog2 + 1
}

#[cfg(test)]
mod test {
    use super::{next_power_of_two, two_pow_n};

    #[test]
    fn test() {
        assert_eq!(two_pow_n(0), 1);
        assert_eq!(two_pow_n(1), 2);
        assert_eq!(two_pow_n(2), 4);
        assert_eq!(two_pow_n(3), 8);
        assert_eq!(two_pow_n(4), 16);
        assert_eq!(two_pow_n(5), 32);
        assert_eq!(two_pow_n(6), 64);
        assert_eq!(two_pow_n(7), 128);
        assert_eq!(two_pow_n(8), 256);
        assert_eq!(two_pow_n(9), 512);
        assert_eq!(two_pow_n(10), 1024);
        assert_eq!(u64::pow(2, 10), 1024);

        assert_eq!(next_power_of_two(1), 0);
        assert_eq!(next_power_of_two(2), 1);
        assert_eq!(next_power_of_two(3), 2);
        assert_eq!(next_power_of_two(4), 2);
        assert_eq!(next_power_of_two(5), 3);
        assert_eq!(next_power_of_two(6), 3);
        assert_eq!(next_power_of_two(7), 3);
        assert_eq!(next_power_of_two(8), 3);
        assert_eq!(next_power_of_two(9), 4);
        assert_eq!(next_power_of_two(255), 8);
        assert_eq!(next_power_of_two(256), 8);
        assert_eq!(next_power_of_two(257), 9);
    }
}
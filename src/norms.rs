#[derive(Debug, Copy, Clone)]
pub enum Norm {
    L1,
    L2,
    LInf,
}

pub struct L1;
pub struct L2;
pub struct LInf;

pub trait NormType {
    fn compute_distance(x: usize, y: usize) -> f64;
}

impl NormType for L1 {
    #[inline]
    fn compute_distance(x: usize, y: usize) -> f64 {
        // x and y are unsigned ints, we don't need to take abs
        (x + y) as f64
    }
}

impl NormType for LInf {
    #[inline]
    fn compute_distance(x: usize, y: usize) -> f64 {
        // x and y are unsigned ints, we don't need to take abs
        x.max(y) as f64
    }
}

impl NormType for L2 {
    #[inline]
    fn compute_distance(x: usize, y: usize) -> f64 {
        f64::sqrt((x * x + y * y) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64;

    #[test]
    fn test_norm_l1_distance() {
        assert_eq!(L1::compute_distance(3, 4), 7.0);
        assert_eq!(L1::compute_distance(0, 5), 5.0);
        assert_eq!(L1::compute_distance(10, 0), 10.0);
    }

    #[test]
    fn test_norm_linf_distance() {
        assert_eq!(LInf::compute_distance(3, 4), 4.0);
        assert_eq!(LInf::compute_distance(0, 5), 5.0);
        assert_eq!(LInf::compute_distance(10, 2), 10.0);
    }

    #[test]
    fn test_norm_l2_distance() {
        assert_eq!(L2::compute_distance(3, 4), 5.0);
        assert_eq!(L2::compute_distance(0, 5), 5.0);
        assert_eq!(L2::compute_distance(1, 1), f64::consts::SQRT_2);
    }
}

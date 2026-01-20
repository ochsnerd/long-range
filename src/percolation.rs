use rand::Rng;
use rand::{SeedableRng, distr::StandardUniform};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use union_find_rs::prelude::*;

use crate::norms::*;

#[derive(Debug, Clone, Copy)]
pub struct Observables {
    /// <https://arxiv.org/pdf/1610.00200>
    /// S, see Equation (5)
    pub average_size: f64,
    /// Q_G, see Equation (6)
    pub size_spread: f64,
}

pub fn simulate(
    norm: Norm,
    l: usize,
    alpha: f64,
    beta: f64,
    n_samples: u64,
    seed: u64,
) -> Vec<Observables> {
    (0..n_samples)
        .into_par_iter()
        .map(|i| {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            rng.set_stream(i);
            realize(norm, l, alpha, beta, &mut rng)
        })
        .collect()
}

pub fn realize<R: Rng + ?Sized>(
    norm: Norm,
    l: usize,
    alpha: f64,
    beta: f64,
    rng: &mut R,
) -> Observables {
    let clusters = match norm {
        Norm::L1 => lr_percolation_2d::<L1, _>(l, alpha, beta, rng),
        Norm::L2 => lr_percolation_2d::<L2, _>(l, alpha, beta, rng),
        Norm::LInf => lr_percolation_2d::<LInf, _>(l, alpha, beta, rng),
    };
    Observables::new(l, clusters)
}

/// Return the number of failures before the first success,
/// for a Bernoulli(p) RV
///    
/// p >= 1 => skip=0 (always success).
/// p <= epsilon => skip=large (never success).
/// Otherwise uses log-based approach for a geometric distribution.
fn geometric_skip<R: Rng + ?Sized>(p: f64, rng: &mut R) -> usize {
    // Doing this check here (as opposed to when p is first computed)
    // seems to be more efficient at larger l. My hypothesis is that the
    // branch predictor can work really well here
    match p {
        p if p >= 1.0 => 0,
        p if p <= 1E-16 => usize::MAX,
        _ => {
            let u: f64 = rng.sample(StandardUniform);
            (u.log2() / (1.0 - p).log2()) as usize
        }
    }
}

type Clusters = DisjointSets<usize>;

/// 2D long-range percolation with skip-based sampling.
/// Probability p_l = min(1, beta / l^(2 + alpha)).
fn lr_percolation_2d<N: NormType, R: Rng + ?Sized>(
    l: usize,
    alpha: f64,
    beta: f64,
    rng: &mut R,
) -> Clusters {
    let mut clusters = Clusters::with_capacity(l * l);
    for i in 0..l * l {
        clusters.make_set(i).unwrap();
    }
    for x in 0..l {
        for y in 0..l {
            if x == 0 && y == 0 {
                continue;
            }
            let periodic_dx = x.min(l - x);
            let periodic_dy = y.min(l - y);
            let distance = N::compute_distance(periodic_dx, periodic_dy);

            if distance < 1E-16 {
                // TODO: is this correct?
                // Would not (d << 1) => (p = 1) => geometric_skip = 0 => one big cluster?
                // I mean we don't want that, but why are we allowed to circumvent it?
                continue;
            }
            let p = beta / distance.powf(2.0 + alpha);

            let mut i: usize = 0;
            while i < l * l {
                i = i.saturating_add(geometric_skip(p, rng));
                if i >= l * l {
                    break;
                }

                let dx = (i / l + x) % l;
                let dy = (i % l + y) % l;
                clusters
                    .union(
                        &clusters.find_set(&i).unwrap(),
                        &clusters.find_set(&(dx * l + dy)).unwrap(),
                    )
                    .unwrap();

                i = i.saturating_add(1);
            }
        }
    }
    clusters
}

impl Observables {
    fn new(l: usize, clusters: Clusters) -> Self {
        // To prevent overflow, f64 instead of an int
        let mut sum_power2: f64 = 0.0;
        let mut sum_power4: f64 = 0.0;
        for size in clusters.into_iter().map(|c| c.len() as f64) {
            sum_power2 += size.powi(2);
            sum_power4 += size.powi(4);
        }

        Observables {
            average_size: sum_power2 / (l * l) as f64,
            size_spread: sum_power4 / sum_power2.powi(2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_geometric_skip_edge_cases() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // When p >= 1.0, should always return 0 (immediate success)
        assert_eq!(geometric_skip(1.0, &mut rng), 0);
        assert_eq!(geometric_skip(1.5, &mut rng), 0);

        // When p is very small, should return usize::MAX
        assert_eq!(geometric_skip(1e-17, &mut rng), usize::MAX);

        // For p=0.5, the distribution is well-defined
        // We test with a fixed seed for deterministic behavior
        let mut fixed_rng = ChaCha8Rng::seed_from_u64(123);
        let skip = geometric_skip(0.5, &mut fixed_rng);
        // With seed 123, the value should be deterministic
        // (Actual value depends on the exact RNG implementation)
        assert!(skip < 100); // Sanity check - should be reasonable
    }

    #[test]
    fn test_simple_percolation() {
        let l = 10;
        let alpha = 0.0;
        let beta = 1.0; // connect everything

        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let clusters = lr_percolation_2d::<L1, _>(l, alpha, beta, &mut rng);

        assert_eq!(clusters.into_iter().count(), 1);
    }

    #[test]
    fn test_no_percolation() {
        let l = 10;
        let alpha = 1.0;
        let beta = 0.0; // connect nothing

        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let clusters = lr_percolation_2d::<L1, _>(l, alpha, beta, &mut rng);

        for c in clusters.into_iter() {
            assert_eq!(c.len(), 1);
        }
    }

    #[test]
    fn test_cluster_property_invariant() {
        // Test that total sum of clusters equals grid size
        let l = 4; // 4x4 grid
        let alpha = 1.5;
        let beta = 0.5;
        let mut rng = ChaCha8Rng::seed_from_u64(123);

        let clusters = lr_percolation_2d::<L1, _>(l, alpha, beta, &mut rng);

        // Sum of all cluster sizes should equal total grid size
        let sum_sizes: usize = clusters.into_iter().map(|c| c.len()).sum();
        assert_eq!(sum_sizes, l * l);
    }
}

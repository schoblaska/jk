/// Cosine similarity between two vectors.
pub fn cosine(a: &[f64], b: &[f64]) -> f64 {
    debug_assert_eq!(a.len(), b.len());
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for (ai, bi) in a.iter().zip(b.iter()) {
        dot += ai * bi;
        na += ai * ai;
        nb += bi * bi;
    }
    na = na.sqrt();
    nb = nb.sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na * nb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_vectors() {
        assert!((cosine(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn orthogonal_vectors() {
        assert!((cosine(&[1.0, 0.0], &[0.0, 1.0])).abs() < 1e-9);
    }

    #[test]
    fn opposite_vectors() {
        assert!((cosine(&[1.0, 0.0], &[-1.0, 0.0]) + 1.0).abs() < 1e-9);
    }

    #[test]
    fn zero_vector() {
        assert_eq!(cosine(&[0.0, 0.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn scale_invariant() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        let b_scaled = [400.0, 500.0, 600.0];
        assert!((cosine(&a, &b) - cosine(&a, &b_scaled)).abs() < 1e-9);
    }
}

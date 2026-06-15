//! The [`HasRng`] capability trait.

/// A context that can produce uniform random numbers.
///
/// The only required method is [`next_f64`](HasRng::next_f64), a uniform draw
/// from `[0.0, 1.0)`; the convenience methods are derived from it. Keeping the
/// source of randomness in the context (rather than a global or a library
/// dependency) means trees stay deterministic and unit-testable: a test can
/// supply a context whose draws are a fixed queue, while a game supplies one
/// backed by the engine RNG.
///
/// ```
/// use btree::prelude::*;
///
/// struct Det { values: Vec<f64>, at: usize }
/// impl HasRng for Det {
///     fn next_f64(&mut self) -> f64 {
///         let v = self.values[self.at % self.values.len()];
///         self.at += 1;
///         v
///     }
/// }
/// ```
pub trait HasRng {
    /// A uniform random draw from the half-open range `[0.0, 1.0)`.
    fn next_f64(&mut self) -> f64;

    /// A uniform random index in `0..n` (returns `0` when `n == 0`).
    fn next_below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        // `next_f64` is in `[0, 1)`, so the product is in `[0, n)`; the `min`
        // guards against a rounding draw of exactly `n`.
        ((self.next_f64() * n as f64) as usize).min(n - 1)
    }

    /// `true` with probability `p` (clamped to `[0, 1]`).
    fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }
}

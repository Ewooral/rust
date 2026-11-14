//! Small "kernels" (tight loops) that the micro-benchmarks and examples
//! compare: data layout, iterators vs loops, and bounds checks.

// ---------------------------------------------------------------------------
// Data layout: array of structs (AoS) vs struct of arrays (SoA).
// ---------------------------------------------------------------------------

/// One particle, stored the usual way: all its fields side by side.
/// 8 fields x 4 bytes = 32 bytes per particle.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Particle {
    /// Position.
    pub x: f32,
    /// Position.
    pub y: f32,
    /// Position.
    pub z: f32,
    /// Velocity.
    pub vx: f32,
    /// Velocity.
    pub vy: f32,
    /// Velocity.
    pub vz: f32,
    /// Mass.
    pub mass: f32,
    /// Charge.
    pub charge: f32,
}

/// The same particles stored "struct of arrays": one `Vec` per field.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Particles {
    /// Every particle's x, then every y, and so on.
    pub x: Vec<f32>,
    /// Positions.
    pub y: Vec<f32>,
    /// Positions.
    pub z: Vec<f32>,
    /// Velocities.
    pub vx: Vec<f32>,
    /// Velocities.
    pub vy: Vec<f32>,
    /// Velocities.
    pub vz: Vec<f32>,
    /// Masses.
    pub mass: Vec<f32>,
    /// Charges.
    pub charge: Vec<f32>,
}

impl Particles {
    /// Converts AoS to SoA.
    pub fn from_aos(particles: &[Particle]) -> Self {
        let n = particles.len();
        let mut soa = Particles {
            x: Vec::with_capacity(n),
            y: Vec::with_capacity(n),
            z: Vec::with_capacity(n),
            vx: Vec::with_capacity(n),
            vy: Vec::with_capacity(n),
            vz: Vec::with_capacity(n),
            mass: Vec::with_capacity(n),
            charge: Vec::with_capacity(n),
        };
        for p in particles {
            soa.x.push(p.x);
            soa.y.push(p.y);
            soa.z.push(p.z);
            soa.vx.push(p.vx);
            soa.vy.push(p.vy);
            soa.vz.push(p.vz);
            soa.mass.push(p.mass);
            soa.charge.push(p.charge);
        }
        soa
    }
}

/// `n` particles with simple, deterministic values.
pub fn make_particles(n: usize) -> Vec<Particle> {
    (0..n)
        .map(|i| {
            let f = i as f32;
            Particle {
                x: f,
                y: f * 0.5,
                z: 1.0,
                vx: 0.1,
                vy: 0.2,
                vz: 0.3,
                mass: 1.0 + (i % 7) as f32,
                charge: if i.is_multiple_of(2) { 1.0 } else { -1.0 },
            }
        })
        .collect()
}

/// How many particles are heavier than `limit`, AoS: to read 4 bytes of
/// mass, the CPU pulls the whole 32-byte particle into its cache, so 7/8 of
/// every load is wasted.
pub fn count_heavy_aos(particles: &[Particle], limit: f32) -> usize {
    particles.iter().filter(|p| p.mass > limit).count()
}

/// The same, SoA: every byte loaded is a mass, so the cache is used fully
/// and the compiler can compare several masses per instruction (SIMD).
pub fn count_heavy_soa(particles: &Particles, limit: f32) -> usize {
    particles.mass.iter().filter(|&&m| m > limit).count()
}

// ---------------------------------------------------------------------------
// Iterators vs loops.
// ---------------------------------------------------------------------------

/// Sum of the squares of the even numbers, with an index loop.
pub fn sum_even_squares_loop(values: &[u64]) -> u64 {
    let mut total = 0;
    let mut i = 0;
    while i < values.len() {
        let v = values[i];
        if v.is_multiple_of(2) {
            total += v * v;
        }
        i += 1;
    }
    total
}

/// The same with an iterator chain: no index, no bounds check, and it
/// compiles to the same (or better) machine code: a zero-cost abstraction.
pub fn sum_even_squares_iter(values: &[u64]) -> u64 {
    values
        .iter()
        .filter(|&&v| v.is_multiple_of(2))
        .map(|&v| v * v)
        .sum()
}

// ---------------------------------------------------------------------------
// Bounds checks.
// ---------------------------------------------------------------------------

/// `out[i] = a[i] + b[i]` with indexing. Nothing tells the compiler that `b`
/// and `out` are as long as `a`, so every step checks two indexes, and a
/// loop that might panic half-way can't be turned into SIMD instructions.
#[allow(clippy::needless_range_loop)] // the index loop is the point here
pub fn add_indexed(a: &[f32], b: &[f32], out: &mut [f32]) {
    for i in 0..a.len() {
        out[i] = a[i] + b[i];
    }
}

/// Re-slicing everything to the same length FIRST: one check before the loop
/// (it panics there if a slice is too short), then the compiler can see that
/// every `i` is in range and drops the checks inside the loop.
#[allow(clippy::needless_range_loop)]
pub fn add_resliced(a: &[f32], b: &[f32], out: &mut [f32]) {
    let n = a.len();
    let (b, out) = (&b[..n], &mut out[..n]);
    for i in 0..n {
        out[i] = a[i] + b[i];
    }
}

/// `zip` walks the three slices together and stops at the shortest: no
/// indexes, so nothing to check.
pub fn add_zip(a: &[f32], b: &[f32], out: &mut [f32]) {
    for ((o, x), y) in out.iter_mut().zip(a).zip(b) {
        *o = x + y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aos_and_soa_agree() {
        let aos = make_particles(1_000);
        let soa = Particles::from_aos(&aos);
        assert_eq!(soa.mass.len(), 1_000);
        assert_eq!(count_heavy_aos(&aos, 4.0), count_heavy_soa(&soa, 4.0));
        assert_eq!(count_heavy_soa(&soa, 4.0), 428); // masses 5, 6 and 7
    }

    #[test]
    fn loop_and_iterator_agree() {
        let values: Vec<u64> = (0..1_000).collect();
        assert_eq!(
            sum_even_squares_loop(&values),
            sum_even_squares_iter(&values)
        );
        assert_eq!(sum_even_squares_iter(&[1, 2, 3, 4]), 4 + 16);
    }

    #[test]
    fn element_wise_adds_agree() {
        let a: Vec<f32> = (0..100u16).map(f32::from).collect();
        let b = vec![0.5; 100];
        let mut expected = vec![0.0; 100];
        add_indexed(&a, &b, &mut expected);
        assert_eq!(expected[99], 99.5);
        for add in [add_resliced, add_zip] {
            let mut out = vec![0.0; 100];
            add(&a, &b, &mut out);
            assert_eq!(out, expected);
        }
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn indexed_add_panics_when_b_is_shorter() {
        add_indexed(&[1.0, 2.0], &[1.0], &mut [0.0, 0.0]);
    }
}

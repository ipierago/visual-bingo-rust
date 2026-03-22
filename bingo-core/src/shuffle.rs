pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    pub fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6d2b79f5);
        let mut t = self.state;
        t = t.wrapping_mul(t ^ (t >> 15)).wrapping_mul(1 | t);
        t ^= t.wrapping_add(t.wrapping_mul(t ^ (t >> 7)).wrapping_mul(61 | t));
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }

    pub fn next_usize(&mut self, n: usize) -> usize {
        (self.next_f64() * n as f64) as usize
    }
}

/// FNV-1a hash — same as TypeScript version
pub fn hash_seed(seed: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for byte in seed.bytes() {
        h ^= byte as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

/// Fisher-Yates shuffle
pub fn shuffle<T: Clone>(items: &[T], seed: &str) -> Vec<T> {
    let mut rng = Rng::new(hash_seed(seed));
    let mut out = items.to_vec();
    let len = out.len();
    for i in (1..len).rev() {
        let j = rng.next_usize(i + 1);
        out.swap(i, j);
    }
    out
}

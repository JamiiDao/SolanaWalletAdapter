use core::fmt;

use zeroize::{Zeroize, ZeroizeOnDrop};

pub struct RandomBytes<const N: usize>([u8; N]);

impl<const N: usize> RandomBytes<N> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_with_buffer(buffer: &mut [u8; N]) {
        use rand_chacha::ChaCha12Rng;
        use rand_core::{RngCore, SeedableRng};

        let mut rng = ChaCha12Rng::from_os_rng();

        rng.fill_bytes(buffer);
    }

    pub fn generate() -> Self {
        let mut buffer = Self::new();

        Self::generate_with_buffer(&mut buffer.0);

        buffer
    }

    pub const fn expose(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> Zeroize for RandomBytes<N> {
    fn zeroize(&mut self) {
        self.0.fill(0u8);

        assert_eq!(self.0, [0u8; N]);
    }
}
impl<const N: usize> ZeroizeOnDrop for RandomBytes<N> {}

impl<const N: usize> Default for RandomBytes<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}

impl<const N: usize> fmt::Debug for RandomBytes<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RandomBytes(REDACTED[{N}])")
    }
}

impl<const N: usize> fmt::Display for RandomBytes<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

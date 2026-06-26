//! Wrapper seguro para operações de string com verificação formal Kani.

use core::slice;

/// String segura com verificação de bounds.
#[derive(Debug, Clone, Default)]
pub struct SafeString {
    inner: Vec<u8>,
}

impl SafeString {
    /// Cria uma nova string segura a partir de um slice de bytes.
    pub fn new(bytes: &[u8]) -> Self {
        Self { inner: bytes.to_vec() }
    }

    /// Busca um caractere, garantindo que não ultrapasse o buffer.
    #[inline]
    #[cfg_attr(kani, kani::ensures(
        match result {
            Some(pos) => pos < self.inner.len() && self.inner[pos] == c,
            None => !self.inner.contains(&c),
        }
    ))]
    pub fn find_char(&self, c: u8) -> Option<usize> {
        self.inner.iter().position(|&x| x == c)
    }

    /// Versão `unsafe` que deve ser provada correta via Kani.
    /// # Safety
    /// - `ptr` deve ser válido e apontar para um buffer de pelo menos `len` bytes
    /// - O buffer deve conter bytes UTF-8 válidos (ou ser tratado como raw)
    #[inline]
    pub unsafe fn find_char_raw(ptr: *const u8, len: usize, c: u8) -> Option<usize> {
        let slice = slice::from_raw_parts(ptr, len);
        slice.iter().position(|&x| x == c)
    }
}

// ============================================================
// Kani Proofs
// ============================================================

#[cfg(kani)]
mod verification {
    use super::*;

    /// Prove que find_char nunca ultrapassa o buffer.
    #[kani::proof]
    fn verify_find_char_bounds() {
        let bytes = kani::any::<[u8; 256]>();
        let s = SafeString { inner: bytes.to_vec() };
        let c = kani::any::<u8>();
        let result = s.find_char(c);

        match result {
            Some(pos) => {
                assert!(pos < s.inner.len());
                assert_eq!(s.inner[pos], c);
            }
            None => {
                assert!(!s.inner.contains(&c));
            }
        }
    }

    /// Prove que find_char_raw é segura: nunca lê além do buffer.
    #[kani::proof]
    fn verify_find_char_raw_safety() {
        let bytes = kani::any::<[u8; 256]>();
        let len = kani::any::<usize>();
        kani::assume(len <= 256);

        let ptr = bytes.as_ptr();
        let c = kani::any::<u8>();

        unsafe {
            let result = SafeString::find_char_raw(ptr, len, c);
            match result {
                Some(pos) => {
                    assert!(pos < len);
                    assert_eq!(bytes[pos], c);
                }
                None => {
                    // Garantia: nenhuma posição encontrada
                    assert!(!bytes.iter().take(len).any(|&x| x == c));
                }
            }
        }
    }

    /// Prove que o padrão Squidbleed NÃO acontece no SafeString.
    /// O problema original era: strchr sem verificação de \0.
    /// Aqui provamos que find_char sempre respeita os limites.
    #[kani::proof]
    fn verify_no_squidbleed_pattern() {
        let bytes = kani::any::<[u8; 256]>();
        let s = SafeString { inner: bytes.to_vec() };
        let c = kani::any::<u8>();

        // A função find_char nunca pode retornar uma posição >= len
        let result = s.find_char(c);
        match result {
            Some(pos) => assert!(pos < s.inner.len()),
            None => assert!(true), // Sempre seguro
        }
    }
}

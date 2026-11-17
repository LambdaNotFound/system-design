//! Short codes: 8 base62 characters drawn from the OS CSPRNG.
//!
//! See design-doc/url-shortener/rfc.md, Link Creation; requirement.md, NFR8
//! and NFR12.

use std::fmt;

/// The base62 symbols in the order `code_from_bytes` indexes them.
const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// The largest multiple of 62 that is at most 256: bytes from here up are
/// redrawn so that `byte % 62` stays uniform.
const REJECT_FROM: u8 = 248;

/// A short code: exactly 8 ASCII characters from `0-9`, `A-Z` and `a-z`,
/// case-sensitive.
///
/// See design-doc/url-shortener/requirement.md, NFR12; rfc.md, Estimates,
/// Code space.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Code([u8; 8]);

impl Code {
    /// The code `s` spells, or `None` unless `s` is exactly 8 base62
    /// characters. A redirect path that is not a code is `404` with no store
    /// read (rfc.md, Redirect Resolver, step 1).
    pub fn parse(s: &str) -> Option<Code> {
        let bytes: [u8; 8] = s.as_bytes().try_into().ok()?;
        bytes
            .iter()
            .all(u8::is_ascii_alphanumeric)
            .then_some(Code(bytes))
    }

    /// The code's 8 characters.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("a Code holds only base62 ASCII")
    }
}

impl fmt::Debug for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Code({:?})", self.as_str())
    }
}

/// The OS random source failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RngError;

/// Where Link Creation draws fresh codes from. A trait so a test can hand
/// out a code that is already taken.
///
/// See design-doc/url-shortener/rfc.md, Link Creation, Phase 2.
pub trait CodeSource: Send + Sync {
    /// One fresh code.
    fn draw(&self) -> Result<Code, RngError>;
}

/// Maps a stream of random bytes to a code, each character uniform over the
/// 62 symbols when the bytes are: a byte below 248 gives the symbol at
/// `byte % 62` in `0-9A-Za-z`, and a byte of 248 or more is dropped and the
/// next one taken, since 248 is the largest multiple of 62 below 256. Takes
/// bytes until 8 are kept; the first error from `next` is returned.
///
/// See design-doc/url-shortener/requirement.md, NFR8 and NFR12.
pub fn code_from_bytes(mut next: impl FnMut() -> Result<u8, RngError>) -> Result<Code, RngError> {
    let mut code = [0; 8];
    let mut kept = 0;
    while kept < code.len() {
        let byte = next()?;
        if byte < REJECT_FROM {
            code[kept] = ALPHABET[usize::from(byte % 62)];
            kept += 1;
        }
    }
    Ok(Code(code))
}

/// Codes from the OS CSPRNG through `getrandom`, mapped by
/// [`code_from_bytes`]. No code predicts another (NFR8).
///
/// See design-doc/url-shortener/requirement.md, NFR8.
#[derive(Clone, Copy, Debug, Default)]
pub struct OsCodes;

impl CodeSource for OsCodes {
    fn draw(&self) -> Result<Code, RngError> {
        // 16 bytes cover the 8 kept ones unless 9 or more of them are
        // redrawn, with probability sum over k >= 9 of C(16, k) p^k (1-p)^(16-k)
        // at p = 8/256, about 2.7e-10; then the buffer refills.
        let mut buf = [0; 16];
        let mut used = buf.len();
        code_from_bytes(|| {
            if used == buf.len() {
                getrandom::fill(&mut buf).map_err(|_| RngError)?;
                used = 0;
            }
            used += 1;
            Ok(buf[used - 1])
        })
    }
}

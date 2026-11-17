//! Link Creation: canonicalize, look the key up, and insert a fresh code
//! when the URL is new.
//!
//! See design-doc/url-shortener/rfc.md, Link Creation.

use std::fmt;

use crate::canon::{Reject, UrlKey, canonicalize_v1};
use crate::code::{Code, CodeSource, RngError};
use crate::store::{InsertError, Link, LinkStore, StoreError};

/// The first draw and 5 redraws (rfc.md, Link Creation, Phase 2). Each
/// insert spends a draw, so this also bounds the inserts of one submission,
/// `KeyTaken` reruns included.
const MAX_DRAWS: usize = 6;

/// A successful submission's code.
///
/// See design-doc/url-shortener/rfc.md, API: `201` for `New`, `200` for
/// `Existing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Created {
    /// This call inserted the link.
    New(Code),
    /// The URL, or one the same under FR4, already had this code (FR3).
    Existing(Code),
}

impl Created {
    /// The code, new or existing.
    pub fn code(&self) -> &Code {
        match self {
            Created::New(code) | Created::Existing(code) => code,
        }
    }
}

/// Why a submission got no code.
///
/// See design-doc/url-shortener/rfc.md, Link Creation and API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateError {
    /// The canonicalizer rejected the URL; the store was not called.
    Rejected(Reject),
    /// The same URL under FR4 has a disabled code (FR9).
    Disabled,
    /// `url_index` points at a row whose URL has another key: never returned
    /// as `Existing` (NFR7).
    Integrity,
    /// Six inserts, the first draw and 5 redraws, wrote nothing. Each insert
    /// spends a draw, whether it lost to a taken code or to a taken key whose
    /// winner the rerun of Phase 1 did not find, as when a serialization
    /// conflict aborts both racers. So this answers a code space that keeps
    /// colliding and a key race that keeps failing alike.
    CodesExhausted,
    /// The code source failed while a new link needed a code. A failed draw
    /// is not retried.
    Rng(RngError),
    /// The store failed, after one retry when the failure was a timeout.
    Store(StoreError),
}

/// The Create Handler's logic past FR8's charge, which belongs to the Abuse
/// Limiter.
///
/// - Phase 1: `find_by_key`. A disabled row is `Disabled`; a row whose URL
///   canonicalizes to another key is `Integrity`; otherwise `Existing`.
/// - Phases 2 and 3: `insert` with a fresh code. `CodeTaken` draws again;
///   `KeyTaken` reruns Phase 1 (FR3), and an absent key there draws again.
///   Draws are counted per submission, at most 5 redraws after the first, so
///   reruns are bounded too: past the budget the answer is `CodesExhausted`.
/// - A code is drawn only after Phase 1 finds the key absent, so a failing
///   code source never blocks the answer to a repeat submission (FR3).
/// - A timed-out transaction is retried once (rfc.md, Failure Modes, "The
///   store is slow"). The one retry covers the whole submission, whether the
///   find or the insert timed out. A find that fails is never read as absent.
///   When the retry's Phase 1 finds the code whose insert timed out, that
///   insert had committed, and the answer is `New`.
///
/// See design-doc/url-shortener/rfc.md, Link Creation.
pub struct LinkCreator<S, G> {
    store: S,
    codes: G,
}

impl<S, G> fmt::Debug for LinkCreator<S, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinkCreator").finish_non_exhaustive()
    }
}

impl<S: LinkStore, G: CodeSource> LinkCreator<S, G> {
    /// A creator writing to `store` and drawing new codes from `codes`.
    pub fn new(store: S, codes: G) -> Self {
        Self { store, codes }
    }

    /// Submits `raw`: the code for it, new or existing.
    ///
    /// See design-doc/url-shortener/requirement.md, FR1, FR3, FR9, NFR7.
    pub fn create(&self, raw: &str) -> Result<Created, CreateError> {
        let (url, key) = canonicalize_v1(raw).map_err(CreateError::Rejected)?;
        let mut retry_left = true;
        let mut draws = 0;
        let mut timed_out_code = None;
        loop {
            match self.store.find_by_key(&key) {
                Ok(Some((code, link))) => {
                    check_existing(&key, &link)?;
                    return Ok(if timed_out_code == Some(code) {
                        Created::New(code)
                    } else {
                        Created::Existing(code)
                    });
                }
                Ok(None) => {}
                Err(StoreError::Timeout) if retry_left => {
                    retry_left = false;
                    continue;
                }
                Err(err) => return Err(CreateError::Store(err)),
            }
            loop {
                if draws == MAX_DRAWS {
                    return Err(CreateError::CodesExhausted);
                }
                let code = self.codes.draw().map_err(CreateError::Rng)?;
                draws += 1;
                match self.store.insert(&code, &key, &url) {
                    Ok(()) => return Ok(Created::New(code)),
                    Err(InsertError::CodeTaken) => {}
                    Err(InsertError::KeyTaken) => break,
                    Err(InsertError::Store(StoreError::Timeout)) if retry_left => {
                        retry_left = false;
                        timed_out_code = Some(code);
                        break;
                    }
                    Err(InsertError::Store(err)) => return Err(CreateError::Store(err)),
                }
            }
        }
    }
}

/// Phase 1's checks on the row `url_index[key]` points at: FR9's disable,
/// then NFR7's re-canonicalization of the stored URL.
fn check_existing(key: &UrlKey, link: &Link) -> Result<(), CreateError> {
    if link.disabled {
        return Err(CreateError::Disabled);
    }
    match canonicalize_v1(&link.url) {
        Ok((_, stored_key)) if stored_key == *key => Ok(()),
        _ => Err(CreateError::Integrity),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::canon::SubmittedUrl;
    use crate::store::MemStore;

    /// Commits the first insert and then reports `Timeout`, as a commit
    /// whose reply was lost.
    #[derive(Default)]
    struct LostReply {
        mem: MemStore,
        lost: Mutex<bool>,
    }

    impl LinkStore for LostReply {
        fn find_by_key(&self, key: &UrlKey) -> Result<Option<(Code, Link)>, StoreError> {
            self.mem.find_by_key(key)
        }

        fn insert(&self, code: &Code, key: &UrlKey, url: &SubmittedUrl) -> Result<(), InsertError> {
            self.mem.insert(code, key, url)?;
            let mut lost = self.lost.lock().unwrap();
            if *lost {
                Ok(())
            } else {
                *lost = true;
                Err(InsertError::Store(StoreError::Timeout))
            }
        }
    }

    /// Never finds the key and refuses every insert with `KeyTaken`: a key
    /// race whose winner never becomes visible.
    struct KeyRaceNeverSettles;

    impl LinkStore for KeyRaceNeverSettles {
        fn find_by_key(&self, _key: &UrlKey) -> Result<Option<(Code, Link)>, StoreError> {
            Ok(None)
        }

        fn insert(
            &self,
            _code: &Code,
            _key: &UrlKey,
            _url: &SubmittedUrl,
        ) -> Result<(), InsertError> {
            Err(InsertError::KeyTaken)
        }
    }

    /// Distinct codes, counting the draws.
    #[derive(Default)]
    struct Counting(Mutex<usize>);

    impl CodeSource for Counting {
        fn draw(&self) -> Result<Code, RngError> {
            let mut n = self.0.lock().unwrap();
            *n += 1;
            Ok(Code::parse(&format!("k{:07}", *n)).unwrap())
        }
    }

    /// `KeyTaken` reruns spend the per-submission draw budget, so a key race
    /// that never settles ends in `CodesExhausted` after 6 draws instead of
    /// looping.
    #[test]
    fn key_race_that_never_settles_exhausts_the_draws() {
        let creator = LinkCreator::new(KeyRaceNeverSettles, Counting::default());
        assert_eq!(
            creator.create("http://race.example/"),
            Err(CreateError::CodesExhausted)
        );
        assert_eq!(*creator.codes.0.lock().unwrap(), MAX_DRAWS);
    }

    struct One(Code);

    impl CodeSource for One {
        fn draw(&self) -> Result<Code, RngError> {
            Ok(self.0)
        }
    }

    /// The API's `201`: a retry that finds this submission's own timed-out
    /// insert answers `New`, and a later submission `Existing`.
    #[test]
    fn own_commit_found_on_retry_is_new() {
        let c = Code::parse("ownwrite").unwrap();
        let creator = LinkCreator::new(LostReply::default(), One(c));
        assert_eq!(creator.create("http://lost.example/"), Ok(Created::New(c)));
        assert_eq!(
            creator.create("http://lost.example/"),
            Ok(Created::Existing(c))
        );
    }
}

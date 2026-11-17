//! The store as two traits, the write side Link Creation uses and the read
//! side the Redirect Resolver uses, with an in-memory implementation.
//!
//! See design-doc/url-shortener/rfc.md, Data model and storage; Link
//! Creation; Redirect Resolver.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use crate::canon::{SubmittedUrl, UrlKey};
use crate::code::Code;

/// A store call that did not complete.
///
/// See design-doc/url-shortener/rfc.md, Redirect Resolver, steps 3 and 4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    /// The call passed its deadline.
    Timeout,
    /// A follower read's nearest replica is further behind than
    /// `max_staleness`.
    TooStale,
    /// The store could not serve the call.
    Unavailable,
}

/// A `links` row as the mechanisms need it: the submitted URL, never updated
/// (FR5), and whether an operator has disabled it (FR6).
///
/// See design-doc/url-shortener/rfc.md, Data model and storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The submitted spelling, sent as `Location` (FR2).
    pub url: Arc<str>,
    /// Set by an operator's disable; a lookup then answers `410` (FR6).
    pub disabled: bool,
}

/// Why an insert wrote nothing.
///
/// See design-doc/url-shortener/rfc.md, Link Creation, Phases 2 and 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertError {
    /// `links[code]` exists: the handler draws another code.
    CodeTaken,
    /// `url_index[key]` exists: a concurrent first submission won, and the
    /// handler reruns Phase 1.
    KeyTaken,
    /// The store failed; the handler retries a `Timeout` once.
    Store(StoreError),
}

/// The write side of the store.
///
/// See design-doc/url-shortener/rfc.md, Link Creation.
pub trait LinkStore: Send + Sync {
    /// Phase 1: `url_index[key]`, then `links[code]` for the code found.
    fn find_by_key(&self, key: &UrlKey) -> Result<Option<(Code, Link)>, StoreError>;

    /// Phases 2 and 3, atomic: inserts `links[code] = url`, then
    /// `url_index[key] = code`. On any error nothing is written. Never
    /// overwrites a row (FR5).
    fn insert(&self, code: &Code, key: &UrlKey, url: &SubmittedUrl) -> Result<(), InsertError>;
}

/// The read side of the store. Each call carries its own deadline: a
/// blocking call cannot be cut off by its caller, so the timeout belongs to
/// the store client, the way a statement timeout does.
///
/// See design-doc/url-shortener/rfc.md, Redirect Resolver.
pub trait LinkReader: Send + Sync {
    /// Follower read at the nearest replica; `TooStale` when that replica is
    /// further behind than `max_staleness`.
    fn read_local(
        &self,
        code: &Code,
        max_staleness: Duration,
        timeout: Duration,
    ) -> Result<Option<Link>, StoreError>;

    /// Consistent read at the leaseholder: sees every committed write.
    fn read_consistent(&self, code: &Code, timeout: Duration) -> Result<Option<Link>, StoreError>;
}

impl<T: LinkStore + ?Sized> LinkStore for &T {
    fn find_by_key(&self, key: &UrlKey) -> Result<Option<(Code, Link)>, StoreError> {
        (**self).find_by_key(key)
    }

    fn insert(&self, code: &Code, key: &UrlKey, url: &SubmittedUrl) -> Result<(), InsertError> {
        (**self).insert(code, key, url)
    }
}

impl<T: LinkStore + ?Sized> LinkStore for Arc<T> {
    fn find_by_key(&self, key: &UrlKey) -> Result<Option<(Code, Link)>, StoreError> {
        (**self).find_by_key(key)
    }

    fn insert(&self, code: &Code, key: &UrlKey, url: &SubmittedUrl) -> Result<(), InsertError> {
        (**self).insert(code, key, url)
    }
}

impl<T: LinkReader + ?Sized> LinkReader for &T {
    fn read_local(
        &self,
        code: &Code,
        max_staleness: Duration,
        timeout: Duration,
    ) -> Result<Option<Link>, StoreError> {
        (**self).read_local(code, max_staleness, timeout)
    }

    fn read_consistent(&self, code: &Code, timeout: Duration) -> Result<Option<Link>, StoreError> {
        (**self).read_consistent(code, timeout)
    }
}

impl<T: LinkReader + ?Sized> LinkReader for Arc<T> {
    fn read_local(
        &self,
        code: &Code,
        max_staleness: Duration,
        timeout: Duration,
    ) -> Result<Option<Link>, StoreError> {
        (**self).read_local(code, max_staleness, timeout)
    }

    fn read_consistent(&self, code: &Code, timeout: Duration) -> Result<Option<Link>, StoreError> {
        (**self).read_consistent(code, timeout)
    }
}

/// An in-memory store of one process. Both reads see the latest state, and
/// `insert` is atomic under one lock.
#[derive(Debug, Default)]
pub struct MemStore {
    tables: Mutex<Tables>,
}

#[derive(Debug, Default)]
struct Tables {
    links: HashMap<Code, Link>,
    url_index: HashMap<UrlKey, Code>,
}

impl MemStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }

    // A panic under this lock cannot leave an index entry without its row:
    // `insert` writes `links` before `url_index`, so the worst it leaves is
    // an unindexed code, and `disable` is one flag store.
    fn tables(&self) -> MutexGuard<'_, Tables> {
        self.tables.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn link(&self, code: &Code) -> Option<Link> {
        self.tables().links.get(code).cloned()
    }

    /// Marks the link disabled, as an operator's disable does; false when no
    /// link has `code`. Its URL is untouched (FR5).
    ///
    /// See design-doc/url-shortener/rfc.md, Disables.
    pub fn disable(&self, code: &Code) -> bool {
        match self.tables().links.get_mut(code) {
            Some(link) => {
                link.disabled = true;
                true
            }
            None => false,
        }
    }
}

impl LinkStore for MemStore {
    fn find_by_key(&self, key: &UrlKey) -> Result<Option<(Code, Link)>, StoreError> {
        let tables = self.tables();
        Ok(tables.url_index.get(key).map(|code| {
            let link = tables
                .links
                .get(code)
                .expect("insert writes both rows under one lock, and no row is removed");
            (*code, link.clone())
        }))
    }

    fn insert(&self, code: &Code, key: &UrlKey, url: &SubmittedUrl) -> Result<(), InsertError> {
        let mut tables = self.tables();
        if tables.links.contains_key(code) {
            return Err(InsertError::CodeTaken);
        }
        if tables.url_index.contains_key(key) {
            return Err(InsertError::KeyTaken);
        }
        tables.links.insert(
            *code,
            Link {
                url: url.as_str().into(),
                disabled: false,
            },
        );
        tables.url_index.insert(*key, *code);
        Ok(())
    }
}

impl LinkReader for MemStore {
    fn read_local(
        &self,
        code: &Code,
        _max_staleness: Duration,
        _timeout: Duration,
    ) -> Result<Option<Link>, StoreError> {
        Ok(self.link(code))
    }

    fn read_consistent(&self, code: &Code, _timeout: Duration) -> Result<Option<Link>, StoreError> {
        Ok(self.link(code))
    }
}

//! The core mechanisms of the URL shortener: the frozen v1 canonicalizer,
//! link creation over a store, and the redirect resolver's tiers.
//!
//! See design-doc/url-shortener/rfc.md, Work Breakdown, Tasks 1 to 3.

pub mod canon;
pub mod code;
pub mod create;
pub mod resolve;
pub mod store;

pub use canon::{Reject, SERVICE_DOMAIN, SubmittedUrl, UrlKey, canonical_form_v1, canonicalize_v1};
pub use code::{Code, CodeSource, OsCodes, RngError, code_from_bytes};
pub use create::{CreateError, Created, LinkCreator};
pub use resolve::{Clock, Resolution, Resolver, ResolverConfig, ResolverStats};
pub use store::{InsertError, Link, LinkReader, LinkStore, MemStore, StoreError};

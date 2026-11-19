//! The v1 canonicalizer: FR10's conversion, FR7's validation, FR4's
//! normalization and FR9's host and userinfo rejections, then the URL key.
//!
//! See design-doc/url-shortener/rfc.md, Canonicalizer.

use std::ops::Range;

use idna::AsciiDenyList;
use sha2::{Digest, Sha256};

/// The service's own domain, which FR9 forbids as a target host along with
/// its subdomains.
///
/// See design-doc/url-shortener/rfc.md, Terminologies.
pub const SERVICE_DOMAIN: &str = "s.example";

/// A long URL after FR10's conversion: ASCII and valid under FR7. It is what
/// the store keeps and what a redirect's `Location` carries, in the caller's
/// own spelling apart from the conversion.
///
/// See design-doc/url-shortener/requirement.md, FR2 and FR10.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SubmittedUrl(String);

impl SubmittedUrl {
    /// The converted URL as submitted.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// SHA-256 of a URL's canonical form, the `url_index` key: two URLs that are
/// the same under FR4 share a key, and different ones do not.
///
/// See design-doc/url-shortener/rfc.md, Data model and storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UrlKey([u8; 32]);

impl UrlKey {
    /// The 32 bytes of the SHA-256 digest.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Why a submission was rejected. Each maps to the API's `400` reason.
///
/// See design-doc/url-shortener/rfc.md, API; requirement.md, FR7 and FR9.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    /// The scheme is syntactically valid but is not `http` or `https` (FR7).
    BadScheme,
    /// No valid scheme, a string outside RFC 3986's `URI` rule, no authority
    /// or an empty host, or a host that UTS #46 ToASCII rejects (FR7, FR10).
    /// The host is judged on the submitted URL, so one that ToASCII maps to
    /// nothing, such as a lone soft hyphen, is empty.
    InvalidUrl,
    /// More than 2,048 characters after FR10's conversion (FR7).
    TooLong,
    /// The host, one trailing dot ignored, is `s.example` or a subdomain of
    /// it, either after FR4 or in one of its browser forms: its escapes
    /// decoded as UTF-8, then WHATWG domain to ASCII, as is (the current form)
    /// or after UTS #46's transitional pre-mapping (the transitional form). A
    /// host with no browser form is judged after FR4 only (FR9).
    ///
    /// See design-doc/url-shortener/rfc.md, Canonicalizer, step 4, and
    /// Appendix, Browser forms of a host.
    OwnDomain,
    /// The authority holds a user or password part, even an empty one (FR9).
    Userinfo,
}

impl Reject {
    /// The API's reason string: `bad_scheme`, `invalid_url`, `too_long`,
    /// `own_domain` or `userinfo`.
    ///
    /// See design-doc/url-shortener/rfc.md, API.
    pub fn reason(&self) -> &'static str {
        match self {
            Reject::BadScheme => "bad_scheme",
            Reject::InvalidUrl => "invalid_url",
            Reject::TooLong => "too_long",
            Reject::OwnDomain => "own_domain",
            Reject::Userinfo => "userinfo",
        }
    }
}

/// Converts, validates, normalizes and checks `raw`, returning the submitted
/// URL and its key. Steps, in order: convert (FR10), validate (FR7: scheme
/// present, scheme `http`/`https`, RFC 3986 grammar, a non-empty host,
/// length), normalize (FR4), reject (FR9: userinfo, then own domain, judged on
/// the normalized host and on its current and transitional browser forms),
/// hash. The browser forms only reject: neither enters the canonical form.
///
/// Frozen: `url_index` keys depend on its output. Any change to its output,
/// a bug fix included, re-keys stored URLs and is a v2 with a backfill.
///
/// See design-doc/url-shortener/rfc.md, Canonicalizer, Versioning.
pub fn canonicalize_v1(raw: &str) -> Result<(SubmittedUrl, UrlKey), Reject> {
    let (url, form) = canonical_form_v1(raw)?;
    Ok((url, UrlKey(Sha256::digest(form.as_bytes()).into())))
}

/// The same steps as [`canonicalize_v1`], returning the canonical string the
/// key hashes in place of the key, so golden tests can show which FR4 rule
/// an output breaks.
///
/// See design-doc/url-shortener/rfc.md, Canonicalizer.
pub fn canonical_form_v1(raw: &str) -> Result<(SubmittedUrl, String), Reject> {
    let submitted = convert(raw)?;
    let parts = validate(&submitted)?;
    let form = normalize_and_judge(&parts)?;
    Ok((SubmittedUrl(submitted), form))
}

/// FR7's length limit, in characters of the submitted URL.
const MAX_LEN: usize = 2_048;

/// A URL cut into RFC 3986 §3's components, borrowed from the string.
struct Parts<'a> {
    scheme: &'a str,
    authority: Option<Authority<'a>>,
    path: &'a str,
    query: Option<&'a str>,
    fragment: Option<&'a str>,
}

struct Authority<'a> {
    userinfo: Option<&'a str>,
    host: &'a str,
    /// Where `host` starts in the whole string, so step 1 can replace it.
    host_range: Range<usize>,
    /// The text after the host's `:`; `Some("")` for an empty port.
    port: Option<&'a str>,
}

/// Cuts `s` at its delimiters, or `None` when it has no syntactically valid
/// `scheme ":"` prefix. Says nothing about whether the components are valid.
fn split(s: &str) -> Option<Parts<'_>> {
    let colon = s.find(':')?;
    let scheme = &s[..colon];
    if !is_scheme(scheme) {
        return None;
    }
    let rest = &s[colon + 1..];
    let (rest, fragment) = match rest.find('#') {
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
        None => (rest, None),
    };
    let (hier, query) = match rest.find('?') {
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
        None => (rest, None),
    };
    let (authority, path) = match hier.strip_prefix("//") {
        Some(after) => {
            let end = after.find('/').unwrap_or(after.len());
            let at = colon + "://".len();
            (Some(split_authority(&after[..end], at)), &after[end..])
        }
        None => (None, hier),
    };
    Some(Parts {
        scheme,
        authority,
        path,
        query,
        fragment,
    })
}

/// Cuts an authority that starts at byte `at` of the whole string.
fn split_authority(a: &str, at: usize) -> Authority<'_> {
    let (userinfo, host_port, host_at) = match a.rfind('@') {
        Some(i) => (Some(&a[..i]), &a[i + 1..], at + i + 1),
        None => (None, a, at),
    };
    let host_len = if host_port.starts_with('[') {
        // An IP literal ends at its `]`, unless what follows is not a port;
        // then the whole text is taken as the host and fails validation.
        match host_port.find(']') {
            Some(i) if host_port[i + 1..].is_empty() || host_port[i + 1..].starts_with(':') => {
                i + 1
            }
            _ => host_port.len(),
        }
    } else {
        host_port.find(':').unwrap_or(host_port.len())
    };
    Authority {
        userinfo,
        host: &host_port[..host_len],
        host_range: host_at..host_at + host_len,
        port: host_port[host_len..].strip_prefix(':'),
    }
}

/// Step 1, FR10. A host with non-ASCII goes through UTS #46 ToASCII,
/// nontransitional; every other non-ASCII character is percent-encoded as
/// UTF-8. ASCII is never touched.
///
/// The ToASCII deny list is the WHATWG one (`%#/:<>?@[\]^|`, controls and
/// space), checked after mapping. Without it, a full-width `／`, `＠` or `：`
/// would map to a delimiter and move the URL's component boundaries after
/// the caller's text was read; with it, such a host is `InvalidUrl`.
fn convert(raw: &str) -> Result<String, Reject> {
    let mut out = String::with_capacity(raw.len());
    let host = split(raw).and_then(|p| p.authority).map(|a| a.host_range);
    match host {
        Some(r) if !raw[r.clone()].is_ascii() && !raw[r.clone()].starts_with('[') => {
            percent_encode_non_ascii(&raw[..r.start], &mut out);
            let ascii = idna::domain_to_ascii_cow(raw[r.clone()].as_bytes(), AsciiDenyList::URL)
                .map_err(|_| Reject::InvalidUrl)?;
            out.push_str(&ascii);
            percent_encode_non_ascii(&raw[r.end..], &mut out);
        }
        _ => percent_encode_non_ascii(raw, &mut out),
    }
    Ok(out)
}

fn percent_encode_non_ascii(s: &str, out: &mut String) {
    for ch in s.chars() {
        if ch.is_ascii() {
            out.push(ch);
        } else {
            let mut buf = [0; 4];
            for b in ch.encode_utf8(&mut buf).bytes() {
                push_escape(b, out);
            }
        }
    }
}

/// Step 2, FR7, in order: a valid scheme prefix, `http` or `https`, the
/// RFC 3986 `URI` rule, then the length.
fn validate(s: &str) -> Result<Parts<'_>, Reject> {
    let parts = split(s).ok_or(Reject::InvalidUrl)?;
    if !parts.scheme.eq_ignore_ascii_case("http") && !parts.scheme.eq_ignore_ascii_case("https") {
        return Err(Reject::BadScheme);
    }
    if !follows_uri_rule(&parts) {
        return Err(Reject::InvalidUrl);
    }
    if s.len() > MAX_LEN {
        return Err(Reject::TooLong);
    }
    Ok(parts)
}

/// RFC 3986 §3 for a string `split` accepted: `hier-part`, query and
/// fragment. `split` already ensures a path without an authority does not
/// start with `//`.
fn follows_uri_rule(p: &Parts<'_>) -> bool {
    if let Some(a) = &p.authority {
        let userinfo_ok = a.userinfo.is_none_or(|u| only(u, b":"));
        let host_ok = match a.host.strip_prefix('[') {
            Some(literal) => literal.strip_suffix(']').is_some_and(is_ip_literal_body),
            None => only(a.host, b""),
        };
        let port_ok = a
            .port
            .is_none_or(|port| port.bytes().all(|b| b.is_ascii_digit()));
        if !(userinfo_ok && host_ok && port_ok) {
            return false;
        }
    }
    only(p.path, b":@/")
        && p.query.is_none_or(|q| only(q, b":@/?"))
        && p.fragment.is_none_or(|f| only(f, b":@/?"))
}

/// Whether `s` is made only of unreserved characters, sub-delims, complete
/// percent-escapes and the bytes in `extra`.
fn only(s: &str, extra: &[u8]) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            if !(i + 2 < b.len() && b[i + 1].is_ascii_hexdigit() && b[i + 2].is_ascii_hexdigit()) {
                return false;
            }
            i += 3;
        } else if is_unreserved(b[i]) || is_sub_delim(b[i]) || extra.contains(&b[i]) {
            i += 1;
        } else {
            return false;
        }
    }
    true
}

/// `scheme = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )`.
fn is_scheme(s: &str) -> bool {
    let mut bytes = s.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_alphabetic())
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
}

fn is_unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
}

fn is_sub_delim(b: u8) -> bool {
    matches!(
        b,
        b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'='
    )
}

/// The text between `[` and `]`: `IPv6address / IPvFuture`.
fn is_ip_literal_body(s: &str) -> bool {
    match s.strip_prefix(['v', 'V']) {
        Some(future) => is_ipv_future_tail(future),
        None => is_ipv6(s),
    }
}

/// `IPvFuture` after its `v`: `1*HEXDIG "." 1*( unreserved / sub-delims / ":" )`.
fn is_ipv_future_tail(s: &str) -> bool {
    let Some((version, rest)) = s.split_once('.') else {
        return false;
    };
    !version.is_empty()
        && version.bytes().all(|b| b.is_ascii_hexdigit())
        && !rest.is_empty()
        && rest
            .bytes()
            .all(|b| is_unreserved(b) || is_sub_delim(b) || b == b':')
}

/// RFC 3986's `IPv6address`. Its nine alternatives come to: groups of 1 to 4
/// hex digits split by `:`, an IPv4 address allowed only as the last two
/// groups, and either exactly 8 groups or at most 7 around a single `::`.
fn is_ipv6(s: &str) -> bool {
    let (left, right) = match s.split_once("::") {
        Some((_, r)) if r.contains("::") => return false,
        Some((l, r)) => (l, Some(r)),
        None => (s, None),
    };
    let groups = |side: &str, may_end_in_ipv4: bool| -> Option<usize> {
        if side.is_empty() {
            return Some(0);
        }
        let pieces: Vec<&str> = side.split(':').collect();
        let mut count = 0;
        for (i, piece) in pieces.iter().enumerate() {
            let last = i + 1 == pieces.len();
            if last && may_end_in_ipv4 && piece.contains('.') {
                is_ipv4(piece).then_some(())?;
                count += 2;
            } else if (1..=4).contains(&piece.len()) && piece.bytes().all(|b| b.is_ascii_hexdigit())
            {
                count += 1;
            } else {
                return None;
            }
        }
        Some(count)
    };
    match right {
        None => groups(left, true) == Some(8),
        Some(right) => match (groups(left, false), groups(right, true)) {
            (Some(l), Some(r)) => l + r <= 7,
            _ => false,
        },
    }
}

/// `IPv4address = dec-octet "." dec-octet "." dec-octet "." dec-octet`, each
/// octet 0 to 255 with no leading zero.
fn is_ipv4(s: &str) -> bool {
    let octets: Vec<&str> = s.split('.').collect();
    octets.len() == 4
        && octets.iter().all(|o| {
            !o.is_empty()
                && o.len() <= 3
                && o.bytes().all(|b| b.is_ascii_digit())
                && !(o.len() > 1 && o.starts_with('0'))
                && o.parse::<u16>().is_ok_and(|v| v <= 255)
        })
}

/// Steps 3 and 4, FR4 then FR9: the canonical string, or the rejection.
fn normalize_and_judge(p: &Parts<'_>) -> Result<String, Reject> {
    let scheme = p.scheme.to_ascii_lowercase();
    let mut form = String::with_capacity(MAX_LEN);
    form.push_str(&scheme);
    form.push(':');
    match &p.authority {
        Some(a) => {
            if a.userinfo.is_some() {
                return Err(Reject::Userinfo);
            }
            let mut host = String::with_capacity(a.host.len());
            normalize_escapes(a.host, true, &mut host);
            if is_own_domain(&host) {
                return Err(Reject::OwnDomain);
            }
            form.push_str("//");
            form.push_str(&host);
            if let Some(port) = a.port.filter(|port| !is_default_port(&scheme, port)) {
                form.push(':');
                form.push_str(port);
            }
            let mut path = String::with_capacity(p.path.len());
            normalize_escapes(p.path, false, &mut path);
            let path = remove_dot_segments(&path);
            form.push_str(if path.is_empty() { "/" } else { &path });
        }
        // A hostless URL (impl-run.md, D1) keeps its dot-segments. Removing
        // them could start the path with `//`, as `/.//x/y` becomes
        // `//x/y`, and the form would then spell `http://x/y`, another URL
        // that would share its key (NFR7).
        None => normalize_escapes(p.path, false, &mut form),
    }
    if let Some(q) = p.query {
        form.push('?');
        normalize_escapes(q, false, &mut form);
    }
    if let Some(f) = p.fragment {
        form.push('#');
        normalize_escapes(f, false, &mut form);
    }
    Ok(form)
}

/// RFC 3986 §6.2.2.1 and §6.2.2.2 on validated text: an escape of an
/// unreserved character is decoded, every other escape gets uppercase hex.
/// With `lowercase`, the host's case rule: every letter outside an escape
/// that remains is lowercased, decoded ones included.
fn normalize_escapes(s: &str, lowercase: bool, out: &mut String) {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let (byte, escaped) = if b[i] == b'%' {
            let v = hex_value(b[i + 1]) << 4 | hex_value(b[i + 2]);
            i += 3;
            (v, !is_unreserved(v))
        } else {
            i += 1;
            (b[i - 1], false)
        };
        if escaped {
            push_escape(byte, out);
        } else if lowercase {
            out.push(char::from(byte.to_ascii_lowercase()));
        } else {
            out.push(char::from(byte));
        }
    }
}

fn hex_value(h: u8) -> u8 {
    match h {
        b'0'..=b'9' => h - b'0',
        b'a'..=b'f' => h - b'a' + 10,
        _ => h - b'A' + 10,
    }
}

fn push_escape(byte: u8, out: &mut String) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push('%');
    out.push(char::from(HEX[usize::from(byte >> 4)]));
    out.push(char::from(HEX[usize::from(byte & 0xF)]));
}

/// RFC 3986 §3.2.3 omits a port equal to the scheme's default. It is judged
/// by decimal value, so `:080` is the default; any other port keeps its
/// digits as written (impl.md, Mechanisms, 1, the decision on ports).
fn is_default_port(scheme: &str, port: &str) -> bool {
    let value = port.trim_start_matches('0');
    port.is_empty() || (scheme == "http" && value == "80") || (scheme == "https" && value == "443")
}

/// FR9 on the normalized host, one trailing dot ignored.
fn is_own_domain(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    host.strip_suffix(SERVICE_DOMAIN)
        .is_some_and(|label| label.is_empty() || label.ends_with('.'))
}

/// RFC 3986 §5.2.4.
fn remove_dot_segments(mut input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    while !input.is_empty() {
        if let Some(rest) = input.strip_prefix("../") {
            input = rest;
        } else if let Some(rest) = input.strip_prefix("./") {
            input = rest;
        } else if input.starts_with("/./") {
            input = &input[2..];
        } else if input == "/." {
            input = "/";
        } else if input.starts_with("/../") {
            input = &input[3..];
            truncate_last_segment(&mut output);
        } else if input == "/.." {
            input = "/";
            truncate_last_segment(&mut output);
        } else if input == "." || input == ".." {
            input = "";
        } else {
            let from = usize::from(input.starts_with('/'));
            let end = input[from..].find('/').map_or(input.len(), |i| i + from);
            output.push_str(&input[..end]);
            input = &input[end..];
        }
    }
    output
}

fn truncate_last_segment(output: &mut String) {
    let cut = output.rfind('/').unwrap_or(0);
    output.truncate(cut);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR10 with the WHATWG deny list: a non-ASCII host whose mapping yields
    /// a delimiter is invalid rather than re-cut into other components.
    #[test]
    fn mapped_delimiters_in_a_host_are_invalid() {
        for raw in [
            "http://a\u{ff0f}s.example/",
            "http://u\u{ff20}\u{fc}.example/",
            "http://\u{fc}\u{ff1a}80/",
            "http://\u{fc}%41.example/",
        ] {
            assert_eq!(
                canonicalize_v1(raw).map(|_| ()),
                Err(Reject::InvalidUrl),
                "{raw:?}"
            );
        }
    }

    /// RFC 3986 §3.2.2's nine `IPv6address` alternatives and `IPvFuture`.
    #[test]
    fn ip_literal_grammar() {
        for ok in [
            "1:2:3:4:5:6:7:8",
            "1:2:3:4:5:6:1.2.3.4",
            "::1:2:3:4:5:6:7",
            "1:2:3:4:5:6:7::",
            "1::8",
            "::",
            "::ffff:255.255.255.255",
            "1:2:3:4:5::1.2.3.4",
            "vA.x:y",
        ] {
            assert!(is_ip_literal_body(ok), "{ok:?}");
        }
        for bad in [
            "1:2:3:4:5:6:7",
            "1:2:3:4:5:6:7:8::",
            "::1:2:3:4:5:6:7:8",
            ":1::2",
            "1::2:",
            ":::",
            "12345::",
            "1.2.3.4::",
            "::1.2.3.4:1",
            "::01.2.3.4",
            "::256.1.1.1",
            "1:2:3:4:5:6:7:1.2.3.4",
            "v.x",
            "vA.",
            "vG.x",
        ] {
            assert!(!is_ip_literal_body(bad), "{bad:?}");
        }
    }

    /// RFC 3986 §5.2.4's own examples.
    #[test]
    fn remove_dot_segments_matches_the_rfc_examples() {
        assert_eq!(remove_dot_segments("/a/b/c/./../../g"), "/a/g");
        assert_eq!(remove_dot_segments("mid/content=5/../6"), "mid/6");
        assert_eq!(remove_dot_segments("/.."), "/");
        assert_eq!(remove_dot_segments("/a/..b"), "/a/..b");
    }
}

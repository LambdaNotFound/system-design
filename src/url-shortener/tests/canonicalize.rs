//! The frozen v1 canonicalizer. These goldens are the data format of
//! `url_index`: a change to any expected string here re-keys stored URLs.
//!
//! See design-doc/url-shortener/rfc.md, Canonicalizer.

use url_shortener::{Reject, canonical_form_v1, canonicalize_v1};

/// (input, submitted URL, canonical form). Each block exercises one FR4 or
/// FR10 rule.
#[rustfmt::skip]
const GOLDEN: &[(&str, &str, &str)] = &[
    // Scheme case.
    ("HTTP://example.com/", "HTTP://example.com/", "http://example.com/"),
    ("HtTpS://example.com/", "HtTpS://example.com/", "https://example.com/"),
    // Host case.
    ("http://EXAMPLE.com/", "http://EXAMPLE.com/", "http://example.com/"),
    // Hex case, with reserved and non-ASCII escapes kept encoded.
    ("http://example.com/a%2fb%c3%a9", "http://example.com/a%2fb%c3%a9", "http://example.com/a%2Fb%C3%A9"),
    ("http://example.com/%20x", "http://example.com/%20x", "http://example.com/%20x"),
    // Unreserved escapes decoded, every unreserved kind.
    ("http://example.com/%7Euser", "http://example.com/%7Euser", "http://example.com/~user"),
    ("http://example.com/%7e%2D%2e%5F%30%41%7A", "http://example.com/%7e%2D%2e%5F%30%41%7A", "http://example.com/~-._0Az"),
    // An escape in the host decodes, then lowercases.
    ("http://%41.example/", "http://%41.example/", "http://a.example/"),
    // An escape kept in the host has uppercase hex; only the host's other
    // letters are lowercased (RFC 3986 §3.2.2, §6.2.2.1).
    ("http://A%2ab.example/", "http://A%2ab.example/", "http://a%2Ab.example/"),
    // Dot-segments, after the decode.
    ("http://example.com/a/./b/../c", "http://example.com/a/./b/../c", "http://example.com/a/c"),
    ("http://example.com/a/%2E%2E/b", "http://example.com/a/%2E%2E/b", "http://example.com/b"),
    ("http://example.com/a/%2e/b", "http://example.com/a/%2e/b", "http://example.com/a/b"),
    ("http://example.com/../a", "http://example.com/../a", "http://example.com/a"),
    ("http://example.com/a/b/..", "http://example.com/a/b/..", "http://example.com/a/"),
    // Dot-segments only in the path.
    ("http://example.com/p?a/../b#c/./d", "http://example.com/p?a/../b#c/./d", "http://example.com/p?a/../b#c/./d"),
    // Default ports dropped, only for their own scheme.
    ("http://example.com:80/x", "http://example.com:80/x", "http://example.com/x"),
    ("https://example.com:443/x", "https://example.com:443/x", "https://example.com/x"),
    ("http://example.com:443/x", "http://example.com:443/x", "http://example.com:443/x"),
    ("https://example.com:80/x", "https://example.com:80/x", "https://example.com:80/x"),
    ("http://example.com:8080/x", "http://example.com:8080/x", "http://example.com:8080/x"),
    // The default port is judged by its decimal value, so leading zeros drop
    // with it; any other port keeps its digits as written, even out of range.
    ("http://example.com:080/x", "http://example.com:080/x", "http://example.com/x"),
    ("https://example.com:0443/x", "https://example.com:0443/x", "https://example.com/x"),
    ("http://example.com:0443/x", "http://example.com:0443/x", "http://example.com:0443/x"),
    ("http://example.com:08080/x", "http://example.com:08080/x", "http://example.com:08080/x"),
    ("http://example.com:99999/x", "http://example.com:99999/x", "http://example.com:99999/x"),
    // An empty port dropped.
    ("http://example.com:/x", "http://example.com:/x", "http://example.com/x"),
    // An empty path with an authority becomes `/`.
    ("http://example.com", "http://example.com", "http://example.com/"),
    ("http://example.com:80", "http://example.com:80", "http://example.com/"),
    ("http://example.com?q=1", "http://example.com?q=1", "http://example.com/?q=1"),
    ("http://example.com#f", "http://example.com#f", "http://example.com/#f"),
    // Query and fragment kept, normalized by the percent rules only.
    ("http://example.com/p?Q=%7e&b=%2f#Frag%41", "http://example.com/p?Q=%7e&b=%2f#Frag%41", "http://example.com/p?Q=~&b=%2F#FragA"),
    // An empty query or fragment keeps its delimiter.
    ("http://example.com/?", "http://example.com/?", "http://example.com/?"),
    ("http://example.com/#", "http://example.com/#", "http://example.com/#"),
    // A trailing dot in the host is kept.
    ("http://example.com./", "http://example.com./", "http://example.com./"),
    // The RFC's Example.
    ("HTTPS://Example.com:443/a/../b", "HTTPS://Example.com:443/a/../b", "https://example.com/b"),
    // FR10: a non-ASCII host through ToASCII, other non-ASCII as UTF-8 escapes.
    ("http://Bücher.example/", "http://xn--bcher-kva.example/", "http://xn--bcher-kva.example/"),
    // Nontransitional: ß is kept, not mapped to ss.
    ("http://faß.example/", "http://xn--fa-hia.example/", "http://xn--fa-hia.example/"),
    ("http://example.com/café?q=ü#ß", "http://example.com/caf%C3%A9?q=%C3%BC#%C3%9F", "http://example.com/caf%C3%A9?q=%C3%BC#%C3%9F"),
    // IP literals.
    ("http://[2001:DB8::1]:80/", "http://[2001:DB8::1]:80/", "http://[2001:db8::1]/"),
    ("http://[::1]:8080/", "http://[::1]:8080/", "http://[::1]:8080/"),
    ("http://127.0.0.1/", "http://127.0.0.1/", "http://127.0.0.1/"),
];

fn canonical(raw: &str) -> String {
    match canonical_form_v1(raw) {
        Ok((_, form)) => form,
        Err(reject) => panic!("{raw:?} was rejected as {reject:?}"),
    }
}

fn submitted(raw: &str) -> String {
    match canonicalize_v1(raw) {
        Ok((url, _)) => url.as_str().to_owned(),
        Err(reject) => panic!("{raw:?} was rejected as {reject:?}"),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn assert_rejected(raw: &str, want: Reject) {
    assert_eq!(canonicalize_v1(raw).map(|_| ()), Err(want), "for {raw:?}");
}

fn assert_accepted(raw: &str) {
    if let Err(reject) = canonicalize_v1(raw) {
        panic!("{raw:?} was rejected as {reject:?}");
    }
}

/// `None` when `raw` gets `want`; otherwise a line for a failure message
/// that lists every wrong case, with a long input cut short.
fn mismatch(raw: &str, want: Result<(), Reject>) -> Option<String> {
    let got = canonicalize_v1(raw).map(|_| ());
    (got != want).then(|| {
        let shown = match raw.char_indices().nth(40) {
            Some((cut, _)) => format!("{:?}... ({} bytes)", &raw[..cut], raw.len()),
            None => format!("{raw:?}"),
        };
        format!("{shown}: got {got:?}, want {want:?}")
    })
}

fn same_key(a: &str, b: &str) -> bool {
    let (_, ka) = canonicalize_v1(a).unwrap_or_else(|r| panic!("{a:?} rejected as {r:?}"));
    let (_, kb) = canonicalize_v1(b).unwrap_or_else(|r| panic!("{b:?} rejected as {r:?}"));
    ka == kb
}

/// Proves FR4: each normalization rule, and only those, applies.
#[test]
fn golden_canonical_forms_v1() {
    for &(raw, want_submitted, want_canonical) in GOLDEN {
        let (url, form) =
            canonical_form_v1(raw).unwrap_or_else(|r| panic!("{raw:?} was rejected as {r:?}"));
        assert_eq!(url.as_str(), want_submitted, "submitted URL of {raw:?}");
        assert_eq!(form, want_canonical, "canonical form of {raw:?}");
        assert_eq!(submitted(raw), want_submitted, "canonicalize_v1 of {raw:?}");
    }
}

/// Proves FR4: the key is SHA-256 of the canonical form. The expected keys
/// were computed outside the crate with `printf '%s' <form> | shasum -a 256`.
#[test]
fn golden_url_keys_v1() {
    let cases = [
        (
            "HTTPS://Example.com:443/a/../b",
            "d7fe568b31ae0fb9ee8a1311409b521aeb6a1b50f459bdf15c6f4a3f29dde53b",
        ),
        (
            "http://Bücher.example/",
            "ef6bde6ded919b75d751a0960d3c96ff54a2c01a2a6887d92a1a5b34dbb28e70",
        ),
        (
            "http://example.com/café?q=ü#ß",
            "631b5e21501b2cfd13468cba6c8a46d35e7310e6277794cb297d4dbd0983a54c",
        ),
        (
            "http://faß.example/",
            "e52ad9f0d40a9d8f631cfcf9ea176ed8ad6408b7764f0c956f895310c0c8e4f5",
        ),
    ];
    for (raw, want) in cases {
        let (_, key) = canonicalize_v1(raw).unwrap_or_else(|r| panic!("{raw:?} rejected as {r:?}"));
        assert_eq!(hex(key.as_bytes()), want, "key of {raw:?}");
    }
}

/// Proves FR4 at its boundary: URLs equal after normalization share a key,
/// and every difference normalization does not remove gives another key.
#[test]
fn equal_urls_share_a_key_and_different_ones_do_not() {
    let equal = [
        ("HTTPS://Example.com:443/a/../b", "https://example.com/b"),
        ("http://example.com/%7e", "http://example.com/~"),
        ("http://example.com/%2f", "http://example.com/%2F"),
        ("http://example.com:80/x", "http://example.com/x"),
        ("HTTP://EXAMPLE.COM", "http://example.com/"),
        ("http://a%2ab.example/", "http://a%2Ab.example/"),
        ("http://example.com:080/", "http://example.com/"),
        ("https://example.com:0443/", "https://example.com/"),
    ];
    for (a, b) in equal {
        assert!(same_key(a, b), "{a:?} and {b:?} should share a key");
    }

    let different = [
        ("http://example.com/A", "http://example.com/a"),
        ("http://example.com/a%2Fb", "http://example.com/a/b"),
        ("http://example.com/a", "http://example.com/a/"),
        ("http://example.com/", "https://example.com/"),
        ("http://example.com:8080/", "http://example.com/"),
        ("http://example.com:08080/", "http://example.com:8080/"),
        ("http://faß.example/", "http://fass.example/"),
        ("http://example.com/?q=1", "http://example.com/?q=2"),
        ("http://example.com/?q=1", "http://example.com/?Q=1"),
        ("http://example.com/?", "http://example.com/"),
        ("http://example.com/#a", "http://example.com/"),
        ("http://example.com./", "http://example.com/"),
        // FR10 escapes UTF-8 bytes as given; only the host is NFC-normalized.
        (
            "http://example.com/caf\u{e9}",
            "http://example.com/cafe\u{301}",
        ),
    ];
    for (a, b) in different {
        assert!(
            !same_key(a, b),
            "{a:?} and {b:?} should have different keys"
        );
    }
}

/// Proves FR3 and NFR7: canonicalizing a submitted URL returns it and its key
/// unchanged, so Phase 1's re-check of a stored URL reproduces the key.
#[test]
fn submitted_url_is_a_fixed_point() {
    for &(raw, _, _) in GOLDEN {
        let (url, key) =
            canonicalize_v1(raw).unwrap_or_else(|r| panic!("{raw:?} rejected as {r:?}"));
        let (again, again_key) = canonicalize_v1(url.as_str())
            .unwrap_or_else(|r| panic!("submitted {:?} rejected as {r:?}", url.as_str()));
        assert_eq!(again, url, "submitted URL of {raw:?} is not a fixed point");
        assert_eq!(
            again_key, key,
            "key of {raw:?} changes on re-canonicalizing"
        );
    }
}

/// Proves FR2 and FR10: the submitted URL is the caller's spelling, with
/// only non-ASCII converted; an ASCII host keeps its case.
#[test]
fn submitted_url_keeps_the_callers_spelling() {
    for raw in [
        "HTTPS://Example.com:443/a/../b",
        "http://Example.COM/A%7e?Q=%2f#F",
        "http://example.com:/x",
    ] {
        assert_eq!(submitted(raw), raw);
    }
}

/// Proves FR10 and FR9: a non-ASCII host goes through UTS #46 ToASCII,
/// nontransitional, which maps case and width and applies NFC before FR9
/// looks at it.
#[test]
fn non_ascii_host_goes_through_idna() {
    assert_eq!(
        submitted("http://Bücher.example/"),
        "http://xn--bcher-kva.example/"
    );
    assert_eq!(
        submitted("http://Bu\u{308}cher.example/"),
        "http://xn--bcher-kva.example/",
        "decomposed ü must give the composed form's host"
    );
    assert!(same_key(
        "http://B\u{fc}cher.example/",
        "http://Bu\u{308}cher.example/"
    ));
    assert_eq!(
        submitted("http://Bücher.example:8080/p"),
        "http://xn--bcher-kva.example:8080/p"
    );
    assert_eq!(
        submitted("http://faß.example/"),
        "http://xn--fa-hia.example/",
        "transitional processing would give fass.example"
    );
    // Full-width letters and dots map to the service's own domain.
    assert_rejected("http://\u{ff53}.example/", Reject::OwnDomain);
    assert_rejected("http://\u{ff53}\u{ff0e}example/", Reject::OwnDomain);
}

/// Proves FR10: non-ASCII outside the host is percent-encoded as UTF-8 with
/// uppercase hex, and ASCII is never encoded.
#[test]
fn non_ascii_elsewhere_is_percent_encoded_as_utf8() {
    assert_eq!(
        submitted("http://example.com/café?q=ü#ß"),
        "http://example.com/caf%C3%A9?q=%C3%BC#%C3%9F"
    );
    assert_eq!(
        submitted("http://example.com/€"),
        "http://example.com/%E2%82%AC"
    );
    assert_eq!(
        submitted("http://example.com/\u{1f600}"),
        "http://example.com/%F0%9F%98%80"
    );
    // A space is ASCII, so it is not encoded and stays invalid.
    assert_rejected("http://example.com/café au lait", Reject::InvalidUrl);
}

/// Proves FR7: only http and https, in any case; a missing or malformed
/// scheme is invalid; the scheme is judged before the grammar.
#[test]
fn scheme_must_be_http_or_https() {
    assert_accepted("HtTpS://example.com/");
    assert_accepted("HTTP://example.com/");
    for raw in [
        "ftp://example.com/",
        "javascript:alert(1)",
        "mailto:a@example.com",
        "example.com:80/x",
        "httpx://example.com/",
        "ftp://example.com/a b",
    ] {
        assert_rejected(raw, Reject::BadScheme);
    }
    for raw in [
        "",
        "example.com/x",
        "//example.com/x",
        "1http://example.com/",
        ":x",
    ] {
        assert_rejected(raw, Reject::InvalidUrl);
    }
}

/// Proves FR7: a string outside RFC 3986's `URI` rule, or with a host that
/// ToASCII rejects, is invalid.
#[test]
fn invalid_rfc3986_is_rejected() {
    for raw in [
        "http://example.com/a b",
        "http://exa mple.com/",
        "http://example.com/<x>",
        "http://example.com/^",
        "http:\\\\example.com\\",
        "http://example.com/%zz",
        "http://example.com/%4",
        "http://example.com/#a#b",
        "http://[::1/",
        "http://[::g]/",
        "http://[1:2:3:4:5:6:7:8:9]/",
        "http://[1::2::3]/",
        "http://example.com:8o/",
        // A label that starts with a combining mark, and a joiner out of
        // context: errors under every UTS #46 option.
        "http://\u{301}b.example/",
        "http://a\u{200d}b.example/",
    ] {
        assert_rejected(raw, Reject::InvalidUrl);
    }
}

/// Proves FR7 and FR4: IP literals are hosts; IPv6 hex is lowercased in the
/// canonical form only.
#[test]
fn ip_literals_are_hosts() {
    assert_eq!(canonical("http://[::1]:8080/"), "http://[::1]:8080/");
    assert_eq!(canonical("http://[::1]/"), "http://[::1]/");
    assert_eq!(canonical("http://[2001:DB8::1]/"), "http://[2001:db8::1]/");
    assert_eq!(submitted("http://[2001:DB8::1]/"), "http://[2001:DB8::1]/");
    assert_eq!(
        canonical("http://[::ffff:192.0.2.1]/"),
        "http://[::ffff:192.0.2.1]/"
    );
    assert_eq!(canonical("http://127.0.0.1:80/"), "http://127.0.0.1/");
}

/// Proves FR7 and FR10: the limit is 2,048 characters of the submitted URL,
/// after the conversion and before normalization, checked after the grammar
/// and before FR9.
#[test]
fn length_is_judged_after_conversion() {
    let prefix = "http://example.com/";
    assert_eq!(prefix.len(), 19);

    assert_accepted(&format!("{prefix}{}", "a".repeat(2_029)));
    assert_rejected(&format!("{prefix}{}", "a".repeat(2_030)), Reject::TooLong);

    // 358 characters raw; 338 × 6 escape characters + 1 after conversion.
    let accented = format!("{prefix}{}", "é".repeat(338));
    assert_accepted(&format!("{accented}a"));
    assert_rejected(&format!("{accented}aa"), Reject::TooLong);

    // Normalizing would shorten it under the limit; the submitted URL counts.
    assert_rejected(&format!("{prefix}{}aa", "%7E".repeat(676)), Reject::TooLong);

    // The grammar is checked first, FR9 after.
    assert_rejected(
        &format!("{prefix} {}", "a".repeat(2_030)),
        Reject::InvalidUrl,
    );
    assert_rejected(
        &format!("http://u@example.com/{}", "a".repeat(2_030)),
        Reject::TooLong,
    );
    assert_rejected(
        &format!("http://s.example/{}", "a".repeat(2_040)),
        Reject::TooLong,
    );
}

/// Proves FR9: the service's domain and its subdomains are rejected after
/// FR4, with one trailing dot ignored for this check; look-alikes are not.
#[test]
fn own_domain_and_subdomains_are_rejected() {
    for raw in [
        "http://s.example/",
        "https://S.EXAMPLE/x",
        "http://s.example./",
        "http://a.b.s.example/x",
        "http://%73.example/",
        "https://s.example:443/",
        "http://s.example:8080/",
    ] {
        assert_rejected(raw, Reject::OwnDomain);
    }
    for raw in [
        "http://nots.example/",
        "http://s.example.com/",
        "http://s.example../",
        "http://s-example/",
        "http://example.com/s.example",
    ] {
        assert_accepted(raw);
    }
}

/// Proves FR9 as amended and FR4: a host is also judged as a browser
/// resolves it, its escapes decoded as UTF-8 and mapped, so `%EF%BD%93`
/// (U+FF53) counts as `s`. The browser forms only reject: an accepted host
/// keeps its escaped canonical form and its key.
#[test]
fn percent_encoded_host_is_judged_as_a_browser_resolves_it() {
    assert_rejected("http://%ef%bd%93.example/", Reject::OwnDomain);
    assert_rejected("http://%EF%BD%93.example/", Reject::OwnDomain);
    // An escape of an unreserved character decodes under FR4 too.
    assert_rejected("http://%73.example/", Reject::OwnDomain);

    assert_eq!(
        canonical("http://%c3%bc.example/"),
        "http://%C3%BC.example/"
    );
    assert_eq!(
        submitted("http://%c3%bc.example/"),
        "http://%c3%bc.example/"
    );
    assert!(same_key("http://%c3%bc.example/", "http://%C3%BC.example/"));
    // Its browser form is `xn--tda.example`, the host of `ü.example`, yet the
    // two keep the keys they had before the amendment.
    assert!(!same_key("http://%C3%BC.example/", "http://ü.example/"));
    assert_eq!(submitted("http://ü.example/"), "http://xn--tda.example/");
}

/// Proves FR9 as amended, and FR1 against a step 4 that rejects more.
/// Rejected by the current browser form (WHATWG domain to ASCII, UTS #46
/// nontransitional), and by the transitional form only, where a joiner fails
/// the current form's CheckJoiners but a transitional client drops it. The
/// one trailing dot is judged on each form, after mapping. Accepted:
/// look-alikes whose forms are not the service's domain, hosts with no
/// browser form (not UTF-8, a decoded delimiter) or an empty one, and hosts
/// whose transitional form differs but is not `s.example`.
#[test]
fn browser_forms_of_the_host_are_judged() {
    let mut wrong = Vec::new();
    let mut check = |raw: &str, want: Result<(), Reject>| wrong.extend(mismatch(raw, want));

    // The current form: soft hyphen and zero-width space ignored, full-width
    // letters mapped, U+3002 mapped to a dot.
    for raw in [
        "http://s.ex%C2%ADample/",
        "http://%E2%80%8Bs.example/",
        "http://%EF%BC%B3.EXAMPLE/",
        "http://s%E3%80%82example/",
        "http://a.%EF%BD%93.example/",
        "http://%EF%BD%93.example./",
        "https://%EF%BD%93.example:8443/x",
    ] {
        check(raw, Err(Reject::OwnDomain));
    }
    // A trailing dot that only the mapping makes, U+3002 or U+FF0E: the one
    // dot is dropped from the browser form, not from the escaped host. Two
    // dots after mapping leave one, which is not the service's domain.
    for raw in [
        "http://s.example%E3%80%82/",
        "http://%EF%BD%93.example%EF%BC%8E/",
    ] {
        check(raw, Err(Reject::OwnDomain));
    }
    check("http://s.example.%E3%80%82/", Ok(()));
    // The transitional form only: U+200D and U+200C dropped.
    for raw in [
        "http://%E2%80%8Ds.example/",
        "http://%E2%80%8Cs.example/",
        "http://s.ex%E2%80%8Dample/",
    ] {
        check(raw, Err(Reject::OwnDomain));
    }
    // Look-alikes: two trailing dots, a suffix, `nots.example`.
    for raw in [
        "http://%EF%BD%93.example../",
        "http://%EF%BD%93.example.com/",
        "http://n%EF%BD%8Fts.example/",
    ] {
        check(raw, Ok(()));
    }
    // No browser form: bytes that are not UTF-8, a decoded `/`, and a host
    // that maps to nothing; each fails in a browser.
    for raw in [
        "http://%FF.example/",
        "http://s.example%2F.evil.com/",
        "http://%C2%AD/s.example/x",
    ] {
        check(raw, Ok(()));
    }
    // The transitional form's cost does not reach `s.example`: `paß` gives
    // `pass`, `ς` gives `σ`.
    for raw in ["http://pa%C3%9F.example/", "http://%CF%82.example/"] {
        check(raw, Ok(()));
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));

    assert_eq!(
        canonical("http://pa%C3%9F.example/"),
        "http://pa%C3%9F.example/"
    );
}

/// Proves FR9: any `@` in the authority rejects, even an empty user, and
/// before the own-domain check; an `@` elsewhere does not.
#[test]
fn any_userinfo_is_rejected() {
    for raw in [
        "http://user@example.com/",
        "http://user:pw@example.com/",
        "http://:@example.com/",
        "http://@example.com/",
        "http://s.example@evil.com/",
        "http://user@s.example/",
        "http://ü@example.com/",
    ] {
        assert_rejected(raw, Reject::Userinfo);
    }
    assert_accepted("http://example.com/@user");
    assert_accepted("http://example.com/?a=@");
}

/// Proves FR7 as amended: an http(s) URL needs an authority with a
/// non-empty host, judged on the submitted URL after FR10, so a host that
/// ToASCII maps to nothing is empty. The host check comes after the scheme
/// and the grammar, and before the length and FR9's userinfo check.
#[test]
fn hostless_urls_are_rejected() {
    let mut wrong = Vec::new();
    let mut check = |raw: &str, want: Result<(), Reject>| wrong.extend(mismatch(raw, want));

    // No authority. `http:/.//x/y` must never take the form of `http://x/y`.
    for raw in [
        "HTTP:foo",
        "http:s.example/x",
        "http:/x",
        "http:/.//x/y",
        "http:",
    ] {
        check(raw, Err(Reject::InvalidUrl));
    }
    // An authority with an empty host.
    for raw in [
        "http:///x",
        "https:///s.example/x",
        "http://:80/x",
        "http://?q",
        "http://#f",
    ] {
        check(raw, Err(Reject::InvalidUrl));
    }
    // Step 1 maps a lone soft hyphen to nothing, which leaves no host.
    for raw in ["http://\u{AD}/s.example/x", "http://\u{AD}:80/"] {
        check(raw, Err(Reject::InvalidUrl));
    }
    // Order: the host check comes before FR9's userinfo check and the length,
    // and after the scheme.
    check("http://u@/x", Err(Reject::InvalidUrl));
    check("ftp:///x", Err(Reject::BadScheme));
    check(
        &format!("http:///{}", "a".repeat(2_048)),
        Err(Reject::InvalidUrl),
    );
    check(
        &format!("http:{}", "a".repeat(2_048)),
        Err(Reject::InvalidUrl),
    );
    // A one-character host is a host.
    check("http://a/", Ok(()));
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

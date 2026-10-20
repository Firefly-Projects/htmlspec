//! Where an asset is stored: its address, derived from its contents.
//!
//! An asset's address is a function of its bytes, so identical bytes have one
//! address and differing bytes have distinct ones. That is the whole mechanism. It
//! makes two properties hold at once that no fixed layout can satisfy together: a
//! project stores each library exactly once, and a document authored earlier keeps
//! rendering with the bytes it was authored against.
//!
//! The failure this replaces was not drift through maintenance. A second copy of a
//! stylesheet was written *already stale* — the project's copy was correct and the
//! copy beside the document was an older revision, so a document shipped markup for
//! a skip link against a stylesheet with no rule for it. A check comparing copies
//! catches that only after the bytes are committed. An address derived from the
//! bytes cannot express the disagreement in the first place.

/// The digest algorithm named in every address.
///
/// Spelled at each reference site rather than assumed, so replacing the algorithm is
/// a visible change across the repository instead of a silent reinterpretation of
/// paths that look the same.
pub const ALGORITHM: &str = "sha256";

/// How many hex characters of the digest an address carries.
///
/// 16 characters is 64 bits. The full 64 would be paid in every reference in every
/// document and in every diff, which is a readability cost with no security return
/// here: this store is closed, every entry is recorded with its full digest, and the
/// checks compare the full digest rather than the address.
///
/// [`crate::verify_vendored_assets`] fails on a truncated-form collision between two
/// entries, so an alias cannot form quietly.
pub const ADDRESS_HEX_LEN: usize = 16;

/// The directory name an asset with this digest is stored under.
///
/// Takes the full lowercase-hex digest and truncates it. Panics on a digest too
/// short to truncate, because that is a malformed lockfile rather than a runtime
/// condition — the lockfile is committed alongside the assets it pins.
pub fn address_dir(sha256_hex: &str) -> String {
    assert!(
        sha256_hex.len() >= ADDRESS_HEX_LEN,
        "a digest of {} characters cannot address an asset: {sha256_hex:?}",
        sha256_hex.len()
    );
    assert!(
        sha256_hex.bytes().all(|b| b.is_ascii_hexdigit()),
        "an address needs a hex digest: {sha256_hex:?}"
    );
    format!("{ALGORITHM}-{}", &sha256_hex[..ADDRESS_HEX_LEN])
}

/// The path an asset is stored at, relative to the project's `assets/` directory.
///
/// The file name is kept inside the hash-named directory rather than replaced by the
/// hash. The directory name already carries the uniqueness, and keeping the name
/// preserves the extension a browser uses to choose a content type, and keeps a
/// directory listing legible to a human who can still tell Mermaid from Vega.
///
/// `file_name` is the base name, not a path: a stored asset is flat inside its
/// address, so `vendor/mermaid.min.js` stores as
/// `sha256-18327bef70d96fb5/mermaid.min.js`.
pub fn stored_path(sha256_hex: &str, file_name: &str) -> String {
    format!("{}/{}", address_dir(sha256_hex), base_name(file_name))
}

/// The final component of a `/`-separated path.
///
/// Applied to the lockfile's `path` field, which records the old nested layout, so
/// the derivation takes either a bare name or a nested one and yields the same
/// stored path.
pub fn base_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_names_its_algorithm_and_truncates_the_digest() {
        let digest = "18327bef70d96fb505fe7287d9f6a7362ebf07ff6576ddfaffb1a06f3e1a2954";
        assert_eq!(address_dir(digest), "sha256-18327bef70d96fb5");
        assert_eq!(
            address_dir(digest).len(),
            ALGORITHM.len() + 1 + ADDRESS_HEX_LEN,
            "an address is the algorithm, a hyphen, and the truncated digest"
        );
    }

    #[test]
    fn a_stored_path_keeps_the_file_name_inside_its_address() {
        let digest = "18327bef70d96fb505fe7287d9f6a7362ebf07ff6576ddfaffb1a06f3e1a2954";
        assert_eq!(
            stored_path(digest, "vendor/mermaid.min.js"),
            "sha256-18327bef70d96fb5/mermaid.min.js",
            "the extension must survive, because a browser reads it"
        );
        assert_eq!(
            stored_path(digest, "mermaid.min.js"),
            stored_path(digest, "vendor/mermaid.min.js"),
            "a nested path and a bare name address the same stored file"
        );
    }

    /// The point of the whole mechanism, asserted directly: differing bytes cannot
    /// share an address, so one cannot overwrite the other.
    #[test]
    fn differing_bytes_get_differing_addresses() {
        let a = crate::sha256_hex(b"--hs-measure: 36rem");
        let b = crate::sha256_hex(b"--hs-measure: 34rem");
        assert_ne!(
            stored_path(&a, "htmlspec.css"),
            stored_path(&b, "htmlspec.css"),
            "two revisions of one stylesheet must not collapse onto one location"
        );
    }

    /// And the other half: identical bytes address one location, which is what makes
    /// a project store each library once however many documents reference it.
    #[test]
    fn identical_bytes_get_one_address() {
        let once = crate::sha256_hex(b"the same bytes");
        let again = crate::sha256_hex(b"the same bytes");
        assert_eq!(stored_path(&once, "x.js"), stored_path(&again, "x.js"));
    }

    #[test]
    #[should_panic(expected = "cannot address an asset")]
    fn a_truncated_digest_is_rejected_rather_than_padded() {
        address_dir("18327bef");
    }

    #[test]
    #[should_panic(expected = "needs a hex digest")]
    fn a_non_hex_digest_is_rejected() {
        address_dir("not-a-hex-digest-but-long-enough-to-truncate");
    }
}

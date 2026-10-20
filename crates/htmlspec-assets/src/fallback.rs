//! The fallback stylesheet every document the toolchain writes carries in its head.
//!
//! A document reaches its stylesheet in the project's one asset store, in a folder above the
//! document. Safari, opening a document from disk, refuses a stylesheet in a parent folder, so there
//! a document renders as bare HTML. The fallback keeps it readable, and it changes nothing wherever
//! the store's stylesheet does load.
//!
//! **Inert by the cascade, not by a script.** A declaration in a named cascade layer loses to every
//! declaration outside a layer, whatever their specificity and whatever their order, and the store's
//! stylesheet declares nothing in a layer. So the fallback sets only properties that the store's
//! stylesheet also sets, unconditionally, on the same selectors: each one is overridden whenever that
//! stylesheet loads. `the_fallback_is_inert_against_the_store_stylesheet` holds that rule against the
//! stylesheet this crate embeds, and it fails the first time either side moves. No script is involved,
//! because a document may run none of its own.
//!
//! The values are the store's own, with its custom properties resolved, so a page without its
//! stylesheet still reads in the house serif and ink. The body's padding is the one exception: it
//! centres a column of at most 44rem, where the store's stylesheet builds its column from a grid.
//!
//! **The skip link comes back on focus.** The fallback moves it off-screen, as the store's stylesheet
//! does, and so it must bring it back when a keyboard user reaches it, as the store's does. Without
//! that rule the fallback is still inert and still hides the link, which is why
//! `the_fallback_brings_the_skip_link_back_on_focus` checks it apart from inertness.
//!
//! **Not pinned in the lockfile.** The lockfile pins bytes that come from somewhere: vendored
//! libraries, the assets a project stores, and the templates. This is text the toolchain writes into
//! a document, like the head of a run's record, and the tests that render it are what hold it.
//!
//! **Canonical as written.** A new document is written as rendered and validated, never formatted,
//! and the formatter keeps a `<style>` element's text byte for byte. So [`ELEMENT`] carries the
//! indentation of a document's head, and the template tests hold every rendered template canonical.

/// The cascade layer the fallback declares itself in.
///
/// Also how an existing fallback is recognized in a document: a `<style>` element whose text opens
/// with this layer is the toolchain's fallback, current or stale.
pub const LAYER: &str = "htmlspec-fallback";

/// The fallback stylesheet as a `<style>` element, in the canonical form a document's head holds it.
///
/// The leading indentation of the element itself is the head's and is supplied by whoever places
/// it: the template's placeholder line, or the formatter.
pub const ELEMENT: &str = r#"<style>
      @layer htmlspec-fallback {
        body { margin: 0; padding: 2rem max(1rem, calc((100% - 44rem) / 2)); background: #fdfcfa; color: #1a1a1a; font-family: Charter, Georgia, "Times New Roman", Times, serif; font-size: 1.0625rem; line-height: 1.55; }
        code, kbd, samp { font-family: ui-monospace, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace; font-size: 0.88em; }
        pre { font-family: ui-monospace, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace; font-size: 0.8rem; line-height: 1.45; background: #f2efe9; border-left: 2px solid #d6d2ca; padding: 0.7rem 0.9rem; margin: 0.4rem 0; overflow-x: auto; }
        .hs-skip-link { position: absolute; left: -9999px; top: 0; }
        .hs-skip-link:focus { left: 1.5rem; top: 1.5rem; }
      }
    </style>"#;

/// The CSS [`ELEMENT`] carries: its text, without the tags.
pub fn css() -> &'static str {
    ELEMENT
        .strip_prefix("<style>")
        .and_then(|rest| rest.strip_suffix("</style>"))
        .expect("the fallback element is one <style> element")
}

/// Whether a `<style>` element's text is the toolchain's fallback, current or stale.
pub fn is_fallback_text(text: &str) -> bool {
    text.trim_start()
        .strip_prefix("@layer")
        .and_then(|rest| rest.trim_start().strip_prefix(LAYER))
        .is_some_and(|after| after.starts_with(|c: char| c.is_whitespace() || c == '{'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_element_is_recognized_as_the_fallback() {
        assert!(is_fallback_text(css()));
        assert!(!is_fallback_text("body { margin: 0 }"));
        assert!(!is_fallback_text("@layer another { body { margin: 0 } }"));
        assert!(!is_fallback_text(
            "@layer htmlspec-fallbacks { body { margin: 0 } }"
        ));
    }
}

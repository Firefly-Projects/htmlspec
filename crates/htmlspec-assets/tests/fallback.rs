//! The fallback stylesheet is inert wherever the store's stylesheet loads.
//!
//! A declaration in a named cascade layer loses to every declaration outside a layer. So the fallback
//! changes nothing on a page whose stylesheet loaded exactly when every property it sets on a selector
//! is also set, outside any layer and outside any at-rule, by a rule of the store's stylesheet naming
//! the same selector. These tests hold that rule against the stylesheet this crate embeds, so either
//! side moving fails the build rather than a reader's page.
//!
//! The checker is tested on its own inputs too. A test that passes against the real texts proves
//! nothing about a checker that would also pass against a broken fallback.

use htmlspec_assets::fallback;

/// One rule: the at-rules enclosing it, its selector prelude, and the properties it declares.
#[derive(Debug)]
struct Rule {
    context: Vec<String>,
    selectors: Vec<String>,
    properties: Vec<String>,
}

/// The rules of `css`, innermost blocks only, each with the at-rule preludes around it.
///
/// Enough CSS for the two texts in question: comments, nested at-rule blocks, and declaration blocks.
/// Neither text puts a brace inside a string or a comment inside a string, and the tests that read the
/// real texts would fail loudly if one began to.
fn rules(css: &str) -> Vec<Rule> {
    let css = strip_comments(css);
    let mut stack: Vec<String> = Vec::new();
    let mut found = Vec::new();
    let mut segment = String::new();
    for c in css.chars() {
        match c {
            '{' => {
                stack.push(segment.trim().to_owned());
                segment.clear();
            }
            '}' => {
                let prelude = stack.pop().expect("the stylesheet's braces balance");
                let body = segment.trim().to_owned();
                segment.clear();
                if !prelude.starts_with('@') {
                    found.push(Rule {
                        context: stack.clone(),
                        selectors: split_top_level(&prelude, ',')
                            .into_iter()
                            .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
                            .collect(),
                        properties: split_top_level(&body, ';')
                            .into_iter()
                            .filter_map(|declaration| {
                                let (property, _) = declaration.split_once(':')?;
                                Some(property.trim().to_ascii_lowercase())
                            })
                            .filter(|property| !property.is_empty())
                            .collect(),
                    });
                }
            }
            _ => segment.push(c),
        }
    }
    assert!(stack.is_empty(), "the stylesheet's braces balance");
    found
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = match rest[start + 2..].find("*/") {
            Some(end) => &rest[start + 2 + end + 2..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// Splits on `separator` outside parentheses and quotes.
fn split_top_level(text: &str, separator: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut current = String::new();
    for c in text.chars() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(') => depth += 1,
            (None, ')') => depth = depth.saturating_sub(1),
            (None, _) if c == separator && depth == 0 => {
                parts.push(current.trim().to_owned());
                current.clear();
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_owned());
    }
    parts
}

/// Each `(selector, property)` the fallback sets that no unconditional store rule also sets.
fn uncovered(fallback_css: &str, store_css: &str) -> Vec<(String, String)> {
    let store = rules(store_css);
    let mut missing = Vec::new();
    for rule in rules(fallback_css) {
        for selector in &rule.selectors {
            for property in &rule.properties {
                let covered = store.iter().any(|candidate| {
                    candidate.context.is_empty()
                        && candidate.selectors.iter().any(|s| s == selector)
                        && candidate.properties.iter().any(|p| p == property)
                });
                if !covered {
                    missing.push((selector.clone(), property.clone()));
                }
            }
        }
    }
    missing
}

/// Everything wrong with the fallback's own shape, apart from what it sets.
fn shape_faults(fallback_css: &str) -> Vec<String> {
    let mut faults = Vec::new();
    for forbidden in ["url(", "@import", "!important"] {
        if fallback_css.contains(forbidden) {
            faults.push(format!("it carries {forbidden}"));
        }
    }
    let rules = rules(fallback_css);
    if rules.is_empty() {
        faults.push("it declares no rule".to_owned());
    }
    let layer = format!("@layer {}", fallback::LAYER);
    for rule in &rules {
        if rule.context != [layer.clone()] {
            faults.push(format!(
                "{:?} sits in {:?}, not directly in {layer}",
                rule.selectors, rule.context
            ));
        }
    }
    faults
}

/// Each fault in how the fallback treats the skip link.
///
/// A fallback that moves the skip link off-screen and never brings it back leaves a keyboard user
/// on an invisible link, exactly where the fallback applies. So each property that moves the link
/// must be set again on focus. Inertness does not catch this: a fallback with no focus rule sets
/// nothing the store's stylesheet does not, and is inert, and still hides the link.
fn skip_link_faults(fallback_css: &str) -> Vec<String> {
    let rules = rules(fallback_css);
    let sets = |selector: &str, property: &str| {
        rules.iter().any(|rule| {
            rule.selectors.iter().any(|s| s == selector)
                && rule.properties.iter().any(|p| p == property)
        })
    };
    ["left", "top"]
        .into_iter()
        .filter(|property| {
            sets(".hs-skip-link", property) && !sets(".hs-skip-link:focus", property)
        })
        .map(|property| {
            format!("it moves the skip link with `{property}` and does not set it again on focus")
        })
        .collect()
}

fn store_stylesheet() -> String {
    let asset = htmlspec_assets::all_assets()
        .into_iter()
        .find(|asset| asset.name() == "htmlspec.css")
        .expect("the toolchain embeds htmlspec.css");
    String::from_utf8(asset.bytes.to_vec()).expect("the stylesheet is UTF-8")
}

#[test]
fn the_fallback_is_inert_against_the_store_stylesheet() {
    let missing = uncovered(fallback::css(), &store_stylesheet());
    assert!(
        missing.is_empty(),
        "the fallback sets properties the store's stylesheet does not also set unconditionally on \
         the same selector, so they would show on a page whose stylesheet loaded: {missing:?}"
    );
}

#[test]
fn the_fallback_brings_the_skip_link_back_on_focus() {
    let faults = skip_link_faults(fallback::css());
    assert!(
        faults.is_empty(),
        "where the fallback applies, a keyboard user tabs onto a skip link nobody can see: {faults:?}"
    );
}

#[test]
fn the_skip_link_check_reports_a_link_hidden_for_good() {
    // The fallback as first written: it moved the link off-screen and never brought it back.
    let hidden =
        "@layer htmlspec-fallback { .hs-skip-link { position: absolute; left: -9999px; top: 0; } }";
    assert_eq!(skip_link_faults(hidden).len(), 2);
    let restored =
        "@layer htmlspec-fallback { .hs-skip-link { position: absolute; left: -9999px; top: 0; } \
                    .hs-skip-link:focus { left: 1.5rem; top: 1.5rem; } }";
    assert!(skip_link_faults(restored).is_empty());
    assert!(skip_link_faults("@layer htmlspec-fallback { body { margin: 0; } }").is_empty());
}

#[test]
fn the_store_stylesheet_declares_no_cascade_layer() {
    // An unlayered declaration beats a layered one. A store stylesheet that adopted layers would
    // put its own declarations in competition with the fallback's, and the order of layers, not
    // their presence, would decide which wins.
    assert!(
        !strip_comments(&store_stylesheet()).contains("@layer"),
        "the store's stylesheet declares a cascade layer, so the fallback is no longer inert by \
         construction"
    );
}

#[test]
fn the_fallback_is_one_layer_that_reaches_nothing() {
    let faults = shape_faults(fallback::css());
    assert!(
        faults.is_empty(),
        "the fallback's shape is wrong: {faults:?}"
    );
}

#[test]
fn the_checker_reports_a_property_the_store_does_not_set() {
    let store = "body { margin: 0; color: #000; }";
    let fallback_css = "@layer htmlspec-fallback { body { margin: 0; text-transform: none; } }";
    assert_eq!(
        uncovered(fallback_css, store),
        [("body".to_owned(), "text-transform".to_owned())]
    );
}

#[test]
fn the_checker_reports_a_selector_the_store_does_not_name() {
    let store = "code, kbd { font-size: 1em; }";
    let fallback_css = "@layer htmlspec-fallback { code, samp { font-size: 1em; } }";
    assert_eq!(
        uncovered(fallback_css, store),
        [("samp".to_owned(), "font-size".to_owned())]
    );
}

#[test]
fn the_checker_does_not_count_a_store_rule_inside_an_at_rule() {
    // A rule inside @media overrides only while its media matches, so it covers nothing.
    let store = "@media (max-width: 34rem) { body { font-size: 1rem; } }";
    let fallback_css = "@layer htmlspec-fallback { body { font-size: 1rem; } }";
    assert_eq!(
        uncovered(fallback_css, store),
        [("body".to_owned(), "font-size".to_owned())]
    );
}

#[test]
fn the_shape_check_reports_what_would_break_the_layer() {
    assert!(
        !shape_faults("body { margin: 0; }").is_empty(),
        "a rule outside the layer"
    );
    assert!(
        !shape_faults("@layer htmlspec-fallback { body { margin: 0 !important; } }").is_empty()
    );
    assert!(
        !shape_faults("@layer htmlspec-fallback { body { background: url(x.png); } }").is_empty()
    );
    assert!(shape_faults("@layer htmlspec-fallback { body { margin: 0; } }").is_empty());
}

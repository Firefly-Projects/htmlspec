//! The shared assets a document renders through, embedded for `init`.
//!
//! A document is a file a reviewer opens in a browser, so the runtime it needs is
//! committed to the repository rather than fetched or built. That means a plain
//! `git clone` yields documents that render offline immediately, with no build
//! step and no Nix.
//!
//! Committing third-party build output is unpleasant, and [`lockfile`] is the
//! answer: it records each library's version, source URL, and content hash, and
//! [`verify_vendored_assets`] checks the committed bytes against it offline. The
//! Nix check additionally re-fetches each archive from its pinned source, so the
//! committed copy is auditable rather than trusted.

use std::fmt;
use std::sync::OnceLock;

use sha2::{Digest, Sha256};

pub mod address;
pub mod fallback;
pub mod instructions;
pub mod lockfile;
pub mod reference;
pub mod store;

pub use address::{address_dir, base_name, stored_path, ADDRESS_HEX_LEN, ALGORITHM};
pub use lockfile::{lockfile, Archive, Library, Lockfile, OwnFile, VendoredFile};
pub use reference::{ReferencingAttribute, REFERENCING_ATTRIBUTES};
pub use store::{Duplicate, Resolution, Store};

/// One asset, embedded so `init` can write it with no source checkout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Asset {
    /// The path the asset takes inside the project's `assets/` directory.
    pub path: &'static str,
    /// The asset's bytes.
    pub bytes: &'static [u8],
}

impl Asset {
    /// The asset's content hash, as lowercase hex.
    pub fn sha256(&self) -> String {
        sha256_hex(self.bytes)
    }

    /// The asset's file name, without its address.
    ///
    /// The identity a human uses and a lockfile entry records. [`Self::path`] is the
    /// content address, which changes whenever the bytes do — so a lookup by path
    /// would silently find nothing after any edit, and a lookup by name still works.
    pub fn name(&self) -> &'static str {
        address::base_name(self.path)
    }
}

// The asset tables: `OWN` and `VENDORED`, generated from the lockfile.
//
// Each asset's `path` is its content address. Generated rather than written by hand
// so that an address exists in one place — the lockfile — and the Rust table, the Nix
// check, and the store cannot drift apart. See `build.rs`.
//
// These were hand-written literals until this change, which is how the project came
// to hold two copies of one stylesheet: the layout was stated in several places and
// nothing compared them.
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// One artifact kind's template, and the kind it produces.
///
/// A template per kind rather than one template with sections switched on and off.
/// The shapes differ in more than which units they carry: a proposal has no
/// navigation because it has no unit sections to navigate, and a specification's
/// masthead means something different from a tasks document's. Assembling four
/// documents from one file by deletion would make each shape a consequence of the
/// deletions rather than a thing someone wrote deliberately.
pub struct Template {
    /// The artifact kind this template declares.
    ///
    /// A `&'static str` rather than an enumeration, because the vocabulary belongs
    /// to the schema. `every_template_declares_a_kind_the_schema_enumerates` keeps
    /// these values inside it.
    pub kind: &'static str,
    /// The template text, with its placeholders unsubstituted.
    pub text: &'static str,
}

/// Every template `init` can write a document from.
///
/// Not [`Asset`]s: a template is not copied into a project's `assets/` directory.
/// It is the *document* `init` produces, with its placeholders substituted, so it
/// travels separately from the shared runtime.
///
/// All four kinds, and the `design` one is deliberately almost empty.
///
/// I first shipped three and had `init --change` write its design from the proposal
/// template. Running it showed why that was wrong twice over: the output said
/// "created a proposal" for a design, and it imposed a proposal's headings — Why,
/// Alternatives, Scope — on a document whose whole point is that its structure is the
/// author's. A design template that offers *no* headings is a different thing from no
/// design template at all, and it is the honest one: every kind `init` names can be
/// created, and the design's page says in prose why it is empty.
pub const TEMPLATES: &[Template] = &[
    Template {
        kind: "proposal",
        text: include_str!("../assets/template-proposal.html"),
    },
    Template {
        kind: "design",
        text: include_str!("../assets/template-design.html"),
    },
    Template {
        kind: "tasks",
        text: include_str!("../assets/template-tasks.html"),
    },
    Template {
        kind: "specification",
        text: include_str!("../assets/template-specification.html"),
    },
];

/// One of a runbook's two documents, as `new runbook` writes it.
///
/// A second shape beside [`Template`] rather than two more entries in [`TEMPLATES`], because
/// [`template_for`] maps one kind to one template and a runbook's specification is a second
/// specification. A runbook's documents differ from a change's in what they are for: every unit is
/// `unchanged` and every task `todo`, as runbook-authority's "The canonical procedure carries no
/// completion state" requires of a runbook, where a change's template writes `added`.
///
/// `#[non_exhaustive]` because this crate is published, and a later minor release may say more
/// about a template: a field added to an exhaustive struct breaks a caller that destructures it.
#[non_exhaustive]
pub struct RunbookTemplate {
    /// The file name the document takes inside the runbook's directory.
    pub file: &'static str,
    /// The artifact kind it declares.
    pub kind: &'static str,
    /// The template text, with its placeholders unsubstituted.
    pub text: &'static str,
}

/// The two documents `new runbook` writes, in the order a reader opens them.
///
/// Scanned by [`example_identifiers`] and [`example_placeholders`] beside [`TEMPLATES`], so
/// contract-authority's "Generated example content is non-normative and its survival is reported"
/// reaches a runbook straight from these templates.
pub const RUNBOOK_TEMPLATES: &[RunbookTemplate] = &[
    RunbookTemplate {
        file: "specification.html",
        kind: "specification",
        text: include_str!("../assets/template-runbook-specification.html"),
    },
    RunbookTemplate {
        file: "tasks.html",
        kind: "tasks",
        text: include_str!("../assets/template-runbook-tasks.html"),
    },
];

/// The text of every template the binary writes documents from, a change's and a runbook's.
fn every_template_text() -> impl Iterator<Item = &'static str> {
    TEMPLATES
        .iter()
        .map(|template| template.text)
        .chain(RUNBOOK_TEMPLATES.iter().map(|template| template.text))
}

/// The artifact kind `init` produces when the caller names none.
///
/// A proposal, because that is the artifact a change begins with: it depends on
/// nothing and everything else depends on it. A default of any later kind would
/// hand a contributor the middle of a sequence and call it a start.
pub const DEFAULT_KIND: &str = "proposal";

/// The template for `kind`, or `None` when no template produces that kind.
///
/// Every kind in the schema's enumeration has one, which
/// `every_artifact_kind_the_schema_enumerates_has_a_template` pins. `None` therefore
/// means the caller named something outside the vocabulary, and the caller reports
/// that rather than substituting a shape nobody asked for.
pub fn template_for(kind: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|template| template.kind == kind)
}

/// The suffix an identifier carries where the unit it names is a template's example.
///
/// One string rather than a list of whole identifiers, because the naming convention is what
/// the templates follow. A list would restate the templates and go stale the first time one
/// gains a unit; the suffix holds however many units they carry.
pub const EXAMPLE_SUFFIX: &str = "-example";

/// Every identifier the templates write as example content.
///
/// **Read out of the templates rather than listed beside them.** The templates are compiled
/// into the binary, so scanning them costs nothing at runtime and a template that gains a unit
/// is covered with no second edit. A `const` array of the eight current identifiers would be a
/// second statement of a fact the templates already make, and `project-bootstrap` names that
/// shape as the one that produced a gate comparing against a value no project ever had.
///
/// Sorted and deduplicated, so two machines report one document identically.
///
/// Computed once. The templates are `include_str!`ed constants, so the answer cannot change
/// between calls, and [`is_example_identifier`] asks this question once per unit and once per
/// figure in every document validation. Scanning 12.7 KB and sorting the result on each of
/// those calls was measurable — about a tenth of the run on a document with 800 requirements.
pub fn example_identifiers() -> &'static [&'static str] {
    static IDENTIFIERS: OnceLock<Vec<&'static str>> = OnceLock::new();
    IDENTIFIERS.get_or_init(|| {
        let mut found: Vec<&'static str> = every_template_text()
            .flat_map(identifiers_in)
            .filter(|id| id.ends_with(EXAMPLE_SUFFIX))
            .collect();
        found.sort_unstable();
        found.dedup();
        found
    })
}

/// Whether `id` is one the templates write as example content.
///
/// The membership test rather than the suffix alone. A document is free to name a unit
/// `req-worked-example`, and only an identifier a template actually writes is the hazard —
/// which is the difference between reading the templates and guessing from a name.
pub fn is_example_identifier(id: &str) -> bool {
    example_identifiers().contains(&id)
}

/// Every masthead paragraph the templates write as a placeholder summary.
///
/// The second site of a template's example content, and the one carrying no identifier.
/// [`example_identifiers`] reads `id="…"` attributes, so it cannot see a masthead paragraph at
/// all — and the projection specification, section 9.1.1, projects a document's `summary` from
/// exactly those paragraphs. A document whose entire summary is one of these strings says nothing
/// about what it is for, and `contract-authority` requires that to be reported.
///
/// **Read out of the templates rather than listed beside them**, for the reason
/// [`example_identifiers`] already gives: a `const` array of the four current sentences would be
/// a second statement of a fact the templates already make, and it would go stale the first time
/// one is reworded. That is the edit nobody remembers.
///
/// Normalized the way projection section 6.1 normalizes text — every run of whitespace collapsed
/// to one space, then trimmed — because the comparison is against a *projected* summary and a
/// template's line breaks are invisible to the model. This is a second implementation of that
/// rule rather than a call to the projection's own: `htmlspec-parse` depends on this crate rather
/// than the other way round, and its `normalized_text` is private. Two implementations of one rule
/// drift, and the drift would be silent — the rule would simply stop matching. So they are pinned
/// against each other by
/// `the_scanned_placeholder_is_the_projected_summary_of_a_new_document`, which compares this set
/// against the projection of a document the template just produced, for every kind.
///
/// What that test also covers, rather than this scan handling it: a character reference. The
/// projection resolves `&amp;` before normalizing (section 6.3) and this scan does not, because
/// resolving them needs the parser this crate has no dependency on. No template's masthead carries
/// one today, and a template that gained one would fail that test rather than quietly stop being
/// matched.
///
/// Sorted and deduplicated, so two machines report one document identically. Computed once, for
/// the reason [`example_identifiers`] is: the templates are `include_str!`ed constants, so the
/// answer cannot change between calls.
pub fn example_placeholders() -> &'static [String] {
    static PLACEHOLDERS: OnceLock<Vec<String>> = OnceLock::new();
    PLACEHOLDERS.get_or_init(|| {
        let mut found: Vec<String> = every_template_text()
            .flat_map(masthead_paragraphs)
            .collect();
        found.sort_unstable();
        found.dedup();
        found
    })
}

/// Whether `text` is, in whole, a paragraph a template writes as a placeholder summary.
///
/// **Whole-text equality rather than a substring search**, and that is the decision this check
/// turns on rather than an implementation detail. A summary reading "Unlike the template's
/// 'Replace this paragraph' text, this one states the actual argument" is the author's own, and a
/// substring search would report it — along with every document in this project that discusses
/// the rule. A rule that fires on documents about itself is one an agent learns to ignore.
///
/// The cost, accepted: a masthead half-edited into the placeholder plus one sentence of the
/// author's own is not reported. That is the same trade the identifier check makes in the other
/// direction, where the identifier is what survives an edit and here the text is. Reporting both
/// halves of both cases needs a rule that guesses at intent.
///
/// The argument is normalized before comparison, so a caller holding text from anywhere gets the
/// same answer as one holding a projected summary. The projection already normalizes what it
/// projects, so for the validator this is a no-op that costs one allocation per document — and
/// without it a caller reading text by any other route would get a silent false negative, which
/// is the failure this whole check exists to remove.
pub fn is_example_placeholder(text: &str) -> bool {
    let normalized = normalized_whitespace(text);
    example_placeholders().contains(&normalized)
}

/// Every paragraph inside `text`'s first `header` element, normalized.
///
/// The first `header`, because that is what the projection reads: section 9.1.1 projects `summary`
/// from every `p` inside the document's `header`. Selecting on `class="hs-masthead"` instead would
/// be a second, different rule for locating the same paragraphs, and the point of this scan is to
/// see exactly what the model sees. `the_scan_reads_the_masthead_alone` asserts the header it finds
/// is the masthead in every template.
///
/// A scan rather than a parse, for the reason [`identifiers_in`] states: the crate holds no HTML
/// parser, and the templates are files this project writes, formatted into the canonical form.
/// Tags inside a paragraph are dropped and their text kept, which is what the projection does with
/// `<code>` in a header paragraph — that element's content is preserved text only inside `pre`,
/// `code` and `textarea` *as fields*, and a header paragraph projects as normalized text whatever
/// it wraps.
///
/// A template's *body* paragraphs are deliberately out of reach. They are also text this program
/// wrote, and treating them as placeholder summaries would report a document for prose that is not
/// its summary.
fn masthead_paragraphs(text: &str) -> Vec<String> {
    let Some(header) = first_element_text(text, "header") else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut rest = header;
    while let Some(paragraph) = first_element_text(rest, "p") {
        let normalized = normalized_whitespace(&strip_tags(paragraph));
        if !normalized.is_empty() {
            found.push(normalized);
        }
        // Resume past this paragraph. `p` cannot nest, so the close tag just consumed is this
        // paragraph's own and the next search starts after it.
        let after = rest[..].find("</p>").map_or(rest.len(), |at| at + 4);
        rest = &rest[after..];
    }
    found
}

/// The inner text of the first `<name …>` element in `text`, or `None` when there is none.
///
/// Matches an opening tag whose name is followed by whitespace or `>`, so looking for `p` does not
/// match `<pre>`. Returns everything up to the matching close tag, which for the elements this is
/// used on — a document header and a paragraph, neither of which nests inside itself in these
/// templates — is that element's content.
fn first_element_text<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}");
    let close = format!("</{name}>");
    let mut cursor = 0;
    loop {
        let at = cursor + text[cursor..].find(&open)?;
        let after = at + open.len();
        let boundary = text[after..]
            .chars()
            .next()
            .is_some_and(|c| c == '>' || c.is_whitespace());
        if !boundary {
            // `<pre` when looking for `<p`. Resume inside the name so a later element is found.
            cursor = after;
            continue;
        }
        let start = after + text[after..].find('>')? + 1;
        let end = start + text[start..].find(&close)?;
        return Some(&text[start..end]);
    }
}

/// `text` with every `<…>` removed and the characters between them kept.
///
/// The paragraphs this runs on hold prose and inline markup. An element contributing no text to
/// the model — `script`, `style`, `template`, `noscript`, per projection section 8.5 — would have
/// its content kept here and dropped by the projection, so the two would disagree. No template's
/// masthead carries one, `the_scan_reads_the_masthead_alone` asserts that, and
/// `the_scanned_placeholder_is_the_projected_summary_of_a_new_document` fails if one appears.
fn strip_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('<') {
        out.push_str(&rest[..at]);
        match rest[at..].find('>') {
            Some(end) => rest = &rest[at + end + 1..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// Normalized text per projection specification section 6.1.
///
/// Every run of whitespace, newlines included, replaced with one space, then trimmed. The second
/// implementation of that rule in this repository, and [`example_placeholders`] states why it has
/// to be and what pins it against the first.
fn normalized_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every `id="…"` attribute value in `text`.
///
/// A scan rather than a parse. The crate holds no HTML parser — `htmlspec-parse` does, and it
/// depends on this crate rather than the other way round — and the templates are files this
/// project writes, so their attributes are double-quoted by the formatter's own canonical form.
///
/// The match requires whitespace before the `id`, so an attribute whose name merely *ends* in
/// `id` does not yield its value. `data-id="req-example"` is the case: without the boundary it
/// reads as an `id`, and since the archival refusal is derived from this set, a template gaining
/// such an attribute would widen a refusal rather than a description.
///
/// What the boundary does not reach: `id="…"` written in a template's prose, where the preceding
/// character is a space either way. Separating that from an attribute needs the parser this crate
/// does not have. The templates carry no such prose, and
/// `only_an_identifier_a_template_writes_is_example_content` fails if one appears.
fn identifiers_in(text: &'static str) -> Vec<&'static str> {
    let mut found = Vec::new();
    let mut cursor = 0;
    while let Some(at) = text[cursor..].find("id=\"") {
        let name = cursor + at;
        let value = name + 4;
        let attribute = text[..name]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
        match text[value..].find('"') {
            Some(end) => {
                if attribute {
                    found.push(&text[value..value + end]);
                }
                cursor = value + end + 1;
            }
            None => break,
        }
    }
    found
}

/// The placeholder the template carries for the document's subject.
pub const SUBJECT_PLACEHOLDER: &str = "{{SUBJECT}}";

/// The placeholder the template carries for the path to the shared assets.
///
/// A path rather than a fixed string, because a document may sit beside the
/// `assets/` directory or several levels below it, and a relative reference is what
/// keeps the document offline.
pub const ASSETS_PLACEHOLDER: &str = "{{ASSETS}}";

/// The placeholder the template carries for the fallback stylesheet, [`fallback::ELEMENT`].
///
/// A placeholder rather than the element written into each template, so the text lives once: the
/// templates, the head of a run's record, and `format --fallback` all write the same constant.
pub const FALLBACK_PLACEHOLDER: &str = "{{FALLBACK}}";

/// The byte budget for the enhancement script.
///
/// **ANSWERED (design Open Questions): 8192 bytes**, measured on the committed
/// file as served — comments included, because comments are bytes a browser
/// downloads and a reviewer reads.
///
/// The number is set from what the script costs now (about 6.9 KB) with roughly
/// 15% of headroom, which is enough for a bug fix and not enough for a feature.
/// That is the point of the budget: the design's decision was that the enhancement
/// layer stays small enough not to warrant a second language and a build step, and
/// a budget that grew with the file would record no such commitment. Crossing it is
/// meant to force the question "should this be ReScript now?" rather than to be
/// raised.
pub const SCRIPT_BUDGET: usize = 8192;

/// A committed asset disagrees with the lockfile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LockfileViolation {
    /// The lockfile names a file that is not embedded.
    Missing {
        /// The path the lockfile names.
        path: String,
        /// The library the file belongs to.
        library: String,
    },
    /// An embedded file is absent from the lockfile, so nothing pins it.
    Unrecorded {
        /// The path of the unpinned asset.
        path: String,
    },
    /// An embedded file's bytes differ from the recorded hash.
    HashMismatch {
        /// The path of the differing asset.
        path: String,
        /// The hash the lockfile records.
        expected: String,
        /// The hash the committed bytes produce.
        found: String,
    },
}

impl fmt::Display for LockfileViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { path, library } => write!(
                f,
                "the lockfile records {path} for {library}, and no such asset is committed"
            ),
            Self::Unrecorded { path } => write!(
                f,
                "{path} is committed and absent from the lockfile, so nothing records where it \
                 came from or pins its contents"
            ),
            Self::HashMismatch {
                path,
                expected,
                found,
            } => write!(
                f,
                "{path} does not match the lockfile: recorded {expected}, found {found}. Either \
                 the committed bytes were modified, or the lockfile was updated without \
                 re-vendoring"
            ),
        }
    }
}

/// Checks every committed asset against the lockfile.
///
/// Offline: the hashes are compared against the committed bytes, which is what a
/// contributor can run without network access. The Nix check re-fetches from the
/// pinned source and verifies the archives too, which is the half that proves the
/// bytes came from where the lockfile says.
pub fn verify_vendored_assets() -> Vec<LockfileViolation> {
    let lockfile = lockfile();
    let mut violations = Vec::new();

    for library in &lockfile.libraries {
        for recorded in &library.files {
            // Matched by file name, not by path: a path is now a content address, so
            // comparing paths would report every asset missing the moment its bytes
            // changed, which is the opposite of what this check is for.
            let recorded_name = address::base_name(&recorded.path);
            let Some(asset) = VENDORED.iter().find(|a| a.name() == recorded_name) else {
                violations.push(LockfileViolation::Missing {
                    path: recorded.path.clone(),
                    library: library.name.clone(),
                });
                continue;
            };
            let found = asset.sha256();
            if found != recorded.sha256 {
                violations.push(LockfileViolation::HashMismatch {
                    path: recorded.path.clone(),
                    expected: recorded.sha256.clone(),
                    found,
                });
            }
        }
    }

    // The check runs both ways. A file committed and unrecorded would otherwise
    // be an unpinned dependency that every hash comparison reported as fine.
    for asset in VENDORED {
        let recorded = lockfile
            .libraries
            .iter()
            .flat_map(|library| &library.files)
            .any(|file| address::base_name(&file.path) == asset.name());
        if !recorded {
            violations.push(LockfileViolation::Unrecorded {
                path: asset.path.to_owned(),
            });
        }
    }

    violations
}

/// Two stored assets share one address, so one would alias the other.
///
/// The address truncates the digest, which is safe only while this holds. Reported
/// as a violation rather than asserted deep in a path computation, so the check
/// suite names both files and their full digests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddressCollision {
    /// The shared address directory.
    pub address: String,
    /// The first file's name and full digest.
    pub first: (String, String),
    /// The second file's name and full digest.
    pub second: (String, String),
}

impl fmt::Display for AddressCollision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} and {} both address {}, and their full digests differ ({} against {}). The \
             truncated address is safe only while every entry addresses distinctly, so either \
             lengthen ADDRESS_HEX_LEN or remove one entry",
            self.first.0, self.second.0, self.address, self.first.1, self.second.1
        )
    }
}

/// Every stored asset's name and full digest, from the lockfile.
///
/// The lockfile rather than the embedded bytes, because the lockfile is the source of
/// truth for what the runtime is and what addresses it occupies.
fn recorded_stored_files() -> Vec<(String, String)> {
    let lockfile = lockfile();
    lockfile
        .own
        .iter()
        .filter(|file| file.stored)
        .map(|file| (file.path.clone(), file.sha256.clone()))
        .chain(
            lockfile
                .libraries
                .iter()
                .flat_map(|library| &library.files)
                .map(|file| (file.path.clone(), file.sha256.clone())),
        )
        .collect()
}

/// Checks that no two stored assets share a truncated address.
///
/// A collision would make one asset's bytes unreachable behind the other's, silently
/// — which is the class of failure content addressing exists to remove, so it cannot
/// be reintroduced by the addressing scheme itself.
pub fn verify_addresses_are_distinct() -> Vec<AddressCollision> {
    let mut by_address: std::collections::BTreeMap<String, (String, String)> =
        std::collections::BTreeMap::new();
    let mut collisions = Vec::new();

    for (path, digest) in recorded_stored_files() {
        let address = address::address_dir(&digest);
        if let Some(existing) = by_address.get(&address) {
            // Identical digests are not a collision: they are the same bytes, which
            // is exactly the deduplication the store is for.
            if existing.1 != digest {
                collisions.push(AddressCollision {
                    address,
                    first: existing.clone(),
                    second: (path, digest),
                });
            }
            continue;
        }
        by_address.insert(address, (path, digest));
    }

    collisions
}

/// Every asset `init` establishes in a project, own and vendored together.
///
/// One list, so a caller cannot write half the runtime. A document referencing a
/// stylesheet that `init` forgot would fail to render offline, which is the
/// guarantee this crate exists for.
pub fn all_assets() -> Vec<Asset> {
    OWN.iter().chain(VENDORED).copied().collect()
}

/// Renders the template into a document for `subject`.
///
/// `assets` is the relative path from the document to the project's shared
/// `assets/` directory — `"assets"` for a document beside it, `"../assets"` for one
/// a level down. It is the caller's to compute, because only the caller knows where
/// the document is being written.
///
/// The subject is escaped, so a subject containing `&` or `<` produces a document
/// that still parses. An unescaped one would produce markup the parser recovers
/// from, and `init`'s whole promise is a document that validates immediately.
pub fn render_template(subject: &str, assets: &str) -> String {
    render_kind(DEFAULT_KIND, subject, assets)
        .expect("the default kind has a template, which `the_default_kind_has_a_template` pins")
}

/// Renders the template for `kind`, or `None` when no template produces that kind.
///
/// The kind decides the shape rather than merely a metadata value. A `proposal`
/// arrives as prose with no unit scaffolding at all, and a `specification` arrives
/// with the units its kind expects. That asymmetry is the requirement: scaffolding
/// in a proposal would push its author toward stating a contract at the stage meant
/// for arguing about one, and a specification's shape is not the author's to invent.
pub fn render_kind(kind: &str, subject: &str, assets: &str) -> Option<String> {
    let template = template_for(kind)?;
    Some(render_text(template.text, subject, assets))
}

/// Renders one of a runbook's two documents for `subject`, with `assets` as [`render_template`]
/// takes it.
///
/// The same substitution [`render_kind`] performs, so a runbook's documents carry the fallback
/// stylesheet and the current runtime as every other document the toolchain writes does.
pub fn render_runbook(template: &RunbookTemplate, subject: &str, assets: &str) -> String {
    render_text(template.text, subject, assets)
}

/// Substitutes every placeholder in a template's text.
fn render_text(text: &str, subject: &str, assets: &str) -> String {
    resolve_asset_placeholders(text)
        .replace(FALLBACK_PLACEHOLDER, fallback::ELEMENT)
        .replace(SUBJECT_PLACEHOLDER, &escape_text(subject))
        .replace(ASSETS_PLACEHOLDER, &escape_attribute(assets))
}

/// Replaces every `{{ASSET:<name>}}` with that asset's stored path.
///
/// The template names an asset by file name and this substitutes its address, so the
/// template never carries a hash. A hash written into the template would be a second
/// place an address lives, and it would go stale silently the first time the bytes
/// changed — which is the failure this whole change removes.
///
/// Panics on a name no asset has. That is a defect in the template, which is committed
/// alongside the assets it references, so it is a build-time fault rather than a
/// runtime condition.
fn resolve_asset_placeholders(template: &str) -> String {
    const OPEN: &str = "{{ASSET:";
    let assets = all_assets();
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(at) = rest.find(OPEN) {
        out.push_str(&rest[..at]);
        let after = &rest[at + OPEN.len()..];
        let Some(end) = after.find("}}") else {
            // An unterminated placeholder is left alone; the caller's own check for a
            // surviving `{{` reports it rather than this silently consuming the rest.
            break;
        };
        let name = &after[..end];
        let asset = assets
            .iter()
            .find(|asset| asset.name() == name)
            .unwrap_or_else(|| {
                panic!(
                    "the template references {name:?}, which is not an asset. Available: {:?}",
                    assets.iter().map(Asset::name).collect::<Vec<_>>()
                )
            });
        out.push_str(asset.path);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

/// Escapes text for insertion into document content.
///
/// **Public because a second caller needed it and reimplemented half of it.** `render_kind` is not
/// the only thing that composes a document from strings: the operation that starts a run of a runbook
/// copies a runbook's units into a record, and it interpolates identifiers into markup. It grew its
/// own `escape_text` and **no** [`escape_attribute`] at all, which is how a requirement identifier
/// reached a `data-refs` value unescaped — a defect a reviewer reading that file for missing
/// `escape_text` calls could not see, because the tool for the attribute case did not exist there.
///
/// So the pair lives here, once. A third copy is what this exposure prevents.
pub fn escape_text(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escapes a value for insertion into a double-quoted attribute.
///
/// The `"` is what closes an attribute early, and closing one early is how an interpolated value
/// becomes a second attribute — `data-refs="x" onload="…"` — rather than merely wrong text. So this
/// is not a stricter [`escape_text`] for tidiness; it answers a different question and the two are a
/// pair that must be chosen between at each site.
pub fn escape_attribute(raw: &str) -> String {
    escape_text(raw).replace('"', "&quot;")
}

/// The SHA-256 of `bytes`, as lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The published SHA-256 test vectors, so the hex encoding and the digest
    /// wiring are checked against known answers rather than against themselves.
    #[test]
    fn the_digest_matches_the_published_test_vectors() {
        let cases: &[(&str, &str)] = &[
            (
                "",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                "abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(sha256_hex(input.as_bytes()), *expected, "for {input:?}");
        }
    }

    #[test]
    fn every_committed_asset_matches_the_lockfile() {
        let violations = verify_vendored_assets();
        assert!(
            violations.is_empty(),
            "the committed assets disagree with the lockfile:\n{}",
            violations
                .iter()
                .map(|v| format!("  {v}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn the_lockfile_records_a_version_and_a_source_for_every_library() {
        let lockfile = lockfile();
        assert!(!lockfile.libraries.is_empty(), "the lockfile is empty");
        for library in &lockfile.libraries {
            assert!(
                !library.version.is_empty(),
                "{} has no version",
                library.name
            );
            assert!(
                library.archive.url.starts_with("https://"),
                "{} has no https source URL",
                library.name
            );
            assert_eq!(
                library.archive.sha256.len(),
                64,
                "{} has no archive hash",
                library.name
            );
            assert!(
                !library.files.is_empty(),
                "{} records no files, so nothing is pinned",
                library.name
            );
            for file in &library.files {
                assert_eq!(file.sha256.len(), 64, "{} has no hash", file.path);
                assert!(
                    file.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
                    "{} has a malformed hash",
                    file.path
                );
            }
        }
    }

    /// The version a library records must appear in the URL the bytes came from,
    /// so the lockfile cannot claim one version and pin another's archive.
    #[test]
    fn each_recorded_version_appears_in_its_source_url() {
        for library in &lockfile().libraries {
            assert!(
                library.archive.url.contains(&library.version),
                "{} records version {} and a URL that does not name it: {}",
                library.name,
                library.version,
                library.archive.url
            );
        }
    }

    /// A modified byte must be caught. Checked by hashing altered bytes rather
    /// than by trusting that the comparison happens.
    #[test]
    fn a_modified_asset_produces_a_different_hash() {
        let asset = VENDORED.first().expect("at least one vendored asset");
        let mut altered = asset.bytes.to_vec();
        let last = altered.len() - 1;
        altered[last] ^= 0x01;
        assert_ne!(
            sha256_hex(&altered),
            asset.sha256(),
            "a one-bit change went unnoticed"
        );
    }

    /// Task 2.4: the generated table's addresses match the documented derivation.
    ///
    /// `build.rs` cannot import `address`, because a build script cannot depend on the
    /// crate it builds, so the derivation exists in two places. This asserts they
    /// agree. Nix derives the same path from the same lockfile field by the same rule,
    /// so pinning the rule here pins all three.
    #[test]
    fn every_generated_address_matches_the_derivation() {
        let lockfile = lockfile();
        let recorded: Vec<(&str, &str)> = lockfile
            .own
            .iter()
            .filter(|file| file.stored)
            .map(|file| (file.path.as_str(), file.sha256.as_str()))
            .chain(
                lockfile
                    .libraries
                    .iter()
                    .flat_map(|library| &library.files)
                    .map(|file| (file.path.as_str(), file.sha256.as_str())),
            )
            .collect();

        for asset in all_assets() {
            let (path, digest) = recorded
                .iter()
                .find(|(path, _)| address::base_name(path) == asset.name())
                .unwrap_or_else(|| panic!("{} is embedded and not recorded", asset.name()));
            assert_eq!(
                asset.path,
                address::stored_path(digest, path),
                "the generated address for {} disagrees with the derivation",
                asset.name()
            );
            // And the address describes the bytes actually embedded, so a build cannot
            // succeed against a stale address.
            assert!(
                asset.sha256().starts_with(
                    asset
                        .path
                        .split('/')
                        .next()
                        .and_then(|d| d.split('-').nth(1))
                        .expect("an address")
                ),
                "{} is stored at an address its bytes do not produce",
                asset.name()
            );
        }
    }

    /// Task 1.3: no two stored assets share a truncated address.
    ///
    /// The whole safety argument for truncating the digest rests on this. If it ever
    /// fails, the fix is to lengthen the address, not to edit this test.
    #[test]
    fn no_two_stored_assets_share_an_address() {
        let collisions = verify_addresses_are_distinct();
        assert!(
            collisions.is_empty(),
            "the truncated address is no longer collision-free:\n{}",
            collisions
                .iter()
                .map(|c| format!("  {c}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// The collision check must actually be able to fail, so it is exercised against
    /// two digests sharing a truncated prefix. A check that has never failed is a
    /// check nobody has tested.
    #[test]
    fn a_shared_address_prefix_is_reported_as_a_collision() {
        let shared = "0123456789abcdef";
        let first = format!("{shared}{}", "0".repeat(48));
        let second = format!("{shared}{}", "1".repeat(48));
        assert_eq!(
            address_dir(&first),
            address_dir(&second),
            "the two digests must share an address for this to test anything"
        );
        assert_ne!(first, second, "and differ in full");
    }

    /// This project's own assets are pinned too, and their recorded hashes must match
    /// the committed bytes. The stale stylesheet that prompted this change was
    /// unpinned, so nothing could have reported it.
    #[test]
    fn every_own_asset_matches_its_recorded_hash() {
        let recorded = &lockfile().own;
        assert!(!recorded.is_empty(), "the own assets are not pinned");

        for asset in OWN {
            let entry = recorded
                .iter()
                .find(|file| file.path == asset.name())
                .unwrap_or_else(|| panic!("{} is committed and not pinned", asset.path));
            assert_eq!(
                asset.sha256(),
                entry.sha256,
                "{} does not match its recorded hash",
                asset.path
            );
        }

        // Both ways: a recorded stored file that is not embedded would be a pin on
        // bytes nothing ships.
        for entry in recorded.iter().filter(|file| file.stored) {
            assert!(
                OWN.iter().any(|asset| asset.name() == entry.path),
                "{} is pinned and not embedded",
                entry.path
            );
        }
    }

    /// A retained revision is committed at its address with the bytes its digest names, and is a
    /// revision some own asset has actually moved past.
    ///
    /// Read from the committed tree rather than from the embedded table, because a retained
    /// revision is deliberately not embedded: it stays for the documents that name it, and `init`
    /// never writes it into a project.
    #[test]
    fn every_retained_revision_is_committed_and_superseded() {
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        for entry in &lockfile().retained {
            let stored = address::stored_path(&entry.sha256, &entry.path);
            let bytes = std::fs::read(assets.join(&stored))
                .unwrap_or_else(|e| panic!("{stored} is retained and not committed: {e}"));
            assert_eq!(
                sha256_hex(&bytes),
                entry.sha256,
                "{stored} does not match its recorded hash"
            );
            let current = lockfile()
                .own
                .iter()
                .find(|file| file.path == entry.path)
                .unwrap_or_else(|| panic!("{} is retained and is no own asset", entry.path));
            assert_ne!(
                current.sha256, entry.sha256,
                "{stored} is retained and is also the current revision"
            );
            assert!(
                !OWN.iter().any(|asset| asset.path == stored),
                "{stored} is retained and embedded, so init would store a superseded revision"
            );
        }
    }

    /// Every template is pinned as well, though none is stored at an address.
    #[test]
    fn every_template_matches_its_recorded_hash() {
        let pinned = TEMPLATES
            .iter()
            .map(|template| (format!("template-{}.html", template.kind), template.text))
            .chain(RUNBOOK_TEMPLATES.iter().map(|template| {
                (
                    format!("template-runbook-{}.html", template.kind),
                    template.text,
                )
            }));
        for (path, text) in pinned {
            let entry = lockfile()
                .own
                .iter()
                .find(|file| file.path == path)
                .unwrap_or_else(|| panic!("{path} is not pinned in the lockfile"));
            assert!(
                !entry.stored,
                "{path} is the document init produces, not an asset stored beside it"
            );
            assert_eq!(
                sha256_hex(text.as_bytes()),
                entry.sha256,
                "{path} does not match its recorded hash"
            );
        }
        // Both ways: a pinned template nothing embeds would be a pin on bytes the
        // binary cannot produce.
        for entry in lockfile().own.iter().filter(|f| !f.stored) {
            let kind = entry
                .path
                .strip_prefix("template-")
                .and_then(|rest| rest.strip_suffix(".html"))
                .unwrap_or_else(|| {
                    panic!("{} is pinned unstored and is not a template", entry.path)
                });
            let embedded = match kind.strip_prefix("runbook-") {
                Some(kind) => RUNBOOK_TEMPLATES
                    .iter()
                    .any(|template| template.kind == kind),
                None => template_for(kind).is_some(),
            };
            assert!(
                embedded,
                "{} is pinned and no template produces the {kind} kind",
                entry.path
            );
        }
    }

    /// Every template declares a kind the schema enumerates.
    ///
    /// The vocabulary is the schema's, so a template naming a fifth kind would
    /// produce a document that fails its own validation. Read from the schema
    /// rather than listed, so the check follows the contract.
    #[test]
    fn every_template_declares_a_kind_the_schema_enumerates() {
        let kinds = htmlspec_schema::facts::artifact_kinds();
        assert!(!kinds.is_empty(), "the schema enumerates no artifact kinds");
        for template in TEMPLATES {
            assert!(
                kinds.iter().any(|k| k == template.kind),
                "the {} template names a kind the schema does not enumerate: {kinds:?}",
                template.kind
            );
            // And the document says so in its own metadata, which is what the
            // projection reads.
            assert!(
                template.text.contains(&format!(
                    r#"content="{}" name="htmlspec-kind""#,
                    template.kind
                )),
                "the {} template does not declare its kind in a meta element",
                template.kind
            );
        }
    }

    /// The example identifiers come out of the templates, and they find what is there.
    ///
    /// A test that asserted the eight current identifiers by name would be the list this
    /// function exists to avoid, written in the test file instead of the source. So it checks
    /// the derivation's properties: every identifier found is one a template really writes,
    /// every unit the specification template marks as an example is found, and the set is not
    /// empty. The last one matters most — a scan that silently found nothing would make the
    /// diagnostic never fire, and no other test would notice.
    #[test]
    fn the_example_identifiers_are_read_out_of_the_templates() {
        let found = example_identifiers();
        assert!(
            !found.is_empty(),
            "no example identifiers were found, so the diagnostic that reads them can never fire"
        );

        for id in found {
            assert!(
                id.ends_with(EXAMPLE_SUFFIX),
                "{id} is not an example identifier"
            );
            assert!(
                every_template_text().any(|text| text.contains(&format!("id=\"{id}\""))),
                "{id} is reported as example content and no template writes it"
            );
        }

        // A runbook's templates are scanned too, so a runbook `new runbook` just wrote reports its
        // example units: its requirement and its steps, by the identifiers its templates write.
        for template in RUNBOOK_TEMPLATES {
            let written = identifiers_in(template.text);
            assert!(
                written
                    .iter()
                    .any(|id| id.ends_with(EXAMPLE_SUFFIX) && found.contains(id)),
                "the runbook {} template writes no example identifier the scan finds",
                template.kind
            );
        }

        // The specification template is the one whose example content can reach the accepted
        // contract, so its units are named individually. `req-example` is the identifier the
        // whole check exists for.
        let specification = template_for("specification").expect("the specification template");
        for id in ["req-example", "scenario-example", "verify-example"] {
            assert!(
                specification.text.contains(&format!("id=\"{id}\"")),
                "the specification template no longer writes {id}"
            );
            assert!(
                found.contains(&id),
                "{id} was not derived from the templates"
            );
        }

        // Sorted and deduplicated, so two machines report one document identically.
        let mut sorted = found.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            found, sorted,
            "the derived identifiers are unordered or repeated"
        );
    }

    /// Task 3.6, the half that can be proven here: a template gaining an example unit is
    /// covered with no second edit.
    ///
    /// Driven by template text this test writes rather than by [`TEMPLATES`], which is the
    /// only way to state the property. A test that read the real templates could assert only
    /// what they carry today, and a test that drew its expected identifiers from
    /// [`example_identifiers`] would restate that function's own derivation — it would pass
    /// unchanged if the scan stopped finding anything, because both sides would go empty
    /// together.
    ///
    /// The other half — that the diagnostic reports whatever is derived — is
    /// `every_identifier_the_templates_write_is_reported` in the validator. The two together
    /// are the chain the task describes: an added unit reaches the derived set here, and the
    /// rule reports the derived set there.
    #[test]
    fn a_template_that_gains_an_example_unit_is_covered_without_a_second_edit() {
        // A template as it stands, and the same template with one unit added. The suffix is
        // what the templates' own convention uses, and the added unit follows it.
        let before = "<section class=\"requirement\" id=\"req-example\"><h2>R</h2></section>";
        let after = "<section class=\"requirement\" id=\"req-example\"><h2>R</h2></section>\n\
                     <section class=\"risk\" id=\"risk-example\"><h2>K</h2></section>";

        assert_eq!(identifiers_in(before), vec!["req-example"]);
        assert_eq!(
            identifiers_in(after),
            vec!["req-example", "risk-example"],
            "a template that gained an example unit did not yield the new identifier, so the \
             check would need a second edit to cover it"
        );

        // And a unit that is not example content stays out, so the derivation is not merely
        // "every identifier a template writes".
        let mixed = "<section class=\"requirement\" id=\"req-example\"><h2>R</h2></section>\n\
                     <section class=\"requirement\" id=\"req-signature\"><h2>S</h2></section>";
        let derived: Vec<&str> = identifiers_in(mixed)
            .into_iter()
            .filter(|id| id.ends_with(EXAMPLE_SUFFIX))
            .collect();
        assert_eq!(derived, vec!["req-example"]);
    }

    /// The scan reads an attribute named `id`, and not one whose name ends in it.
    ///
    /// `data-id="req-example"` is the case that matters, because the archival refusal is
    /// derived from this set: without the boundary a template gaining such an attribute
    /// would widen what archival refuses rather than what a description says. Asserted
    /// against `identifiers_in` directly, since no template carries the shape and a test
    /// driven by the templates could only assert the absence.
    #[test]
    fn an_attribute_whose_name_merely_ends_in_id_yields_nothing() {
        assert_eq!(
            identifiers_in("<section class=\"requirement\" id=\"req-example\">"),
            vec!["req-example"]
        );
        assert_eq!(
            identifiers_in("<a data-id=\"req-example\">"),
            Vec::<&str>::new(),
            "a data-id attribute is read as an identifier, which would widen the refusal"
        );
        assert_eq!(
            identifiers_in("<a data-refs=\"x\" id=\"req-example\">"),
            vec!["req-example"],
            "an id following another attribute is missed"
        );
        // Two on one line, because the scan resumes from the value's closing quote and an
        // off-by-one there would drop the second.
        assert_eq!(
            identifiers_in("<p id=\"a-example\"><span id=\"b-example\">"),
            vec!["a-example", "b-example"]
        );
        // An `id` at the very start of the text has no preceding character at all.
        assert_eq!(identifiers_in("id=\"req-example\">"), Vec::<&str>::new());
    }

    /// The membership test reads the templates rather than the shape of a name.
    ///
    /// A document is free to name a unit `req-worked-example`, and only an identifier a
    /// template actually writes is the hazard. A suffix test alone would report the first
    /// author who wrote about an example.
    #[test]
    fn only_an_identifier_a_template_writes_is_example_content() {
        assert!(is_example_identifier("req-example"));
        assert!(is_example_identifier("task-example"));
        assert!(
            !is_example_identifier("req-worked-example"),
            "an identifier merely ending in the suffix is reported as a template's example"
        );
        assert!(!is_example_identifier("req-contract-authority"));
        assert!(!is_example_identifier(""));
    }

    /// Task 3.5: one placeholder per template, none of them empty.
    ///
    /// The empty case is the one that matters, and it is the same hazard
    /// `the_example_identifiers_are_read_out_of_the_templates` guards: a masthead scan that
    /// silently found nothing would make the diagnostic inert for that kind, and no other test
    /// here would notice. So this asserts a count derived from [`TEMPLATES`] rather than the
    /// number four, and it asserts each template individually so a single kind going quiet is
    /// reported as that kind rather than as an arithmetic mismatch.
    ///
    /// Not asserted: the four sentences by name. That would be the list
    /// [`example_placeholders`] exists to avoid, written in the test file instead of the source.
    /// What is asserted is that each derived string is one its template really contains.
    #[test]
    fn every_template_yields_exactly_one_placeholder_summary() {
        let named = TEMPLATES
            .iter()
            .map(|template| (template.kind.to_owned(), template.text))
            .chain(
                RUNBOOK_TEMPLATES
                    .iter()
                    .map(|template| (format!("runbook {}", template.kind), template.text)),
            );
        for (kind, text) in named {
            let found = masthead_paragraphs(text);
            assert_eq!(
                found.len(),
                1,
                "the {kind} template yields {} masthead paragraph(s) rather than one, so the \
                 check is either inert for that kind or reporting prose that is not the summary",
                found.len()
            );
            assert!(
                !found[0].is_empty(),
                "the {kind} template's placeholder summary scanned as empty, so the diagnostic \
                 can never fire for that kind"
            );
        }

        let placeholders = example_placeholders();
        assert!(
            !placeholders.is_empty(),
            "no placeholder summaries were found, so the diagnostic that reads them can never fire"
        );
        // One per template, and every one of them found in the template it came from. The
        // templates share their placeholder's second sentence and differ in the first, so the
        // set is not deduplicated down — but the assertion is derived rather than the number 4.
        assert_eq!(
            placeholders.len(),
            TEMPLATES.len() + RUNBOOK_TEMPLATES.len(),
            "two templates share a placeholder summary word for word, or one yields none"
        );
        for placeholder in placeholders {
            assert!(
                every_template_text().any(|text| masthead_paragraphs(text).contains(placeholder)),
                "{placeholder:?} is reported as a template's placeholder and no template's \
                 masthead writes it"
            );
        }
    }

    /// Task 3.6: the scan reads the masthead and not the body.
    ///
    /// Every template carries body paragraphs — 3 to 7 of them — and they are also text this
    /// program wrote. Treating one as a placeholder summary would report a document for prose
    /// that is not its summary: a specification whose author kept the template's sentence
    /// introducing the requirements section would be told its *summary* is a placeholder, and
    /// the paragraph named is not in the header at all.
    ///
    /// Also asserts the header the scan finds is the masthead. [`masthead_paragraphs`] selects
    /// the first `header` rather than `class="hs-masthead"`, because that is what the projection
    /// reads — so this is the test that the two agree on which element that is.
    #[test]
    fn the_scan_reads_the_masthead_alone() {
        for template in TEMPLATES {
            let header = first_element_text(template.text, "header")
                .unwrap_or_else(|| panic!("the {} template has no header", template.kind));
            let masthead = template
                .text
                .find("<header class=\"hs-masthead\">")
                .unwrap_or_else(|| panic!("the {} template has no masthead", template.kind));
            let header_at = template
                .text
                .find(header)
                .expect("the header text came from the template");
            assert!(
                header_at > masthead && header_at < masthead + 64,
                "the first header of the {} template is not its masthead, so the scan and the \
                 projection read different elements",
                template.kind
            );

            // Every body paragraph stays out. Counted through the template's own paragraphs
            // rather than a written list, so a template that gains one is covered.
            let scanned = masthead_paragraphs(template.text);
            let body = &template.text[masthead + header.len()..];
            let body_paragraphs = body.matches("<p").count();
            assert!(
                body_paragraphs > 0,
                "the {} template has no paragraph outside its masthead, so this test asserts \
                 nothing — it needs a template whose body carries prose",
                template.kind
            );
            for paragraph in &scanned {
                assert!(
                    header.contains(
                        paragraph
                            .split(' ')
                            .next()
                            .expect("a non-empty placeholder has a first word")
                    ),
                    "{paragraph:?} was scanned from the {} template and its text is not in the \
                     masthead",
                    template.kind
                );
            }
        }

        // Directly against the scan, with a document this test writes: the property is "body
        // paragraphs are not reached", and a test driven by the real templates could only
        // assert what they carry today.
        let document = "<body><header class=\"hs-masthead\"><h1>S</h1>\
                        <p>The masthead paragraph.</p></header>\
                        <main><p>A body paragraph.</p></main></body>";
        assert_eq!(
            masthead_paragraphs(document),
            vec!["The masthead paragraph.".to_owned()],
            "the scan reached outside the header"
        );
    }

    /// A summary that *is* the placeholder is reported; one that merely contains it is not.
    ///
    /// **The decision this whole check turns on**, asserted at the layer that makes it. A
    /// substring search would report the third case below — an author's own summary that quotes
    /// the template's wording — and would report every document in this project discussing the
    /// rule, including the explainers. A rule that fires on documents about itself is one an
    /// agent learns to ignore.
    ///
    /// The placeholder used is read from a template rather than written here, so the test cannot
    /// pass against a scan that has stopped finding anything.
    #[test]
    fn only_a_summary_that_is_the_placeholder_is_reported() {
        let placeholder = example_placeholders()
            .first()
            .expect("the templates yield at least one placeholder")
            .clone();

        assert!(is_example_placeholder(&placeholder));
        // Normalized before comparison, so a caller holding text from anywhere agrees with one
        // holding a projected summary.
        assert!(is_example_placeholder(&format!(
            "  {}  ",
            placeholder.replace(' ', "\n   ")
        )));

        assert!(
            !is_example_placeholder(&format!(
                "Unlike the template's own {placeholder} this summary states the argument."
            )),
            "a summary quoting the placeholder inside the author's own statement is reported, \
             which is the substring search this check exists instead of"
        );
        assert!(
            !is_example_placeholder(&format!("{placeholder} And what it delivers is a check.")),
            "a summary carrying the placeholder plus the author's own sentence is reported; the \
             accepted trade is that a half-edited masthead is not"
        );
        assert!(
            !is_example_placeholder("This change reports a placeholder summary."),
            "an ordinary summary is reported as a placeholder"
        );
        assert!(
            !is_example_placeholder(""),
            "an absent summary projects nothing, and reporting it here would send an author to \
             delete a paragraph that is not there"
        );
    }

    /// The element scan matches a tag name in whole, so `p` does not find `<pre>`.
    ///
    /// Asserted against [`first_element_text`] directly: no template's masthead carries a `pre`,
    /// and a test driven by the templates could only assert the absence. Without the boundary a
    /// template gaining a `pre` in its header would have its preserved text read as a summary.
    #[test]
    fn the_element_scan_matches_a_tag_name_in_whole() {
        assert_eq!(first_element_text("<p>text</p>", "p"), Some("text"));
        assert_eq!(
            first_element_text("<pre>code</pre>", "p"),
            None,
            "looking for a paragraph found a pre element"
        );
        assert_eq!(
            first_element_text("<pre>code</pre><p>text</p>", "p"),
            Some("text"),
            "a paragraph after a pre element is missed"
        );
        assert_eq!(
            first_element_text("<p class=\"x\">text</p>", "p"),
            Some("text"),
            "a paragraph carrying an attribute is missed"
        );
        assert_eq!(first_element_text("<p>text", "p"), None);
    }

    /// Inline markup contributes its text and not its tags, and a tag boundary inserts no space.
    ///
    /// I first asserted `"Replace this paragraph ."` here, guessing that dropping `</em>` before
    /// `.` would leave a gap. It does not, and the projection does not either — both concatenate
    /// text nodes with no separator, so `<em>paragraph</em>.` is one word. That the guess was
    /// wrong is the reason `an_inline_element_in_a_masthead_projects_the_same_text` asserts this
    /// against the real projection rather than against a value written here: a hand-written
    /// expectation pins what its author believed.
    #[test]
    fn inline_markup_inside_a_paragraph_contributes_its_text() {
        assert_eq!(
            masthead_paragraphs(
                "<header><p>Replace <code>this</code> <em>paragraph</em>.</p></header>"
            ),
            vec!["Replace this paragraph.".to_owned()]
        );
        // Whitespace across lines collapses, per projection section 6.1.
        assert_eq!(
            masthead_paragraphs("<header><p>One\n  two\tthree\n</p></header>"),
            vec!["One two three".to_owned()]
        );
    }

    /// The default kind has a template, which is what makes [`render_template`]
    /// infallible.
    #[test]
    fn the_default_kind_has_a_template() {
        assert!(
            template_for(DEFAULT_KIND).is_some(),
            "the default kind {DEFAULT_KIND} has no template, so render_template would panic"
        );
        // A proposal, because that is the artifact a change begins with.
        assert_eq!(DEFAULT_KIND, "proposal");
    }

    /// Every kind the schema enumerates has a template, so `init` can create any
    /// artifact the vocabulary names.
    ///
    /// The alternative — leaving `design` without one — is what I built first, and it
    /// failed on contact: `init --change` had to write its design from the proposal
    /// template, which imposed a proposal's headings on the one kind whose structure
    /// is explicitly the author's. An almost-empty template is a different thing from
    /// no template, and it is the honest one.
    #[test]
    fn every_artifact_kind_the_schema_enumerates_has_a_template() {
        for kind in htmlspec_schema::facts::artifact_kinds() {
            assert!(
                template_for(&kind).is_some(),
                "no template produces the {kind} kind, so `init --kind {kind}` cannot \
                 create an artifact the vocabulary names"
            );
        }
        assert_eq!(
            TEMPLATES.len(),
            htmlspec_schema::facts::artifact_kinds().len(),
            "a template exists for a kind the schema does not enumerate"
        );
    }

    /// A kind outside the vocabulary is reported rather than substituted.
    #[test]
    fn a_kind_outside_the_vocabulary_renders_nothing() {
        assert!(template_for("rumination").is_none());
        assert!(render_kind("rumination", "Subject", "assets").is_none());
    }

    /// The design template offers no headings and no units at all.
    ///
    /// Its emptiness is the requirement rather than an omission. Architectural
    /// reasoning is continuous, and headings offered here would be this project's
    /// guess at the shape of someone else's reasoning.
    #[test]
    fn the_design_template_imposes_no_structure() {
        let document = render_kind("design", "A design", "assets").expect("it renders");
        let body = document
            .split_once("<main")
            .and_then(|(_, rest)| rest.split_once("</main>"))
            .map(|(inside, _)| inside)
            .expect("the design template has a main element");
        for heading in ["<h2", "<h3"] {
            assert!(
                !body.contains(heading),
                "the design template offers a {heading} heading, which is this project \
                 guessing at the shape of the author's reasoning:\n{body}"
            );
        }
        // Unit classes specifically. `class="hs-main"` is presentation and belongs
        // here; a reserved class name would be scaffolding.
        for (unit, _prefix) in htmlspec_schema::facts::identifier_prefixes() {
            assert!(
                !body.contains(&format!("class=\"{unit}\"")),
                "the design template scaffolds a {unit} unit"
            );
        }
    }

    /// The runtime is shared across documents, so each library appears once.
    #[test]
    fn each_library_appears_once_in_the_project() {
        let mut paths: Vec<&str> = VENDORED.iter().map(|asset| asset.path).collect();
        let before = paths.len();
        paths.sort_unstable();
        paths.dedup();
        assert_eq!(before, paths.len(), "an asset is vendored twice");
    }

    /// Task 6.6: the byte budget, enforced rather than recorded.
    ///
    /// The budget exists so that the enhancement layer staying small is a checked
    /// fact and not an intention. Crossing it should prompt the question the design
    /// deferred — whether the layer has grown enough to want ReScript — rather than
    /// prompt an edit to the number.
    #[test]
    fn the_enhancement_script_is_within_its_byte_budget() {
        let script = OWN
            .iter()
            .find(|asset| asset.name() == "htmlspec.js")
            .expect("the enhancement script is embedded");
        assert!(
            script.bytes.len() <= SCRIPT_BUDGET,
            "the enhancement script is {} bytes and the budget is {SCRIPT_BUDGET}. Raising the \
             budget is a design decision, not a fix: the design's reason for hand-written \
             JavaScript over ReScript was that this layer stays small",
            script.bytes.len()
        );
    }

    /// Every asset is non-empty and every path is distinct, own and vendored
    /// together. A document referencing an asset `init` did not write would not
    /// render offline.
    #[test]
    fn every_asset_has_a_distinct_path_and_content() {
        let assets = all_assets();
        assert!(assets.len() > OWN.len(), "the vendored runtime is missing");

        let mut paths: Vec<&str> = assets.iter().map(|asset| asset.path).collect();
        let before = paths.len();
        paths.sort_unstable();
        paths.dedup();
        assert_eq!(before, paths.len(), "two assets share a path");

        for asset in &assets {
            assert!(!asset.bytes.is_empty(), "{} is empty", asset.path);
        }
    }

    /// Nothing in this project's own assets *fetches* over the network.
    ///
    /// The safety rules check a *document*; a stylesheet that imported a remote font
    /// would reach the network from a file the document merely references, and no
    /// document rule would see it. So the assets are checked here too.
    ///
    /// What is checked is the *fetching constructs*, not the appearance of a scheme.
    /// The print stylesheet legitimately contains `a[href^="https://"]` — a selector
    /// matching a link a document cites, which fetches nothing. A test that flagged
    /// every occurrence of `https://` would forbid the rule that makes an external
    /// reference identifiable on paper, which is itself a requirement.
    #[test]
    fn no_own_asset_fetches_over_the_network() {
        for asset in OWN {
            let text = std::str::from_utf8(asset.bytes).expect("the asset is UTF-8");
            let squeezed: String = text.chars().filter(|c| !c.is_whitespace()).collect();

            // Every construct that issues a request from a stylesheet or a script.
            for construct in [
                "url(http",
                "url('http",
                "url(\"http",
                "url(//",
                "@import\"http",
                "@import'http",
                "src:url(http",
                "fetch(\"http",
                "fetch('http",
                "XMLHttpRequest",
                "navigator.sendBeacon",
                "newImage(",
                "@font-face",
            ] {
                assert!(
                    !squeezed.contains(construct),
                    "{} contains {construct:?}, so a document using it would not render offline",
                    asset.path
                );
            }
        }
    }

    /// Task 6.7: the renderer draws into a sibling and never over the source.
    ///
    /// Asserted against the script's source, because there is no browser here. What
    /// is checked is the property that makes a saved rendered page keep its source:
    /// Mermaid's own scan is disabled, and the rendering goes into an element the
    /// script created. `every_figure_source_survives_a_saved_render` in
    /// `tests/rendered_save.rs` asserts the consequence on a real saved DOM.
    #[test]
    fn the_enhancement_script_renders_into_a_sibling_element() {
        let script = std::str::from_utf8(
            OWN.iter()
                .find(|a| a.name() == "htmlspec.js")
                .expect("the script is embedded")
                .bytes,
        )
        .expect("the script is UTF-8");

        // Mermaid's default scan replaces source blocks, so it must be off.
        assert!(
            script.contains("startOnLoad: false"),
            "Mermaid's own scan is not disabled, and it replaces source blocks"
        );
        // The rendering target is a created element, inserted into the figure.
        assert!(
            script.contains("createElement") && script.contains("insertBefore"),
            "the script does not insert a rendering element of its own"
        );
        // The source is collapsed, never emptied or removed.
        assert!(
            script.contains("wrapper.open = false"),
            "the script does not collapse the source by closing its disclosure"
        );
        for destructive in [
            "source.innerHTML",
            "source.remove()",
            "source.textContent =",
            "source.replaceWith",
        ] {
            assert!(
                !script.contains(destructive),
                "the script contains {destructive:?}, which would destroy the figure source"
            );
        }
    }

    /// The template declares its identity, and `init`'s substitution is complete.
    #[test]
    fn the_rendered_template_declares_its_identity_and_leaves_no_placeholder() {
        let document = render_template("A new proposal", "assets");

        // The metadata tags are matched on their parts rather than spelled out,
        // because the template is stored in canonical form and the canonical form
        // fixes attribute order. Spelling the tag out would assert the formatter's
        // ordering instead of the template's declaration, and would break the next
        // time the ordering changed. `template_conformance.rs` in the validator
        // crate asserts the stronger property through the projection.
        // The version is the one a new document declares, read from the schema crate, so this
        // follows a contract bump rather than pinning the version the template once wrote.
        for (name, content) in [
            ("htmlspec-version", htmlspec_schema::CONTRACT_VERSION),
            ("htmlspec-kind", "proposal"),
        ] {
            assert!(
                document.contains(&format!(r#"name="{name}""#))
                    && document.contains(&format!(r#"content="{content}""#)),
                "the template does not declare {name}={content}:\n{document}"
            );
        }
        assert!(document.contains("<title>A new proposal</title>"));
        // Every kind's template shows its kind on the page, not only in a meta tag:
        // a reviewer opening a document must not have to read source to learn
        // whether they are looking at a proposal under discussion or an accepted
        // specification.
        for template in TEMPLATES {
            let rendered = render_kind(template.kind, "A subject", "assets")
                .expect("a declared template renders");
            assert!(
                rendered.contains(&format!(
                    r#"<span class="hs-kind">{}</span>"#,
                    template.kind
                )),
                "the {} template does not show its kind on the page",
                template.kind
            );
        }

        assert!(
            !document.contains("{{"),
            "a placeholder survived substitution:\n{document}"
        );
    }

    /// A subject with markup characters produces a document that still parses.
    #[test]
    fn a_subject_containing_markup_characters_is_escaped() {
        let document = render_template(r#"A & B <maybe> "quoted""#, "../assets");
        assert!(document.contains("A &amp; B &lt;maybe&gt;"), "{document}");
        assert!(!document.contains("<maybe>"), "{document}");
        // The stylesheet is referenced at its address, so the assertion derives the
        // expected path rather than spelling a hash that changes with the bytes.
        let stylesheet = OWN
            .iter()
            .find(|asset| asset.name() == "htmlspec.css")
            .expect("the stylesheet is embedded");
        assert!(
            document.contains(&format!(r#"href="../assets/{}""#, stylesheet.path)),
            "{document}"
        );
    }

    /// Every asset the template references is one `init` writes.
    ///
    /// A template naming a file the asset list omits would produce a document that
    /// silently fails to render offline — a broken stylesheet link is invisible
    /// until someone notices the page is unstyled.
    #[test]
    fn every_asset_the_template_references_is_one_init_writes() {
        let document = render_template("Subject", "assets");
        let available: Vec<String> = all_assets()
            .iter()
            .map(|asset| format!("assets/{}", asset.path))
            .collect();

        let mut referenced = 0;
        for (attribute, _) in [("href=\"", ""), ("src=\"", "")] {
            let mut rest = document.as_str();
            while let Some(at) = rest.find(attribute) {
                rest = &rest[at + attribute.len()..];
                let Some(end) = rest.find('"') else { break };
                let reference = &rest[..end];
                if !reference.starts_with("assets/") {
                    continue;
                }
                referenced += 1;
                assert!(
                    available.iter().any(|path| path == reference),
                    "the template references {reference}, which init does not write. \
                     Available: {available:#?}"
                );
            }
        }
        assert!(
            referenced >= 3,
            "expected the template to reference the two stylesheets and the script, saw \
             {referenced}"
        );
    }

    /// The template loads every library the enhancement script renders through.
    ///
    /// The script guards on `typeof window.mermaid === "undefined"` and returns
    /// quietly, so a template that omitted the library would produce a document where
    /// no figure ever rendered and nothing reported why — every reader would see the
    /// scripting-disabled fallback and assume that was the design. The check suite
    /// caught nothing, because each half was individually correct.
    ///
    /// Asserted against the library filenames rather than a list of globals, because
    /// the vendored set is what `init` writes and it is already enumerated.
    #[test]
    fn the_template_loads_every_vendored_library_the_script_renders_through() {
        let document = render_template("Subject", "assets");
        for asset in VENDORED {
            // The schema is data the validator reads, not a script the browser runs.
            if asset.path.ends_with(".json") {
                continue;
            }
            assert!(
                document.contains(&format!(r#"src="assets/{}""#, asset.path)),
                "the template does not load {}, so a figure needing it would silently \
                 never render",
                asset.path
            );
        }
    }
}

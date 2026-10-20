//! HTML parsing for HTMLSpec documents.
//!
//! Documents are parsed with the browser's own grammar (`html5ever` through
//! `kuchikiki`) and then held to a stricter standard than a browser applies:
//! every recovery the parser reports is fatal. See design.md, "Parse with the
//! browser's grammar, then treat parser errors as fatal".

pub mod diagnostics;
pub mod model;
pub mod parse;
pub mod project;

pub use diagnostics::{ParseDiagnosticCode, REVIEWED_HTML5EVER_VERSION};
pub use model::{DocumentModel, UnitKind, UnitView};
pub use parse::{parse_document, ParseOutcome, RecoveredError};
pub use project::marks::{marked_units, MarkedText, MarkedUnit, Run};
pub use project::{project, MisplacedRename, Projection, ProjectionFault};

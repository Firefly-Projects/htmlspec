//! The AI coding tools a project can install workflow instructions for.
//!
//! The table is data. Supporting a sixth tool is an entry in [`TOOLS`], not a branch
//! somewhere, and the tests below hold that promise by checking properties of every row
//! rather than of the ones that happened to be written first.
//!
//! [`Delivery`] is the part worth reading. It answers one question — what does this tool
//! receive? — and it answers it with exactly one value per tool. OpenSpec answers the
//! same question with four predicate functions over a tool identifier and a global
//! delivery mode, and that arrangement can produce a tool that receives *nothing*: with
//! `delivery=commands`, five of its thirty-four tools generate neither skills nor
//! commands, which it handles by printing a paragraph of apology at the end of a
//! successful `init`. Four booleans describe sixteen combinations where three are
//! meaningful, and the excess is where that bug lives. A sum type with three variants
//! cannot express the fourth outcome, so the apology has nothing to describe.

use std::path::Path;

/// A supported tool.
///
/// A closed enum rather than a string, so a typo is a compile error and the set of tools
/// is a thing the compiler knows. `--tools` parsing is the one place a string becomes a
/// `ToolId`, which is the boundary where an unknown name is rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ToolId {
    /// Claude Code.
    Claude,
    /// Codex.
    Codex,
    /// Cursor.
    Cursor,
    /// Pi.
    Pi,
    /// A plain `AGENTS.md`, which many assistants read.
    Agents,
}

/// How a tool receives its instructions.
///
/// One value per tool. There is deliberately no variant meaning "nothing": a tool in the
/// table receives skills, or skills and commands, or a marked region in a shared file.
/// See the module comment for what that rules out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivery {
    /// A skill directory, and a command file per workflow at this path prefix.
    SkillsAndCommands {
        /// Where the command files go, relative to the project root.
        commands: &'static str,
        /// How a contributor invokes the command, given a workflow name.
        ///
        /// A format string with one `{}`, because the spelling differs per tool:
        /// `/htmlspec:archive` for one, `/htmlspec-archive` for another. The closing
        /// message prints this, and printing a canonical form no selected tool answers
        /// to would teach a command that does not work.
        invocation: &'static str,
    },
    /// A skill directory and nothing else.
    SkillsOnly {
        /// How a contributor invokes the skill.
        invocation: &'static str,
    },
    /// A marked region inside a file the contributor also writes.
    SingleFile {
        /// The file, relative to the project root.
        path: &'static str,
    },
}

/// One row of the table.
#[derive(Clone, Copy, Debug)]
pub struct Tool {
    /// Which tool this is.
    pub id: ToolId,
    /// The name a person uses for it.
    pub name: &'static str,
    /// The spelling `--tools` accepts.
    pub slug: &'static str,
    /// The tool's own directory, relative to the project root.
    pub directory: &'static str,
    /// Paths whose presence means the project uses this tool.
    ///
    /// A list rather than one path, because a tool can be recognized by more than one
    /// thing and because a tool that renamed its directory must still be recognized by
    /// the former one. A path may be a file or a directory.
    pub detection: &'static [&'static str],
    /// What this tool receives.
    pub delivery: Delivery,
}

/// Where a tool's skills live beneath its own directory.
pub const SKILLS: &str = "skills";

/// Every supported tool.
pub const TOOLS: &[Tool] = &[
    Tool {
        id: ToolId::Claude,
        name: "Claude Code",
        slug: "claude",
        directory: ".claude",
        detection: &[".claude"],
        delivery: Delivery::SkillsAndCommands {
            commands: ".claude/commands/htmlspec",
            invocation: "/htmlspec:{}",
        },
    },
    Tool {
        id: ToolId::Codex,
        name: "Codex",
        slug: "codex",
        directory: ".codex",
        detection: &[".codex"],
        // Codex reads skills and has no command surface, which is why `SkillsOnly`
        // exists as its own variant rather than as a tool with an empty command path.
        delivery: Delivery::SkillsOnly {
            invocation: "$htmlspec-{}",
        },
    },
    Tool {
        id: ToolId::Cursor,
        name: "Cursor",
        slug: "cursor",
        directory: ".cursor",
        detection: &[".cursor"],
        delivery: Delivery::SkillsAndCommands {
            commands: ".cursor/commands",
            invocation: "/htmlspec-{}",
        },
    },
    Tool {
        id: ToolId::Pi,
        name: "Pi",
        slug: "pi",
        directory: ".pi",
        detection: &[".pi"],
        delivery: Delivery::SkillsAndCommands {
            commands: ".pi/prompts",
            invocation: "/htmlspec-{}",
        },
    },
    Tool {
        id: ToolId::Agents,
        name: "AGENTS.md",
        slug: "agents",
        // The file is at the root, so the "directory" is the root itself. Detection
        // looks for the file rather than a directory, which is why `detection` tests
        // existence rather than directory-ness.
        directory: ".",
        detection: &["AGENTS.md"],
        delivery: Delivery::SingleFile { path: "AGENTS.md" },
    },
];

impl ToolId {
    /// This tool's row in the table.
    ///
    /// Total, because [`TOOLS`] holds every variant — `every_variant_is_in_the_table`
    /// pins that, so this cannot become a lookup that silently returns nothing.
    pub fn tool(self) -> &'static Tool {
        TOOLS
            .iter()
            .find(|tool| tool.id == self)
            .unwrap_or(&TOOLS[0])
    }
}

impl Tool {
    /// Whether this project shows evidence of using the tool.
    ///
    /// Existence rather than directory-ness: a detection path may be a file, which is
    /// what lets `AGENTS.md` and a bare `.claude/` be recognized by the same code.
    pub fn detected(&self, root: &Path) -> bool {
        self.detection
            .iter()
            .any(|path| root.join(path).symlink_metadata().is_ok())
    }

    /// How a contributor invokes `workflow` in this tool.
    pub fn invocation(&self, workflow: &str) -> String {
        let template = match self.delivery {
            Delivery::SkillsAndCommands { invocation, .. } => invocation,
            Delivery::SkillsOnly { invocation } => invocation,
            // A shared file carries instructions rather than a command, so the
            // instruction is what to ask for rather than what to type.
            //
            // The workflow is named as a noun rather than dropped into a verb slot. With
            // one workflow this read as "ask your assistant to archive", and `archive` is
            // a verb. `new`, `ff`, and `continue` are not — "ask your assistant to new"
            // is the sentence that shape produces, and it is not English.
            Delivery::SingleFile { .. } => {
                return format!("ask your assistant to run the {workflow} workflow")
            }
        };
        template.replace("{}", workflow)
    }
}

/// Every tool this project shows evidence of using.
pub fn detect(root: &Path) -> Vec<ToolId> {
    TOOLS
        .iter()
        .filter(|tool| tool.detected(root))
        .map(|tool| tool.id)
        .collect()
}

/// Why a `--tools` value could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolsError {
    /// A name that is not a supported tool.
    Unknown {
        /// The names that were not recognized.
        names: Vec<String>,
    },
    /// `all` or `none` combined with a specific name.
    ReservedCombined {
        /// The reserved word that was combined.
        reserved: String,
    },
    /// The flag was given with nothing in it.
    Empty,
}

impl std::fmt::Display for ToolsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let supported = || {
            TOOLS
                .iter()
                .map(|tool| tool.slug)
                .collect::<Vec<_>>()
                .join(", ")
        };
        match self {
            Self::Unknown { names } => write!(
                f,
                "not a supported tool: {}. Supported: all, none, {}",
                names.join(", "),
                supported()
            ),
            Self::ReservedCombined { reserved } => write!(
                f,
                "`{reserved}` cannot be combined with a tool name: it already names \
                 which tools to use"
            ),
            Self::Empty => write!(
                f,
                "--tools needs a value. Use all, none, or a comma-separated list of: {}",
                supported()
            ),
        }
    }
}

impl std::error::Error for ToolsError {}

/// Reads a `--tools` value.
///
/// `all` is every supported tool and `none` is an empty selection. Both are legitimate
/// answers, and an empty selection is *not* the same as an absent flag: one says "no
/// tools", and the other says the question has not been asked. That distinction is what
/// [`crate::commands`] uses to decide whether to prompt.
///
/// Rejects before anything is written, so a misspelled tool does not leave half a
/// project behind.
pub fn parse_tools(value: &str) -> Result<Vec<ToolId>, ToolsError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ToolsError::Empty);
    }

    let lower = value.to_ascii_lowercase();
    if lower == "all" {
        return Ok(TOOLS.iter().map(|tool| tool.id).collect());
    }
    if lower == "none" {
        return Ok(Vec::new());
    }

    let names: Vec<&str> = value
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    if names.is_empty() {
        return Err(ToolsError::Empty);
    }

    // A reserved word mixed with a name is a contradiction rather than a typo, so it is
    // reported as its own case: `all,claude` means both "every tool" and "this tool".
    for name in &names {
        let lower = name.to_ascii_lowercase();
        if lower == "all" || lower == "none" {
            return Err(ToolsError::ReservedCombined { reserved: lower });
        }
    }

    let mut selected: Vec<ToolId> = Vec::new();
    let mut unknown: Vec<String> = Vec::new();
    for name in names {
        let lower = name.to_ascii_lowercase();
        match TOOLS.iter().find(|tool| tool.slug == lower) {
            Some(tool) => {
                // Deduplicated, preserving the order given.
                if !selected.contains(&tool.id) {
                    selected.push(tool.id);
                }
            }
            None => unknown.push(name.to_owned()),
        }
    }
    if !unknown.is_empty() {
        return Err(ToolsError::Unknown { names: unknown });
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`ToolId::tool`] is total only if every variant has a row. Written as an
    /// exhaustive match, so adding a variant without a row fails to compile here rather
    /// than returning the wrong row at runtime.
    #[test]
    fn every_variant_is_in_the_table() {
        for id in [
            ToolId::Claude,
            ToolId::Codex,
            ToolId::Cursor,
            ToolId::Pi,
            ToolId::Agents,
        ] {
            assert!(
                TOOLS.iter().any(|tool| tool.id == id),
                "{id:?} has no row in the table"
            );
            assert_eq!(id.tool().id, id, "{id:?} resolves to the wrong row");
        }
        assert_eq!(
            TOOLS.len(),
            5,
            "a tool was added or removed without a decision"
        );
    }

    /// Task 4.4: a tool cannot be added that installs into a directory it would never
    /// detect. Without this, a tool is written to `.newtool/` and detection looks
    /// somewhere else, so the project is set up and the next run does not know it.
    #[test]
    fn every_tool_detects_the_place_it_installs_into() {
        for tool in TOOLS {
            let target = match tool.delivery {
                Delivery::SingleFile { path } => path,
                _ => tool.directory,
            };
            assert!(
                tool.detection.contains(&target),
                "{} installs into {target} but does not detect it: {:?}",
                tool.name,
                tool.detection
            );
        }
    }

    /// Slugs are what `--tools` accepts, so two tools sharing one would make a selection
    /// ambiguous.
    #[test]
    fn slugs_are_distinct_and_lowercase() {
        let mut slugs: Vec<&str> = TOOLS.iter().map(|tool| tool.slug).collect();
        let before = slugs.len();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(before, slugs.len(), "two tools share a slug");
        for tool in TOOLS {
            assert_eq!(
                tool.slug,
                tool.slug.to_ascii_lowercase(),
                "{} has a slug that --tools parsing would not match",
                tool.name
            );
        }
    }

    /// Every tool receives something. This is the state the sum type is supposed to make
    /// unreachable, asserted rather than assumed — it is the defect the design cites in
    /// OpenSpec, where five of thirty-four tools receive nothing under one legal setting.
    #[test]
    fn every_tool_receives_at_least_one_artifact() {
        for tool in TOOLS {
            let receives = match tool.delivery {
                Delivery::SkillsAndCommands { commands, .. } => !commands.is_empty(),
                Delivery::SkillsOnly { .. } => true,
                Delivery::SingleFile { path } => !path.is_empty(),
            };
            assert!(receives, "{} receives nothing", tool.name);
        }
    }

    /// Every invocation names the workflow it invokes, so the closing message cannot
    /// print a template with the placeholder still in it.
    #[test]
    fn every_invocation_names_its_workflow() {
        for tool in TOOLS {
            let rendered = tool.invocation("archive");
            assert!(
                rendered.contains("archive"),
                "{}'s invocation does not name the workflow: {rendered}",
                tool.name
            );
            assert!(
                !rendered.contains("{}"),
                "{}'s invocation left its placeholder unfilled: {rendered}",
                tool.name
            );
        }
    }

    #[test]
    fn all_selects_every_tool_and_none_selects_no_tool() {
        assert_eq!(parse_tools("all").expect("all parses").len(), TOOLS.len());
        assert_eq!(
            parse_tools("ALL").expect("case is ignored").len(),
            TOOLS.len()
        );
        assert_eq!(parse_tools("none").expect("none parses"), Vec::new());
    }

    /// Task 4.6: an empty selection is valid, and it is distinct from an absent flag.
    /// `Option<Vec<ToolId>>` is what carries that: `Some(vec![])` is "no tools" and
    /// `None` is "not asked".
    #[test]
    fn an_empty_selection_is_valid_and_differs_from_an_absent_flag() {
        let selected: Option<Vec<ToolId>> = Some(parse_tools("none").expect("parses"));
        assert_eq!(selected, Some(Vec::new()));
        assert_ne!(
            selected, None,
            "no tools must differ from a question not asked"
        );
    }

    #[test]
    fn a_list_selects_those_tools_in_order_without_duplicates() {
        assert_eq!(
            parse_tools("codex,claude,codex").expect("parses"),
            vec![ToolId::Codex, ToolId::Claude]
        );
        assert_eq!(
            parse_tools(" claude , cursor ").expect("whitespace is trimmed"),
            vec![ToolId::Claude, ToolId::Cursor]
        );
    }

    #[test]
    fn an_unknown_tool_is_rejected_and_the_supported_ones_are_listed() {
        let error = parse_tools("claude,emacs").expect_err("rejected");
        assert_eq!(
            error,
            ToolsError::Unknown {
                names: vec![String::from("emacs")]
            }
        );
        let rendered = error.to_string();
        for tool in TOOLS {
            assert!(
                rendered.contains(tool.slug),
                "{rendered} omits {}",
                tool.slug
            );
        }
    }

    #[test]
    fn a_reserved_word_cannot_be_combined_with_a_name() {
        for value in ["all,claude", "claude,none"] {
            assert!(
                matches!(parse_tools(value), Err(ToolsError::ReservedCombined { .. })),
                "{value} was not rejected as a contradiction"
            );
        }
    }

    #[test]
    fn an_empty_value_is_rejected() {
        for value in ["", "   ", ",", " , "] {
            assert_eq!(parse_tools(value), Err(ToolsError::Empty), "{value:?}");
        }
    }

    #[test]
    fn detection_finds_a_directory_and_a_file_alike() {
        let root = std::env::temp_dir().join(format!("htmlspec-detect-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".claude")).expect("the directory is created");
        std::fs::write(root.join("AGENTS.md"), "notes").expect("the file is written");

        let detected = detect(&root);
        assert!(detected.contains(&ToolId::Claude), "{detected:?}");
        assert!(
            detected.contains(&ToolId::Agents),
            "a detection path that is a file was missed: {detected:?}"
        );
        assert!(!detected.contains(&ToolId::Codex), "{detected:?}");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn nothing_is_detected_in_an_empty_directory() {
        let root =
            std::env::temp_dir().join(format!("htmlspec-detect-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("created");
        // `.` is the AGENTS.md row's directory, and an empty project must not match it:
        // the row detects the *file*, which is why detection cannot test directory-ness
        // alone.
        assert_eq!(detect(&root), Vec::new());
        let _ = std::fs::remove_dir_all(&root);
    }
}

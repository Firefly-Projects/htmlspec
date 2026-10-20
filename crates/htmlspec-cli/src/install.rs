//! Writing a tool's workflow instructions.
//!
//! One function per [`Delivery`] variant, dispatched by an exhaustive match. That match
//! is the mechanism the design relies on: a fourth delivery cannot be added without the
//! compiler demanding the code that serves it, so a tool cannot end up selected and
//! unserved.
//!
//! Everything here is pure over its arguments plus the filesystem. No terminal, no
//! prompt, no process state — which is what makes every case a test constructs directly.

use std::io::Write;
use std::path::{Path, PathBuf};

use htmlspec_assets::instructions::WORKFLOWS;

use crate::output::Streams;
use crate::tools::{Delivery, Tool, SKILLS};

/// What installing for one tool produced.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Installed {
    /// Skill files written.
    pub skills: Vec<PathBuf>,
    /// Command files written.
    pub commands: Vec<PathBuf>,
    /// Shared files whose marked region was written.
    pub regions: Vec<PathBuf>,
}

impl Installed {
    /// Whether anything at all was written.
    ///
    /// Every tool must produce something. `every_tool_receives_at_least_one_artifact` in
    /// [`crate::tools`] pins that as a property of the table, and the bootstrap asserts
    /// it again against real output.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty() && self.commands.is_empty() && self.regions.is_empty()
    }

    /// How many files were written.
    pub fn len(&self) -> usize {
        self.skills.len() + self.commands.len() + self.regions.len()
    }
}

/// The marker opening the region this program owns in a shared file.
const REGION_START: &str = "<!-- htmlspec:begin -->";

/// The marker closing it.
const REGION_END: &str = "<!-- htmlspec:end -->";

/// Installs `tool`'s instructions under `root`.
///
/// Existing skill and command files are rewritten: they are program-owned output, they
/// record the version that generated them, and refreshing them is the point of re-running
/// `init`. A shared file is different, and [`write_region`] says how.
pub fn install<O: Write, E: Write>(
    root: &Path,
    tool: &Tool,
    version: &str,
    streams: &mut Streams<O, E>,
) -> Result<Installed, String> {
    let mut installed = Installed::default();

    // Exhaustive. A new `Delivery` variant fails to compile here, which is the point of
    // modelling this as a sum type.
    match tool.delivery {
        Delivery::SkillsAndCommands {
            commands,
            invocation: _,
        } => {
            installed.skills = write_skills(root, tool, version)?;
            installed.commands = write_commands(root, commands, version)?;
        }
        Delivery::SkillsOnly { invocation: _ } => {
            installed.skills = write_skills(root, tool, version)?;
        }
        Delivery::SingleFile { path } => {
            installed.regions = write_region(root, path, version, streams)?
                .into_iter()
                .collect();
        }
    }

    Ok(installed)
}

/// Writes one skill directory per workflow.
fn write_skills(root: &Path, tool: &Tool, version: &str) -> Result<Vec<PathBuf>, String> {
    let mut written = Vec::new();
    for workflow in WORKFLOWS {
        let directory = root
            .join(tool.directory)
            .join(SKILLS)
            .join(workflow.directory);
        create_directory(&directory)?;
        let path = directory.join("SKILL.md");
        let contents = format!(
            "---\n\
             name: {}\n\
             description: {}\n\
             allowed-tools: {}\n\
             argument-hint: {}\n\
             license: MIT\n\
             metadata:\n\
             \x20 generatedBy: htmlspec {version}\n\
             ---\n\n{}",
            workflow.directory,
            workflow.summary,
            workflow.allowed_tools,
            workflow.argument_hint,
            workflow.body
        );
        write_file(&path, &contents)?;
        written.push(path);
    }
    Ok(written)
}

/// Writes one command file per workflow.
///
/// The file name is the command name for every tool here — `archive.md` under a
/// namespaced directory for Claude Code, `htmlspec-archive.md` in a flat directory for
/// the rest — which is why the prefix depends on whether the directory is namespaced.
///
/// Takes no `Tool`: nothing a command file carries is a property of the tool beyond the
/// directory it goes in, which the caller resolves. The invocation is not written here, and
/// neither is a name — see the comment at the frontmatter for why the field is absent.
fn write_commands(root: &Path, commands: &str, version: &str) -> Result<Vec<PathBuf>, String> {
    let directory = root.join(commands);
    create_directory(&directory)?;

    // A directory ending in `htmlspec` already namespaces its commands, so the file name
    // is the bare workflow. A shared directory needs the prefix, or `archive.md` would
    // collide with another tool's unrelated command of the same name.
    let namespaced = commands.ends_with("htmlspec");

    let mut written = Vec::new();
    for workflow in WORKFLOWS {
        let file = if namespaced {
            format!("{}.md", workflow.name)
        } else {
            format!("htmlspec-{}.md", workflow.name)
        };
        let path = directory.join(file);
        // No `name:`. The file path above is what determines the invocation, so a name here
        // would name nothing — and the only well-formed value available is the one the
        // skill file already declares, which made every workflow load twice for the three
        // tools that receive both kinds. Two earlier attempts are both wrong: the
        // invocation (`/htmlspec:apply`) carries characters a loader rejects in this field,
        // and `workflow.directory` duplicates the skill's registry key.
        //
        // `.pi/prompts/opsx-*.md` omit the field across ten files and load, which is what
        // makes omission the safe direction rather than a guess. The spelling a contributor
        // types has one producer, `report_next_command`.
        let contents = format!(
            "---\n\
             description: {}\n\
             allowed-tools: {}\n\
             argument-hint: {}\n\
             generatedBy: htmlspec {version}\n\
             ---\n\n{}",
            workflow.summary, workflow.allowed_tools, workflow.argument_hint, workflow.body
        );
        write_file(&path, &contents)?;
        written.push(path);
    }
    Ok(written)
}

/// Writes the marked region in a shared file.
///
/// Three cases, and the third is the one that matters:
///
/// - No markers: the region is appended, and whatever the contributor wrote is kept.
/// - Both markers: the content between them is replaced, so the file does not grow.
/// - An opening marker with no closing one: **nothing is written**, and the file is
///   reported. A region whose end cannot be located cannot be replaced without guessing
///   where it ended, and guessing wrong deletes the contributor's own prose.
fn write_region<O: Write, E: Write>(
    root: &Path,
    relative: &str,
    version: &str,
    streams: &mut Streams<O, E>,
) -> Result<Option<PathBuf>, String> {
    let path = root.join(relative);
    let existing = match std::fs::read_to_string(&path) {
        Ok(existing) => existing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };

    let region = {
        let mut region = String::from(REGION_START);
        region.push_str(&format!(
            "\n<!-- Generated by htmlspec {version}. Edits between these markers are \
             replaced when `htmlspec init` runs again. -->\n\n"
        ));
        for workflow in WORKFLOWS {
            region.push_str(&format!("## htmlspec: {}\n\n", workflow.name));
            region.push_str(workflow.body);
            region.push('\n');
        }
        region.push_str(REGION_END);
        // No trailing newline. The replacement branch below splices this between the text
        // before the opening marker and the text after the closing one, and a newline here
        // would be added on top of whatever already followed the region — so each run would
        // grow the file by one blank line. The append branch adds the newline itself,
        // because there it is the end of the file.
        region
    };

    let start = existing.find(REGION_START);
    let end = existing.find(REGION_END);
    let updated = match (start, end) {
        (Some(start), Some(end)) if end > start => {
            let mut updated = String::with_capacity(existing.len() + region.len());
            updated.push_str(&existing[..start]);
            updated.push_str(&region);
            updated.push_str(&existing[end + REGION_END.len()..]);
            updated
        }
        (Some(_), _) => {
            // Opened and not closed. Report and write nothing.
            let _ = streams.failure(&format!(
                "{} opens the htmlspec region with {REGION_START} and never closes it \
                 with {REGION_END}, so the region's end cannot be located. Nothing was \
                 written to this file: restore the closing marker, or delete the opening \
                 one and let init add the region back",
                path.display()
            ));
            return Ok(None);
        }
        (None, _) => {
            let mut updated = existing.clone();
            if !updated.is_empty() && !updated.ends_with('\n') {
                updated.push('\n');
            }
            if !updated.is_empty() {
                updated.push('\n');
            }
            updated.push_str(&region);
            updated.push('\n');
            updated
        }
    };

    if let Some(parent) = path.parent() {
        create_directory(parent)?;
    }
    write_file(&path, &updated)?;
    Ok(Some(path))
}

fn create_directory(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path)
        .map_err(|error| format!("cannot create {}: {error}", path.display()))
}

fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    std::fs::write(path, contents)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::Form;
    use crate::tools::{ToolId, TOOLS};
    use htmlspec_assets::instructions::PLACEHOLDER;
    use std::collections::BTreeMap;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("htmlspec-install-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("created");
            Self(path)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Runs an installation, discarding the streams.
    fn install_for(root: &Path, id: ToolId) -> Result<Installed, String> {
        let mut streams = Streams::new(Form::Human, Vec::new(), Vec::new());
        install(root, id.tool(), "0.1.0-test", &mut streams)
    }

    /// Task 6.6: every tool in the table receives at least one artifact, asserted against
    /// real files rather than against the table's shape.
    #[test]
    fn every_tool_receives_at_least_one_file() {
        let scratch = Scratch::new("every-tool");
        for tool in TOOLS {
            let installed = install_for(&scratch.0, tool.id).expect("installed");
            assert!(
                !installed.is_empty(),
                "{} received nothing, which the Delivery sum type exists to prevent",
                tool.name
            );
            for path in installed
                .skills
                .iter()
                .chain(&installed.commands)
                .chain(&installed.regions)
            {
                assert!(
                    path.exists(),
                    "{} was reported but not written",
                    path.display()
                );
            }
        }
    }

    /// Task 6.6: a skills-only tool receives no command file.
    #[test]
    fn a_skills_only_tool_receives_no_command_file() {
        let scratch = Scratch::new("skills-only");
        let installed = install_for(&scratch.0, ToolId::Codex).expect("installed");
        assert!(!installed.skills.is_empty());
        assert!(
            installed.commands.is_empty(),
            "a skills-only tool got command files: {:?}",
            installed.commands
        );
    }

    #[test]
    fn a_commands_tool_receives_both() {
        let scratch = Scratch::new("both");
        let installed = install_for(&scratch.0, ToolId::Claude).expect("installed");
        assert_eq!(installed.skills.len(), WORKFLOWS.len());
        assert_eq!(installed.commands.len(), WORKFLOWS.len());
        // Claude Code's commands are namespaced, so the file is the bare workflow name.
        assert!(
            scratch
                .0
                .join(".claude/commands/htmlspec/archive.md")
                .exists(),
            "the command was not written where Claude Code reads it"
        );
        assert!(scratch
            .0
            .join(".claude/skills/htmlspec-archive-change/SKILL.md")
            .exists());
    }

    /// A flat command directory gets the prefix, or the file would collide with another
    /// tool's command of the same name.
    #[test]
    fn a_flat_command_directory_prefixes_the_file_name() {
        let scratch = Scratch::new("flat");
        install_for(&scratch.0, ToolId::Cursor).expect("installed");
        assert!(scratch
            .0
            .join(".cursor/commands/htmlspec-archive.md")
            .exists());
    }

    /// Generated instructions record the version that produced them, so an installation
    /// predating its instructions is identifiable.
    #[test]
    fn generated_files_record_the_generating_version() {
        let scratch = Scratch::new("version");
        let installed = install_for(&scratch.0, ToolId::Claude).expect("installed");
        for path in installed.skills.iter().chain(&installed.commands) {
            let contents = std::fs::read_to_string(path).expect("readable");
            assert!(
                contents.contains("0.1.0-test"),
                "{} does not record its generating version",
                path.display()
            );
        }
    }

    /// Every `name:` this program writes is loadable: lowercase letters, digits, and
    /// hyphens only.
    ///
    /// Asserted against the files an install produces rather than against `WORKFLOWS`,
    /// because the value is assembled by a format string at write time. `workflow.name`
    /// and `workflow.directory` both conform, so a check over the table cannot see a
    /// fault the writer introduces.
    #[test]
    fn every_generated_name_is_loadable() {
        // The strictest rule observed across the tools that load these files: "must be
        // lowercase a-z, 0-9, hyphens only". A name legal under it is legal under a
        // looser one; the reverse is what produces a warning on somebody else's machine.
        fn offending_character(name: &str) -> Option<char> {
            name.chars()
                .find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
        }

        let scratch = Scratch::new("loadable-names");
        let mut faults: Vec<String> = Vec::new();
        let mut checked = 0usize;
        for tool in TOOLS {
            let installed = install_for(&scratch.0, tool.id).expect("installed");
            let mut failing = 0usize;
            for path in installed.skills.iter().chain(&installed.commands) {
                let contents = std::fs::read_to_string(path).expect("readable");
                // A file declaring no name has none to malform. Command files do not, so
                // this check reads the skill files for a tool that receives both.
                let Some(name) = declared_name(&contents) else {
                    continue;
                };
                checked += 1;
                if let Some(bad) = offending_character(name) {
                    failing += 1;
                    faults.push(format!(
                        "{}: name `{name}` carries `{bad}`, which is outside lowercase \
                         a-z, 0-9, and hyphens",
                        path.display()
                    ));
                }
            }
            if failing > 0 {
                faults.push(format!(
                    "  -> {} has {failing} malformed file(s) of {}",
                    tool.name,
                    installed.skills.len() + installed.commands.len()
                ));
            }
        }
        assert!(
            faults.is_empty(),
            "generated names a tool may reject:\n{}",
            faults.join("\n")
        );
        // Guards against the check passing because it read nothing. Command files stopped
        // declaring a name, and an empty sweep would look identical to a clean one.
        assert_eq!(
            checked,
            TOOLS
                .iter()
                .filter(|tool| !matches!(tool.delivery, Delivery::SingleFile { .. }))
                .count()
                * WORKFLOWS.len(),
            "expected one declared name per workflow per tool that receives skills"
        );
    }

    /// The `name:` value from a generated file's frontmatter, if it declares one.
    ///
    /// Optional rather than required: a command file declares no name, because its
    /// invocation is determined by its file path and a name there would duplicate the
    /// skill file's. See `write_commands`.
    fn declared_name(contents: &str) -> Option<&str> {
        contents
            .lines()
            .find_map(|line| line.strip_prefix("name: "))
    }

    /// No two files one tool receives declare the same name.
    ///
    /// A declared name is a registry key for the tool that loads it, so one workflow
    /// declaring its name in both a skill file and a command file presents itself as two.
    /// `every_generated_name_is_loadable` cannot see this: both halves of a duplicate pair
    /// are well-formed, and it checks characters rather than repeats.
    ///
    /// Scoped per tool. Two tools both declaring `htmlspec-apply-change` is correct —
    /// each tool's namespace is its own, and only a repeat *within* one tool collides.
    #[test]
    fn no_tool_receives_a_name_twice() {
        let scratch = Scratch::new("distinct-names");
        let mut faults: Vec<String> = Vec::new();
        for tool in TOOLS {
            let installed = install_for(&scratch.0, tool.id).expect("installed");

            // Every name this tool received, with the file that declared it. A file
            // carrying no name contributes nothing: it cannot collide.
            let mut seen: BTreeMap<String, PathBuf> = BTreeMap::new();
            let mut repeats = 0usize;
            for path in installed.skills.iter().chain(&installed.commands) {
                let contents = std::fs::read_to_string(path).expect("readable");
                let Some(name) = declared_name(&contents) else {
                    continue;
                };
                if let Some(first) = seen.get(name) {
                    repeats += 1;
                    faults.push(format!(
                        "{}: name `{name}` is already declared by {}",
                        path.display(),
                        first.display()
                    ));
                } else {
                    seen.insert(name.to_owned(), path.clone());
                }
            }
            if repeats > 0 {
                faults.push(format!(
                    "  -> {} declares {repeats} name(s) twice across {} file(s)",
                    tool.name,
                    installed.skills.len() + installed.commands.len()
                ));
            }
        }
        assert!(
            faults.is_empty(),
            "one tool would load the same name from two files:\n{}",
            faults.join("\n")
        );
    }

    /// Every invocable file declares an argument and consumes one.
    ///
    /// Asserted against the files an install produces rather than against `WORKFLOWS`, for
    /// the reason `every_generated_name_is_loadable` states: both halves are assembled at
    /// write time, so a check over the table passes while a writer that omitted one ships.
    /// That is not hypothetical here — it is the fault this test was added for. Every
    /// generated file went seven workflows and four tools without either half, and no stage
    /// of the suite noticed, because the loss happens in a tool this program does not
    /// control and only a transcript shows it.
    ///
    /// Both halves, in one test rather than two. A file declaring a hint whose body never
    /// substitutes announces an argument it discards; a body substituting one no
    /// frontmatter declared relies on a tool volunteering it. Neither half is meaningful
    /// alone, so a failure should name the file rather than the half.
    ///
    /// The region-delivered tool is not reached, and by construction: `Installed.regions`
    /// is a separate field from `skills` and `commands`, so iterating the latter two cannot
    /// see it. A marked region in a shared file has no invocation and therefore no argument
    /// to pass — which is a different thing from an omission, and the reason this is not
    /// written as an exception in the loop.
    #[test]
    fn every_invocable_file_declares_and_consumes_an_argument() {
        let scratch = Scratch::new("arguments");
        let mut faults: Vec<String> = Vec::new();
        let mut checked = 0usize;
        for tool in TOOLS {
            let installed = install_for(&scratch.0, tool.id).expect("installed");
            for path in installed.skills.iter().chain(&installed.commands) {
                let contents = std::fs::read_to_string(path).expect("readable");
                checked += 1;
                if !contents
                    .lines()
                    .any(|line| line.starts_with("argument-hint: "))
                {
                    faults.push(format!(
                        "{}: declares no `argument-hint`, so a tool has nothing to \
                         substitute and what a contributor typed is dropped",
                        path.display()
                    ));
                }
                if !contents.contains(PLACEHOLDER) {
                    faults.push(format!(
                        "{}: body never substitutes {PLACEHOLDER}, so an argument a tool \
                         passes reaches no agent",
                        path.display()
                    ));
                }
            }
        }
        assert!(
            faults.is_empty(),
            "generated files that would discard a contributor's input:\n{}",
            faults.join("\n")
        );
        // Guards against passing because it read nothing, as the name check does. An empty
        // sweep and a clean one are otherwise indistinguishable.
        assert_eq!(
            checked,
            TOOLS
                .iter()
                .map(|tool| match tool.delivery {
                    Delivery::SkillsAndCommands { .. } => WORKFLOWS.len() * 2,
                    Delivery::SkillsOnly { .. } => WORKFLOWS.len(),
                    // No invocation, so no argument. Counted as zero rather than skipped,
                    // so adding a region-delivered tool does not silently change the total.
                    Delivery::SingleFile { .. } => 0,
                })
                .sum::<usize>(),
            "expected every invocable file to be checked"
        );
    }

    /// A re-run rewrites the skill rather than duplicating or appending to it.
    #[test]
    fn a_second_run_refreshes_rather_than_grows() {
        let scratch = Scratch::new("refresh");
        install_for(&scratch.0, ToolId::Claude).expect("installed");
        let path = scratch
            .0
            .join(".claude/skills/htmlspec-archive-change/SKILL.md");
        let first = std::fs::read_to_string(&path).expect("readable");

        install_for(&scratch.0, ToolId::Claude).expect("installed again");
        let second = std::fs::read_to_string(&path).expect("readable");
        assert_eq!(first, second, "a re-run changed a program-owned file");
    }

    // -------------------------------------------------------------------------
    // The marked region
    // -------------------------------------------------------------------------

    #[test]
    fn a_region_is_appended_and_existing_content_is_kept() {
        let scratch = Scratch::new("region-append");
        std::fs::write(scratch.0.join("AGENTS.md"), "# My notes\n\nMine.\n").expect("written");

        install_for(&scratch.0, ToolId::Agents).expect("installed");
        let contents = std::fs::read_to_string(scratch.0.join("AGENTS.md")).expect("readable");
        assert!(
            contents.starts_with("# My notes"),
            "the contributor's content moved"
        );
        assert!(
            contents.contains("Mine."),
            "the contributor's content was lost"
        );
        assert!(contents.contains(REGION_START) && contents.contains(REGION_END));
    }

    #[test]
    fn a_region_is_created_when_the_file_does_not_exist() {
        let scratch = Scratch::new("region-new");
        install_for(&scratch.0, ToolId::Agents).expect("installed");
        let contents = std::fs::read_to_string(scratch.0.join("AGENTS.md")).expect("readable");
        assert!(contents.contains(REGION_START));
    }

    /// The region carries every workflow, in the table's order.
    ///
    /// A shared-file tool receives the whole instruction set as one document, so the order
    /// is what a reader meets rather than an implementation detail. `the_table_is_in_lifecycle_order`
    /// in the assets crate pins the table; this pins that the region follows it. Those are
    /// different claims: a `write_region` that sorted or reversed its iteration would keep
    /// the table's test passing and still hand a contributor the lifecycle backwards.
    #[test]
    fn the_region_carries_every_workflow_in_lifecycle_order() {
        let scratch = Scratch::new("region-order");
        install_for(&scratch.0, ToolId::Agents).expect("installed");
        let contents = std::fs::read_to_string(scratch.0.join("AGENTS.md")).expect("readable");

        let positions: Vec<(usize, &str)> = WORKFLOWS
            .iter()
            .map(|workflow| {
                let heading = format!("## htmlspec: {}\n", workflow.name);
                let at = contents.find(&heading).unwrap_or_else(|| {
                    panic!(
                        "the region omits {}, so a contributor at that stage finds nothing:\n{contents}",
                        workflow.name
                    )
                });
                (at, workflow.name)
            })
            .collect();

        let mut sorted = positions.clone();
        sorted.sort_unstable();
        assert_eq!(
            sorted, positions,
            "the region's sections are not in the table's order"
        );
    }

    /// A repeated installation replaces the region rather than adding a second one, so
    /// the file does not grow without bound.
    #[test]
    fn a_repeated_region_write_does_not_grow_the_file() {
        let scratch = Scratch::new("region-repeat");
        std::fs::write(scratch.0.join("AGENTS.md"), "# Mine\n").expect("written");

        install_for(&scratch.0, ToolId::Agents).expect("installed");
        let first = std::fs::read_to_string(scratch.0.join("AGENTS.md")).expect("readable");
        install_for(&scratch.0, ToolId::Agents).expect("installed again");
        let second = std::fs::read_to_string(scratch.0.join("AGENTS.md")).expect("readable");

        assert_eq!(
            first, second,
            "the region was added twice rather than replaced"
        );
        assert_eq!(second.matches(REGION_START).count(), 1);
        assert!(second.starts_with("# Mine"));
    }

    /// The case that would otherwise delete a contributor's prose.
    #[test]
    fn an_unclosed_region_is_reported_and_nothing_is_written() {
        let scratch = Scratch::new("region-unclosed");
        let path = scratch.0.join("AGENTS.md");
        let original = format!("# Mine\n\n{REGION_START}\nsomething\n\nMy own prose below.\n");
        std::fs::write(&path, &original).expect("written");

        let mut streams = Streams::new(Form::Human, Vec::new(), Vec::new());
        let installed = install(
            &scratch.0,
            ToolId::Agents.tool(),
            "0.1.0-test",
            &mut streams,
        )
        .expect("the call itself succeeds");

        assert!(
            installed.regions.is_empty(),
            "a file was reported as written"
        );
        assert_eq!(
            std::fs::read_to_string(&path).expect("readable"),
            original,
            "the file was modified despite the unlocatable region end"
        );
        let (_, err) = streams.into_parts();
        let err = String::from_utf8(err).expect("UTF-8");
        assert!(err.contains("never closes it"), "{err}");
        assert!(err.contains("Nothing was written"), "{err}");
    }
}

# What an agent now reads, installed

Section 5. The amended instruction bodies are compiled into the binary, so the check is
what `htmlspec init` writes rather than what the source says. Installed into a scratch
project from this tree.

```
htmlspec init --tools claude
```

## The need, in two forms

`.claude/skills/htmlspec-new-change/SKILL.md`:

```
**State the need concretely, and a need takes one of two forms.** Neither outranks the
other:

- **Something existing misbehaves.** State it as a measurement, an error message, or a
  behavior somebody hit.
- **The project cannot do something yet.** State who or what wants the capability, what
  they cannot do without it, and what working looks like the first time you see it.

The second form is not a lesser case. A project's first change has nothing to measure and
no error to quote, and neither does any change that adds a capability rather than repairing
one. If you find yourself asking a contributor for a problem when they have described
something new, the answer is that there is no problem and the description was already
enough.
```

The old demand is gone: `grep -c "State the problem concretely"` returns **0**. Its three
examples survive as the first form, which is correct for the case they were written for.

The last sentence is aimed at the recorded failure specifically. The agent in that
transcript reasoned its way to "an absence of capability rather than a broken system" and
asked for a problem anyway, so the guidance now names that exact move and answers it.

## The name

```
**Where a contributor describes the work and names nothing, derive the name yourself.** The
rule above is what a good name is, not who has to supply it — it applies identically to a
name you derive. "A browser application that adds named tally counters, incrementing on
click, with no backend" yields `add-tally-counters` in one step.

State the derived name in your report so a contributor who disagrees can say so. Do not stop
and ask them to approve it first: a change directory is renamed by moving it, so the name is
the cheapest thing in the change to correct, and a round trip to confirm one that follows
from the description already given is a refusal in a politer form.
```

The worked example is the request from the transcript, so an agent meeting the same input
has the derivation in front of it rather than a rule to apply.

## Asking is still right when nothing was described

```
**Ask only for what the description does not carry.** Asking again for a scope or an
exclusion the contributor already stated presents their description as incomplete, and
leaves them guessing which part fell short. Where a request carries no description of the
work at all, asking is right — a need nobody stated must not be invented.
```

Task 5.4 required this to survive, and it did. The repair removes a refusal that was wrong
and keeps the one that was right.

## What was NOT verified

**No agent was run against the recorded request.** The guidance is installed and reads
correctly, and whether an agent now creates the change on the first exchange is a
behavioral property observable only by running one. This is the same limit the previous
change recorded about a session load: reading a file does not establish what a reader does
with it.

That limit is sharper here than usual. The fault under repair was not an agent failing to
know something — it was an agent knowing and refusing anyway. So the text reading correctly
is weaker evidence than it would be for an ordinary documentation fix, and the real check is
the next `/htmlspec-new` invocation against a described change.

`explore.md` was not touched, and `ff.md:41` needed no repair — it already said "the value,
the need, the scope", which is the contract's own wording.

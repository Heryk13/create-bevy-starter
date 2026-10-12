# CLAUDE.md

@CONTRIBUTING.md

## The plan

The CLI is designed in `bevy-starter`, in `../bevy-starter/docs/specs/create-bevy-starter/design.md`,
worked through one step per commit, in its order. What the build changes from the plan goes in
that spec's "What changed while building it".

## Every implementation

Whenever a change is made, by the user or by you, before calling it done:

1. Read the diff and check it against every rule in `CONTRIBUTING.md`:
   the loop, file layout, separators, comments, patterns, naming, tests and docs.
2. Run `just check` yourself. Never report it as passing without running it.
   Skip it for a docs-only change: there is nothing in it for `just check` to check.
3. Report a short checklist: what follows the conventions, what is missing,
   and a suggested commit message.

At the end of every step of the plan, before the commit, run that review unasked, with an
explicit pass over **every name** (constants, types, functions, parameters, tests) and over
the **order** of sections and functions. A rename leaves variables whose names now lie, and
the compiler never complains about those.

Say plainly what was and was not verified: `just check` covers compiling, lints and tests, and
it says nothing about what the CLI looks like in a terminal or what it leaves on disk. Ask the
user to run it for that.

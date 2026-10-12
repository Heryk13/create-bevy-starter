# Contributing

How every change in this project is made. These are `bevy-starter`'s rules, less the ones only
a Bevy game needs; the CLI's design lives in `bevy-starter`, in
`docs/specs/create-bevy-starter/design.md`.

Every rule below exists for a reason. When the rule and its reason disagree, the reason
wins: do what makes the code better, and write the exception down in a comment where it
lives, saying why the usual rule does not fit there. An exception with its reason beside
it is a decision. An exception without one is drift.

## The loop

1. **Plan**: find where the change belongs, by what it does: the questions in `prompts.rs`,
   the decisions in `choices.rs`, the disk in `scaffold.rs`. A change bigger than one commit
   is designed first, in a spec with an implementation order of one commit per step.
2. **Write** following the [code conventions](#code-conventions).
3. **Test** the logic (see [Tests](#tests)).
4. **Fix and check**: `just fix`, then `just check`.
5. **Run**: `just run <name> <folder>` into a scratch folder, and look at what it made.
6. **Docs**: update `README.md` if what the CLI asks, does or needs changed.
7. **Commit** (see [Commits](#commits)).

## Code conventions

### File layout

Every `.rs` file follows the same order, skipping the sections it doesn't have:

1. imports: external crates first, a blank line, then crate-local ones (`use crate::...`)
2. child modules (`mod ...;`)
3. `Config`: tunable constants
4. types
5. the functions the file is for, in the order they run
6. `Helpers`: pure functions, next to the tests that cover them
7. `Tests`

### Splitting files

- Split by what the code does, never by kind of code: `prompts.rs` and `scaffold.rs`, not
  `types.rs` and `functions.rs`. Names like `common`, `utils` or `global` describe status
  instead of content: everything fits in them and nothing ever leaves.
- Keep a file whole while it reads as one thing. Split when a part makes sense on its own,
  usually around 300 to 400 lines.
- A split module is `prompts.rs` next to a `prompts/` folder with the parts, never `mod.rs`;
  clippy's `mod_module_files` fails `just check` on one.
- Unit tests stay in the file they test, `tests/` is only for integration tests.

### Separators

Every `.rs` file splits its sections with separator lines ending at column 80.
The character tells the kind of boundary:

```rust
// ==== Helpers ================================================================
// ---- file edits: text in, text out -----------------------------------------
```

- `====`: a section of the file. Title Case.
- `----`: a group inside a section, lowercase, may carry a short note.
  Inside a function body it is indented with the code and still ends at column 80.

### Comments

- Comments explain *why*, not *what*. Code that already says what it does gets no comment.
- Every function and type has a `///` doc comment written as a sentence:
  capitalized, ending with a period (`/// Returns the branch a game is cloned from.`).
- Plain `//` comments are lowercase, without a final period.

### Patterns

- Nothing touches the disk before the user confirms, and only `scaffold.rs` touches it.
- Every message the user reads is in English, and an error says what to do next.
- Prefer the call that can say no over the one that panics: `get()` over `[0]`, `?` over
  `unwrap()`. A wrong answer or a failed clone ends in a message, never a panic.
- A value that can only be one of a few is an enum, not a string or a number, and a choice
  the CLI cannot build has no variant.
- Ask for the minimum: `&` to read, ownership only to keep.
- Tunable values are named constants in the `Config` section, never magic numbers.

### Naming

- Functions say what they give back or do (`branch_for`, `title_from_name`, `clone_template`).
- Constants: `SCREAMING_SNAKE_CASE`, with the unit in the name when there is one.

## Tests

- Logic lives in pure functions: plain values in, plain values out. The code that asks or
  writes stays thin: read, call the function, use the result.
- Tests sit at the bottom of the same file, in `#[cfg(test)] mod tests` under `// ==== Tests ====`.
- One case per test, so a failure names the case and the others still run.
- Every test asserts something, and has been seen failing: break the code it covers once, and
  watch it go red.
- The name says what the test guarantees, as a sentence about the subject
  (`doubled_separators_leave_no_empty_word`), not what the function under test is called.
- A test that touches the disk works in a folder of its own under `std::env::temp_dir()`,
  emptied first, so tests running in parallel never share files.

## Commits

- [Conventional Commits](https://www.conventionalcommits.org/), in English, imperative, short title:
  `feat: ask for the cameras`, `fix: ...`, `refactor: ...`, `test: ...`, `docs: ...`, `build: ...`.
- One change per commit.
- `git commit --amend` only before pushing.

## Dependencies

- New crates are fine when they save real work, but check first if the standard library
  already does it. Every crate is one more thing to keep up to date.

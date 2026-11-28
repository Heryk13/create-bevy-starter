# `create-bevy-starter` 0.5.0: leaving all of an option out

Status: **approved** 2026-11-28, building
Date: 2026-11-28
CLI: 0.4.3, to 0.5.0
Template: 1.1.1 (`3d/player` at `37b2f77`)
Follows: `bevy-starter`'s `docs/specs/create-bevy-starter-0-4/design.md`, whose "Out of scope"
(the docs a game keeps still describe the options it left out) and "The locales" this spec
takes back.

## Goal

A game has nothing of an option it was not given: no code only that option reads, no comment
or doc that names it, no line that is there only because of it. A 3D game with the first-person
camera alone, no split screen and no multiplayer has no `GuestPad`, no `CameraRays`, no
`Without<GuestPad>` filter, and no sentence in `docs/flow.md` about F5, and it still builds.
And the release check fails the day a template change brings such a leftover back.

Today the CLI can only delete a whole line ending with an option's marker (`remove_marked_lines`,
`src/choices.rs`), in `Cargo.toml` and the Rust sources. That misses three things the first
real game made with it (`biscuit-game`, first person alone) had to fix by hand:

1. a run of lines, such as `GuestPad` with its doc and its section separator, where a marker on
   every line is noise and a `///` line cannot carry one (the marker becomes part of the doc);
2. a line that has to **change**, not go: without the split screen, `gamepads` loses its
   `Without<GuestPad>` filter, and deleting it breaks `Controls::on_any_pad` and
   `Controls::stick`;
3. the Markdown docs and the locales, which the CLI never reads.

## Decided

| Question | Decision |
| --- | --- |
| Where the fix lives | In the CLI, with new kinds of marker, not by bending the template's code so that deleting lines is enough (decided before this spec) |
| Kinds of marker | Three: the **line** (exists), the **block** (`<marker>: begin` … `<marker>: end`) and the **without line** (`without <marker>: <line>`), a commented-out line that becomes code when the option is left out. `without` over `unless`: it reads as "in a game without the split screen, this line". Syntax below |
| Reword before marking | A sentence that names an option only as an example is rewritten once, in the template, so it is true in every game, with no marker. Markers are for what the option is or does: its code, what only it reads, the docs that describe it. A marker costs every reader of the template a line of noise, and a rewording costs nothing after it is written |
| Where markers are read | Every `.rs`, `.toml`, `.md` and `.ftl` file of the game, `docs/specs/` aside, which is emptied before |
| Comment by kind of file | `.rs`: `//`. `.toml` and `.ftl`: `#`. `.md`: `<!-- … -->`. One style per file, chosen by its extension: today both `//` and `#` are matched everywhere, and in Markdown `# multiplayer` is a heading |
| Matched whole | A marker is matched whole, in every kind. For without lines that needs a rule on the list of options: **no marker followed by `: ` starts another marker**, or `// without camera: switch: …` could be read as the line `switch: …` of a marker `camera`. A test over the list of every option holds it |
| Blocks inside blocks | Allowed, of any markers: removing the outer one takes the inner one with it. Every file's fences are checked together, all options at once, as brackets: a `begin` with no `end`, an `end` with no `begin`, and blocks that cross (A begin, B begin, A end, B end) are `TemplateChanged`. Guessing where a block ends could eat the rest of a file, and a crossing has no right answer: removing A would cut B in half |
| A without line in Markdown holding `-->` | `TemplateChanged`: the comment would end there as GitHub renders the template, and the rest of the line would show |
| Markdown tables | A line marker after a row's last pipe renders nothing on GitHub (checked with its Markdown API, as a repository file renders): a cell past the header's count is dropped, so no column appears. Allowed, after the last pipe, where the line ends with it |
| A kept option, in code (`.rs`, `.toml`, `.ftl`) | Its line markers and block fences stay, as line markers stay today: they show the game what belongs to it, and are where swapping it for something else starts. Its without lines are deleted: commented-out code the game will never use |
| A kept option, in Markdown | Its markers go too: a line marker is cut off the end of its line, which stays, and a block's fences are deleted, with what is between them kept. In the docs a marker serves nothing (the docs are read, never edited by marker), and inside a code block it shows as text in the game's own docs |
| Options a kind does not offer | Left out too, by their markers. `GuestPad` lives in `main`'s `input.rs` (the split screen is `3d/player`'s, but `Controls` is `main`'s), so every game but a 3D player with the split screen has to lose it. Their files are deleted if they are there and never checked: only a kind that offers an option reports `TemplateChanged` when its files are missing |
| Old CLIs | 0.4.x clones the same branches and knows only line markers. The template stays buildable by them: see "Staying buildable by 0.4" |
| The locales | Now cleaned: Fluent has no comment at the end of a line, but it has comment lines, so blocks work there. 0.4 kept the keys because it had no way to mark them |
| After removing | Unchanged: `cargo fmt` (a without line comes out at the comment's indentation, and a call can now fit one line) and `cargo metadata` |
| The search in `just check-combos` | Each `Removal` carries the words that name its option (below). After making each game, `just check-combos` searches the files markers are read in, `docs/roadmap/` aside, for the words of every option the game left out, offered or not, and names the game, the file and the word. It only reports from step 6, and fails from step 20 on: failing sooner would block the release the template's marking waits for. It is what keeps the next template feature from bringing the problem back. The words are read only by that test (see "What changed while building it", 5) |
| `docs/roadmap/` | Kept, as now: a game may want the template's history, and `biscuit-game` kept it. If it ever goes, as a choice of the CLI, off by default. The search skips it |
| `choices.rs` | Already 466 lines; the text edits (`replace_line`, `remove_section`, the markers) move to `src/edits.rs`, their own thing: text in, text out. `CONTRIBUTING.md`'s "The loop" names the new file |

## The markers

Each option keeps the marker it has: `multiplayer`, `camera: first-person`,
`camera: third-person`, `camera: switch`, `split screen`. Every line is matched trimmed, so
indentation does not matter.

### Rust (`//`), and TOML the same with `#`

```rust
mod split_screen; // split screen

// split screen: begin
// ==== Components =============================================================

/// Marks a gamepad another player on this PC plays with, as in a split screen: the actions and
/// sticks `Controls` reads, the first player's, do not hear it. Whatever gave it out reads it.
#[derive(Component)]
pub(crate) struct GuestPad;
// split screen: end

    // split screen: begin
    // the first player's: a gamepad another local player took is that player's alone
    gamepads: Query<'w, 's, &'static Gamepad, Without<GuestPad>>,
    // split screen: end
    // without split screen: gamepads: Query<'w, 's, &'static Gamepad>,
```

| Marker | Option left out | Option kept |
| --- | --- | --- |
| line: ends with `// <marker>` | the line goes | stays |
| block: a line `// <marker>: begin`, a line `// <marker>: end` | both, and everything between, go | stays |
| without line: starts with `// without <marker>: ` | becomes what follows the `: `, at the comment's indentation | goes |

Two rules come from `rustfmt`, checked on this spec's examples with `rustfmt --edition 2024`:

- a comment after a `{` is moved to the next line, so a line that opens a block (`fn … {`,
  `match … {`, `pub(crate) enum … {`) takes a block, never a line marker;
- block fences and without lines, being comment lines, are left as they are, also between `///`
  lines and inside a `match`. A without line past column 100 is left alone too.

A line marker on a `///` line becomes part of the doc, so doc comments take blocks: the
sentence about the option starts on a line of its own, and the block holds its lines.

### Markdown (`<!-- … -->`)

```markdown
physics_debug.rs  F6: every collider's outline, then the third-person camera's rays <!-- camera: third-person -->
<!-- without camera: third-person: physics_debug.rs  F6: every collider's outline, on and off -->

<!-- multiplayer: begin -->
- Multiplayer over a LAN or Radmin VPN, up to 10 players: …
<!-- multiplayer: end -->

| `multiplayer.rs` | playing over the network | <!-- multiplayer -->
```

The same three kinds; a left-out option as in the table above, a kept one as in "Decided". As
GitHub renders the template (checked with its Markdown API):

- a line marker at the end of a line, in a paragraph or after a table row's last pipe, shows
  nothing;
- a block fence or a without line on its own line starts an HTML block, which ends the
  paragraph it sits in. So blocks go between paragraphs and list items, a paragraph is written
  so that deleting its marked lines is enough (one clause per line where an option starts or
  ends), and without lines are kept for code blocks and lists;
- inside a code block (the tree in `docs/template.md`, the diagrams in `docs/flow.md`) every
  marker shows as text. Accepted in the template, where it says which option a line is; a
  game loses it, kept or not.

### Fluent (`#`)

Blocks only, on whole lines: a `#` after a value is part of the value. The three files get the
same blocks, since a test in `src/i18n.rs` checks that every key is in all of them.

```
# multiplayer: begin
notice-could-not-join = Could not join that game.
# multiplayer: end
```

### The words of each option

What the search in `just check-combos` looks for, case-sensitive, anywhere in a line. Each word
names the option and nothing else in the template; words that do not (`shoulder`: the
capsule is "half a shoulder width"; `first-person`: `look.rs` cites Bevy's first-person example;
`F5`) are left out.

| Option | Words |
| --- | --- |
| `multiplayer` | `multiplayer`, `Multiplayer`, `lightyear`, `others.rs`, `find-server`, `Find Server`, `is_shared_game`, `LockedProps` |
| `camera: first-person` | `first_person`, `FirstPerson`, `first person` |
| `camera: third-person` | `third_person`, `ThirdPerson`, `third person`, `third-person`, `CameraRays`, `CAMERA_RAYS`, `swap-shoulder` |
| `camera: switch` | `camera_switch`, `CameraView`, `CAMERA_VIEWS`, `switch-camera` |
| `split screen` | `split_screen`, `split screen`, `GuestPad` |

Run on the template as 0.4 leaves it (each option removed alone, `docs/specs/` and
`docs/roadmap/` aside), these words find nothing outside the sweep below, but
`docs/template.md:11` (now a rewording) and lines of the `## Branches` sections the CLI already
removes. They do not find every row of it: the sentences about the host and a joined game carry
none of them. The search catches a forgotten file or type, not every sentence, so a template
step still reads its option's docs.

## Staying buildable by 0.4

0.4.x clones the branch tip like 0.5 does, so after this lands an old CLI makes games from a
template that has blocks and without lines it cannot read. It deletes line-marked lines only,
leaving fences and without lines as plain comments. Its games keep building if:

- **lines that name an option's files keep their line markers**, as today (`mod split_screen;`,
  the plugins, the `use` lines): 0.4 deletes those files, so it must delete those lines;
- **every new marker on code is a block**, never a line marker. 0.4 leaves a block's code where
  it is, which is the template's own code minus the option's files: what 0.4 already makes,
  which builds. A line marker on, say, `Colliders => CameraRays` would be deleted by 0.4 while
  the without line that replaces it stays a comment, and the `match` would no longer compile;
- **a line a without line replaces is inside a block**, for the same reason.

New line markers on comments and in Markdown are safe: 0.4 never reads Markdown, and a comment
gone builds. The last step runs 0.4.3's `just check-combos` against the new template to prove it.

## What the sweep found

The template's five branches, each option simulated on its own the way 0.4 removes it (files
and line-marked lines gone), then searched for what still names it. Lines are those of template
1.1.1. A **place** is one spot the template marks: a block, a run of marked lines, a without
group. **Reword** is a sentence rewritten once, with no marker.

### How many markers the rewording saves

| Option | Places before | Reworded | Places now |
| --- | --- | --- | --- |
| split screen | 7 | 4 | 3 |
| multiplayer | 27 | 0 | 27 |
| third-person camera | 15 | 0 | 15 |
| camera switch | 5 | 0 | 5 |
| first-person camera | 2 | 0 | 2 |
| every camera | 1 | 0 | 1 |
| **all** | **57** | **4** | **53** |

Five rewordings in all: the four of the split screen, plus `docs/template.md:11`, which the
word search found and the first draft had kept. The first person's three rewordings were
already in the first draft. Considered and kept marked, because the option is the point of the
sentence rather than an example of it: `edits.rs:44-46` and `map.rs:60-61` (multiplayer is the
reason `MapEdit` and `PropId` are built as they are), `first_person.rs:25-26` and
`third_person.rs:86-87` (what the switch does to the camera), and `split_screen.rs:30` (that
guests are not sent to a shared game).

### The split screen (every kind but a 3D player with it)

| Branch | Where | What | Fix |
| --- | --- | --- | --- |
| `main` | `src/input.rs:76-77` | the comment and `gamepads … Without<GuestPad>` | block, and a without line with no filter |
| `main` | `src/input.rs:156-161` | `// ==== Components`, the doc and `GuestPad` | block |
| `3d/base` | `src/screens/gameplay/playroom.rs:111-112` | `PlayroomView`'s doc: "as a split screen's other players' (`player/split_screen.rs` on `3d/player`)" | reword: what a view alone is for, with no example or one every game has |
| `3d/base` | `src/screens/gameplay/playroom.rs:156-157` | `PlayroomCamera`'s doc: "the one camera, or in a split screen the first player's" | reword: the player's camera, the one the game follows |
| `3d/player` | `src/screens/gameplay/player.rs:203` | `capsule_body`'s doc: "The player's, and a split screen's other players'." | reword: the sentence goes, the first one says it |
| `3d/player` | `src/screens/gameplay/player/look.rs:144-145` | `turn_view`'s doc: "the player's and a split screen's other players'" | reword: "The look of every body that looks around." |
| `3d/player` | `docs/template.md:155` | the tree's `split_screen.rs` | line |

### Multiplayer (every kind)

| Branch | Where | What | Fix |
| --- | --- | --- | --- |
| `main` | `src/dev_tools/prop_editor.rs:19-20` | "which multiplayer sends to the host" and `pub(crate) mod files;` | block, and `without multiplayer: mod files;` |
| `main` | `src/dev_tools/prop_editor/edits.rs:44-46` | the plugin's doc: "so multiplayer can later send the same edit to the host first" (and "later" is stale) | block |
| `main` | `edits.rs:91-93`, `:114-116`, `:166-168` | the docs of `MapEdit`, `MapChange` and `apply_map_change`: the host, the games that joined | blocks |
| `main` | `edits.rs:68`, `:150-154`, `:253`, `:281-284` | `LockedProps`, always empty without multiplayer: its `init_resource`, the resource, the `locked` parameter, the comment and the `&& !locked…` of the pick-up | blocks |
| `main` | `src/dev_tools/prop_editor/files.rs:18-21`, `:41-42`, `:125-126`, `:222-223` | docs and comments: the host, a joined game | blocks |
| `main` | `src/screens/gameplay/map.rs:60-61` | `PropId`'s doc: "so multiplayer can send it" | block and without lines |
| `main` | `assets/locales/*/main.ftl` | `menu-find-server`, `pause-open-to-lan`, `pause-close-lan`, `dialog-ok`, the three `notice-…` of joining and hosting, and the `## find server` group | 5 blocks, in each of the three files |
| `main` | `README.md:27-31` | the Multiplayer item of Features | block |
| `main` | `README.md:50` | "all of it shared in a multiplayer game", inside the dev tools item | rewrap, line |
| `main` | `docs/flow.md:54` | `[Find Server]` in the menu's diagram | line |
| `main` | `docs/flow.md:65-94` | the Multiplayer section | block |
| `main` | `docs/template.md:11` | "the cameras, split screen and multiplayer not chosen" | reword: "the options not chosen" |
| `main` | `docs/template.md:99-107` | the tree's `multiplayer.rs` and `multiplayer/` | block |
| `2d/player`, `3d/player` | `docs/template.md:135` and `:154` | the tree's `others.rs` | line |
| `3d/player` | `src/screens/gameplay/player/split_screen.rs:30` | "nor sent to a shared game": a game with the split screen and without multiplayer | block |

Stays: `Serialize` and `Deserialize` on `MapEdit`, `MapChange`, `PropFileEdit` and
`PropFileChange`, and the `transform_parts` they use. An edit that can be written down is sound
without the network, nothing in it says "multiplayer" once the docs above go, and taking it out
would mean blocks on four derives and a module for no line a reader trips over.

### The third-person camera (`3d/player`)

| Where | What | Fix |
| --- | --- | --- |
| `src/dev_tools.rs:15-16` | "read by the third-person camera…" and `pub(crate) mod physics_debug;` | block, and `without camera: third-person: mod physics_debug;` |
| `src/dev_tools/physics_debug.rs:19` | `CAMERA_RAYS_WORDS` | block |
| `physics_debug.rs:22-27` | the plugin's doc: two views, the rays | block and without lines |
| `physics_debug.rs:63-64` | the enum's doc ("A camera reads it…") and its `pub(crate)` | block, and without lines with no `pub(crate)` |
| `physics_debug.rs:69` | `CameraRays` | block |
| `physics_debug.rs:78` | `step_physics_debug`'s doc: "the camera's rays" | block and without line |
| `physics_debug.rs:106` | the `CameraRays` arm of `show_physics_debug_status` | block |
| `physics_debug.rs:143-144` | `next_view`: `Colliders => CameraRays`, `CameraRays => Off` | block, and `without …: PhysicsDebugView::Colliders => PhysicsDebugView::Off,` |
| `physics_debug.rs:155` | the test's name, `f6_shows_the_colliders_then_the_rays_then_nothing` | block, and a without line naming it `f6_shows_the_colliders_then_nothing` |
| `physics_debug.rs:160-161` | the test's step to `CameraRays` and its assert | block |
| `assets/locales/*/main.ftl` | `action-swap-shoulder` | block, in each of the three files |
| `docs/flow.md:361-363` | `ThirdPersonView`, the shoulder, V, the three rays | rewrap, lines |
| `docs/flow.md:401-403` | F6's second view, the rays | rewrap, lines |
| `docs/template.md:97` | `physics_debug.rs`: "then the third-person camera's rays" | line, and a without line |
| `docs/template.md:156` | the tree's `third_person.rs` | line |

### The camera switch (`3d/player`, fewer than two cameras)

| Where | What | Fix |
| --- | --- | --- |
| `src/screens/gameplay/player/first_person.rs:25-26` | `view_in_first_person`'s doc: "the switch (`camera_switch.rs`) then keeps…" | block |
| `src/screens/gameplay/player/third_person.rs:86-87` | the same sentence in `view_in_third_person`'s doc | block |
| `assets/locales/*/main.ftl` | `action-switch-camera` | block, in each of the three files |
| `docs/flow.md:363-365` | `camera_switch.rs`, `GameData::camera`, F5, `CAMERA_VIEWS` | rewrap, lines |
| `docs/template.md:151` | the tree's `camera_switch.rs` | line |

### The first-person camera (`3d/player`)

| Where | What | Fix |
| --- | --- | --- |
| `docs/flow.md:352` | "the playroom, walked in first person" | reword: wrong with the third person too |
| `docs/flow.md:360-361` | `FirstPersonView` at the eyes | rewrap, line |
| `docs/flow.md:378-379` | "rather than in the first camera's file, which a game may delete" | reword: "a camera's file" |
| `docs/template.md:152` | the tree's `first_person.rs` | line |
| `src/dev_tools/free_camera.rs:23` | `// ---- turning: as first person turns` | reword: the turning is `look.rs`'s |

### Every camera (`3d/player`)

`docs/flow.md:375-379`, the paragraph that lists the three camera markers: one marker per line,
each with its own line marker, so a game lists only the markers it still has.

## Out of scope

- **The branch tree in a game's docs**: `docs/flow.md` and `docs/template.md` say "on
  `3d/player`", "on `2d/player`" in every kind of game, and the code's "read by each genre" too.
  The same problem for kinds instead of options, and the 0.4 spec's "Out of scope" already names
  it. Its own spec, if wanted.
- **Emptying `docs/roadmap/`**: kept, see "Decided".
- **Taking a kept option's markers out of its code**: they stay, see "Decided".

## Implementation order

One commit per step. The template may use a new marker only once a published CLI reads it, so
the CLI goes first, all the way to crates.io.

### In `create-bevy-starter`

1. `refactor: move the text edits into their own file`: `replace_line`, `remove_section`,
   `remove_marked_lines` and their tests from `choices.rs` to `src/edits.rs`; `CONTRIBUTING.md`
   names it in "The loop". A cut-and-paste move, Claude's to do.
2. `feat: remove marked blocks`: the block in Rust and TOML. The list of every option comes in
   here, since a file's fences are checked for all of them at once. Tests: a block removed, a
   block inside another, and each broken case (no `end`, no `begin`, two blocks crossing) as
   `TemplateChanged`, one test each.
3. `feat: leave out what a kind does not offer`: `removals_for` gives every option not chosen,
   and says which ones the kind offers, so only those check their files. Tests: a 2D game
   leaves out the split screen; an Empty game does not ask for `split_screen.rs`.
4. `feat: turn without lines into code`: put back for an option left out, deleted for one kept,
   which needs the kept options too, not only the removals. Tests: one per row of the table, and
   no marker followed by `: ` starting another, over the list of every option.
5. `feat: read markers in Markdown and Fluent`: the comment style from the extension, `.md` and
   `.ftl` added to the files read; a kept option's markers taken out of the Markdown; a without
   line holding `-->` as `TemplateChanged`. Tests: each kind in each style, a kept option's line
   marker cut and its fences gone in Markdown, `-->` refused, and `# multiplayer` as a Markdown
   heading staying.
6. `test: search every game for the options it left out`: `words` on each `Removal`, the pure
   search with a test of its own (a word found, a word in `docs/roadmap/` skipped), and
   `just check-combos` **reporting** what it finds: each game, file and word, without changing
   whether it passes. Failing here would make a cycle: step 8 runs `just check-combos` before
   publishing, and the template is marked only in steps 9 to 19, which need 0.5.0 published.
   Until then the report is the list of what is left to mark.
7. `docs: describe the markers`: `README.md`'s steps, and this spec's "What changed while
   building it".
8. `build: release 0.5.0`: `cargo publish`, the user's to run.

Steps 2 to 5 pass `just check`, and `just run` of a 3D game into a scratch folder against the
current template, which has no new marker yet: the game must come out as 0.4.3 makes it, its
Markdown aside.

### In `bevy-starter`, after 0.5.0 is on crates.io

Each step on the branch named, merged down as `CONTRIBUTING.md`'s Branches says, `just check` on
every branch it reaches, then pushed and checked with `create-bevy-starter` 0.5.0.

9. `main`, `docs: the block and without markers`: `CONTRIBUTING.md`'s Branches section and
   `docs/specs/create-bevy-starter/design.md`'s "Camera markers" get the three kinds, the
   comment styles, "Reword before marking", the words of each option, and "Staying buildable
   by 0.4".
10. `main`, the split screen: `input.rs`.
11. `main`, multiplayer in the code and the locales.
12. `main`, multiplayer in `README.md`, `docs/flow.md` and `docs/template.md`.
13. `3d/base`, the split screen: the two rewordings in `playroom.rs`.
14. `2d/player`, multiplayer: `docs/template.md`'s `others.rs`.
15. `3d/player`, the third-person camera.
16. `3d/player`, the camera switch.
17. `3d/player`, the split screen: the two rewordings and the tree line.
18. `3d/player`, the first-person camera, its rewordings, and the markers paragraph.
19. `3d/player`, multiplayer: `others.rs` in `docs/template.md`, `split_screen.rs:30`.

### Back in `create-bevy-starter`

20. `test: fail check-combos on a left-out option's words`: the report of step 6 becomes a
    failure, once a run of `just check-combos` against the marked template reports nothing. No
    release: `just check-combos` is an ignored test, run only by whoever works on the CLI, so
    nothing `cargo install` builds changes.

### Checking it

21. In `create-bevy-starter`, no commit unless something fails:
    - `just check-combos`: the 20 games, each one's `just check`, and the word search, now
      failing on anything it finds;
    - 0.4.3's `just check-combos` (a worktree at `8750385`) against the same template: every
      game still builds, untidy as 0.4 leaves it;
    - a 3D game with the first-person camera alone, no split screen, no multiplayer: its
      `just check`, then a search of its `.rs` and `.md` files, `docs/roadmap/` aside, for
      `GuestPad`, `CameraRays`, `third_person`, `split_screen`, `camera_switch` and
      `CAMERA_VIEWS`, which finds nothing.

`just check` covers compiling, lints and tests. What a game's docs read like, and that a kept
option's docs are whole, is for the user to look at.

## What changed while building it

Steps 1 to 7 built 2026-11-28, each its own commit.

1. **What becomes of an option is an enum** (step 3): `Fate { Kept, LeftOut, NotOffered }`,
   from `fate_of(choices, option)`, and `build_game` goes through `OPTIONS` asking it.
   `removals_for` stays as it was, what a kind offers and was not given, and `fate_of` reads
   it: its tests needed no change.
2. **The check covers without lines too** (step 5): `are_blocks_whole` became
   `are_markers_whole`, and `check_blocks` `check_markers`. A Markdown without line never
   closed is refused as well as one holding `-->`: its comment would hide the rest of the file
   as GitHub renders the template.
3. **The markers got a file of their own**, `src/markers.rs`, in a commit after step 5:
   `edits.rs` had reached 622 lines, and the markers read as one thing. `edits.rs` keeps
   `replace_line` and `remove_section`; `CONTRIBUTING.md`'s "The loop" names both.
4. **Markers are read in the whole game**, every `.rs`, `.toml`, `.md` and `.ftl` at any depth,
   not only `Cargo.toml` and `src/`. So the template's docs may **quote a marker only inside a
   line** (in backticks, mid-sentence), never as a line of its own or at a line's end, outside
   the sections the CLI removes first (`## Branches`): such a line would be read as a marker.
   Step 9 writes the rules into `CONTRIBUTING.md`'s Branches section, which is removed before
   the markers are read.
5. **`words` needs no `expect(dead_code)`** (step 6): the derived `PartialEq` of `Removal`
   reads every field, so the compiler never sees it unread, and the expectation went
   unfulfilled.

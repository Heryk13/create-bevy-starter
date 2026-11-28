# create-bevy-starter

Builds a new Bevy game from the [`bevy-starter`](https://github.com/Heryk13/bevy-starter)
template, from a few questions, the way `create-next-app` does for Next.js.

```
create-bevy-starter [name] [folder]
```

Both arguments are optional: what they leave open is asked. Nothing is written before the last
question, the confirmation.

## What it asks

| Question     | Enter gives                                                                   |
| ------------ | ----------------------------------------------------------------------------- |
| Name         | `bevy-template`                                                               |
| Folder       | `.`, where the command runs; the game goes into `<folder>/<name>`             |
| Title        | from the name: `bevy-template` becomes `Bevy Template`                        |
| Kind         | Empty; 3D is a test level, 2D a room in tiles                                 |
| Cameras      | asked of a 3D game, any ticked with Enter or Space, Done going on. None, the  |
|              | level under a still camera; First person and Third person each add physics    |
|              | and a player walking it, and both together switch on F5; Isometric is coming  |
|              | soon                                                                          |
| Player       | asked of a 2D game. None, the room alone; Top-down square adds a square that  |
|              | walks it, the camera following                                                |
| Extras       | any ticked with Enter or Space, Done going on. Split screen, offered to a 3D  |
|              | game with a camera, lets a second player join on a gamepad (Select), the      |
|              | window cut top and bottom; Multiplayer lets friends join the game over the    |
|              | LAN, one hosting from the pause menu, the others finding it from the main     |
|              | menu                                                                          |

The answers pick the template branch the game is cloned from:

| Kind  | Cameras or player  | Branch      |
| ----- | ------------------ | ----------- |
| Empty |                    | `main`      |
| 3D    | None               | `3d/base`   |
| 3D    | One or both        | `3d/player` |
| 2D    | None               | `2d/base`   |
| 2D    | Top-down square    | `2d/player` |

What was not chosen does not come: outside its own files, the template marks everything that
names a camera, the F5 switch, the split screen or multiplayer, in the code, the docs and the
locales, and the CLI deletes those files and what they mark. A marker is a comment of the
file's kind (`//` in Rust, `#` in TOML and Fluent, `<!-- -->` in Markdown):

| Marker                               | Option left out                | Option kept |
| ------------------------------------ | ------------------------------ | ----------- |
| a line ending `// multiplayer`       | the line goes                  | stays       |
| `// multiplayer: begin` … `: end`    | the block goes, fences and all | stays       |
| `// without multiplayer: mod files;` | becomes `mod files;`           | goes        |

In the Markdown docs a kept option's markers go too: they serve a game nothing. A broken
marker (a block never closed, two that cross, a Markdown without line holding `-->`) stops the
CLI before anything is removed. The rules for the template are in `docs/specs/markers/design.md`.

## What it does

1. Clones the template's branch for the kind, cameras and player chosen into `<folder>/<name>`.
2. Deletes its `.git`: the game starts a history of its own.
3. Sets the game's name in `Cargo.toml` and the `justfile`, and its title in `src/main.rs`.
4. Removes what only the template needs: what is inside `docs/specs/` (the folder stays, with a
   `.gitkeep`), the `## Branches` section of the README and of `CONTRIBUTING.md`, and the
   `tools/` folder, which keeps the template's branches in step.
5. Removes what was not chosen: the cameras not ticked, F5's switch with fewer than two, the
   split screen and multiplayer when the answer was No, and what a kind never offers (a 2D
   game has no split screen): each file, and every line and block marked with it, its without
   lines turned into code. Of what was chosen, only the without lines go, and the markers in
   the docs.
6. Runs `git init` and makes a first commit.

It only writes into a folder that does not exist or is empty, and if any step fails, it deletes
the folder it made: a half-built game is worse than none.

## Install

```
cargo install create-bevy-starter
```

It needs [Rust](https://rustup.rs/) 1.88 or newer, and [Git](https://git-scm.com/) with
`user.name` and `user.email` set, for the game's first commit.

## Working on it

```
just run <name> <folder>
```

| Command             | What it does                                                     |
| ------------------- | ---------------------------------------------------------------- |
| `just run`          | runs the CLI, passing on its arguments                           |
| `just check`        | formatting, lints and tests                                      |
| `just fix`          | applies clippy's fixes and formats the code                      |
| `just test`         | the tests only                                                   |
| `just check-combos` | makes every game the CLI can make, from the template on GitHub,  |
|                     | runs each one's `just check`, and lists the words of an option   |
|                     | a game left out that it still has: an hour or so, before release |

It needs [just](https://github.com/casey/just). How changes are made is in
[`CONTRIBUTING.md`](CONTRIBUTING.md); the design lives in `bevy-starter`, in
`docs/specs/create-bevy-starter/design.md`.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option,
like Bevy itself.

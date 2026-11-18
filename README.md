# create-bevy-starter

Builds a new Bevy game from the [`bevy-starter`](https://github.com/Heryk13/bevy-starter)
template, from a few questions, the way `create-next-app` does for Next.js.

```
create-bevy-starter [name] [folder]
```

Both arguments are optional: what they leave open is asked. Nothing is written before the last
question, the confirmation.

## What it asks

| Question    | Enter gives                                                                    |
| ----------- | ------------------------------------------------------------------------------ |
| Name        | `bevy-template`                                                                |
| Folder      | `.`, where the command runs; the game goes into `<folder>/<name>`              |
| Title       | from the name: `bevy-template` becomes `Bevy Template`                         |
| Kind        | Empty; 3D is a test level, 2D a room in tiles                                  |
| Camera      | asked of a 3D game. None, the level under a still camera; First person adds    |
|             | physics and a player walking it; Third person and Isometric are coming soon    |
| Player      | asked of a 2D game. None, the room alone; Top-down square adds a square that   |
|             | walks it, the camera following                                                 |
| Multiplayer | No; Yes is coming soon                                                         |

The answers pick the template branch the game is cloned from:

| Kind  | Camera or player | Branch      |
| ----- | ---------------- | ----------- |
| Empty |                  | `main`      |
| 3D    | None             | `3d/base`   |
| 3D    | First person     | `3d/player` |
| 2D    | None             | `2d/base`   |
| 2D    | Top-down square  | `2d/player` |

## What it does

1. Clones the template's branch for the kind, camera and player chosen into `<folder>/<name>`.
2. Deletes its `.git`: the game starts a history of its own.
3. Sets the game's name in `Cargo.toml` and the `justfile`, and its title in `src/main.rs`.
4. Removes what only the template needs: what is inside `docs/specs/` (the folder stays, with a
   `.gitkeep`), the `## Branches` section of the README and of `CONTRIBUTING.md`, and the
   `tools/` folder, which keeps the template's branches in step.
5. Runs `git init` and makes a first commit.

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

| Command      | What it does                                   |
| ------------ | ---------------------------------------------- |
| `just run`   | runs the CLI, passing on its arguments         |
| `just check` | formatting, lints and tests                    |
| `just fix`   | applies clippy's fixes and formats the code    |
| `just test`  | the tests only                                 |

It needs [just](https://github.com/casey/just). How changes are made is in
[`CONTRIBUTING.md`](CONTRIBUTING.md); the design lives in `bevy-starter`, in
`docs/specs/create-bevy-starter/design.md`.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option,
like Bevy itself.

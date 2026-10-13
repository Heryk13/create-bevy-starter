# create-bevy-starter

Builds a new Bevy game from the [`bevy-starter`](https://github.com/Heryk13/bevy-starter)
template, from a few questions, the way `create-next-app` does for Next.js.

```
create-bevy-starter [name] [folder]
```

Both arguments are optional: what they leave open is asked. Nothing is written before the last
question, the confirmation.

## What it asks

| Question    | Enter gives                                                       |
| ----------- | ----------------------------------------------------------------- |
| Name        | `bevy-template`                                                   |
| Folder      | `.`, where the command runs; the game goes into `<folder>/<name>` |
| Title       | from the name: `bevy-template` becomes `Bevy Template`            |
| Kind        | Empty; 3D is the level with a still camera, 2D is coming soon     |
| Multiplayer | No; Yes is coming soon                                            |

## What it does

1. Clones the template's branch for the kind chosen into `<folder>/<name>`.
2. Deletes its `.git`: the game starts a history of its own.
3. Sets the game's name in `Cargo.toml` and the `justfile`, and its title in `src/main.rs`.
4. Removes what only the template needs: what is inside `docs/specs/`, and the README's
   `## Branches` section.
5. Runs `git init` and makes a first commit.

It only writes into a folder that does not exist or is empty, and if any step fails, it deletes
the folder it made: a half-built game is worse than none.

## Requirements

- [Git](https://git-scm.com/), with `user.name` and `user.email` set for the first commit.
- [Rust](https://rustup.rs/) and [just](https://github.com/casey/just), to run it from here.
- Access to the template's repository, which is private for now.

## Usage

Not published yet: it runs from this repository.

```
just run <name> <folder>
```

| Command      | What it does                                   |
| ------------ | ---------------------------------------------- |
| `just run`   | runs the CLI, passing on its arguments         |
| `just check` | formatting, lints and tests, as CI would       |
| `just fix`   | applies clippy's fixes and formats the code    |
| `just test`  | the tests only                                 |

How changes are made is in [`CONTRIBUTING.md`](CONTRIBUTING.md); the design lives in
`bevy-starter`, in `docs/specs/create-bevy-starter/design.md`.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option,
like Bevy itself.

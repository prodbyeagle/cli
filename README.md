# eagle

A lightweight macOS CLI (Rust) to automate personal workflows — project scaffolding, Minecraft server management, EagleCord, and more.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/prodbyeagle/cli/main/installer.sh | bash
```

Installs `eagle` to `/usr/local/bin`.

## Commands

| Command      | Alias | Description                                      |
|--------------|-------|--------------------------------------------------|
| `version`    | `v`   | Show current version                             |
| `update`     | `u`   | Update eagle in place from GitHub                |
| `uninstall`  | `rem` | Remove eagle from the system                     |
| `goto`       | `g`   | Fuzzy-jump to a project directory                |
| `create`     |       | Scaffold a new project from a template           |
| `minecraft`  |       | Start or create a Minecraft server               |
| `eaglecord`  |       | Install or update EagleCord (Vencord fork)       |
| `init`       |       | Install zsh shell integrations                   |
| `help`       |       | Show help                                        |

## Shell integration

Run once to enable the `g` shortcut for `goto`:

```sh
eagle init
```

Then restart your shell or run `source ~/.zshrc`. `g <query>` will fuzzy-search your projects and `cd` into the selected one.

The development root defaults to `~/Development`. Override with `--root` or `$EAGLE_DEV_ROOT`.

## Goto

```sh
eagle goto              # interactive fuzzy picker
eagle goto --root ~/Projects
```

Expected structure: `<root>/<category>/<project>/`

## Create

```sh
eagle create
```

Defaults to `~/Development`. Override with `--root` or `$EAGLE_CREATE_ROOT`.

## Minecraft

```sh
# Interactive server picker
eagle minecraft

# Create a new server
eagle minecraft create --name my-server --type paper --version 1.21.4
```

## Update

```sh
# Pull latest release from GitHub
eagle update

# Install local debug build (dev mode)
eagle update --dev

# Install a specific local binary
eagle update --dev path/to/eagle
```

## Dev mode

Debug builds automatically enable dev mode: version shows as `vX.Y.Z-dev` and each command logs timing and dispatch info to stderr.

```sh
cargo build           # dev mode on
cargo build --release # dev mode off
```

## Development

```sh
cargo run -- help
cargo fmt && cargo clippy && cargo test
```

## License

MIT — see [LICENSE](LICENSE).

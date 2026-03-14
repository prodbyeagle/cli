# eagle

A lightweight Windows CLI (Rust) to automate personal workflows — project scaffolding, Minecraft server management, EagleCord, and more.

## Install

```powershell
Invoke-WebRequest -UseBasicParsing `
    https://raw.githubusercontent.com/prodbyeagle/cli/main/installer.ps1 |
    Invoke-Expression
```

Installs `eagle.exe` to `C:\Scripts` and adds it to your PATH.

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
| `init`       |       | Install PowerShell shell integrations            |
| `help`       |       | Show help                                        |

## Shell integration

Run once to enable the `g` shortcut for `goto`:

```powershell
eagle init
```

Then restart your shell. `g <query>` will fuzzy-search your projects and `cd` into the selected one.

The development root defaults to `~/Development`. Override with `--root` or `$EAGLE_DEV_ROOT`.

## Goto

```powershell
eagle goto              # interactive fuzzy picker
eagle goto --root D:\Projects
```

Expected structure: `<root>/<category>/<project>/`

## Create

```powershell
eagle create
```

Defaults to `%USERPROFILE%\Development`. Override with `--root` or `$EAGLE_CREATE_ROOT`.

## Minecraft

```powershell
# Interactive server picker
eagle minecraft

# Create a new server
eagle minecraft create --name my-server --type paper --version 1.21.4
```

## Update

```powershell
# Pull latest release from GitHub
eagle update

# Install local debug build (dev mode)
eagle update --dev

# Install a specific local binary
eagle update --dev path\to\eagle.exe
```

## Dev mode

Debug builds automatically enable dev mode: version shows as `vX.Y.Z-dev` and each command logs timing and dispatch info to stderr.

```powershell
cargo build           # dev mode on
cargo build --release # dev mode off
```

## Development

```powershell
cargo run -- help
.\scripts\check.ps1
```

## License

MIT — see [LICENSE](LICENSE).

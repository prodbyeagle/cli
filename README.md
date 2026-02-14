A lightweight native CLI (Rust) to automate a few personal workflows
(Spicetify, EagleCord, project templates, Minecraft server launcher, etc.).

Windows-only CLI.

## Install (Windows)

```powershell
Invoke-WebRequest -UseBasicParsing `
	https://raw.githubusercontent.com/prodbyeagle/cli/main/installer.ps1 |
	Invoke-Expression
```

Installs `eagle.exe` to `C:\Scripts` and sets a PowerShell alias `eagle`.

## Usage

```powershell
eagle help
eagle <command> [args]
```

### Minecraft

```powershell
# Start an existing server (interactive selector)
eagle minecraft

# Create a new server
eagle minecraft create --name my-server --type paper --version 1.21.4
```

### Create

`eagle create` defaults to `%USERPROFILE%\Development\.YY`.

You can override the base root with:

- `--root C:\some\path`
- `%EAGLE_CREATE_ROOT%`

### Codex

```powershell
eagle codex
```

Opens a Windows Terminal session in `D:\development\.26\eagle` and runs:
`codex --yolo`.

## Dev

```powershell
cargo run -- help
.\scripts\check.ps1
.\scripts\release.ps1 -SetVersion '3.1.0'
# dry run:
.\scripts\release.ps1 -SetVersion '3.1.0' -DryRun
```

On macOS/Linux:

```sh
./scripts/check.sh
```

# Security Policy

## Supported Versions

Only the latest release of eagle is supported with security updates.

| Version | Supported |
|---------|-----------|
| Latest  | ✅        |
| Older   | ❌        |

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Instead, report them privately via GitHub's built-in security advisory system:

1. Go to the [Security tab](https://github.com/prodbyeagle/cli/security)
2. Click **"Report a vulnerability"**
3. Fill in the details

I'll respond as soon as possible. Once confirmed, a fix will be released and the advisory published.

## Scope

This is a personal Windows CLI tool. The main surfaces worth reporting:

- **Self-update** (`eagle update`) — binary download, SHA-256 verification, and file replacement
- **PowerShell script execution** — delayed process replacement via spawned `powershell.exe`
- **External process invocation** — `git`, `bun`, `winget`, and Minecraft server JARs

## Out of Scope

- Issues that require physical access to the machine
- Vulnerabilities in third-party dependencies (report those upstream)

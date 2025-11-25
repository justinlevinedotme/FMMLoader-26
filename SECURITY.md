# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in FMMLoader26, please report it responsibly:

1. **Do NOT open a public GitHub issue** for security vulnerabilities
2. **Email:** Send details to justin@justinlevine.me
3. **Discord:** DM @jalco on the [Discord server](https://discord.gg/AspRvTTAch)

### What to include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Any suggested fixes (optional)

### Response timeline

- **Acknowledgment:** Within 48 hours
- **Initial assessment:** Within 1 week
- **Fix/patch:** Depends on severity, typically within 2 weeks for critical issues

### Scope

This policy covers:
- The FMMLoader26 application (Tauri/Rust backend, React frontend)
- Official releases from this repository

This policy does NOT cover:
- Third-party mods installed via FMMLoader26
- Football Manager 2026 itself
- User-generated content

## Security Features

FMMLoader26 includes several security measures:
- Automatic backup creation before file modifications
- Sandboxed Tauri environment with minimal permissions
- No network requests except for update checks to GitHub releases
- All mod operations are local and reversible

Thank you for helping keep FMMLoader26 secure!

# Security

## Reporting a vulnerability

Please **do not** open a public GitHub issue for undisclosed security problems.

1. **Email:** [yuzizhan000@gmail.com](mailto:yuzizhan000@gmail.com) — use subject `[Keycard] Security report`.
2. **GitHub:** If the repository has [private vulnerability reporting](https://docs.github.com/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability) enabled, use **Security → Report a vulnerability** on GitHub instead of a public issue.

Include:

- A short description and impact
- Steps to reproduce (if safe to share)
- Affected versions or commit, if known

We will aim to acknowledge within a few business days. This is a best-effort commitment for a community project.

**Repository owners:** Turn on GitHub **Settings → General → Security → Private vulnerability reporting** so reporters can use “Report a vulnerability” without a public issue.

## Scope

In scope: Keycard **application and CLI** as shipped from this repository (crypto, vault format, local storage, IPC surface).

Out of scope: generic malware on the user’s machine, leaked backups, weak master passwords chosen by users, or third-party Automator/scripts misconfigured by users.

## Security notes for contributors

- Do not log or echo decrypted secrets. See `README.md` → Security notes and `docs/MANUAL_TEST.md`.
- Threat model and crypto design: `docs/superpowers/specs/2026-03-29-keycard-design.md`.

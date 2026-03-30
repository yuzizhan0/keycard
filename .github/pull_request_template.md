## What

<!-- Briefly describe the change -->

## Why

<!-- Motivation, issue link: Fixes #… -->

## Checklist

- [ ] No secrets, real keys, or personal paths in commits
- [ ] `cargo test -p keycard-core` (and `-p keycard-cli` if CLI touched) pass locally
- [ ] If user-facing: `README.md` / `docs/cli-setup-*.md` updated when behavior or paths change
- [ ] Security: no logging of decrypted vault material

## Notes for reviewers

<!-- Risk areas, follow-ups, screenshots -->

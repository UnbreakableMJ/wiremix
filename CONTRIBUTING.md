# Contributing to wiremix

Thank you for your interest. Please read this document before opening an issue
or pull request — it sets honest expectations for both sides so no one's time is
wasted.

## Project Stance

wiremix is a **personal hobby project** under the Spacecraft Software umbrella,
shaped around the maintainer's own use case and developed at hobby pace. It is a
fork of the upstream `wiremix` by Thomas Sowell (see [`CREDITS.md`](./CREDITS.md)).

This is **not** a community-driven project, but external input is welcome and
appreciated within the bounds set out below.

## What Is Welcome

- **Bug reports** — clear, reproducible, with environment details (OS, kernel,
  Rust toolchain version, PipeWire version, shell, relevant config).
- **Suggestions** — features, refactors, design feedback.
- **Pull requests** — small, focused, aligned with the Spacecraft Software
  Standard and the dual-mode CLI contract (see below).
- **Documentation fixes** — typos, inaccuracies, broken links, clarifications.
- **Test coverage improvements** — almost always merge-worthy.

## What Is Not Guaranteed

- **PR acceptance.** Direction, scope, and quality bar are set by the maintainer
  alone. A correct, well-written PR that passes CI is still not a guaranteed
  merge. Rejection reflects fit, not quality.
- **Response time.** Hobby pace — expect days to weeks, not hours.
- **Roadmap influence.** Suggestions may inform direction but do not override the
  maintainer's plans.
- **API/CLI stability for in-progress work.** Pre-1.0 surfaces may break in any
  release; the `--json` schema follows the deprecation policy in the manual.

## Before Opening a PR

1. **Open an issue first** for non-trivial changes. Discuss the design before
   writing code.
2. **Read the Spacecraft Software Standard.** Stability (memory safety first) →
   performance → hardened security, in that order. Rust where viable.
   POSIX-compliant CLI. GPL-3.0-or-later with REUSE/SPDX headers on every file.
3. **Honor the dual-mode CLI contract.** Every data-returning subcommand must
   support `--json`, emit the standard envelope, use the canonical exit codes,
   and surface structured errors on stderr (see the `spacecraft-cli-standard`
   and `spacecraft-agentic-cli` skills, and the manual). stdout is data only.
4. **Run the full local gate** (inside `nix develop`):
   `cargo fmt --all --check`, `cargo clippy --all-features --all-targets -- -D
   warnings`, `cargo test --all-features --all-targets`, and `reuse lint`.
   PRs that don't pass CI will not be reviewed.
5. **Add REUSE headers** to new files (two SPDX tags). New work is
   `GPL-3.0-or-later`; do not strip upstream `MIT OR Apache-2.0` headers from
   files you don't substantially rewrite.
6. **Sign your commits cryptographically and with sign-off.** Commits to a
   Spacecraft Software remote must be both GPG/SSH-signed (showing "Verified",
   Standard §6.3) and carry a Developer Certificate of Origin sign-off
   (`git commit -S -s`).

## Commit Style

- Conventional Commits prefix (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`,
  `chore:`, `perf:`, `build:`, `ci:`).
- Subject ≤ 72 characters, imperative mood ("add" not "added").
- Body wrapped at 72 columns; explain *why*, not just *what*.
- Reference issues by number (`Closes #42`).

## Forking

If your needs diverge from the maintainer's, **fork it** — that is exactly what
GPL-3.0-or-later is for. Keep the source open under a compatible license,
preserve copyright notices, and pass the same freedoms downstream.

## Reporting Security Issues

For security-sensitive bugs, do **not** open a public issue. Email
&lt;Mohamed.Hammad@SpacecraftSoftware.org&gt; with details. A coordinated-disclosure
window of 90 days from acknowledgment is the default; this can be shortened or
lengthened by mutual agreement.

## License of Contributions

By submitting a contribution, you agree it will be licensed under
**GPL-3.0-or-later**, the same terms as the project. You retain copyright in your
contributions; no CLA is required.

---

**Maintainer:** Mohamed Hammad &lt;Mohamed.Hammad@SpacecraftSoftware.org&gt;
**License:** GPL-3.0-or-later
**Website:** <https://Wiremix.SpacecraftSoftware.org/>

*--- Forged in Spacecraft Software ---*

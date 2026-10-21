# sloptopop quality standard (every repo, every language)

Nothing may be skipped. Each repo's `scripts/gate.sh` enforces every item
below; a missing tool **fails** the gate, never skips a step. The full gate
runs on `git push` (pre-push hook) and in CI.

## Required checks

| # | Standard | Requirement |
|---|---|---|
| 1 | Pinned toolchain | Exact compiler/runtime version pinned in a file in the repo and verified by the gate (fails on any other version). Exact versions of gate tools pinned via the language's lockfile / tool manifest where it has one. |
| 2 | Formatting | Formatter check passes with no diff. |
| 3 | Zero warnings | Compiler/type checker and the strictest practical linter configuration; every warning is an error. Suppressions only inline, per occurrence, with a written reason. |
| 4 | All tests pass | Unit tests, the example program, tests against the real `sloptopop` binary, and every case in `cli/conformance/cases`. |
| 5 | 100% coverage | Every metric the language's coverage tool offers (lines + branches/regions + functions where available) at 100% on product code. Only test code is excluded. |
| 6 | Mutation testing | A mutation tool runs over product code; every mutant must be killed (timeouts count as killed). Where no maintained OSS tool exists for the language, the repo ships its own mutation script with the same rule. |
| 7 | Dependency vulnerabilities | Every dependency (runtime and dev) is scanned for known vulnerabilities; any finding fails. Use OSV-Scanner (`osv-scanner scan source -r .`, Apache-2.0) on the lockfile, plus the language's native auditor where one exists. A repo with no dependencies must assert that it has none. |
| 8 | Dependency licences | Every dependency's licence is checked against the allow-list: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, 0BSD, Zlib, CC0-1.0, Unicode-3.0, MPL-2.0, PSF-2.0, BlueOak-1.0.0, BSL-1.0. Anything else fails. |
| 9 | Docs | Public API documentation complete (the linter/doc tool enforces it where the language has one). |
| 10 | Pre-push hook | `.githooks/pre-push` runs `scripts/gate.sh`; `git config core.hooksPath .githooks` is set. |
| 11 | CI | `.github/workflows/gate.yml` runs the full gate. |

## Gate modes

- `scripts/gate.sh` — everything above (pre-push and CI).
- `scripts/gate.sh --fast` — 1–5 and 9 (Claude Code runs this before ending a turn).
- `scripts/gate.sh --check` — 1–3 only, in seconds (Claude Code runs this after each edit).

The modes only change *when* a check runs, never *whether*: nothing reaches a
remote without the full gate.

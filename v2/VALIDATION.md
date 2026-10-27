# Observed validation

Own source **b1edb8f7837a148fef50fdf79fcfe917a7aed952**, branch **feat/native-v2-rust**. Stable compiler is **rustc 1.98.0 (88d9e12ae 2026-08-18)**; compatibility compiler is **1.85.0**. `validation-results.json` records the 28 real commands and their exit codes; all returned zero. Disposable logs are under `.scratch/validation`, excluded from the archive. Run `python3 scripts/validate.py` to reproduce.

| Check | Result | Scope |
|---|---|---|
| Formatter oracle | PASS | 780 real Prettier 2.5.1 comparisons, four indentation modes, pinned upstream inputs, errors and reference-idempotent output |
| Pure Rust tests | PASS on stable + 1.85 | 3 tests: oracle corpus, private/bounded errors, allocation and Unicode representation limits |
| Concrete production adapter harness | PASS on stable + 1.85 | 6 adapter/NativeSessions lifecycle tests plus the original SDK/common's 5 tests; Tauri/router/registry/dispatch are explicit stubs |
| Source oracle | PASS | Actual own TSX, recovered actual locked utils 2.3.0 hook, actual beta useForm initializer, real pinned Prettier, recorded effect sequencing/defaults/validation/copy failure |
| Renderer contract | PASS | Actual pinned and patched FormRuntime/Surface with hook/UI imports stubbed; dropdown options, callback payload, complete secondary submit, one-time applied/rejected replacement consumption using fresh real adapter traces, identical-byte new operations, cloned/unchanged refresh, indentation, copy/retry, cancellation, pure repeated state updaters and legacy reconciliation |
| Manifest | PASS | Actual Draft 2020-12 schema, Ajv 2020, 10 negative cases; both platform verification states remain not-run |
| Evidence | PASS | 309 relative-path records, every size/SHA256, original npm-lock SHA512 integrity, source-map recovery, original icon, actual npm notices, unchanged harness sources |
| Formatting | PASS on stable + 1.85 | Cargo fmt and direct rustfmt for adapter/harness sources; immutable fixtures are never reformatted |
| Strict all-target Clippy | PASS on stable + 1.85 | Both standalone crate and concrete host harness, `-D warnings`; no new-Clippy-only lint allowances |
| Windows GNU check + strict Clippy | PASS on stable + 1.85 | `x86_64-pc-windows-gnu`, all targets, both manifests; no Windows executable was run |
| Cargo packaging | PASS | Original archive members and Git visibility checked, including all indexed evidence and template harness manifests |
| Extracted offline reproduction | PASS on stable + 1.85 | Fresh system-temp extraction; Node oracle/source/renderer/schema/evidence checks and both Rust manifests. Standard cached Cargo dependencies are required; no host checkout/absolute source path is used |
| Integration patch application | PASS in disposable copies | Registration/Form patches apply; prior catalog entries remain identical and one Format GraphQL entry is appended |
| Original icon | PASS | 224850 bytes; SHA256 `98addbe45d1d95735dd96b50284d0e6ab6c6d67736ab81f7446fc32f4301d9e4` |
| V1 baseline | Smoke test PASS; typecheck FAIL as originally shipped | `v1-baseline.json`: unchanged locked disposable install; typecheck exit 2 / TS2307 for missing prettier; actual Node lookup MODULE_NOT_FOUND; V1 lock unchanged |

Review round 2 reproduced the parent assertion against the previous actual proposed renderer: after observing a replacement and typing `{a}` again, a cloned heartbeat produced `"{\\n  a\\n}\\n"` instead of the intentional `"{a}"`. The former test incorrectly treated a cloned old response as a new request. The corrected test requires a new `replacementId`, and tests both already-applied and already-rejected results. The adapter commits `{id, base, output}` before persistence/copy awaits, retaining the identity through refresh, indentation, copy completion, cancellation and explicit retry. The mounted renderer consumes each identity on first observation, including rejection, remembers all observed identities across omitted fields, and preserves existing unguarded reconciliation. No immutable beta/base fixture changed.

`node scripts/renderer-tests.cjs` prepares the isolated concrete harness and runs `replacement_identity_trace` to obtain fresh actual responses rather than relying only on fabricated props. Its Rust assertions cover committed state, mutation-free refresh clones, distinct identities for new identical-byte submissions, stable cancelled intent/refresh/retry identity, and recorded copy counts. The trace is disposable under `.scratch`; it is regenerated when running the extracted package checks.

Final XERJ refresh: canonical prefix `ax-ed7d45a32474`. Initial V1 index covered all 3/3 code files (9 searchable files / 75 records; exit 3 completed-with-junk). After adding V2, the refresh aborted in 0.4 seconds:

```text
project new file v2/Cargo.toml: no frozen dataset accepts family docs,
group None, and fields ["end_line", "start_line", "text"];
this requires unsupported dataset/schema evolution
```

Thus the initial V1 code search is covered, but **new V2 files are not covered by the final XERJ index**. No schema/prefix workaround, global service restart or unrelated process cancellation was performed. Current-file `rg`, direct fixture comparison, archive membership/Git visibility checks and executable tests provide V2 verification. Archive/junk/binary coverage is not represented as semantic code search coverage.

Remaining gaps are explicit: missing Prettier in the own V1 lock; test-only utils 2.3.0 missing published license text despite its MIT declaration; isolated UTF-16-surrogate representation; potential untested Unicode-version width or formatting/comment edge cases; native Tabs/storage and retained-form feedback adaptations. These do not become claims of full Raycast parity. The complete limits and behavior are in README/NOTICES/PROVENANCE.

**NOT-RUN:** host registration/rebuild/install, real host registry/catalog dispatch, real macOS/Windows dropdown/UI/keyboard, real clipboard/focus/pop-to-root behavior. No manifest verification state is upgraded. No stage, commit, push, PR, clone deletion, ledger update, host/profile write or real clipboard/focus/paste action was performed. The parent owns integration/review/commit/push/remote verification and eventual clone cleanup.

# Format GraphQL — offline Rust Native V2

This is a compiled-in Rust port of the one `index` command, **Format GraphQL / Formats a GraphQL document**. The canonical ID is `com.tezbar.native.format-graphql`; it replaces `raycast.format-graphql`. It formats locally, offers **Format and Copy** followed by **Only Format**, and keeps the editable form open with feedback. Production contains no JavaScript interpreter, formatter subprocess, network request, clipboard read, selection read, paste, browser route or window/focus operation.

The exact original PNG is `assets/command-icon.png`: 224850 bytes, SHA256 `98addbe45d1d95735dd96b50284d0e6ab6c6d67736ab81f7446fc32f4301d9e4`. Integration copies these bytes to `src/renderer/public/extensions/icons/format-graphql.png`; the manifest uses `/extensions/icons/format-graphql.png`.

## Evidence and behavior

Own source: `almatkai/format-graphql@b1edb8f7837a148fef50fdf79fcfe917a7aed952`, fetched from `origin/main`. The upstream Raycast source/manifest/icon are pinned at `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Original author: **tngranados**; contributor: **xmok**. The own manifest names almatkai; V2 credits the original authors in its description and notices because schema 2 does not support a root `author`.

The own source imports Prettier, but its manifest and lockfile omit it. The unchanged disposable V1 install passes its one smoke test; typechecking fails with `src/index.tsx(3,22): TS2307: Cannot find module 'prettier' or its corresponding type declarations`. Node resolution in that clean install also returns `MODULE_NOT_FOUND`. The beta builder can install missing bare imports without the original version constraint, and its runtime resolves from the extension's Node require context. Ancestor/host packages can therefore affect resolution. Neither that path nor the host's declared `prettier: ^3.5.3` establishes an own locked formatter version.

The **comparison baseline is upstream-locked Prettier 2.5.1**, whose upstream lock records its exact resolved tarball and integrity. It is not claimed to be the own V1 lock. `src/format.rs` and `src/document.rs` translate that printer's GraphQL-specific rules and group/indent/line model to Rust. Apollo Parser 0.8.5 validates grammar; it is not used as an AST pretty printer. The port supports executable documents, schema/type definitions, descriptions, directives, extensions supported by the reference, fragments (including experimental fragment variables), aliases, legacy interface separators, strings, block strings, comments, `prettier-ignore`, original empty lines, normalization of commas, LF and the default print width 80. The fixtures contain 780 independent Prettier comparisons across Tabs and 2/4/8 spaces, including the pinned upstream GraphQL test suite. Expectations are produced by real Prettier, never by Rust. This is evidence of those cases, not a universal parity proof.

`extend schema` is deliberately rejected: Prettier 2.5.1 itself throws `unknown graphql type: "SchemaExtension"`. Invalid input leaves the current editor draft intact. Empty input has required-field feedback; whitespace is accepted by the source validation and formats to an empty string. Numeric indentation values exposed by the source are `2`, `4`, `8`; those retain `parseInt` semantics. Forged variants such as `2x`, `04`, negative or out-of-range widths are rejected instead of expanding the UI's domain. Paired Unicode escapes are decoded before Apollo validation, which lacks paired-surrogate support. Isolated UTF-16 surrogate escapes are explicitly rejected because Rust/IPC strings require valid Unicode; the JS baseline can emit lone UTF-16 surrogates. A fixture records this known representation gap. Literal tabs are accepted; invalid control characters are rejected. As in the baseline, some *escaped* control characters can produce output that the baseline itself cannot reparse; oracle idempotence is checked where the reference is idempotent.

## Form, storage and effects

The actual own TSX, actual locked `@raycast/utils` 2.3.0 `useForm` source recovered from its published source map, and actual pinned beta `useForm` initializer are executable test fixtures. React hooks and Raycast effects are explicit recorders. The real locked hook initially returns `null` values and stable IDs; the beta shim creates itemProps only for known keys and initially omits `indent`. Its renderer derives a fallback ID, so a complete beta form map does not supply the source handler's expected `indent`. Beta `storeValue` is not implemented. These are source/host compatibility gaps, preserved as evidence rather than repaired in V1.

V2 exposes stable `input` and `indent` IDs, uses **Tabs as an explicit native initial default**, and stores only the indentation in the SDK namespace `native-extensions-v2/format-graphql/storage.json`. This implements the source's storage intent; it does not claim observed initial selection behavior on real Raycast or migrate unknown V1 dropdown data. The beta dropdown also exposes its built-in blank Select choice; a blank/forged value gets validation and cannot corrupt storage. Arguments and preferences, if inspected, are the host's `payload.argumentValues` and `payload.preferences`; this command declares neither.

Field callbacks read `formValues.value`; both submit actions require the complete `formValues` map. Source handlers add `copy: true/false`; the native action ID chooses this effect rather than trusting a client-provided copy flag. Unrelated responses omit controlled values. Each successful explicit format creates a unique `replacementId`, committed with its submitted `replacementBase` and output before any await. The required renderer patch consumes each identity once per stable field for the mounted Form, including replacements rejected because the draft differs from the submitted base. Repeating an applied or rejected identity never overwrites a later draft, even after editing back to the old base or after an intervening response omits the fields. Refresh, indentation, copy completion, cancellation and deliberate copy retry retain the committed identity; they cannot re-arm it. A new explicit format creates a new identity and can apply identical base/output bytes, guarded by the current submitted-base comparison on its first response. Unguarded extensions retain beta reconciliation. A newer local draft survives old asynchronous responses, refreshes, validation failures and retry. Back does not clear form state; leaving/disposal ends the native session. Copy feedback is returned after the async acknowledgement; formatting state is checkpointed before awaits so cancellation does not lose the canonical output. There is no detached background copy.

The unchanged pinned SDK serializes stores with its process mutex and writes atomically, preserving unknown keys. The adapter validates the stored indentation on read and inside **every** update transaction; malformed JSON, non-object storage and typed corruption are not overwritten, including corruption introduced after launch. There is no cross-process transaction guarantee and no asserted V1 storage migration.

Each launch and rotated action set receives process-nonce plus monotonic-counter IDs. Stale, cross-session and forged action IDs are rejected. Refresh reuses committed IDs without mutation; search and load-more return unchanged and do not mint tokens in discarded clones. Disposal revokes the session through NativeSessions. A formatted copy intent is saved before the clipboard await. Only an object with `ok:true`, absent/null `error`, and absent/true `success` acknowledges success. Null, booleans, malformed, contradictory and transport replies remain uncertain. Refresh/search never recopy; **Retry Copy (May Copy Again)** is explicit and bound to the saved canonical output. Private transport/parser diagnostics are not shown. Successful acknowledgement remains checkpointed.

The original toast/HUD/pop-to-root sequence is recorded in source tests. There are no genuine SDK equivalents for all of those effects. V2 uses supported Form feedback and stays open; it makes no focus/pop-to-root claim. Beta `window:hide []` hides but returns null, which is not a valid acknowledgement. This port does not call it and needs no hide acknowledgement patch. It uses no Markdown copy controls or custom shortcut/tooltips.

## Bounds

Inputs are UTF-8, at most 65536 bytes; output at most 524288 bytes. Limits are 16384 significant tokens plus comments, 512 comments, 64 nested delimiter pairs, Apollo recursion 256 / validation-token budget 32768, and 4000000 document-print/probe steps. Source structure and comments are bounded by those token/input limits. Limits return explicit errors; no document or output is silently truncated. Indentation is restricted to the four UI choices. Width calculations use pinned unicode-width 0.2.2; rare Unicode-version width differences beyond the tested CJK/emoji/combining cases remain possible.

## Build and reproduce

Rust dependencies use the checked-in `Cargo.lock`; fetch them once on a provisioned machine. All formatter/TSX/renderer/schema oracle tools are pinned test-only archives in `fixtures/library`, extracted under `.scratch` without package installs or lifecycle hooks. Node and Python are required only for these reproduction checks. A fresh air-gapped machine still needs a preseeded Cargo registry; no absolute clone or host paths are used by the extracted-package tests.

From `v2/`:

```sh
python3 scripts/prepare-fixtures.py
node scripts/oracle.cjs --check
node scripts/source-oracle.cjs --check
node scripts/renderer-tests.cjs
node scripts/schema-tests.cjs
python3 scripts/verify-evidence.py
cargo test --locked --offline
cargo +1.85.0 test --locked --offline
cargo fmt -- --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo +1.85.0 clippy --locked --offline --all-targets -- -D warnings
python3 scripts/prepare-harness.py
cargo test --manifest-path .scratch/host-harness/Cargo.toml --locked --offline
cargo +1.85.0 test --manifest-path .scratch/host-harness/Cargo.toml --locked --offline
cargo clippy --manifest-path .scratch/host-harness/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo +1.85.0 clippy --manifest-path .scratch/host-harness/Cargo.toml --locked --offline --all-targets -- -D warnings
```

`tests/harness/Cargo.template.toml` materializes a disposable, relative-path Cargo harness. It compiles the **exact production adapter**, unchanged pinned beta SDK/common/NativeSessions, explicit Tauri/OS/router/registry/dispatch stubs and temporary stores. The SDK/common's five original tests run, plus six concrete adapter/lifecycle tests. The replacement-identity test also exports fresh actual adapter responses to `.scratch/renderer-adapter-trace.json` for the renderer regressions; the renderer script prepares/runs that isolated Rust harness itself. State updater callbacks are exercised twice against the same prior state to check purity. The original NativeSessions' Perplexity-specific unit-test module is excluded by the harness's module cfg; the real NativeSessions production implementation runs in integration tests. This does not exercise the actual host registry or Tauri runtime.

For Windows cross-validation, run `cargo check` and `cargo clippy` with `--locked --offline --all-targets --target x86_64-pc-windows-gnu` for both manifests, adding `-- -D warnings` to Clippy. Repeat with `cargo +1.85.0`. These are cross-compilation checks, **not Windows execution**. Use `python3 scripts/check-package.py` for an archive membership check, extraction and offline reproduction outside this checkout. See `VALIDATION.md` for observed results.

## Integrate and verify separately

Follow `integration/README.md`. A manifest alone cannot execute this extension: vendor the crate and adapter, add the host path dependency, register the module/slug/command/handler and catalog entry, copy the exact icon, apply the narrow Form contract patch, rebuild, then install the compiled canonical ID. Registration and renderer patches are review artifacts and have **not** been applied to the dirty host. They preserve preceding catalog entries/ports and use current committed host HEAD as their integration base, while beta runtime evidence remains pinned at `fd300e276356587726566026f072563eb9c34cb1`.

Real macOS/Windows installation, dropdown/keyboard/UI, clipboard/focus QA and host rebuild/registration are **NOT-RUN**. Both manifest verification states remain `not-run`. V1 stays available for fallback. `PROVENANCE.md`, `NOTICES.md`, and `fixtures/INDEX.json` identify originals, generated fixtures, licenses and evidence gaps.

#!/usr/bin/env python3
"""Materialize a disposable Cargo harness from relative immutable fixtures."""
from pathlib import Path
import shutil
root=Path(__file__).resolve().parents[1]
out=root/'.scratch/host-harness'
(out/'src/native_core/extensions').mkdir(parents=True,exist_ok=True)
(out/'tests').mkdir(exist_ok=True)
shutil.copyfile(root/'tests/harness/Cargo.template.toml',out/'Cargo.toml')
shutil.copyfile(root/'tests/harness/lib.rs',out/'src/lib.rs')
beta=root/'fixtures/beta/src-tauri/src/native_core'
for name in ['common.rs','extensions/native.rs','extensions/v2_sdk.rs','extensions/native_tests.rs']:
 shutil.copyfile(beta/name,out/'src/native_core'/name)
shutil.copyfile(root/'adapter/format_graphql.rs',out/'src/native_core/extensions/format_graphql.rs')
shutil.copyfile(root/'tests/harness/runtime.rs',out/'tests/runtime.rs')
if (root/'tests/harness/Cargo.lock').exists():shutil.copyfile(root/'tests/harness/Cargo.lock',out/'Cargo.lock')
print(out.relative_to(root))

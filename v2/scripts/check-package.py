#!/usr/bin/env python3
"""Package/extract/test offline in a system temp directory, retaining artifacts.

No source, stage, commit, host checkout or profile is modified. The Cargo registry
must already be populated; test-only Node tools ship inside the archive.
"""
from pathlib import Path
import subprocess,tarfile,tempfile,shutil,json
root=Path(__file__).resolve().parents[1]
def run(args,cwd):subprocess.run(args,cwd=cwd,check=True)
run(['cargo','package','--allow-dirty','--locked','--offline','--no-verify'],root)
archive=root/'target/package/tezbar-format-graphql-0.1.0.crate'
listing=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard'],cwd=root,text=True).splitlines()
visible=set(listing)
# Cargo include policy and Git visibility must both cover the deliverable.
required=['adapter/format_graphql.rs','manifest.json','Cargo.lock','scripts/prepare-harness.py','tests/harness/Cargo.template.toml','tests/harness/Cargo.lock','fixtures/oracle.json','fixtures/INDEX.json','fixtures/library/prettier-2.5.1.tgz','fixtures/library/typescript-5.9.3.tgz','integration/registration.patch','integration/form-contract.patch']
for p in required:assert p in visible,('not visible to Git',p)
out=Path(tempfile.mkdtemp(prefix='format-graphql-package-'))
with tarfile.open(archive) as tar:
 names=set(tar.getnames());prefix='tezbar-format-graphql-0.1.0/'
 for p in required:assert prefix+p in names,('missing archive member',p)
 for name in names:
  rel=name.removeprefix(prefix)
  assert not any(part in ['.scratch','target','node_modules','__pycache__'] for part in Path(rel).parts),('disposable artifact in archive',rel)
  assert rel in visible or rel in ['Cargo.toml.orig','.cargo_vcs_info.json'],('archive file not visible to Git',rel)

 for r in json.loads((root/'fixtures/INDEX.json').read_text())['records']:
  assert r['path'] in visible,('evidence not visible to Git',r['path'])
  assert prefix+r['path'] in names,('evidence missing from archive',r['path'])
 for member in tar.getmembers():
  p=Path(member.name);assert not p.is_absolute() and '..' not in p.parts and not member.issym() and not member.islnk()
  tar.extract(member,out)
crate=out/'tezbar-format-graphql-0.1.0'
run(['python3','scripts/prepare-fixtures.py'],crate)
for script in ['oracle.cjs','source-oracle.cjs','representation-gap.cjs']:run(['node','scripts/'+script,'--check'],crate)
for script in ['renderer-tests.cjs','schema-tests.cjs']:run(['node','scripts/'+script],crate)
run(['python3','scripts/prepare-harness.py'],crate)
run(['python3','scripts/verify-evidence.py'],crate)
for toolchain in [None,'+1.85.0']:
 cmd=['cargo']+([toolchain] if toolchain else [])
 run(cmd+['test','--locked','--offline'],crate)
 run(cmd+['test','--manifest-path','.scratch/host-harness/Cargo.toml','--locked','--offline'],crate)
# Apply review patches only to a disposable copy of committed integration files.
patchroot=out/'patch-check';shutil.copytree(crate/'integration/base',patchroot)
snapshot=patchroot/'src-tauri/Cargo.toml.snapshot';snapshot.rename(snapshot.with_name('Cargo.toml'))
for patch in ['registration.patch','form-contract.patch']:
 run(['git','apply','--check',str(crate/'integration'/patch)],patchroot)
 run(['git','apply',str(crate/'integration'/patch)],patchroot)
for name in ['form-runtime.tsx','index.tsx']:
 assert (patchroot/'src/renderer/src/raycast-api'/name).read_bytes()==(crate/'integration/renderer'/name).read_bytes()
catalog=json.loads((patchroot/'src-tauri/src/native_core/extensions/v2_catalog.json').read_text());before=json.loads((crate/'integration/base/src-tauri/src/native_core/extensions/v2_catalog.json').read_text());assert catalog[:-1]==before and catalog[-1]['slug']=='format-graphql'
print('Archive and extracted offline tests passed; patches preserve all prior catalog entries.')
print('Retained extracted review directory:',out)

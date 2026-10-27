#!/usr/bin/env python3
"""Offline integrity checks, independent of any host checkout or absolute path."""
from pathlib import Path,PurePosixPath
import hashlib,json,base64,tarfile
root=Path(__file__).resolve().parents[1]
index=json.loads((root/'fixtures/INDEX.json').read_text());seen=set()
for record in index['records']:
 rel=record['path'];p=PurePosixPath(rel)
 assert not p.is_absolute() and '..' not in p.parts and rel not in seen,rel
 seen.add(rel);data=(root/rel).read_bytes()
 assert len(data)==record['size'] and hashlib.sha256(data).hexdigest()==record['sha256'],rel
actual={p.relative_to(root).as_posix() for folder in ['fixtures','assets','integration/base'] for p in (root/folder).rglob('*') if p.is_file() and p.name!='INDEX.json'}
assert actual==seen,(actual-seen,seen-actual)
known=[('fixtures/own/package-lock.json','@raycast/utils','raycast-utils-2.3.0.tgz'),('fixtures/own/package-lock.json','typescript','typescript-5.9.3.tgz'),('fixtures/upstream/extensions/format-graphql/package-lock.json','prettier','prettier-2.5.1.tgz')]
for lock,name,archive in known:
 entry=json.loads((root/lock).read_text())['packages']['node_modules/'+name]
 alg,digest=entry['integrity'].split('-',1)
 assert base64.b64encode(hashlib.new(alg,(root/'fixtures/library'/archive).read_bytes()).digest()).decode()==digest,archive
with tarfile.open(root/'fixtures/library/raycast-utils-2.3.0.tgz') as archive:
 source_map=json.loads(archive.extractfile('package/dist/main.js.map').read())
 for name in ['useForm.tsx','useLatest.ts']:
  contents=[s for path,s in zip(source_map['sources'],source_map['sourcesContent']) if Path(path).name==name]
  assert len(contents)==1
  assert (root/'fixtures/library/raycast-utils-source'/name).read_bytes()==contents[0].encode(),name
for record in index['records']:
 if not record['path'].startswith('fixtures/licenses/npm/'):continue
 archive,member=record['origin'].split('#',1)
 with tarfile.open(root/archive) as tar:
  assert tar.extractfile(member).read()==(root/record['path']).read_bytes(),record['path']
icon=(root/'assets/command-icon.png').read_bytes()
assert len(icon)==224850 and hashlib.sha256(icon).hexdigest()=='98addbe45d1d95735dd96b50284d0e6ab6c6d67736ab81f7446fc32f4301d9e4'
assert icon==(root/'fixtures/upstream/extensions/format-graphql/assets/command-icon.png').read_bytes()
assert 'node_modules/prettier' not in json.loads((root/'fixtures/own/package-lock.json').read_text())['packages']
if (root/'.scratch/host-harness/src').exists():
 for name in ['common.rs','extensions/native.rs','extensions/v2_sdk.rs','extensions/native_tests.rs']:
  assert (root/'.scratch/host-harness/src/native_core'/name).read_bytes()==(root/'fixtures/beta/src-tauri/src/native_core'/name).read_bytes(),name
 assert (root/'.scratch/host-harness/src/native_core/extensions/format_graphql.rs').read_bytes()==(root/'adapter/format_graphql.rs').read_bytes()
print(f'{len(seen)} evidence files: exact sizes/SHA256, original npm lock SRI, recovered useForm source, actual npm licenses, icon and unchanged harness sources verified.')

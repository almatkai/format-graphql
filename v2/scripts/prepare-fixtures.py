#!/usr/bin/env python3
"""Extract pinned test-only tooling offline. Does not install packages or run hooks."""
from pathlib import Path
import tarfile
root=Path(__file__).resolve().parents[1]
for name in ['prettier-2.5.1','typescript-5.9.3']:
 with tarfile.open(root/'fixtures/library'/f'{name}.tgz') as archive:
  for member in archive.getmembers():
   p=Path(member.name)
   if not member.isfile():continue
   if not p.parts or p.parts[0]!='package' or '..' in p.parts:raise ValueError('unsafe fixture path')
   dest=root/'.scratch/tooling'/name/p
   dest.parent.mkdir(parents=True,exist_ok=True)
   dest.write_bytes(archive.extractfile(member).read())
print('Pinned test-only tooling extracted offline')
for name, version in {'ajv':'8.17.1','fast-deep-equal':'3.1.3','fast-uri':'3.0.1','json-schema-traverse':'1.0.0','require-from-string':'2.0.2'}.items():
 with tarfile.open(root/'fixtures/library'/f'{name}-{version}.tgz') as archive:
  for member in archive.getmembers():
   p=Path(member.name)
   if not member.isfile():continue
   if not p.parts or p.parts[0]!='package' or '..' in p.parts:raise ValueError('unsafe fixture path')
   dest=root/'.scratch/tooling/node_modules'/name/Path(*p.parts[1:])
   dest.parent.mkdir(parents=True,exist_ok=True)
   dest.write_bytes(archive.extractfile(member).read())

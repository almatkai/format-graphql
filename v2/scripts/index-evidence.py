#!/usr/bin/env python3
"""Regenerate hashes/classification only when deliberately updating evidence."""
from pathlib import Path
import json,hashlib
root=Path(__file__).resolve().parents[1]
own='b1edb8f7837a148fef50fdf79fcfe917a7aed952';beta='fd300e276356587726566026f072563eb9c34cb1';upstream='6734d85992290066fd31a7f5bee3cca3d3545a1a'
prettier=json.loads((root/'fixtures/library/prettier-git-reference.json').read_text())['resolvedCommit']
head=(root/'integration/host-head.txt').read_text().strip();records=[]
for p in sorted([p for folder in ['fixtures','assets','integration/base'] for p in (root/folder).rglob('*') if p.is_file()]):
 rel=p.relative_to(root).as_posix()
 if rel=='fixtures/INDEX.json':continue
 kind='original';source='';origin=''
 if rel.startswith('fixtures/own/'):
  path=rel[len('fixtures/own/'):];source=f'git:{own}:{path}';origin=f'https://github.com/almatkai/format-graphql/blob/{own}/{path}'
 elif rel.startswith('fixtures/beta/'):
  path=rel[len('fixtures/beta/'):];source=f'local-host-git:{beta}:{path}';origin='read-only git show; host license fixture included'
 elif rel.startswith('fixtures/upstream/'):
  path=rel[len('fixtures/upstream/'):];source=f'raycast/extensions@{upstream}:{path}';origin=f'https://raw.githubusercontent.com/raycast/extensions/{upstream}/{path}'
 elif rel.startswith('integration/base/'):
  path=rel[len('integration/base/'):].removesuffix('.snapshot');source=f'local-host-git:{head}:{path}';origin='read-only git show; integration base only'
 elif rel=='assets/command-icon.png':
  path='extensions/format-graphql/assets/command-icon.png';source=f'raycast/extensions@{upstream}:{path}';origin=f'https://raw.githubusercontent.com/raycast/extensions/{upstream}/{path}'
 elif rel.startswith('fixtures/library/prettier-source-2.5.1/') or rel.startswith('fixtures/library/prettier-tests-2.5.1/'):
  path=rel.split('/',3)[3];source=f'prettier/prettier@{prettier}:{path}';origin=f'https://raw.githubusercontent.com/prettier/prettier/{prettier}/{path}'
 elif rel.startswith('fixtures/library/raycast-utils-source/'):
  kind='recovered-original';source='@raycast/utils@2.3.0 published source map';origin='fixtures/library/raycast-utils-2.3.0.tgz#package/dist/main.js.map sourcesContent: '+p.name
 elif rel.startswith('fixtures/licenses/npm/'):
  archive=rel.split('/')[3];source=archive+' published archive';origin=f'fixtures/library/{archive}.tgz#package/{p.name}'
 elif rel.startswith('fixtures/licenses/rust/'):
  package=rel.split('/')[3];name,version=package.rsplit('-',1);member='Cargo.toml' if p.name=='published-Cargo.toml.snapshot' else p.name;source=f'crates.io:{name}@{version}';origin=f'https://crates.io/api/v1/crates/{name}/{version}/download#{member}'
 elif rel=='fixtures/licenses/rust-dependencies.json':
  kind='generated-inventory';source='Cargo.lock + actual registry package metadata/license texts';origin='Cargo.lock; fixtures/licenses/rust/'
 elif p.suffix=='.tgz':
  if p.name.startswith('raycast-utils-'):name='@raycast/utils';version='2.3.0';url=f'https://registry.npmjs.org/@raycast/utils/-/utils-{version}.tgz'
  else:name,version=p.stem.rsplit('-',1);url=f'https://registry.npmjs.org/{name}/-/{name}-{version}.tgz'
  source=f'npm:{name}@{version}';origin=url
 elif p.name.endswith('-registry.json'):
  name={'raycast-utils-registry.json':'@raycast/utils/2.3.0','prettier-registry.json':'prettier/2.5.1','typescript-registry.json':'typescript/5.9.3'}[p.name];source='npm registry version metadata';origin='https://registry.npmjs.org/'+name
 elif p.name=='prettier-git-reference.json':source='GitHub git reference + resolved commit';origin='https://api.github.com/repos/prettier/prettier/git/ref/tags/2.5.1';kind='recorded-response'
 elif p.name in ['oracle.json','source-trace.json','representation-gap.json']:
  kind='generated-oracle';source='real pinned Prettier / executable source with explicit recording shims';origin={'oracle.json':'scripts/oracle.cjs','source-trace.json':'scripts/source-oracle.cjs','representation-gap.json':'scripts/representation-gap.cjs'}[p.name]
 else:raise ValueError('Unclassified evidence '+rel)
 data=p.read_bytes();records.append({'path':rel,'size':len(data),'sha256':hashlib.sha256(data).hexdigest(),'kind':kind,'source':source,'origin':origin})
(root/'fixtures/INDEX.json').write_text(json.dumps({'ownSource':own,'beta':beta,'integrationBase':head,'prettierCommit':prettier,'records':records},indent=2,ensure_ascii=False)+'\n')
print(len(records),'evidence records')

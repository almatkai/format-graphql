#!/usr/bin/env python3
"""Reproduce native/oracle checks; all effects use test recorders/temp stores."""
from pathlib import Path
import subprocess,json,time,datetime
root=Path(__file__).resolve().parents[1];logs=root/'.scratch/validation';logs.mkdir(parents=True,exist_ok=True);results=[]
def run(command):
 start=time.monotonic();result=subprocess.run(command,cwd=root,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
 path=logs/f'{len(results):02d}.log';path.write_text(result.stdout)
 results.append({'command':command,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,2),'log':path.relative_to(root).as_posix()})
 print(('PASS' if result.returncode==0 else 'FAIL')+' '+' '.join(command),flush=True)
 if result.returncode:print(result.stdout);raise SystemExit(result.returncode)
for command in [['python3','scripts/prepare-fixtures.py'],['python3','scripts/prepare-harness.py']]:run(command)
for script in ['oracle.cjs','source-oracle.cjs','representation-gap.cjs']:run(['node','scripts/'+script,'--check'])
for script in ['renderer-tests.cjs','schema-tests.cjs']:run(['node','scripts/'+script])
run(['python3','scripts/verify-evidence.py'])
for toolchain in [None,'+1.85.0']:
 cmd=['cargo']+([toolchain] if toolchain else [])
 run(cmd+['fmt','--','--check'])
 for manifest in ['Cargo.toml','.scratch/host-harness/Cargo.toml']:
  run(cmd+['test','--manifest-path',manifest,'--locked','--offline'])
  run(cmd+['clippy','--manifest-path',manifest,'--locked','--offline','--all-targets','--','-D','warnings'])
  run(cmd+['check','--manifest-path',manifest,'--locked','--offline','--all-targets','--target','x86_64-pc-windows-gnu'])
  run(cmd+['clippy','--manifest-path',manifest,'--locked','--offline','--all-targets','--target','x86_64-pc-windows-gnu','--','-D','warnings'])
run(['rustfmt','--check','--edition','2021','--config','skip_children=true','adapter/format_graphql.rs','tests/harness/lib.rs','tests/harness/runtime.rs'])
run(['rustfmt','+1.85.0','--check','--edition','2021','--config','skip_children=true','adapter/format_graphql.rs','tests/harness/lib.rs','tests/harness/runtime.rs'])
(root/'validation-results.json').write_text(json.dumps({'recordedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'rustStable':subprocess.check_output(['rustc','--version'],text=True).strip(),'scope':'Automated isolated checks only. Real host/OS/UI verification not-run. Logs are disposable and excluded from archive.','results':results},indent=2)+'\n')
print('All automated checks passed. Windows checks are cross-compilation, not execution.')

// Actual pinned beta FormRuntime/Surface code, transpiled; explicit hook/UI stubs.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),ts=require(path.join(root,'.scratch/tooling/typescript-5.9.3/package/lib/typescript.js'));
const jsx={jsx:(type,props)=>({type,props}),jsxs:(type,props)=>({type,props})};
function hooks(){let state=[],i=0,effects=[];const equal=(a,b)=>a&&b&&a.length===b.length&&a.every((x,i)=>Object.is(x,b[i]));const react={
 useState(v){const n=i++;if(!(n in state))state[n]=typeof v==='function'?v():v;return [state[n],v=>{if(typeof v==='function'){const a=v(state[n]),b=v(state[n]);assert.deepEqual(a,b,'State updater must be pure when React calls it twice');state[n]=a;}else state[n]=v;}];},
 useRef(v){const n=i++;if(!(n in state))state[n]={current:v};return state[n];},
 useMemo(f,deps){const n=i++;if(!state[n]||!equal(state[n].deps,deps))state[n]={deps,value:f()};return state[n].value;},
 useCallback(f,deps){return react.useMemo(()=>f,deps);},
 useEffect(f,deps){const n=i++;if(!state[n]||!equal(state[n].deps,deps)){state[n]={deps};effects.push(f);}},
 useImperativeHandle(ref,f,deps){react.useMemo(()=>{ref.current=f();},deps);},forwardRef:f=>f,
};return {react,render(f){i=0;effects=[];return f();},flush(){for(const e of effects)e();}};}

function compile(file,imports){const code=ts.transpileModule(fs.readFileSync(file,'utf8'),{fileName:file,compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true}}).outputText;const module={exports:{}};new Function('require','exports','module',code)(k=>{if(!(k in imports))throw Error('Unstubbed import '+k);return imports[k];},module.exports,module);return module.exports;}
function walk(node,predicate){if(!node||typeof node!=='object')return null;if(predicate(node))return node;for(const child of [node.props?.children].flat(Infinity)){const found=walk(child,predicate);if(found)return found;}return null;}
function form(patched){
 const h=hooks(),file=path.join(root,patched?'integration/renderer/form-runtime.tsx':'fixtures/beta/src/renderer/src/raycast-api/form-runtime.tsx');
 const m=compile(file,{'react':h.react,'react/jsx-runtime':jsx,'../../ui/primitives':{Hint:'Hint',HintBar:'HintBar',Kbd:'Kbd',ViewHeader:'ViewHeader'},'./detail-runtime':{parseNativeColorWheel:()=>null,NativeColorWheel:'NativeColorWheel'}});
 const ref={current:null},changes=[];let currentRoot={type:'Form',children:[{type:'Form.TextArea',props:{id:'input',title:'Input',value:''}},{type:'Form.Dropdown',props:{id:'indent',title:'Indentation',value:'tab',actionId:'indent-callback'},children:['tab','2','4','8'].map(value=>({type:'Form.Dropdown.Item',props:{value,title:value}}))}]};
 const props=()=>({root:currentRoot,title:'Format GraphQL',onBack:()=>{},onSubmitForm:()=>{},onChangeField:(...args)=>changes.push(args),onInvokeAction:()=>{},onOpenActions:()=>{}});
 function render(){let tree=h.render(()=>m.FormRuntime(props(),ref));h.flush();tree=h.render(()=>m.FormRuntime(props(),ref));return tree;}
 let tree=render();const select=walk(tree,n=>n.type==='select');assert.equal(select.props.value,'tab');assert.equal(select.props.children.flat(Infinity).filter(Boolean).length,5); // beta adds Select
 walk(tree,n=>n.type==='textarea').props.onChange({target:{value:'{a}'}});select.props.onChange({target:{value:'4'}});tree=render();assert.deepEqual(changes,[['indent-callback','4']]);assert.deepEqual(ref.current.getValues(),{input:'{a}',indent:'4'});
 // Server omits controlled values on an unrelated callback/heartbeat.
 currentRoot=structuredClone(currentRoot);delete currentRoot.children[0].props.value;delete currentRoot.children[1].props.value;render();assert.deepEqual(ref.current.getValues(),{input:'{a}',indent:'4'});
 // A delayed response to the previous submit must not replace a newer draft.
 walk(tree,n=>n.type==='textarea').props.onChange({target:{value:'{newer}'}});render();currentRoot=structuredClone(currentRoot);Object.assign(currentRoot.children[0].props,{value:'{\n  a\n}\n',replacementBase:'{a}',replacementId:'delayed-format-1'});render();assert.equal(ref.current.getValues().input,patched?'{newer}':'{\n  a\n}\n');
 // A rejected identity stays consumed even after editing back to its base.
 tree=render();walk(tree,n=>n.type==='textarea').props.onChange({target:{value:'{a}'}});render();currentRoot=structuredClone(currentRoot);render();assert.equal(ref.current.getValues().input,'{a}');
 if(patched){
  // Only a NEW explicit format request can apply identical base/output bytes.
  currentRoot=structuredClone(currentRoot);currentRoot.children[0].props.replacementId='explicit-format-2';render();assert.equal(ref.current.getValues().input,'{\n  a\n}\n');
  tree=render();walk(tree,n=>n.type==='textarea').props.onChange({target:{value:'{a}'}});render();currentRoot=structuredClone(currentRoot);render();assert.equal(ref.current.getValues().input,'{a}','Applied identity must not replay');
 }
 // Beta parsing derives a fallback id; it does not implement storeValue.
 const fields=[{key:'input:Form.TextArea',id:'input',type:'Form.TextArea',value:null},{key:'Form.Dropdown:1:Form.Dropdown',id:'Form.Dropdown:1',type:'Form.Dropdown',value:undefined}];assert.deepEqual(m.reconcileFormValues(fields),{input:null,'Form.Dropdown:1':''});
 return m.FormRuntime;
}
function surface(patched,FormRuntime){
 const h=hooks();const calls=[];const file=path.join(root,patched?'integration/renderer/index.tsx':'fixtures/beta/src/renderer/src/raycast-api/index.tsx');const imports={'react':h.react,'react/jsx-runtime':jsx,'./action-runtime':{Action:'Action',ActionPanel:'ActionPanel',ActionPanelOverlay:'Overlay'},'./action-runtime-registry':{ActionRegistryContext:{Provider:'Registry'}},'./form-runtime':{FormRuntime},'./detail-runtime':{DetailRuntime:'Detail'},'./grid-runtime':{GridRuntime:'Grid'},'./list-runtime':{ListRuntime:'List'},'./hooks':{NavigationContext:{Provider:'Navigation'},setRuntimeContext:()=>{}}};const m=compile(file,imports);
 const props={sessionId:'fixture',title:'Format GraphQL',extensionId:'com.tezbar.native.format-graphql',commandName:'index',root:{type:'Form'},actions:[{id:'copy',kind:'submit-form'},{id:'only',kind:'submit-form'}],onBack:()=>{},onSearchTextChanged:()=>{},onLoadMore:()=>{},onInvokeAction:(...args)=>calls.push(args)};
 let tree=h.render(()=>m.ExtensionRuntimeSurface(props));let child=walk(tree,n=>n.type===FormRuntime);const values={input:'{draft}',indent:'8'};child.props.ref.current={getValues:()=>values};child.props.onSubmitForm(values);assert.deepEqual(calls.pop(),['copy',values]);child.props.onChangeField('indent-callback','2');assert.deepEqual(calls.pop(),['indent-callback',{value:'2'}]);
 child.props.onOpenActions();tree=h.render(()=>m.ExtensionRuntimeSurface(props));const overlay=walk(tree,n=>n.type==='Overlay');overlay.props.onExecute(props.actions[1]);assert.deepEqual(calls.pop(),patched?['only',values]:['only']);
}
// Produce fresh response objects through the real adapter, SDK and isolated store.
// This also asserts committed identity and cancellation/retry state in Rust.
function adapterTrace(){
 const cp=require('node:child_process');
 cp.execFileSync('python3',['scripts/prepare-harness.py'],{cwd:root,stdio:'pipe'});
 const file=path.join(root,'.scratch/renderer-adapter-trace.json');
 cp.execFileSync('cargo',['test','--manifest-path','.scratch/host-harness/Cargo.toml','--locked','--offline','--test','runtime','replacement_identity_trace','--','--exact'],{cwd:root,env:{...process.env,FORMAT_GRAPHQL_RENDERER_TRACE:file},stdio:'pipe'});
 return JSON.parse(fs.readFileSync(file,'utf8'));
}
function replacementContract(trace){
 function mount(){
  const h=hooks(),ref={current:null};let currentRoot=trace.initial.root;
  const m=compile(path.join(root,'integration/renderer/form-runtime.tsx'),{'react':h.react,'react/jsx-runtime':jsx,'../../ui/primitives':{Hint:'Hint',HintBar:'HintBar',Kbd:'Kbd',ViewHeader:'ViewHeader'},'./detail-runtime':{parseNativeColorWheel:()=>null,NativeColorWheel:'NativeColorWheel'}});
  const props=()=>({root:currentRoot,title:'Format GraphQL',onBack:()=>{},onSubmitForm:()=>{},onChangeField:()=>{},onInvokeAction:()=>{},onOpenActions:()=>{}});
  function render(){let t=h.render(()=>m.FormRuntime(props(),ref));h.flush();return h.render(()=>m.FormRuntime(props(),ref));}
  render();
  return {
   receive(response,clone=false){currentRoot=clone?structuredClone(response.root):response.root;render();},
   edit(input){walk(render(),n=>n.type==='textarea').props.onChange({target:{value:input}});render();},
   value(){return ref.current.getValues().input;},
   render,
  };
 }
 const output=trace.first.root.children[0].props.value;
 for(const rejected of [false,true]){
  const f=mount();f.edit(rejected?'{newer}':'{a}');f.receive(trace.first);
  assert.equal(f.value(),rejected?'{newer}':output,'First observation must guard the submitted base');
  f.edit('{a}');
  // Same root, cloned roots, repeated heartbeat, and an omitted-value callback.
  for(const response of [trace.first,trace.refresh,trace.indent,trace.indentRefresh,trace.first]){
   for(const clone of [false,true]){f.receive(response,clone);assert.equal(f.value(),'{a}','Applied/rejected old ID must stay consumed across refresh/indent');}
  }
  // Real Format and Copy is a NEW operation with the same bytes and a new ID.
  f.edit(rejected?'{newer-copy}':'{a}');f.receive(trace.copy);
  assert.equal(f.value(),rejected?'{newer-copy}':output);
  f.edit('{a}');
  for(const response of [trace.copy,trace.copyRefresh,trace.retry,trace.retryRefresh,trace.indent,trace.first]){
   for(const clone of [false,true]){f.receive(response,clone);assert.equal(f.value(),'{a}','Copy completion/retry/omission/older IDs must not re-arm');}
  }
  f.receive(trace.second,true);assert.equal(f.value(),output,'New explicit Only Format applies identical base/output with new ID');
  f.edit('{a}');
  for(const response of [trace.second,trace.first,trace.retryRefresh,trace.second]){
   f.receive(response,true);assert.equal(f.value(),'{a}','All observed identities stay consumed, not only the last one');
  }
  // Cancellation checkpoints output/ID. First observation is still base guarded.
  const c=mount();c.edit(rejected?'{newer-cancel}':'{a}');c.receive(trace.cancelledRefresh);
  assert.equal(c.value(),rejected?'{newer-cancel}':output);
  c.edit('{a}');
  for(const response of [trace.cancelledRefresh,trace.cancelledRetry,trace.cancelledRetryRefresh]){
   for(const clone of [false,true]){c.receive(response,clone);assert.equal(c.value(),'{a}','Cancellation/retry/refresh must retain a consumed identity');}
  }
 }
}
function legacyReconciliation(){
 const modules=[false,true].map(patched=>compile(path.join(root,patched?'integration/renderer/form-runtime.tsx':'fixtures/beta/src/renderer/src/raycast-api/form-runtime.tsx'),{'react':hooks().react,'react/jsx-runtime':jsx,'../../ui/primitives':{},'./detail-runtime':{}}));
 for(const type of ['Form.TextArea','Form.Checkbox','Form.FilePicker','Form.TagPicker']){
  for(const value of [undefined,null,'','same','changed',false,[]]){
   for(const actionId of [undefined,'controlled']){
    const field={key:'x:'+type,id:'x',type,value,actionId};
    const args=[[field],{x:'draft'},[{...field,value:'same'}]];
    assert.deepEqual(modules[1].reconcileFormValues(...args),modules[0].reconcileFormValues(...args),'Unguarded extensions retain beta reconciliation');
   }
  }
 }
}
global.window={addEventListener(){},removeEventListener(){},clearTimeout(){},tezbar:{setSuppressBlurHide:async()=>{}}};
for(const patched of [false,true]){const f=form(patched);surface(patched,f);}
replacementContract(adapterTrace());legacyReconciliation();
console.log('Pinned beta + patched renderer PASS: applied/rejected IDs consumed once across unchanged/cloned refresh, indent, copy/retry and cancellation; real adapter trace + new identical-byte format; legacy reconciliation, pure repeated updaters, both complete submits. Hook/UI imports are stubs; no OS/UI QA.');

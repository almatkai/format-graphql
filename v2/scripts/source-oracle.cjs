// Executes the immutable own TSX and actual useForm sources with hook/API recorders.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
const ts=require(path.join(root,'.scratch/tooling/typescript-5.9.3/package/lib/typescript.js'));
const prettier=require(path.join(root,'.scratch/tooling/prettier-2.5.1/package'));
function compile(source,file,imports){const code=ts.transpileModule(source,{fileName:file,compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true}}).outputText;const module={exports:{}};new Function('require','module','exports',code)((key)=>{if(!(key in imports))throw Error('Unstubbed import '+key);return imports[key];},module,module.exports);return module.exports;}
function hooks(){let values=[],at=0,effects=[];const react={
 useState(initial){const i=at++;if(!(i in values))values[i]=typeof initial==='function'?initial():initial;return [values[i],v=>{values[i]=typeof v==='function'?v(values[i]):v;}];},
 useRef(initial){const i=at++;if(!(i in values))values[i]={current:initial};return values[i];},
 useCallback(fn){at++;return fn;},useMemo(fn){at++;return fn();},
 useEffect(fn){at++;effects.push(fn);},useImperativeHandle(ref,fn){at++;if(ref)ref.current=fn();},forwardRef:fn=>fn,
};return {react,render(fn){at=0;effects=[];const out=fn();return out;},effects(){for(const effect of effects)effect();}};}
const jsx={jsx:(type,props)=>({type,props}),jsxs:(type,props)=>({type,props})};
const trace=[];
async function run(useBeta){
 const h=hooks();let rejectCopy=false;const calls=[];
 const api={Icon:{Clipboard:'Clipboard',Checkmark:'Checkmark'},Form:Object.assign('Form',{TextArea:'Form.TextArea',Dropdown:Object.assign('Form.Dropdown',{Item:'Form.Dropdown.Item'})}),ActionPanel:'ActionPanel',Action:{SubmitForm:'Action.SubmitForm'},Toast:{Style:{Animated:'animated',Success:'success',Failure:'failure'}},
 showToast:async opts=>{calls.push(['toast',structuredClone(opts)]);return new Proxy(opts,{set(obj,key,value){calls.push(['toast-update',key,value]);obj[key]=value;return true;}});},Clipboard:{copy:async text=>{calls.push(['copy',text]);if(rejectCopy)throw Error('copy failure');}},showHUD:async text=>calls.push(['hud',text]),popToRoot:async()=>calls.push(['popToRoot'])};
 const actualFormat=prettier.format.bind(prettier);const library={format:(input,options)=>{calls.push(['format',input,options]);return actualFormat(input,options);}};
 let utils;
 if(useBeta){
  const source=fs.readFileSync(path.join(root,'fixtures/beta/src/main/extension-runner.ts'),'utf8');const tree=ts.createSourceFile('runner.ts',source,ts.ScriptTarget.Latest,true);
  let node;function walk(n){if(ts.isVariableDeclaration(n)&&n.name.getText(tree)==='useForm'&&n.initializer&&ts.isArrowFunction(n.initializer))node=n;ts.forEachChild(n,walk);}walk(tree);assert(node);
  // Exact initializer, with explicit cache/session/React shims around it.
  const source2='const {useState}=require("react");const cache={remove:()=>{}};const session={commandName:"index"};const FormValidation={Required:"required"};exports.FormValidation=FormValidation;exports.useForm='+node.initializer.getText(tree)+';';
  utils=compile(source2,'useForm-beta.ts',{'react':h.react});
 }else{
  const latest=compile(fs.readFileSync(path.join(root,'fixtures/library/raycast-utils-source/useLatest.ts'),'utf8'),'useLatest.ts',{'react':h.react});
  utils=compile(fs.readFileSync(path.join(root,'fixtures/library/raycast-utils-source/useForm.tsx'),'utf8'),'useForm.tsx',{'react':h.react,'./useLatest':latest,'@raycast/api':api});
 }
 const originalUseForm=utils.useForm;
 utils.useForm=options=>originalUseForm({...options,onSubmit:values=>{calls.push(['submit',structuredClone(values)]);return options.onSubmit(values);}});
 const mod=compile(fs.readFileSync(path.join(root,'fixtures/own/src/index.tsx'),'utf8'),'index.tsx',{'react/jsx-runtime':jsx,'@raycast/api':api,'@raycast/utils':utils,'prettier':library});
 let tree=h.render(mod.default);const fields=()=>tree.props.children;const actions=()=>tree.props.actions.props.children;
 assert.deepEqual(actions().map(a=>a.props.title),['Format and Copy','Only Format']);
 assert.equal(fields()[0].props.id,'input');assert.equal(fields()[1].props.storeValue,true);
 if(!useBeta){assert.equal(fields()[0].props.value,null);assert.equal(fields()[1].props.id,'indent');assert.equal(fields()[1].props.value,null);}else{assert.equal(fields()[1].props.id,undefined);}
 const defaults={input:fields()[0].props.value??null,indent:fields()[1].props.value??null,indentId:fields()[1].props.id??null,storeValue:fields()[1].props.storeValue};
 assert.equal(await actions()[0].props.onSubmit({input:'',indent:'tab'}),false);assert.equal(calls.length,0);
 let before=calls.length;await actions()[1].props.onSubmit({input:'{f(a:1)}',indent:'4',extra:'kept'});assert.deepEqual(calls.slice(before).map(c=>c[0]),['submit','toast','format','toast-update','toast-update']);
 assert.deepEqual(calls[before][1],{input:'{f(a:1)}',indent:'4',extra:'kept',copy:false});
 tree=h.render(mod.default);assert.equal(fields()[0].props.value,'{\n    f(a: 1)\n}\n');
 const output=fields()[0].props.value;before=calls.length;await actions()[0].props.onSubmit({input:output,indent:'tab'});assert.deepEqual(calls.slice(before).map(c=>c[0]),['submit','toast','format','toast-update','toast-update','copy','hud','popToRoot']);
 tree=h.render(mod.default);const last=fields()[0].props.value;
 before=calls.length;await actions()[0].props.onSubmit({input:'{broken',indent:'2'});assert.deepEqual(calls.slice(before).map(c=>c[0]),['submit','toast','format','toast-update','toast-update']);tree=h.render(mod.default);assert.equal(fields()[0].props.value,last);
 // Input onChange supplies the user's draft; invalid submissions do not replace it.
 fields()[0].props.onChange('{draft invalid');tree=h.render(mod.default);await actions()[1].props.onSubmit({input:'{draft invalid',indent:'8'});tree=h.render(mod.default);assert.equal(fields()[0].props.value,'{draft invalid');
 rejectCopy=true;await assert.rejects(actions()[0].props.onSubmit({input:'{a}',indent:'2'}),/copy failure/);tree=h.render(mod.default);assert.equal(fields()[0].props.value,'{\n  a\n}\n');
 // Actual dropdown storage is a Raycast component responsibility, absent from beta.
 if(!useBeta){fields()[1].props.onChange('8');tree=h.render(mod.default);assert.equal(fields()[1].props.value,'8');}
 const remount=hooks(); // independent render state is exercised in each run above.
 assert(remount);
 trace.push({runtime:useBeta?'pinned beta useForm shim':'own locked @raycast/utils 2.3.0 source',defaults,calls});
}
(async()=>{await run(false);await run(true);const encoded=JSON.stringify(trace,null,2)+'\n';const out=path.join(root,'fixtures/source-trace.json');if(process.argv.includes('--check'))assert.equal(fs.readFileSync(out,'utf8'),encoded);else fs.writeFileSync(out,encoded);console.log('Own TSX + locked useForm + pinned beta useForm execution passed; API calls were recorders.');})();

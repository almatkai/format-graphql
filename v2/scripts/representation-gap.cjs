const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');const root=path.resolve(__dirname,'..');const p=require(path.join(root,'.scratch/tooling/prettier-2.5.1/package'));
const rows=['{f(a:"\\uD800")}','{f(a:"\\uDFFF")}'].map(input=>({input,oracleOutputUtf16:Array.from(p.format(input,{parser:'graphql'}),c=>c.charCodeAt(0)),nativeError:'Invalid GraphQL string'}));
const data=JSON.stringify({note:'JS accepts isolated UTF-16 surrogates. Rust requires Unicode scalar strings and rejects explicitly.',rows},null,2)+'\n';const file=path.join(root,'fixtures/representation-gap.json');if(process.argv.includes('--check'))assert.equal(fs.readFileSync(file,'utf8'),data);else fs.writeFileSync(file,data);
console.log('Actual JS UTF-16 representation gap reproduced.');

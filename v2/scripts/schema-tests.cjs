const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..');
const Ajv2020=require(path.join(root,'.scratch/tooling/node_modules/ajv/dist/2020.js')).default;
const schema=JSON.parse(fs.readFileSync(path.join(root,'fixtures/beta/schemas/extension-v2.schema.json')));
const manifest=JSON.parse(fs.readFileSync(path.join(root,'manifest.json')));
const ajv=new Ajv2020({allErrors:true,strict:false}),validate=ajv.compile(schema);
assert(validate(manifest),JSON.stringify(validate.errors));
for(const change of [m=>m.schemaVersion=1,m=>m.runtime='javascript',m=>m.author='unsupported',m=>m.id='raycast.format-graphql',m=>m.icon='/wrong/icon.png',m=>m.commands=[],m=>m.commands[0].platforms=['macos'],m=>m.verification[0].state='verified',m=>m.replaces=[],m=>m.verification=[]]){const bad=structuredClone(manifest);change(bad);assert.equal(validate(bad),false);}
assert.equal(manifest.commands.length,1);assert.equal(manifest.commands[0].name,'index');assert.deepEqual(manifest.verification.map(v=>v.state),['not-run','not-run']);
console.log('Full Draft 2020-12 validation and 10 negative cases passed.');

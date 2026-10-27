// Reproducible offline corpus: expected bytes come exclusively from pinned Prettier.
const fs=require('node:fs'),path=require('node:path');
const root=path.resolve(__dirname,'..');
const prettier=require(path.join(root,'.scratch/tooling/prettier-2.5.1/package'));
const cases=[
'', '  \n', '# comment', '{a}', 'query {a}', 'query ($x:Int=1){f(x:$x)}',
'query Foo($id: ID!, $values:[Int!]=[1,2,3]) @a @b(arg:1) { user(id:$id){ name age } }',
'mutation M($x:Input={x:1,y:[true,false,null]}){alias:create(input:$x){...Frag ... on User @skip(if:false){id} ... @a { name }}} fragment Frag on User { id }',
'subscription S{event{name}}',
'schema @a {query: Query mutation: Mutation subscription: Subscription}',
'"schema doc" schema { query: Query }',
'"doc" scalar Date @specifiedBy(url:"https://example.test")',
'"doc" type User implements & Node & Named @a { "id doc" id: ID! name( "arg" first:Int = 4 @a):String @b }',
'interface A implements Node & B @a {id:ID!}',
'input Filter @a { "hello" field: [String!]! = ["x","y"] @a }',
'enum E @a { "hi" A @b B C }',
'union U @a = | A | B | C',
'directive @custom("desc" a:Int=1,b:[String]) repeatable on | FIELD | QUERY | FIELD_DEFINITION',
'extend type User implements Node @a {name:String}',
'extend interface Node @a {id:ID!}', 'extend scalar Date @a', 'extend enum E @a { D }',
'extend input I @a { x:Int }','extend union U @a = A | B','extend schema @a','extend schema { query: Query }',
'{f(a: {x:1,y:2},b:[1,2,3],c:1e+10,d:-0.01,e:true,f:null)}',
'{f(a:"\\u263a\\uD83D\\uDE00\\n\\\\\\\"\\/")}',
'{f(a:"\\t\\b\\f\\r")}',
'"""One line description""" type T { """line 1\n  line 2""" f(a:String="""hi"""):String }',
'{f(a:"""\n  first\n    second\n  third\n""")}',
'{f(a:"""triple \\""" quote""")}',
'# leading\nquery X { # opening\n a # trailing\n\n b\n # before\n c\n} # end\n# final',
'{a\n\n b\n\n\n c}', 'type A { x:Int }\n\n type B { y:Int }',
'query X(\n# var\n$x: Int # after\n$y: String\n){f(\n# arg\na:$x\n b:$y\n)}',
'{f(a:[\n# in list\n1,2 # after\n3\n],b:{\n# object\nx:1,y:2\n})}',
'# prettier-ignore\nquery Foo{a,b(c:1,d:2)}\nquery Q { a }',
'{\n# prettier-ignore\nf( a:1, b:2 ){a,b}\nother\n}',
'query X # name\n{ a }','type A implements A # interface\n & B { a:Int }',
'query X @veryLongDirective(argument: "'+ 'a'.repeat(80)+'") @b { f }',
'union ReallyLongUnionName = '+Array.from({length:8},(_,i)=>'VeryLongMemberName'+i).join(' | '),
'{ f('+Array.from({length:12},(_,i)=>`argument${i}:${i}`).join(',')+') }',
'query ReallyLongName('+Array.from({length:8},(_,i)=>`$variable${i}:String="default${i}"`).join(',')+'){f}',
'query X{ f(a:{'+Array.from({length:9},(_,i)=>`field${i}:"value${i}"`).join(',')+'}) }',
'{', '{}', 'query', 'type A {}','{ f() }','{f(a:01)}','{f(a:1.)}','{f(a:"\\x")}','{f(a:$x)} trailing',
'fragment F on T {x}', 'type A', 'input I', 'enum E', 'union U',
'\ufeffquery Q\r\n{ a, b }\r\n',
];
for(let n=1;n<=20;n++)for(const shape of [
`{f(a:[${Array.from({length:n},(_,i)=>'"'+'x'.repeat(i+1)+'"').join(',')}])}`,
`{field${n}(a:{x:${n},y:[1,2]},b:"${'a'.repeat(n*4)}"){a b}}`,
`query Q($x:Int=${n}) @a {f(a:$x) {a\n\n b}}`,
`{f(a:"""\n${' '.repeat(n%5)}first\n${' '.repeat(n%5+2)}second\n""")}`,
])cases.push(shape);
const upstreamRoot=path.join(root,'fixtures/library/prettier-tests-2.5.1/tests/format/graphql');
for (const dir of fs.readdirSync(upstreamRoot).sort())for(const file of fs.readdirSync(path.join(upstreamRoot,dir)).sort())if(file.endsWith('.graphql'))cases.push(fs.readFileSync(path.join(upstreamRoot,dir,file),'utf8'));
for (const value of ['日本語😀'.repeat(6), '👩‍💻'.repeat(12), 'é'.repeat(30)])cases.push('{f(a:"'+value+'",b:1)}');
for(const input of ['{f(a:"x\ty")}', '{f(a:"\\t")}', '{f(a:"x\by")}', '{f(a:"""x\u0000y""")}'])cases.push(input);
const rows=[];
for (const input of cases)for(const indent of ['tab','2','4','8']){
 const options={parser:'graphql',useTabs:indent==='tab',tabWidth:indent==='tab'?2:parseInt(indent)};
 try{const output=prettier.format(input,options);let idempotent;try{idempotent=prettier.format(output,options)===output;}catch{idempotent=false;}rows.push({input,indent,output,idempotent});}
 catch(e){rows.push({input,indent,error:true,oracleError:e.message});}
}
const encoded=JSON.stringify({baseline:'Prettier 2.5.1, upstream lock; own lock missing dependency',rows},null,2)+'\n';
const out=path.join(root,'fixtures/oracle.json');
if(process.argv.includes('--check')){if(fs.readFileSync(out,'utf8')!==encoded)throw Error('Oracle fixture drift');}else fs.writeFileSync(out,encoded);
console.log(`${rows.length} pinned Prettier comparisons`);

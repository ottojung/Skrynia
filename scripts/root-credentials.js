#!/usr/bin/env node
'use strict';
// Local operator-only access to the root issuer credential.
const fs=require('node:fs');
const path=require('node:path');
const root=process.env.SKRYNIA_DATA_DIR||'/var/lib/skrynia';
const dir=path.join(root,'state','grants');
const entries=fs.readdirSync(dir).filter(x=>/^[a-f0-9]{32}\.json$/.test(x));
const records=entries.map(name=>JSON.parse(fs.readFileSync(path.join(dir,name),'utf8')));
const roots=records.filter(x=>x.parent===null);
if(roots.length!==1||typeof roots[0].rootIssuerToken!=='string') {
  throw new Error('expected one initialized root grant');
}
process.stdout.write(JSON.stringify({id:roots[0].id,
  issuerToken:roots[0].rootIssuerToken})+'\n');

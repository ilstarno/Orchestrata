import {createEngine} from './try-engine.mjs';
import {compile,describe,report} from './promise-check-model.mjs';
let resources;
self.onmessage=async({data})=>{try{
 resources??=Promise.all([fetch(new URL('../solutions/field-service/dispatch/input.json',import.meta.url)).then(r=>{if(!r.ok)throw Error();return r.json()}),fetch(new URL('./try-engine.wasm',import.meta.url)).then(r=>{if(!r.ok)throw Error();return r.arrayBuffer()}).then(createEngine)]);
 const[source,engine]=await resources;
 const base=compile(source,{overtime:0}),model=compile(source,data.policy);
 const baseline=describe(base,engine.solveProblem(base.problem,4096)),current=describe(model,engine.solveProblem(model.problem,4096));
 self.postMessage({id:data.id,report:report(source,baseline,current,model.policy)});
 }catch{self.postMessage({id:data.id,error:'The sample could not establish a verified plan. No result is available to download.'})}};

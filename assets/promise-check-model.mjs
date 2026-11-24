export const VERSION = 'promise-check-v1';
export const LOCK = Object.freeze({job:'safety_inspection',technician:1,start:545});
export const LIMITS = 'One simultaneous work wave; one job per technician; fixed depot travel estimates; no multi-stop routing, weekly schedule, live traffic, uploads or write-back.';
export function compile(source,{absent=false,urgent=false,overtime=0}={}){
 if(typeof absent!=='boolean'||typeof urgent!=='boolean'||![0,20].includes(overtime))throw Error('Unsupported sample policy');
 const jobs=source.jobs.filter(j=>urgent||j.id!=='urgent_isolation');
 const pairs=jobs.map(j=>source.technicians.map(t=>{const travel=j.travelMinutes[t.id]*2,start=Math.max(t.ready+travel/2,j.earliest),end=start+j.serviceMinutes+travel/2,ot=Math.max(0,end-t.shiftEnd),reasons=[];
 if(absent&&t.id===0)reasons.push('Ana is unavailable');
 if(!t.skills.includes(j.skill))reasons.push('Required skill unavailable');
 if(start>j.latest)reasons.push('Misses latest start');
 if(ot>overtime)reasons.push('Exceeds selected overtime allowance');
 if(j.id===LOCK.job&&(t.id!==LOCK.technician||start!==LOCK.start))reasons.push('Confirmed appointment is locked');
 return {technician:t.name,technicianId:t.id,start,end,travel,overtime:ot,reasons};}));
 const domains=pairs.map((ps,i)=>[...ps.filter(p=>!p.reasons.length).map(p=>p.technicianId),...(jobs[i].id===LOCK.job?[]:[100+i])]);
 if(domains.some(d=>!d.length))throw Error('Locked commitment cannot be met under these rules');
 const constraints=[{kind:'all_different',variables:jobs.map((_,i)=>i),hard_weight:1}];
 domains.forEach((d,i)=>d.filter(v=>v<100).forEach(v=>constraints.push({kind:'prefer_value',variable:i,value:v,soft_weight:jobs[i].unservedPenalty-pairs[i][v].travel-2*pairs[i][v].overtime})));
 return {jobs,pairs,domains,policy:{absent,urgent,overtime},problem:{variables:jobs.map((j,i)=>({name:j.id,domain:domains[i]})),constraints}};
}
export function describe(model,result){
 const s=result.solution??result;const vs=s?.values;
 if(!Array.isArray(vs)||vs.length!==model.jobs.length||s.score?.hard!==0)throw Error('No verified feasible result');
 const used=new Set();let soft=0;
 vs.forEach((v,i)=>{if(!model.domains[i].includes(v)||used.has(v))throw Error('Invalid assignment');used.add(v)});
 for(const c of model.problem.constraints.slice(1))if(vs[c.variable]!==c.value)soft-=c.soft_weight;
 if(s.score.soft!==soft)throw Error('Result score mismatch');
 const rows=vs.map((v,i)=>{const j=model.jobs[i],p=v<100?model.pairs[i][v]:null;return {id:j.id,job:j.name,locked:j.id===LOCK.job,person:p?.technician??null,start:p?.start??null,returnAt:p?.end??null,travelMinutes:p?.travel??0,overtimeMinutes:p?.overtime??0,holdPenalty:p?0:j.unservedPenalty,reason:p?(j.id===LOCK.job?'Confirmed appointment preserved':'Eligible within selected policy'):model.pairs[i].some(x=>!x.reasons.length)?'Eligible people are assigned elsewhere; held under the stated cost model':'No eligible person fits the selected policy',eligibility:model.pairs[i].map(x=>({person:x.technician,reasons:x.reasons}))}});
 const travel=rows.reduce((n,r)=>n+r.travelMinutes,0),ot=rows.reduce((n,r)=>n+r.overtimeMinutes,0),hold=rows.reduce((n,r)=>n+r.holdPenalty,0);
 return {rows,assigned:rows.filter(r=>r.person).length,requested:rows.length,travelMinutes:travel,overtimeMinutes:ot,holdPenalty:hold,modeledCost:travel+2*ot+hold,evaluated:s.evaluated,score:s.score};
}
export function report(source,baseline,current,policy){return {version:VERSION,classification:'Synthetic sample; actual browser WASM solve',policy,lockedCommitment:LOCK,limits:LIMITS,costDefinition:'Fixed round-trip travel minutes + 2 × overtime minutes + held-job penalties. Model units, not currency or measured savings.',heldJobPenalties:Object.fromEntries(source.jobs.map(j=>[j.id,j.unservedPenalty])),baseline,current,changes:current.rows.map(r=>{const b=baseline.rows.find(x=>x.id===r.id);return {job:r.job,before:b?.person??null,after:r.person,status:!b?'Added':!r.person?'Unassigned':b.person!==r.person?'Moved':'Unchanged'}}),input:source};}

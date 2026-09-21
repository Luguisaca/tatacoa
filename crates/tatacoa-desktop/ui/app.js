const invoke=window.__TAURI__.core.invoke;
let activeEngagement=null,activeExecution=null,activeProfile=null,pendingRun=null,pendingResume=null;
const byId=id=>document.getElementById(id),workspace=()=>byId('workspace').value.trim(),lines=value=>value.split('\n').map(x=>x.trim()).filter(Boolean);
function showError(error){const box=byId('error');box.textContent=String(error);box.hidden=false}function clearError(){byId('error').hidden=true}function text(node,value){node.textContent=String(value)}
async function openWorkspace(){clearError();if(!workspace())return showError('Indique la ruta del workspace.');try{const works=await invoke('list_work',{workspace:workspace()});text(byId('workspace-status'),`${works.length} trabajo(s) validado(s)`);const list=byId('work-list');list.replaceChildren();for(const work of works){const button=document.createElement('button');text(button,work.name);button.addEventListener('click',()=>loadSummary(work.id));list.append(button)}}catch(error){showError(error)}}
async function createWork(event){event.preventDefault();clearError();const data=Object.fromEntries(new FormData(event.target));try{const created=await invoke('create_work',{workspace:workspace(),request:data});event.target.reset();await openWorkspace();await loadSummary(created.engagement.id,'Trabajo y contexto autorizado creados. Revise el Scope antes de ejecutar.')}catch(error){showError(error)}}
async function loadSummary(id,feedback=''){
  clearError();
  try{
    const value=await invoke('summarize',{workspace:workspace(),engagementId:id});
    activeEngagement=id;activeExecution=null;activeProfile=value.engagement.security_profile;
    pendingResume=null;byId('resume-review').hidden=true;
    byId('welcome').hidden=true;byId('work-panel').hidden=false;byId('execution-detail').hidden=true;
    text(byId('work-title'),value.engagement.name);text(byId('profile'),value.engagement.security_profile);
    text(byId('work-feedback'),feedback);
    const sessions=byId('session');sessions.replaceChildren();
    for(const item of value.sessions){const option=document.createElement('option');option.value=item.id;text(option,item.name);sessions.append(option)}
    const previous=value.continuity.state.current_session_id;
    if(previous&&value.sessions.some(item=>item.id===previous))sessions.value=previous;
    renderSummary(value);renderExecutions(value.executions);await refreshContext();
  }catch(error){showError(error)}
}
async function refreshContext(){
  if(!activeEngagement||!byId('session').value)return;
  try{const review=await getAuthorizationReview();const c=review.context;
    text(byId('work-context'),`Scope: ${c.scope.name} · Entorno: ${c.environment.name} · Objetivo: ${c.target.label} (${c.target.locator}) · Sesión: ${c.session.name}. Límite autorizado: ${c.scope.authorization_boundary}`);
  }catch(error){showError(error)}
}
const observedDate=value=>{const n=Number(value);return Number.isFinite(n)?new Date(n).toLocaleString('es-CO'):'fecha no disponible'};
function renderSummary(value){
  const root=byId('summary');root.replaceChildren();
  for(const [count,label] of [[value.sessions.length,'sesiones'],[value.executions.length,'ejecuciones'],[value.executions.reduce((n,e)=>n+e.artifact_count,0),'artifacts registrados']]){
    const card=document.createElement('div'),strong=document.createElement('strong');card.className='card';text(strong,count);card.append(strong,document.createTextNode(label));root.append(card);
  }
  const profile=value.engagement.security_profile;
  let notice='Reabrir el trabajo no autoriza nuevas operaciones; revise el contexto antes de continuar.';
  if(profile==='HIGH_SENSITIVITY')notice+=' TATACOA todavía no proporciona cifrado persistente del workspace local.';
  if(profile==='CUSTOM')notice+=' CUSTOM no habilita capacidades de protección sin una política aprobada.';
  text(byId('profile-notice'),notice);
  const continuity=value.continuity,state=continuity.state;
  text(byId('continuity-status'),`${state.status==='PAUSED'?'Trabajo pausado':'Trabajo activo'} · último estado guardado ${observedDate(state.updated_unix_ms_observed)}${continuity.recovery_required?` · RECUPERACIÓN REQUERIDA: ${continuity.incomplete_capture_count} captura(s) incompleta(s)`:''}`);
  byId('resume').hidden=state.status!=='PAUSED';
  byId('continuity-pending').value=state.pending.join('\n');
  const recent=[...value.executions].sort((a,b)=>Number(b.started_unix_ms_observed)-Number(a.started_unix_ms_observed));
  const next=recent[0];
  text(byId('continuity-next'),next?`Última ejecución: ${next.executable} · ${next.capture_status} · ${observedDate(next.started_unix_ms_observed)}. Abra el resultado para continuar desde lo registrado.`:'Todavía no hay ejecuciones. Revise el contexto y prepare una operación autorizada.');
  const activity=byId('activity-list');activity.replaceChildren();
  for(const item of recent.slice(0,5)){const row=document.createElement('div'),button=document.createElement('button');row.className='activity-row';text(button,`Abrir ${item.executable} · ${observedDate(item.started_unix_ms_observed)}`);button.addEventListener('click',()=>openExecution(item.id));row.append(button);activity.append(row)}
  if(state.pending.length){const heading=document.createElement('p');text(heading,'Pendientes guardados:');activity.append(heading);for(const pending of state.pending){const row=document.createElement('p');text(row,`• ${pending}`);activity.append(row)}}
}
function renderExecutions(items){
  const root=byId('executions');root.replaceChildren();
  if(!items.length){const empty=document.createElement('p');text(empty,'Aún no hay resultados capturados.');root.append(empty);return}
  const table=document.createElement('table'),head=document.createElement('thead'),headerRow=document.createElement('tr');
  for(const label of['Cuándo','Herramienta','Captura','Artifacts','']){const cell=document.createElement('th');text(cell,label);headerRow.append(cell)}head.append(headerRow);table.append(head);
  const body=document.createElement('tbody');
  for(const item of [...items].sort((a,b)=>Number(b.started_unix_ms_observed)-Number(a.started_unix_ms_observed))){
    const row=document.createElement('tr');for(const value of[observedDate(item.started_unix_ms_observed),item.executable,item.capture_status,item.artifact_count]){const cell=document.createElement('td');text(cell,value);row.append(cell)}
    const action=document.createElement('td'),button=document.createElement('button');button.className='artifact-button';text(button,'Ver resultado');button.addEventListener('click',()=>openExecution(item.id));action.append(button);row.append(action);body.append(row)
  }table.append(body);root.append(table)
}
function renderRelated(rootId,items,describe,empty){const root=byId(rootId);root.replaceChildren();if(items.length===0){const note=document.createElement('p');text(note,empty);root.append(note);return}for(const item of items){const row=document.createElement('div');row.className='related-object';text(row,describe(item));root.append(row)}}
function recordedContext(manifest){const c=manifest.context,e=manifest.execution,argv=e.invocation.argv.map((arg,index)=>`[${index}] ${arg}`).join('\n')||'(sin argumentos)';const context=c?`Scope: ${c.scope.name}\nLímite autorizado: ${c.scope.authorization_boundary}\nEnvironment: ${c.environment.name}\nTarget: ${c.target.label} (${c.target.locator})\nSession: ${c.session.name}`:'Contexto completo no disponible para esta Execution heredada.';return `${context}\n\nExecutable: ${e.invocation.executable}\nArgumentos efectivos:\n${argv}\nShell: ${e.invocation.shell}\nEstado de captura: ${e.capture_status}\nExit code: ${e.exit_code??'no disponible'}\nDuración observada: ${e.duration_ms_observed} ms`}
function applyWorkflowProfile(){let knowledge='TATACOA reutiliza Execution, contexto, invocación, resultado y artifacts. Añada únicamente la interpretación que requiere criterio humano.';if(activeProfile==='LAB_LEARNING')knowledge+=' Distinga con cuidado qué demuestra la captura y qué todavía necesita validación.';if(activeProfile==='PROFESSIONAL')knowledge='Contexto y hechos capturados reutilizados. Registre solo interpretación, límites y validación humana.';if(activeProfile==='HIGH_SENSITIVITY')knowledge='Contexto capturado reutilizado. Minimice datos adicionales y registre solo lo necesario conforme a la política vigente.';text(byId('knowledge-help'),knowledge)}
async function openExecution(id,feedback=''){
  clearError();
  try{
    const manifest=await invoke('execution_workspace',{workspace:workspace(),engagementId:activeEngagement,executionId:id});
    activeExecution=id;byId('execution-detail').hidden=false;
    const execution=manifest.execution;
    text(byId('execution-meta'),`${execution.invocation.executable} · captura ${execution.capture_status} · ${manifest.artifacts.length} artifacts · ${observedDate(execution.started_unix_ms_observed)}. Los artifacts capturados no son Evidence validada.`);
    text(byId('execution-context'),recordedContext(manifest));
    text(byId('replay-origin'),`Se reutilizarán ${execution.invocation.executable}, sus ${execution.invocation.argv.length} argumento(s) y el contexto original. Preparar una receta no la ejecuta.`);
    text(byId('execution-feedback'),feedback);applyWorkflowProfile();
    renderArtifacts(manifest.artifacts);renderKnowledge(manifest.knowledge_cards);
    renderRelated('replay-list',manifest.replay_recipes,item=>`${item.executable} · ${item.argv_template.length} argumento(s) · guardado sin ejecutar`,'Todavía no hay Replay / Retest preparado.');
    const assistance=await invoke('execution_assistance',{workspace:workspace(),engagementId:activeEngagement,executionId:id});renderAssistance(assistance);
    byId('execution-detail').scrollIntoView({block:'start',behavior:'smooth'});
  }catch(error){showError(error)}
}
function renderArtifacts(items){
  const root=byId('artifacts');root.replaceChildren();
  for(const artifact of items){
    const card=document.createElement('div'),button=document.createElement('button'),details=document.createElement('details'),summary=document.createElement('summary'),technical=document.createElement('pre');
    card.className='related-object';button.className='artifact-button';
    text(button,`Abrir ${artifact.role.toLowerCase()} · ${artifact.size_bytes} bytes · ${artifact.capture_status}`);
    button.addEventListener('click',()=>openArtifact(artifact.id));
    text(summary,'Identidad, integridad y provenance');
    text(technical,`ID: ${artifact.id}\nEstado de Evidence: ${artifact.evidence_state}\nClasificación: ${artifact.classification}\nSHA-256: ${artifact.digest.value}\nPath: ${artifact.path}\nProvenance: ${artifact.provenance.kind}`);
    details.append(summary,technical);card.append(button,details);root.append(card);
  }
}
function renderKnowledge(items){
  const root=byId('knowledge-list');root.replaceChildren();
  if(!items.length){const empty=document.createElement('p');text(empty,'Aún no hay notas o Knowledge asociado; añadirlo es opcional.');root.append(empty);return}
  for(const item of items){
    const card=document.createElement('div'),heading=document.createElement('p'),details=document.createElement('details'),summary=document.createElement('summary'),technical=document.createElement('pre');card.className='related-object';
    text(heading,`${item.review_status==='DRAFT'?'Borrador':'Fuente revisada'} · ${item.observe}`);
    text(summary,'Ver ficha y trazabilidad');text(technical,`Knowledge ID: ${item.id}\nExecution: ${item.execution_id}\nObjetivo: ${item.objective}\nQué demuestra: ${item.proves}\nQué no demuestra: ${item.does_not_prove}\nValidación: ${item.validation}\nFuentes: ${item.references.map(ref=>`${ref.classification}: ${ref.title} (${ref.locator})`).join('; ')}`);
    details.append(summary,technical);card.append(heading,details);root.append(card)
  }
}
function renderAssistance(value){
  text(byId('assistance-level'),`Disponibilidad: ${value.level} · adapter: ${value.adapter}. No representa confianza ni estado de Evidence.`);
  text(byId('assistance-documentation'),value.documentation==='UNAVAILABLE'?'No hay ayuda documental local verificada para esta herramienta (UNAVAILABLE). No se ejecutó ningún probe ni se consultó la red.':'Hay ayuda documental local con fuente consultable en los detalles.');
  const root=byId('assistance-facts');root.replaceChildren();
  for(const fact of value.facts){
    const card=document.createElement('div'),primary=document.createElement('p'),details=document.createElement('details'),summary=document.createElement('summary'),source=document.createElement('p');card.className='related-object';
    text(primary,`${fact.kind==='OBSERVED_FACT'?'Observado':'Documentado'} · ${fact.label}: ${fact.value}`);
    text(summary,'Fuente del hecho');text(source,fact.source);details.append(summary,source);card.append(primary,details);root.append(card)
  }
}
async function openArtifact(id){clearError();try{const value=await invoke('artifact_preview',{workspace:workspace(),engagementId:activeEngagement,executionId:activeExecution,artifactId:id});const decoded=new TextDecoder('utf-8',{fatal:false}).decode(new Uint8Array(value.bytes));text(byId('artifact-content'),decoded+(value.truncated_for_preview?'\n\n[Vista previa limitada a 1 MiB]':''))}catch(error){showError(error)}}
async function getAuthorizationReview(){return invoke('authorization_review',{workspace:workspace(),engagementId:activeEngagement,sessionId:byId('session').value})}function reviewText(review){const c=review.context;return `Engagement: ${review.engagement.name}\nSecurity Profile: ${review.engagement.security_profile}\nScope: ${c.scope.name}\nLímite autorizado: ${c.scope.authorization_boundary}\nEnvironment: ${c.environment.name}\nTarget: ${c.target.label} (${c.target.locator})\nSession: ${c.session.name}`}
async function runTool(event){event.preventDefault();clearError();pendingRun=null;byId('execution-review').hidden=true;const request={engagement_id:activeEngagement,session_id:byId('session').value,executable:byId('executable').value,argv:lines(byId('argv').value),max_stream_bytes:null};try{const review=await getAuthorizationReview();const invocation=`${reviewText(review)}\n\nExecutable: ${request.executable}\nArgumentos efectivos:\n${request.argv.map((arg,index)=>`[${index}] ${arg}`).join('\n')||'(sin argumentos)'}`;pendingRun={workspace:workspace(),engagementId:activeEngagement,request};text(byId('execution-review-text'),invocation);byId('execution-review').hidden=false}catch(error){showError(error)}}
async function confirmRun(){clearError();if(!pendingRun)return showError('No hay una ejecución revisada pendiente de confirmación.');const approved=pendingRun;pendingRun=null;byId('execution-review').hidden=true;try{const captured=await invoke('run_tool',{workspace:approved.workspace,request:approved.request});await loadSummary(approved.engagementId,'Ejecución capturada. Revise el resultado y sus artifacts.');await openExecution(captured.execution.id,'Resultado capturado; los artifacts no son Evidence validada.')}catch(error){showError(error)}}
function cancelRun(){pendingRun=null;byId('execution-review').hidden=true}
async function createKnowledge(event){event.preventDefault();clearError();const data=Object.fromEntries(new FormData(event.target));const executionId=activeExecution,request={engagement_id:activeEngagement,execution_id:executionId,source_reviewed:byId('source-reviewed').checked,...data,references:[{classification:byId('reference-classification').value,title:byId('reference-title').value,locator:byId('reference-locator').value}],related_techniques:lines(byId('related-techniques').value)};try{const card=await invoke('create_knowledge_from_execution',{workspace:workspace(),request});event.target.reset();await openExecution(executionId,`Knowledge Card guardada y localizada en esta Execution: ${card.id}`)}catch(error){showError(error)}}
async function createNote(event){event.preventDefault();clearError();const executionId=activeExecution;try{const card=await invoke('create_note_from_execution',{workspace:workspace(),request:{engagement_id:activeEngagement,execution_id:executionId,note:byId('operator-note').value}});event.target.reset();await openExecution(executionId,`Nota guardada como Knowledge borrador ${card.id}; no es Evidence validada.`)}catch(error){showError(error)}}
function parsePlaceholders(value){return lines(value).map(line=>{const parts=line.split('|');if(parts.length!==4)throw new Error('Cada placeholder debe usar NAME|SECRET|REQUIRED|DESCRIPCIÓN.');return{name:parts[0],secret:parts[1]==='true',required:parts[2]==='true',description:parts[3]}})}
byId('select-workspace').addEventListener('click',async()=>{try{const path=await invoke('select_workspace');if(path){byId('workspace').value=path;await openWorkspace()}}catch(error){showError(error)}});byId('select-export').addEventListener('click',async()=>{try{const path=await invoke('select_export_destination');if(path)byId('export-destination').value=path}catch(error){showError(error)}});byId('select-timestamp-sidecar').addEventListener('click',async()=>{try{const path=await invoke('select_timestamp_sidecar');if(path)byId('timestamp-sidecar').value=path}catch(error){showError(error)}});
async function createReplay(event){event.preventDefault();clearError();try{const executionId=activeExecution,changed=byId('replay-change-invocation').checked,request={engagement_id:activeEngagement,execution_id:executionId,executable_override:changed?byId('replay-executable-override').value:null,argv_template_override:changed?lines(byId('replay-argv-override').value):null,placeholders:parsePlaceholders(byId('replay-placeholders').value),prerequisites:lines(byId('replay-prerequisites').value),authorization_limits:lines(byId('replay-limits').value)};if(changed&&!request.executable_override.trim())throw new Error('Una modificación deliberada requiere indicar el executable.');const recipe=await invoke('create_replay_from_execution',{workspace:workspace(),request});event.target.reset();byId('replay-overrides').hidden=true;await openExecution(executionId,`Replay / Retest guardado, localizado y no ejecutado: ${recipe.id}`)}catch(error){showError(error)}}
async function exportExecution(event){event.preventDefault();clearError();const encrypted=byId('export-mode').value==='ENCRYPTED',password=byId('export-password'),confirmation=byId('export-confirmation');try{if(encrypted&&password.value!==confirmation.value)throw new Error('Las contraseñas no coinciden.');const destination=byId('export-destination').value,mode=byId('export-mode').value,request={engagement_id:activeEngagement,execution_id:activeExecution,destination,mode,acknowledge_plaintext:byId('plain-ack').checked,password:encrypted?password.value:null};await invoke('export_execution',{workspace:workspace(),request});byId('timestamp-bundle').value=destination;byId('timestamp-sidecar').value=`${destination}.tsr`;byId('timestamp-mode').value=mode;text(byId('execution-meta'),'Exportación completada según la política del Security Profile. Timestamp permanece opcional y explícito.');event.target.reset();byId('password-fields').hidden=true}catch(error){showError(error)}finally{password.value='';confirmation.value=''}}
function timestampRequest(includeNetwork){const request={bundle:byId('timestamp-bundle').value,sidecar:byId('timestamp-sidecar').value,mode:byId('timestamp-mode').value,trust_anchor_der:lines(byId('timestamp-trust-anchors').value),trust_intermediate_der:lines(byId('timestamp-trust-intermediates').value),accepted_policy_oids:lines(byId('timestamp-policies').value)};if(includeNetwork){request.tsa_endpoint=byId('timestamp-tsa').value;request.timeout_seconds=Number(byId('timestamp-timeout').value)}return request}function renderTimestampReport(report){const rows=[`ASSURANCE: ${report.assurance}`,`Policy: ${report.policy_oid||'no disponible'}`,...report.checks.map(check=>`${check.status} · ${check.name}: ${check.detail}`)];text(byId('timestamp-report'),rows.join('\n'))}async function requestTimestamp(event){event.preventDefault();clearError();try{if(!byId('timestamp-tsa').value)throw new Error('Configure explícitamente una TSA HTTPS.');const report=await invoke('request_timestamp',{workspace:workspace(),request:timestampRequest(true)});renderTimestampReport(report)}catch(error){showError(error)}}async function verifyTimestamp(){clearError();try{const report=await invoke('verify_timestamp',{workspace:workspace(),request:timestampRequest(false)});renderTimestampReport(report)}catch(error){showError(error)}}
byId('pause').addEventListener('click',async()=>{try{await invoke('pause_work',{workspace:workspace(),engagementId:activeEngagement,sessionId:byId('session').value||null,pending:lines(byId('continuity-pending').value)});await loadSummary(activeEngagement,'Trabajo pausado y pendientes guardados. Para operar de nuevo, revalide la autorización.')}catch(error){showError(error)}});
byId('resume').addEventListener('click',async()=>{clearError();try{const review=await getAuthorizationReview();pendingResume={workspace:workspace(),engagementId:activeEngagement,sessionId:byId('session').value};text(byId('resume-review-text'),reviewText(review));byId('resume-review').hidden=false}catch(error){showError(error)}});
byId('cancel-resume').addEventListener('click',()=>{pendingResume=null;byId('resume-review').hidden=true});
byId('confirm-resume').addEventListener('click',async()=>{if(!pendingResume)return showError('No hay una reanudación revisada pendiente.');const approved=pendingResume;pendingResume=null;byId('resume-review').hidden=true;try{if(approved.workspace!==workspace()||approved.engagementId!==activeEngagement||approved.sessionId!==byId('session').value)throw new Error('El contexto cambió; revise nuevamente antes de reanudar.');await invoke('resume_work',{workspace:approved.workspace,engagementId:approved.engagementId,authorizationRevalidated:true});await loadSummary(approved.engagementId,'Trabajo reanudado tras revalidación explícita. Revise cada nueva ejecución antes de confirmarla.')}catch(error){showError(error)}});
byId('open').addEventListener('click',openWorkspace);byId('create-form').addEventListener('submit',createWork);byId('run-form').addEventListener('submit',runTool);byId('confirm-run').addEventListener('click',confirmRun);byId('cancel-run').addEventListener('click',cancelRun);byId('refresh').addEventListener('click',()=>loadSummary(activeEngagement));byId('knowledge-form').addEventListener('submit',createKnowledge);byId('replay-form').addEventListener('submit',createReplay);byId('export-form').addEventListener('submit',exportExecution);byId('timestamp-form').addEventListener('submit',requestTimestamp);byId('verify-timestamp').addEventListener('click',verifyTimestamp);byId('export-mode').addEventListener('change',()=>{byId('password-fields').hidden=byId('export-mode').value!=='ENCRYPTED'});
byId('replay-change-invocation').addEventListener('change',()=>{byId('replay-overrides').hidden=!byId('replay-change-invocation').checked});
byId('note-form').addEventListener('submit',createNote);
byId('session').addEventListener('change',()=>{cancelRun();pendingResume=null;byId('resume-review').hidden=true;refreshContext()});

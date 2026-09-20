const invoke = window.__TAURI__.core.invoke;
let activeEngagement = null;
const byId = id => document.getElementById(id);
const workspace = () => byId('workspace').value.trim();

function showError(error) { const box=byId('error'); box.textContent=String(error); box.hidden=false; }
function clearError() { byId('error').hidden=true; }
function text(node, value) { node.textContent=String(value); }

async function openWorkspace() {
  clearError();
  if (!workspace()) return showError('Indique la ruta del workspace.');
  try {
    const works = await invoke('list_work', { workspace: workspace() });
    text(byId('workspace-status'), `${works.length} trabajo(s) validado(s)`);
    const list=byId('work-list'); list.replaceChildren();
    for (const work of works) { const button=document.createElement('button'); text(button, work.name); button.addEventListener('click',()=>loadSummary(work.id)); list.append(button); }
  } catch (error) { showError(error); }
}

async function createWork(event) {
  event.preventDefault(); clearError();
  const data=Object.fromEntries(new FormData(event.target));
  try { const created=await invoke('create_work',{workspace:workspace(),request:data}); event.target.reset(); await openWorkspace(); await loadSummary(created.engagement.id); }
  catch(error){showError(error);}
}

async function loadSummary(id) {
  clearError();
  try {
    const value=await invoke('summarize',{workspace:workspace(),engagementId:id}); activeEngagement=id;
    byId('welcome').hidden=true; byId('work-panel').hidden=false; text(byId('work-title'),value.engagement.name); text(byId('profile'),value.engagement.security_profile);
    renderSummary(value);
    const sessions=byId('session'); sessions.replaceChildren(); for(const item of value.sessions){const option=document.createElement('option');option.value=item.id;text(option,item.name);sessions.append(option);}
    renderExecutions(value.executions);
  } catch(error){showError(error);}
}

function renderSummary(value) {
  const root=byId('summary'); root.replaceChildren();
  const values=[[value.sessions.length,'sesiones'],[value.executions.length,'ejecuciones'],[value.executions.reduce((n,e)=>n+e.artifact_count,0),'artifacts']];
  for (const [count,label] of values) { const card=document.createElement('div'); card.className='card'; const strong=document.createElement('strong'); text(strong,count); card.append(strong,document.createTextNode(label)); root.append(card); }
}

function renderExecutions(items){const root=byId('executions');root.replaceChildren();const table=document.createElement('table');const head=document.createElement('thead');const headerRow=document.createElement('tr');for(const label of ['Ejecutable','Estado','Artifacts']){const cell=document.createElement('th');text(cell,label);headerRow.append(cell);}head.append(headerRow);table.append(head);const body=document.createElement('tbody');for(const item of items){const row=document.createElement('tr');for(const value of [item.executable,item.capture_status,item.artifact_count]){const cell=document.createElement('td');text(cell,value);row.append(cell);}body.append(row);}table.append(body);root.append(table);}

async function runTool(event){event.preventDefault();clearError();if(!confirm('Confirme que esta ejecución está dentro del alcance autorizado.'))return;const request={engagement_id:activeEngagement,session_id:byId('session').value,executable:byId('executable').value,argv:byId('argv').value.split('\n').map(x=>x.trim()).filter(Boolean),max_stream_bytes:null};try{const summary=await invoke('run_tool',{workspace:workspace(),request});renderExecutions(summary.executions);await loadSummary(activeEngagement);}catch(error){showError(error);}}

byId('open').addEventListener('click',openWorkspace);byId('create-form').addEventListener('submit',createWork);byId('run-form').addEventListener('submit',runTool);byId('refresh').addEventListener('click',()=>loadSummary(activeEngagement));

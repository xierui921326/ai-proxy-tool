import { invoke } from "@tauri-apps/api/core";

function log(msg){
  const el = document.getElementById('log');
  el.textContent += `\n${new Date().toLocaleTimeString()} ${msg}`;
  el.scrollTop = el.scrollHeight;
}

async function load(){
  try{
    const s = await invoke('load_settings');
    document.getElementById('listen').value = s.listen;
    if(s.tls_listen) document.getElementById('tls_listen').value = s.tls_listen;
    if(s.openai_upstream) document.getElementById('openai').value = s.openai_upstream;
    if(s.anthropic_upstream) document.getElementById('anthropic').value = s.anthropic_upstream;
    updateSvcStatus(false);
    updateHostsStatus('unknown');
  }catch(e){ log(`load_settings error: ${e}`) }
}

document.getElementById('start').onclick = async () => {
  const cfg = {
    listen: document.getElementById('listen').value || null,
    tls_listen: document.getElementById('tls_listen').value || null,
    openai_upstream: document.getElementById('openai').value || null,
    anthropic_upstream: document.getElementById('anthropic').value || null,
  };
  try{ await invoke('start_proxy', { cfg }); log('proxy started'); }catch(e){ log(`start error: ${e}`) }
  updateSvcStatus(true);
}

document.getElementById('applyHosts').onclick = async ()=>{
  try{ await invoke('apply_hosts'); log('hosts applied'); updateHostsStatus('on'); }catch(e){ log(`apply hosts error: ${e}`) }
}

document.getElementById('revertHosts').onclick = async ()=>{
  try{ await invoke('revert_hosts'); log('hosts reverted'); updateHostsStatus('off'); }catch(e){ log(`revert hosts error: ${e}`) }
}

document.getElementById('exportCA').onclick = async ()=>{
  try{ const p = await invoke('export_root_ca_path'); document.getElementById('caPath').value = p; log(`CA at ${p}`);}catch(e){ log(`export CA error: ${e}`) }
}

document.getElementById('save').onclick = async ()=>{
  const cfg = {
    listen: document.getElementById('listen').value,
    tls_listen: document.getElementById('tls_listen').value,
    openai_upstream: document.getElementById('openai').value,
    anthropic_upstream: document.getElementById('anthropic').value,
  };
  try{ await invoke('save_settings', { cfg }); log('saved'); }catch(e){ log(`save error: ${e}`) }
}

document.getElementById('clear').onclick = async ()=>{
  try{ await invoke('clear_data'); log('data cleared'); }catch(e){ log(`clear error: ${e}`) }
}

document.getElementById('launch').onclick = async ()=>{
  const path = document.getElementById('appPath').value;
  const listen = document.getElementById('listen').value;
  const port = parseInt((listen.split(':')[1]||'8777'),10);
  try{ await invoke('launch_target', { appPath: path, port }); log('launched target app'); }catch(e){ log(`launch error: ${e}`) }
}

load();

function updateSvcStatus(on){
  const el = document.getElementById('svcStatus');
  if(!el) return;
  if(on){ el.textContent = '运行中'; el.style.background = '#dcfce7'; el.style.color = '#166534'; }
  else { el.textContent = '未启动'; el.style.background = '#fee2e2'; el.style.color = '#991b1b'; }
}

function updateHostsStatus(state){
  const el = document.getElementById('hostsStatus');
  if(!el) return;
  if(state==='on'){ el.textContent = '已接管'; el.style.background = '#e0e7ff'; el.style.color = '#3730a3'; }
  else if(state==='off'){ el.textContent = '已释放'; el.style.background = '#f3f4f6'; el.style.color = '#374151'; }
  else { el.textContent = '未知'; el.style.background = '#fef3c7'; el.style.color = '#92400e'; }
}

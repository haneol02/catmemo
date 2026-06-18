// store.js — CatMemo data layer (single JSON file via Tauri)

const invoke = (...args) => window.__TAURI__.core.invoke(...args);
const emit   = (...args) => window.__TAURI__.event.emit(...args);

let _cache = null;

async function getData() {
  if (!_cache) {
    const raw = await invoke('load_data');
    _cache = JSON.parse(raw);
    if (!Array.isArray(_cache.memos))   _cache.memos = [];
    if (!_cache.settings) _cache.settings = { breed:'orange', defaultColor:'#fef9c3' };
  }
  return _cache;
}

async function persist() {
  await invoke('save_data', { data: JSON.stringify(_cache) });
  await emit('data-changed', {});
}

export function invalidateCache() { _cache = null; }

// ─── Settings ─────────────────────────────────────────────────────────────────

export async function getSettings() {
  return { ...(await getData()).settings };
}

export async function saveSettings(patch) {
  const d = await getData();
  d.settings = { ...d.settings, ...patch };
  await persist();
}

// ─── Memos ────────────────────────────────────────────────────────────────────

function genId() {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 7);
}

export async function getMemos({ archived = false } = {}) {
  const d = await getData();
  return archived ? d.memos.filter(m => m.isArchived) : d.memos.filter(m => !m.isArchived);
}

export async function getMemo(id) {
  const d = await getData();
  return d.memos.find(m => m.id === id) ?? null;
}

export async function createMemo(color) {
  const d = await getData();
  const memo = {
    id: genId(),
    title: '새 메모',
    content: '',
    color: color || d.settings.defaultColor || '#fef9c3',
    isEncrypted: false,
    isArchived:  false,
    isPinned:    false,
    createdAt:   new Date().toISOString(),
    updatedAt:   new Date().toISOString(),
  };
  d.memos.push(memo);
  await persist();
  return memo;
}

export async function updateMemo(id, patch) {
  const d   = await getData();
  const idx = d.memos.findIndex(m => m.id === id);
  if (idx === -1) return null;
  d.memos[idx] = { ...d.memos[idx], ...patch, updatedAt: new Date().toISOString() };
  await persist();
  return d.memos[idx];
}

export async function deleteMemo(id) {
  const d = await getData();
  d.memos = d.memos.filter(m => m.id !== id);
  await persist();
}

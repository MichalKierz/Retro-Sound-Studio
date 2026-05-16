import { invoke } from './tauri.js';

const stateKey = 'retro-sound-studio-state-v1';

let saveTimer = null;

export async function loadAppState() {
    try {
        const state = await invoke('load_app_state');
        discardLegacyLocalStorageState();
        return state;
    } catch (_error) {
        discardLegacyLocalStorageState();
        return null;
    }
}

export function bindAppStatePersistence(collectState) {
    const saveNow = () => saveAppState(collectState());
    const schedule = () => {
        window.clearTimeout(saveTimer);
        saveTimer = window.setTimeout(saveNow, 250);
    };
    document.addEventListener('input', schedule, true);
    document.addEventListener('change', schedule, true);
    document.addEventListener('mouseup', schedule, true);
    document.addEventListener('keyup', schedule, true);
    window.addEventListener('beforeunload', saveNow);
    window.setInterval(saveNow, 1500);
    saveNow();
}

export function collectFormValues() {
    const values = {};
    document.querySelectorAll('input[id], select[id]').forEach(element => {
        values[element.id] = element.value;
    });
    return values;
}

export function restoreFormValues(values) {
    if (!values || typeof values !== 'object') {
        return;
    }
    for (const [id, value] of Object.entries(values)) {
        const element = document.getElementById(id);
        if (element) {
            element.value = value;
            element.dispatchEvent(new Event('change', { bubbles: true }));
        }
    }
}

export function getActiveTabId() {
    return document.querySelector('.tab-content.active')?.id || 'sound-creation';
}

export function restoreActiveTab(id) {
    const target = document.getElementById(id);
    const button = document.querySelector(`.tab-btn[data-target="${id}"]`);
    if (!target || !button) {
        return;
    }
    document.querySelectorAll('.tab-btn').forEach(item => item.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(item => item.classList.remove('active'));
    button.classList.add('active');
    target.classList.add('active');
}

export function createPersistentMelodyProject(project) {
    return {
        ...project,
        layers: (project.layers || []).map(layer => ({
            ...layer,
            soundSample: null
        }))
    };
}

async function saveAppState(state) {
    try {
        await invoke('save_app_state', { state });
    } catch (_error) {
        const lighterState = {
            ...state,
            melody: createPersistentMelodyProject(state?.melody || {})
        };
        try {
            await invoke('save_app_state', { state: lighterState });
        } catch (_secondError) {
            discardLegacyLocalStorageState();
        }
    }
}

function discardLegacyLocalStorageState() {
    try {
        window.localStorage.removeItem(stateKey);
    } catch (_error) {
    }
}

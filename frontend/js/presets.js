import { invoke } from './tauri.js';

let soundPresets = [];
let melodyWorkflows = [];
let melodyPresets = [];

export async function loadPresetLists() {
    const [sounds, workflows, presets] = await Promise.all([
        invoke('list_sound_presets'),
        invoke('list_melody_presets'),
        invoke('list_melody_layer_presets')
    ]);
    soundPresets = sounds;
    melodyWorkflows = workflows;
    melodyPresets = presets;
    fillSelect(document.getElementById('sound-preset'), soundPresets, 'Select sound preset');
    fillSelect(document.getElementById('melody-preset'), melodyWorkflows, 'Select melody workflow');
}

export function getSoundPresets() {
    return soundPresets.slice();
}

export function getMelodyPresets() {
    return melodyPresets.slice();
}

export function getMelodyWorkflows() {
    return melodyWorkflows.slice();
}

export async function loadSoundPreset(id) {
    return invoke('load_sound_preset', { name: id });
}

export async function saveSoundPresetFile(path, params) {
    return invoke('save_sound_preset_file', { path, params });
}

export async function loadMelodyPreset(id) {
    return invoke('load_melody_preset', { name: id });
}

export async function loadMelodyPresetFile(path) {
    return invoke('load_melody_preset_file', { path });
}

export async function loadMelodyLayerPreset(id) {
    return invoke('load_melody_layer_preset', { name: id });
}

export async function saveMelodyPreset(filename, project, description) {
    return invoke('save_melody_preset', { filename, project, description });
}

export async function saveMelodyPresetFile(path, project) {
    return invoke('save_melody_preset_file', { path, project });
}

export async function getPresetDirectory(kind) {
    return invoke('default_preset_dir', { kind });
}

function fillSelect(select, presets, emptyLabel) {
    if (!select) {
        return;
    }
    select.innerHTML = '';
    const empty = document.createElement('option');
    empty.value = '';
    empty.textContent = emptyLabel;
    select.appendChild(empty);
    for (const preset of presets) {
        const option = document.createElement('option');
        option.value = preset.id;
        option.textContent = preset.name;
        option.dataset.description = preset.description || '';
        select.appendChild(option);
    }
}

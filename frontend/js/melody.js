import {
    getMelodyPresets,
    getPresetDirectory,
    loadMelodyLayerPreset,
    loadMelodyPreset,
    loadMelodyPresetFile,
    loadPresetLists,
    saveMelodyPresetFile
} from './presets.js';
import { confirmDialog, invoke, openFile, saveFile } from './tauri.js';

const colors = ['#6cff8f', '#69d8ff', '#ffd76a', '#ff6bd6', '#b58cff', '#ff8c69', '#8cffd2', '#f4ff69'];
const pitchMin = 24;
const pitchMax = 96;
const beatsVisible = 16;
const snap = 0.25;
const rollLeft = 90;
const baseCanvasWidth = 2200;
const baseCanvasHeight = 1460;
const minZoom = 0.6;
const maxZoom = 2.2;
const pitchNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

let project = createEmptyProject();
let activeLayer = 1;
let selected = null;
let dragMode = null;
let dragOffsetStart = 0;
let dragOffsetPitch = 0;
let editMode = 'draw';
let nextLayerId = 5;
let canvas = null;
let ctx = null;
let playback = null;
let statusWriter = null;
let liveTimer = null;
let playheadSeconds = null;
let playheadDuration = 0;
let pianoZoom = 1;

export function initMelody(playbackController, setStatus) {
    playback = playbackController;
    statusWriter = setStatus;
    canvas = document.getElementById('melody-canvas');
    if (!canvas) {
        return;
    }
    ctx = canvas.getContext('2d');

    document.getElementById('btn-add-layer')?.addEventListener('click', () => {
        const id = nextLayerId;
        nextLayerId += 1;
        project.layers.push(createLayer(id));
        activeLayer = id;
        selected = null;
        renderLayerList();
        drawPianoRoll();
        scheduleLiveUpdate();
    });

    document.getElementById('btn-delete-note')?.addEventListener('click', () => {
        deleteSelectedNote();
    });

    document.getElementById('btn-clear-layer')?.addEventListener('click', () => {
        getLayer(activeLayer).notes = [];
        selected = null;
        drawPianoRoll();
        scheduleLiveUpdate();
    });

    document.getElementById('btn-save-selected-melody')?.addEventListener('click', () => {
        saveLayerMelodyPreset(activeLayer);
    });

    document.getElementById('btn-load-selected-melody')?.addEventListener('click', () => {
        const layer = getLayer(activeLayer);
        if (!layer.melodyPresetId) {
            statusWriter('Select a melody preset for the active layer');
            return;
        }
        loadPresetIntoLayer(activeLayer, layer.melodyPresetId);
    });

    document.getElementById('btn-zoom-out')?.addEventListener('click', () => {
        setPianoZoom(pianoZoom - 0.15);
    });

    document.getElementById('btn-zoom-in')?.addEventListener('click', () => {
        setPianoZoom(pianoZoom + 0.15);
    });

    document.getElementById('btn-note-shorten')?.addEventListener('click', () => {
        resizeSelectedNote(-snap);
    });

    document.getElementById('btn-note-lengthen')?.addEventListener('click', () => {
        resizeSelectedNote(snap);
    });

    document.querySelectorAll('[data-edit-mode]').forEach(button => {
        button.addEventListener('click', () => {
            editMode = button.dataset.editMode;
            updateModeButtons();
        });
    });

    const melodyPreset = document.getElementById('melody-preset');
    document.getElementById('btn-load-melody-preset')?.addEventListener('click', async () => {
        if (!melodyPreset?.value) {
            setStatus('Select a melody workflow first');
            return;
        }
        try {
            project = await hydrateWorkflow(normalizeProject(await loadMelodyPreset(melodyPreset.value)));
            nextLayerId = Math.max(...project.layers.map(layer => layer.id), 0) + 1;
            selected = null;
            activeLayer = project.layers[0]?.id || 1;
            renderLayerList();
            drawPianoRoll();
            setStatus(`Loaded melody workflow ${melodyPreset.options[melodyPreset.selectedIndex].textContent}`);
            refreshPlaybackIfAudible();
        } catch (error) {
            setStatus(error.message);
        }
    });

    document.getElementById('btn-import-melody-preset')?.addEventListener('click', async () => {
        try {
            const defaultPath = await getPresetDirectory('melody-workflows');
            const path = await openFile({
                multiple: false,
                defaultPath,
                filters: [{ name: 'Melody Workflow', extensions: ['json'] }]
            });
            if (!path) {
                return;
            }
            project = await hydrateWorkflow(normalizeProject(await loadMelodyPresetFile(path)));
            nextLayerId = Math.max(...project.layers.map(layer => layer.id), 0) + 1;
            selected = null;
            activeLayer = project.layers[0]?.id || 1;
            renderLayerList();
            drawPianoRoll();
            refreshPlaybackIfAudible();
        } catch (error) {
            setStatus(error.message);
        }
    });

    document.getElementById('btn-save-melody-preset')?.addEventListener('click', async () => {
        try {
            const defaultPath = joinPath(await getPresetDirectory('melody-workflows'), `${safeFileName(project.name || 'melody_workflow')}.json`);
            const path = await saveFile({
                defaultPath,
                filters: [{ name: 'Melody Workflow', extensions: ['json'] }]
            });
            if (!path) {
                return;
            }
            const saved = await saveMelodyPresetFile(path, getMelodyProject());
            await loadPresetLists();
            const select = document.getElementById('melody-preset');
            if (select) {
                select.value = saved.id;
            }
            renderLayerList();
            setStatus(`Saved melody workflow ${saved.name}`);
        } catch (error) {
            setStatus(error.message);
        }
    });

    document.getElementById('btn-reset-melody-workflow')?.addEventListener('click', async () => {
        const confirmed = await confirmDialog('Reset the current melody workflow? This clears all layers, selected sounds and notes.', 'Reset Workflow');
        if (!confirmed) {
            return;
        }
        await resetCurrentWorkflow();
    });

    canvas.addEventListener('mousedown', handleMouseDown);
    canvas.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', () => {
        dragMode = null;
    });

    renderLayerList();
    updateModeButtons();
    setPianoZoom(1);
    drawPianoRoll();
}

export async function playCurrentMelody(playbackController, setStatus) {
    const info = await invoke('play_melody', { project: getMelodyProject() });
    playbackController?.setInfo(info);
    setStatus('');
    return info;
}

export function getMelodyProject() {
    return normalizeProject(project);
}

export function getMelodyUiState() {
    return {
        activeLayer,
        editMode,
        pianoZoom
    };
}

export function setMelodyProject(input) {
    project = normalizeProject(input);
    nextLayerId = Math.max(...project.layers.map(layer => layer.id), 0) + 1;
    activeLayer = project.layers[0]?.id || 1;
    selected = null;
    renderLayerList();
    drawPianoRoll();
}

export function setMelodyUiState(state) {
    if (!state || typeof state !== 'object') {
        return;
    }
    if (project.layers.some(layer => layer.id === Number(state.activeLayer))) {
        activeLayer = Number(state.activeLayer);
    }
    if (state.editMode === 'draw' || state.editMode === 'delete') {
        editMode = state.editMode;
        updateModeButtons();
    }
    if (Number.isFinite(Number(state.pianoZoom))) {
        setPianoZoom(Number(state.pianoZoom));
    } else {
        renderLayerList();
        drawPianoRoll();
    }
}

export async function restoreMelodyProject(input) {
    const restored = normalizeProject(input);
    await hydrateSoundFiles(restored.layers);
    setMelodyProject(await hydrateWorkflow(restored));
}

async function hydrateSoundFiles(layers) {
    for (const layer of layers) {
        await hydrateLayerSound(layer);
    }
}

async function hydrateLayerSound(layer) {
    if (!layer.soundFilePath) {
        return;
    }
    try {
        if (fileExtension(layer.soundFilePath) === 'json') {
            layer.sound = await invoke('load_sound_preset_file', { path: layer.soundFilePath });
            layer.soundSample = null;
        } else {
            layer.soundSample = await invoke('load_audio_sample_file', { path: layer.soundFilePath });
            layer.sound = null;
        }
    } catch (_error) {
        layer.sound = null;
        layer.soundSample = null;
    }
}

export function setMelodyPlayhead(current, duration) {
    playheadSeconds = duration > 0 ? current : null;
    playheadDuration = duration || 0;
    drawPianoRoll();
}

function renderLayerList() {
    const list = document.getElementById('layer-list');
    if (!list) {
        return;
    }
    list.innerHTML = '';
    for (const layer of project.layers) {
        const row = document.createElement('div');
        row.className = `layer-row${layer.id === activeLayer ? ' active' : ''}${layerVolume(layer) <= 0 ? ' silent' : ''}`;
        row.dataset.layer = String(layer.id);

        const titleCell = document.createElement('div');
        titleCell.className = 'layer-title-cell';

        const volumeCell = document.createElement('label');
        volumeCell.className = 'layer-volume';
        volumeCell.title = 'Layer volume';
        const volumeInput = document.createElement('input');
        volumeInput.type = 'range';
        volumeInput.min = '0';
        volumeInput.max = '100';
        volumeInput.step = '1';
        volumeInput.value = String(Math.round(layerVolume(layer) * 100));
        const volumeReadout = document.createElement('span');
        volumeReadout.textContent = `${volumeInput.value}%`;
        volumeInput.addEventListener('input', event => {
            layer.volume = Math.max(0, Math.min(1, Number(event.target.value) / 100));
            layer.muted = layer.volume <= 0;
            volumeReadout.textContent = `${Math.round(layer.volume * 100)}%`;
            activeLayer = layer.id;
            drawPianoRoll();
            scheduleLiveUpdate();
        });
        volumeCell.appendChild(volumeInput);
        volumeCell.appendChild(volumeReadout);

        const selectButton = document.createElement('button');
        selectButton.className = 'layer-name';
        selectButton.textContent = layer.name || `Layer ${layer.id}`;
        selectButton.addEventListener('click', () => {
            activeLayer = layer.id;
            selected = null;
            renderLayerList();
            drawPianoRoll();
        });

        titleCell.appendChild(volumeCell);
        titleCell.appendChild(selectButton);

        const soundButton = document.createElement('button');
        soundButton.className = 'layer-sound-file';
        soundButton.textContent = layer.soundLabel || 'Choose sound';
        soundButton.addEventListener('click', () => {
            chooseLayerSound(layer.id);
        });

        const melodySelect = document.createElement('select');
        melodySelect.appendChild(new Option('Melody preset', ''));
        for (const preset of getMelodyPresets()) {
            const option = new Option(preset.name, preset.id);
            melodySelect.appendChild(option);
        }
        melodySelect.value = layer.melodyPresetId || '';
        melodySelect.addEventListener('change', () => {
            layer.melodyPresetId = melodySelect.value;
            if (!melodySelect.value) {
                layer.notes = [];
                selected = null;
            }
            activeLayer = layer.id;
            renderLayerList();
            drawPianoRoll();
            scheduleLiveUpdate();
        });

        const removeButton = document.createElement('button');
        removeButton.textContent = 'X';
        removeButton.addEventListener('click', () => removeLayer(layer.id));

        row.appendChild(titleCell);
        row.appendChild(soundButton);
        row.appendChild(melodySelect);
        row.appendChild(removeButton);
        list.appendChild(row);
    }
    updateSummary();
}

async function chooseLayerSound(layerId) {
    try {
        const defaultPath = await getPresetDirectory('rendered-sounds');
        const path = await openFile({
            multiple: false,
            defaultPath,
            filters: [
                { name: 'Audio or Retro Sound Preset', extensions: ['wav', 'mp3', 'ogg', 'flac', 'json'] },
                { name: 'Audio Sample', extensions: ['wav', 'mp3', 'ogg', 'flac'] },
                { name: 'Retro Sound Preset', extensions: ['json'] }
            ]
        });
        if (!path) {
            return;
        }
        const layer = getLayer(layerId);
        layer.soundFilePath = path;
        layer.soundLabel = fileName(path);
        await hydrateLayerSound(layer);
        activeLayer = layerId;
        renderLayerList();
        scheduleLiveUpdate();
        statusWriter(`${layer.name || `Layer ${layer.id}`} sound loaded`);
    } catch (error) {
        statusWriter(error.message);
    }
}

async function loadPresetIntoLayer(layerId, presetId) {
    if (!presetId) {
        const layer = getLayer(layerId);
        layer.notes = [];
        layer.melodyPresetId = '';
        selected = null;
        activeLayer = layerId;
        renderLayerList();
        drawPianoRoll();
        scheduleLiveUpdate();
        return;
    }
    try {
        const source = normalizeProject(await loadMelodyLayerPreset(presetId));
        const notes = source.layers.find(layer => layer.notes.length)?.notes.map(note => ({ ...note })) || [];
        if (!notes.length) {
            statusWriter('Selected melody preset has no notes');
            return;
        }
        const layer = getLayer(layerId);
        layer.notes = notes.sort((left, right) => left.start - right.start || left.pitch - right.pitch);
        layer.melodyPresetId = presetId;
        activeLayer = layerId;
        selected = null;
        renderLayerList();
        drawPianoRoll();
        scheduleLiveUpdate();
        statusWriter(`Loaded melody into ${layer.name || `Layer ${layer.id}`}`);
    } catch (error) {
        statusWriter(error.message);
    }
}

async function saveLayerMelodyPreset(layerId) {
    try {
        const layer = getLayer(layerId);
        const defaultName = `${safeFileName(layer.name || `layer_${layer.id}`)}_melody.json`;
        const defaultPath = joinPath(await getPresetDirectory('melody-presets'), defaultName);
        const path = await saveFile({
            defaultPath,
            filters: [{ name: 'Melody Preset', extensions: ['json'] }]
        });
        if (!path) {
            return;
        }
        const presetProject = normalizeProject({
            name: layer.name || `Layer ${layer.id} Melody`,
            description: `${layer.name || `Layer ${layer.id}`} melody preset`,
            tempo: project.tempo,
            sample_rate: project.sample_rate,
            layers: [{
                ...layer,
                id: 1,
                name: layer.name || `Layer ${layer.id}`,
                sound: null,
                soundSample: null,
                soundFilePath: '',
                soundLabel: '',
                melodyPresetId: '',
                muted: false,
                volume: 1
            }]
        });
        const saved = await saveMelodyPresetFile(path, presetProject);
        await loadPresetLists();
        layer.melodyPresetId = saved.id;
        renderLayerList();
        scheduleLiveUpdate();
    } catch (error) {
        statusWriter(error.message);
    }
}

function removeLayer(layerId) {
    if (project.layers.length <= 1) {
        statusWriter('At least one layer is required');
        return;
    }
    project.layers = project.layers.filter(layer => layer.id !== layerId);
    if (activeLayer === layerId) {
        activeLayer = project.layers[0].id;
    }
    selected = null;
    renderLayerList();
    drawPianoRoll();
    scheduleLiveUpdate();
}

function handleMouseDown(event) {
    const point = canvasPoint(event);
    const hit = hitTest(point.x, point.y);
    if (editMode === 'delete') {
        if (hit) {
            hit.layer.notes = hit.layer.notes.filter(note => note !== hit.note);
            selected = null;
            drawPianoRoll();
            scheduleLiveUpdate();
        }
        return;
    }
    if (hit) {
        selected = hit;
        dragMode = hit.edge ? 'resize' : 'move';
        dragOffsetStart = pointToBeat(point.x) - hit.note.start;
        dragOffsetPitch = pointToPitch(point.y) - hit.note.pitch;
    } else if (point.x >= rollLeft) {
        const note = {
            pitch: pointToPitch(point.y),
            start: snapBeat(pointToBeat(point.x)),
            duration: 1,
            velocity: 0.85
        };
        const layer = getLayer(activeLayer);
        layer.notes.push(note);
        selected = { layer, note, edge: false };
        dragMode = 'move';
        dragOffsetStart = 0;
        dragOffsetPitch = 0;
        scheduleLiveUpdate();
    }
    drawPianoRoll();
}

function handleMouseMove(event) {
    if (!dragMode || !selected || editMode !== 'draw') {
        return;
    }
    const point = canvasPoint(event);
    if (dragMode === 'move') {
        selected.note.start = Math.max(0, snapBeat(pointToBeat(point.x) - dragOffsetStart));
        selected.note.pitch = clampPitch(Math.round(pointToPitch(point.y) - dragOffsetPitch));
    }
    if (dragMode === 'resize') {
        const beat = snapBeat(pointToBeat(point.x));
        selected.note.duration = Math.max(snap, snapBeat(beat - selected.note.start));
    }
    drawPianoRoll();
    scheduleLiveUpdate();
}

function drawPianoRoll() {
    if (!ctx) {
        return;
    }
    const width = canvas.width;
    const height = canvas.height;
    ctx.fillStyle = '#07090a';
    ctx.fillRect(0, 0, width, height);
    drawGrid(width, height);
    for (const layer of project.layers) {
        for (const note of layer.notes) {
            drawNote(layer, note);
        }
    }
    drawPlayhead(width, height);
    updateSummary();
}

function setPianoZoom(value) {
    pianoZoom = Math.max(minZoom, Math.min(maxZoom, Number(value)));
    if (canvas) {
        canvas.width = Math.round(baseCanvasWidth * pianoZoom);
        canvas.height = Math.round(baseCanvasHeight * pianoZoom);
        canvas.style.width = `${canvas.width}px`;
        canvas.style.height = `${canvas.height}px`;
    }
    drawPianoRoll();
}

function drawGrid(width, height) {
    const pixelsPerBeat = (width - rollLeft) / beatsVisible;
    const pitchCount = pitchMax - pitchMin + 1;
    const rowHeight = height / pitchCount;
    ctx.fillStyle = '#101417';
    ctx.fillRect(0, 0, rollLeft, height);
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    for (let pitch = pitchMin; pitch <= pitchMax; pitch += 1) {
        const y = (pitchMax - pitch + 0.5) * rowHeight;
        const name = pitchNames[pitch % 12];
        const octave = Math.floor(pitch / 12) - 1;
        if (!name.includes('#')) {
            const isOctave = name === 'C';
            ctx.font = isOctave ? '16px Courier New' : '11px Courier New';
            ctx.fillStyle = isOctave ? '#69d8ff' : '#9fb0a8';
            ctx.fillText(`${name}${octave}`, rollLeft - 10, y);
        }
    }
    ctx.lineWidth = 1;
    for (let beat = 0; beat <= beatsVisible; beat += snap) {
        const x = rollLeft + beat * pixelsPerBeat;
        ctx.strokeStyle = Number.isInteger(beat) ? '#32443c' : '#1d2a27';
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, height);
        ctx.stroke();
    }
    for (let index = 0; index <= pitchCount; index += 1) {
        const y = index * rowHeight;
        ctx.strokeStyle = index % 12 === 0 ? '#32443c' : '#1d2a27';
        ctx.beginPath();
        ctx.moveTo(rollLeft, y);
        ctx.lineTo(width, y);
        ctx.stroke();
    }
}

function drawNote(layer, note) {
    const rect = noteRect(note);
    const isSelected = selected?.note === note;
    ctx.fillStyle = colors[(layer.id - 1) % colors.length];
    const volume = layerVolume(layer);
    ctx.globalAlpha = volume <= 0 ? 0.18 : (layer.id === activeLayer ? 0.95 : 0.42) * (0.35 + volume * 0.65);
    ctx.fillRect(rect.x, rect.y, rect.w, rect.h);
    ctx.globalAlpha = 1;
    ctx.strokeStyle = isSelected ? '#ffffff' : '#07090a';
    ctx.lineWidth = isSelected ? 3 : 1;
    ctx.strokeRect(rect.x, rect.y, rect.w, rect.h);
    ctx.fillStyle = '#071009';
    ctx.fillRect(rect.x + rect.w - 5, rect.y + 2, 3, rect.h - 4);
}

function drawPlayhead(width, height) {
    if (playheadSeconds === null || !playheadDuration) {
        return;
    }
    const beat = playheadSeconds / secondsPerBeat();
    const pixelsPerBeat = (width - rollLeft) / beatsVisible;
    const x = rollLeft + beat * pixelsPerBeat;
    if (x < rollLeft || x > width) {
        return;
    }
    ctx.strokeStyle = '#ffffff';
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
    ctx.stroke();
}

function hitTest(x, y) {
    const layers = project.layers.slice().reverse();
    for (const layer of layers) {
        for (let index = layer.notes.length - 1; index >= 0; index -= 1) {
            const note = layer.notes[index];
            const rect = noteRect(note);
            if (x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h) {
                activeLayer = layer.id;
                renderLayerList();
                return { layer, note, edge: x >= rect.x + rect.w - 10 };
            }
        }
    }
    return null;
}

function noteRect(note) {
    const width = canvas.width;
    const height = canvas.height;
    const pixelsPerBeat = (width - rollLeft) / beatsVisible;
    const pitchCount = pitchMax - pitchMin + 1;
    const rowHeight = height / pitchCount;
    return {
        x: rollLeft + note.start * pixelsPerBeat,
        y: (pitchMax - note.pitch) * rowHeight,
        w: Math.max(note.duration * pixelsPerBeat, 8),
        h: Math.max(rowHeight - 2, 8)
    };
}

function canvasPoint(event) {
    const rect = canvas.getBoundingClientRect();
    return {
        x: (event.clientX - rect.left) * (canvas.width / rect.width),
        y: (event.clientY - rect.top) * (canvas.height / rect.height)
    };
}

function pointToBeat(x) {
    return Math.max(0, (x - rollLeft) / ((canvas.width - rollLeft) / beatsVisible));
}

function pointToPitch(y) {
    const pitchCount = pitchMax - pitchMin + 1;
    return clampPitch(pitchMax - Math.floor(y / (canvas.height / pitchCount)));
}

function snapBeat(value) {
    return Math.round(value / snap) * snap;
}

function clampPitch(value) {
    return Math.max(pitchMin, Math.min(pitchMax, value));
}

function getLayer(id) {
    let layer = project.layers.find(layer => layer.id === id);
    if (!layer) {
        layer = createLayer(id);
        project.layers.push(layer);
        project.layers.sort((left, right) => left.id - right.id);
    }
    return layer;
}

function createLayer(id) {
    return { id, name: `Layer ${id}`, sound: null, soundSample: null, soundFilePath: '', soundLabel: '', melodyPresetId: '', muted: false, volume: 1, notes: [] };
}

function createEmptyProject() {
    return {
        name: 'Current Melody',
        description: 'Editable custom melody',
        tempo: 120,
        sample_rate: 44100,
        layers: [1, 2, 3, 4].map(createLayer)
    };
}

async function resetCurrentWorkflow() {
    try {
        if (playback?.isPlaying()) {
            await playback.stop();
        } else {
            playback?.stopTimer();
        }
    } catch (error) {
        statusWriter(error.message);
    }
    project = createEmptyProject();
    activeLayer = 1;
    nextLayerId = 5;
    selected = null;
    dragMode = null;
    playheadSeconds = null;
    playheadDuration = 0;
    const select = document.getElementById('melody-preset');
    if (select) {
        select.value = '';
    }
    renderLayerList();
    drawPianoRoll();
    document.dispatchEvent(new Event('change', { bubbles: true }));
}

function normalizeProject(input) {
    const layers = input?.layers?.length ? input.layers : createEmptyProject().layers;
    return {
        name: input?.name || 'Current Melody',
        description: input?.description || 'Editable custom melody',
        tempo: Number(input?.tempo || 120),
        sample_rate: Number(input?.sample_rate || 44100),
        layers: layers.map((layer, index) => ({
            id: Number(layer.id || index + 1),
            name: layer.name || `Layer ${Number(layer.id || index + 1)}`,
            sound: layer.sound || null,
            soundSample: layer.soundSample || null,
            soundFilePath: layer.soundFilePath || '',
            soundLabel: layer.soundLabel || (layer.sound || layer.soundSample ? 'Embedded sound' : ''),
            melodyPresetId: layer.melodyPresetId || '',
            muted: Boolean(layer.muted) || Number(layer.volume) === 0,
            volume: Number.isFinite(Number(layer.volume)) ? Math.max(0, Math.min(1, Number(layer.volume))) : (layer.muted ? 0 : 1),
            notes: (layer.notes || []).map(note => ({
                pitch: clampPitch(Number(note.pitch)),
                start: Math.max(0, snapBeat(Number(note.start || 0))),
                duration: Math.max(snap, snapBeat(Number(note.duration || 1))),
                velocity: Number(note.velocity || 0.85)
            }))
        }))
    };
}

function layerVolume(layer) {
    if (Number.isFinite(Number(layer.volume))) {
        return Math.max(0, Math.min(1, Number(layer.volume)));
    }
    return layer.muted ? 0 : 1;
}

function updateModeButtons() {
    document.querySelectorAll('[data-edit-mode]').forEach(button => {
        button.classList.toggle('active', button.dataset.editMode === editMode);
    });
}

function updateSummary() {
    getLayer(activeLayer);
}

function deleteSelectedNote() {
    if (!selected) {
        return;
    }
    selected.layer.notes = selected.layer.notes.filter(note => note !== selected.note);
    selected = null;
    drawPianoRoll();
    scheduleLiveUpdate();
}

function resizeSelectedNote(deltaBeats) {
    if (!selected) {
        return;
    }
    const currentDuration = Number(selected.note.duration || 1);
    selected.note.duration = Math.max(snap, snapBeat(currentDuration + deltaBeats));
    drawPianoRoll();
    scheduleLiveUpdate();
}

function scheduleLiveUpdate() {
    window.clearTimeout(liveTimer);
    if (!playback?.isPlaying()) {
        return;
    }
    liveTimer = window.setTimeout(() => {
        playCurrentMelody(playback, statusWriter).catch(error => statusWriter(error.message));
    }, 220);
}

function refreshPlaybackIfAudible() {
    window.clearTimeout(liveTimer);
    if (!playback?.isPlaying()) {
        return;
    }
    playCurrentMelody(playback, statusWriter).catch(error => statusWriter(error.message));
}

function secondsPerBeat() {
    return 60 / Number(project.tempo || 120);
}

function fileName(path) {
    return String(path).split(/[\\/]/).pop() || 'Sound file';
}

function fileExtension(path) {
    return String(path).split('.').pop()?.toLowerCase() || '';
}

async function hydrateWorkflow(input) {
    const normalized = normalizeProject(input);
    const hydratedLayers = [];
    for (const layer of normalized.layers) {
        await hydrateLayerSound(layer);
        if (!layer.melodyPresetId) {
            hydratedLayers.push(layer);
            continue;
        }
        try {
            const source = normalizeProject(await loadMelodyLayerPreset(layer.melodyPresetId));
            const sourceLayer = source.layers.find(candidate => candidate.notes.length) || source.layers[0];
            hydratedLayers.push({
                ...layer,
                notes: (sourceLayer?.notes || layer.notes).map(note => ({ ...note }))
            });
        } catch (_error) {
            hydratedLayers.push(layer);
        }
    }
    return { ...normalized, layers: hydratedLayers };
}

function safeFileName(value) {
    return String(value)
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9_-]+/g, '_')
        .replace(/^_+|_+$/g, '') || 'melody_workflow';
}

function joinPath(directory, filename) {
    const separator = String(directory).includes('\\') ? '\\' : '/';
    return `${String(directory).replace(/[\\/]+$/, '')}${separator}${filename}`;
}

import {
    bindAppStatePersistence,
    collectFormValues,
    createPersistentMelodyProject,
    getActiveTabId,
    loadAppState,
    restoreActiveTab,
    restoreFormValues
} from './app-state.js';
import { initExport } from './export.js';
import {
    getMelodyProject,
    getMelodyUiState,
    initMelody,
    playCurrentMelody,
    restoreMelodyProject,
    setMelodyPlayhead,
    setMelodyUiState
} from './melody.js';
import { createPlaybackController, getSharedVolume, setSharedVolume } from './playback.js';
import { loadPresetLists } from './presets.js';
import { getSoundParams, initSynth, playCurrentSound, setSoundParams } from './synth.js';
import { createStatus, initUI } from './ui.js';

document.addEventListener('DOMContentLoaded', async () => {
    const setStatus = createStatus();
    let soundPlayback;
    let melodyPlayback;

    soundPlayback = createPlaybackController('sound', () => playCurrentSound(soundPlayback, setStatus), setStatus);
    melodyPlayback = createPlaybackController('melody', () => playCurrentMelody(melodyPlayback, setStatus), setStatus, setMelodyPlayhead);

    initUI(setStatus, [soundPlayback, melodyPlayback]);
    bindKeyboardPlayback(soundPlayback, melodyPlayback, setStatus);

    let initialStatus = 'Ready';
    try {
        await loadPresetLists();
    } catch (error) {
        initialStatus = error.message;
    }

    initSynth(soundPlayback, setStatus);
    initMelody(melodyPlayback, setStatus);
    initExport(setStatus);
    await restoreSavedState(setStatus);
    bindAppStatePersistence(() => ({
        activeTab: getActiveTabId(),
        forms: collectFormValues(),
        melody: createPersistentMelodyProject(getMelodyProject()),
        melodyUi: getMelodyUiState(),
        sound: getSoundParams(),
        volume: getSharedVolume()
    }));
    setStatus(initialStatus);
});

async function restoreSavedState(setStatus) {
    const saved = await loadAppState();
    if (!saved) {
        return;
    }
    try {
        restoreFormValues(saved.forms);
        if (saved.sound) {
            setSoundParams(saved.sound);
        }
        if (saved.melody) {
            await restoreMelodyProject(saved.melody);
        }
        setMelodyUiState(saved.melodyUi);
        if (Number.isFinite(Number(saved.volume))) {
            setSharedVolume(saved.volume, setStatus);
        }
        restoreActiveTab(saved.activeTab);
    } catch (error) {
        setStatus(error.message);
    }
}

function bindKeyboardPlayback(soundPlayback, melodyPlayback, setStatus) {
    document.addEventListener('keydown', event => {
        if (event.code !== 'Space' || event.repeat || isEditableTarget(event.target)) {
            return;
        }
        const activeTab = document.querySelector('.tab-content.active')?.id;
        const controller = activeTab === 'melody-creation' ? melodyPlayback : activeTab === 'sound-creation' ? soundPlayback : null;
        if (!controller) {
            return;
        }
        event.preventDefault();
        controller.togglePlayPause().catch(error => setStatus(error.message));
    });
}

function isEditableTarget(target) {
    const element = target instanceof Element ? target : null;
    if (!element) {
        return false;
    }
    return Boolean(element.closest('input, select, textarea, [contenteditable="true"]'));
}

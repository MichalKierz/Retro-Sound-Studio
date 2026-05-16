import { getSoundParams, sanitizeFilename } from './params.js';
import { getMelodyProject } from './melody.js';
import { invoke, listen, openDirectory } from './tauri.js';

export function initExport(setStatus) {
    initFolder('sounds', 'sound-export-folder', 'btn-sound-export-folder', setStatus);
    initFolder('melodies', 'melody-export-folder', 'btn-melody-export-folder', setStatus);

    document.getElementById('btn-export-sound')?.addEventListener('click', async () => {
        const format = document.getElementById('export-format')?.value || 'wav';
        const filename = sanitizeFilename(document.getElementById('sound-export-name')?.value, 'retro_sound');
        const folder = document.getElementById('sound-export-folder')?.value || null;
        const options = readExportOptions('sound');
        await runExport('sounds', 'sound-export-progress', 'sound-export-progress-label', 'btn-export-sound', setStatus, async () => {
            return invoke('export_sound', { params: getSoundParams(), format, filename, folder, options });
        }, 'Sound');
    });

    document.getElementById('btn-export-melody')?.addEventListener('click', async () => {
        const format = document.getElementById('export-melody-format')?.value || 'wav';
        const filename = sanitizeFilename(document.getElementById('melody-export-name')?.value, 'retro_melody');
        const folder = document.getElementById('melody-export-folder')?.value || null;
        const options = readExportOptions('melody');
        await runExport('melodies', 'melody-export-progress', 'melody-export-progress-label', 'btn-export-melody', setStatus, async () => {
            return invoke('export_melody', { project: getMelodyProject(), format, filename, folder, options });
        }, 'Melody');
    });
}

async function initFolder(kind, inputId, buttonId, setStatus) {
    const input = document.getElementById(inputId);
    if (input) {
        try {
            input.value = await invoke('default_export_dir', { kind });
        } catch (error) {
            setStatus(error.message);
        }
    }
    document.getElementById(buttonId)?.addEventListener('click', async () => {
        try {
            const folder = await openDirectory(input?.value || await invoke('default_export_dir', { kind }));
            if (folder && input) {
                input.value = folder;
            }
        } catch (error) {
            setStatus(error.message);
        }
    });
}

async function runExport(kind, progressId, labelId, buttonId, setStatus, action, label) {
    const progress = document.getElementById(progressId);
    const progressLabel = document.getElementById(labelId);
    const button = document.getElementById(buttonId);
    let unlisten = null;
    if (button) {
        button.disabled = true;
    }
    setProgress(progress, progressLabel, 0, 'Queued');
    try {
        unlisten = await listen('export-progress', event => {
            const payload = event.payload || {};
            if (payload.kind === kind) {
                setProgress(progress, progressLabel, payload.percent, payload.label);
            }
        });
        const path = await action();
        setProgress(progress, progressLabel, 100, 'Done');
        setStatus(`${label} exported to ${path}`);
    } catch (error) {
        setProgress(progress, progressLabel, 0, 'Failed');
        setStatus(error.message);
    } finally {
        if (typeof unlisten === 'function') {
            unlisten();
        }
        if (button) {
            button.disabled = false;
        }
    }
}

function setProgress(progress, label, value, text) {
    if (progress) {
        progress.value = value;
    }
    if (label) {
        label.textContent = text;
    }
}

function readExportOptions(prefix) {
    const value = id => document.getElementById(`${prefix}-${id}`)?.value;
    return {
        wav_bit_depth: value('wav-bit-depth') || '16',
        normalize_peak: value('wav-normalize') !== 'off',
        mp3_bitrate: numberValue(value('mp3-bitrate'), 192),
        mp3_quality: numberValue(value('mp3-quality'), 2),
        ogg_quality: numberValue(value('ogg-quality'), 6),
        ogg_mode: value('ogg-mode') || 'quality',
        ogg_bitrate: numberValue(value('ogg-bitrate'), 128000),
        midi_ticks_per_beat: numberValue(value('midi-ticks'), 480),
        midi_note_beats: numberValue(value('midi-note-beats'), 2),
        midi_velocity_scale: numberValue(value('midi-velocity'), 100)
    };
}

function numberValue(value, fallback) {
    const number = Number(value);
    return Number.isFinite(number) ? number : fallback;
}

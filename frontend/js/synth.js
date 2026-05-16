import { getSoundParams, applySoundParams, bindSoundParamInputs, updateValueLabels } from './params.js';
import { getPresetDirectory, loadPresetLists, loadSoundPreset, saveSoundPresetFile } from './presets.js';
import { invoke, openFile, saveFile } from './tauri.js';
import { drawLiveWaveform, drawSoundVisuals, liveWaveformResolution } from './visualizer.js';

let previewTimer = null;
let liveWaveformTimer = null;

export function initSynth(playback, setStatus) {
    updateValueLabels();
    drawSoundVisuals(getSoundParams());
    scheduleLiveWaveform(getSoundParams());

    bindSoundParamInputs(params => {
        drawSoundVisuals(params);
        scheduleLiveWaveform(params);
        window.clearTimeout(previewTimer);
        if (playback?.isPlaying()) {
            previewTimer = window.setTimeout(() => {
                playCurrentSound(playback, setStatus, 'Live sound updated').catch(error => setStatus(error.message));
            }, 180);
        }
    });

    document.getElementById('btn-load-sound-preset')?.addEventListener('click', async () => {
        const select = document.getElementById('sound-preset');
        if (!select?.value) {
            setStatus('Select a sound preset first');
            return;
        }
        try {
            const preset = await loadSoundPreset(select.value);
            applySoundParams(preset);
            drawSoundVisuals(getSoundParams());
            scheduleLiveWaveform(getSoundParams());
            if (playback?.isPlaying()) {
                await playCurrentSound(playback, setStatus, 'Live sound preset updated');
            }
            setStatus(`Loaded sound preset ${select.options[select.selectedIndex].textContent}`);
        } catch (error) {
            setStatus(error.message);
        }
    });

    document.getElementById('btn-import-sound-preset')?.addEventListener('click', async () => {
        try {
            const defaultPath = await getPresetDirectory('sounds');
            const path = await openFile({
                multiple: false,
                defaultPath,
                filters: [{ name: 'Retro Sound Preset', extensions: ['json'] }]
            });
            if (!path) {
                return;
            }
            const preset = await invoke('load_sound_preset_file', { path });
            applySoundParams(preset);
            drawSoundVisuals(getSoundParams());
            scheduleLiveWaveform(getSoundParams());
            setStatus('Imported external sound preset');
        } catch (error) {
            setStatus(error.message);
        }
    });

    document.getElementById('btn-save-sound-preset')?.addEventListener('click', async () => {
        try {
            const defaultPath = joinPath(await getPresetDirectory('sounds'), 'custom_sound.json');
            const path = await saveFile({
                defaultPath,
                filters: [{ name: 'Retro Sound Preset', extensions: ['json'] }]
            });
            if (!path) {
                return;
            }
            const saved = await saveSoundPresetFile(path, getSoundParams());
            await loadPresetLists();
            const select = document.getElementById('sound-preset');
            if (select) {
                select.value = saved.id;
            }
        } catch (error) {
            setStatus(error.message);
        }
    });
}

export async function playCurrentSound(playback, setStatus, message = '') {
    const params = getSoundParams();
    const info = await invoke('play_sound', { params });
    playback?.setInfo(info);
    setStatus(message);
    return info;
}

export function setSoundParams(params) {
    applySoundParams(params);
    drawSoundVisuals(getSoundParams());
    scheduleLiveWaveform(getSoundParams());
}

export { getSoundParams };

function scheduleLiveWaveform(params) {
    window.clearTimeout(liveWaveformTimer);
    liveWaveformTimer = window.setTimeout(async () => {
        try {
            const waveform = await invoke('render_sound_waveform', { params, resolution: liveWaveformResolution() });
            drawLiveWaveform(waveform);
        } catch (_error) {
            drawLiveWaveform(null);
        }
    }, 120);
}

function joinPath(directory, filename) {
    const separator = String(directory).includes('\\') ? '\\' : '/';
    return `${String(directory).replace(/[\\/]+$/, '')}${separator}${filename}`;
}

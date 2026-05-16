import { invoke } from './tauri.js';

let sharedVolume = 5;

export function createPlaybackController(kind, playCallback, setStatus, progressCallback = null) {
    const root = document.querySelector(`[data-playback="${kind}"]`);
    let duration = 0;
    let startSeconds = 0;
    let startedAt = 0;
    let frame = null;
    let loopEnabled = false;
    let playing = false;
    let paused = false;
    let visualStartSeconds = 0;
    let visualStartedAt = 0;
    let timerSerial = 0;

    function setInfo(info) {
        duration = Number(info?.duration_seconds || 0);
        startSeconds = Number(info?.start_seconds || 0);
        visualStartSeconds = startSeconds;
        visualStartedAt = performance.now();
        loopEnabled = Boolean(info?.loop_enabled);
        paused = Boolean(info?.paused);
        playing = duration > 0;
        startedAt = visualStartedAt;
        setLoopButton();
        startTimer();
        updateView(startSeconds);
    }

    function stopTimer() {
        timerSerial += 1;
        if (frame) {
            cancelAnimationFrame(frame);
            frame = null;
        }
        playing = false;
        paused = false;
        loopEnabled = false;
        startSeconds = 0;
        visualStartSeconds = 0;
        setLoopButton();
        updateView(0);
    }

    function startTimer() {
        timerSerial += 1;
        const serial = timerSerial;
        if (frame) {
            cancelAnimationFrame(frame);
        }
        const tick = () => {
            if (serial !== timerSerial) {
                return;
            }
            if (!duration || paused) {
                updateView(visualStartSeconds);
                frame = requestAnimationFrame(tick);
                return;
            }
            const current = visibleSeconds();
            if (!loopEnabled && current >= duration) {
                playing = false;
                frame = null;
                updateView(duration);
                return;
            }
            updateView(current);
            frame = requestAnimationFrame(tick);
        };
        frame = requestAnimationFrame(tick);
    }

    async function play() {
        const info = await playCallback();
        setInfo(info);
        return info;
    }

    async function pause() {
        const result = await invoke('pause_playback');
        paused = Boolean(result);
        if (paused) {
            startSeconds = currentSeconds();
            visualStartSeconds = startSeconds;
            if (frame) {
                cancelAnimationFrame(frame);
                frame = null;
            }
        } else {
            startedAt = performance.now();
            visualStartedAt = startedAt;
            startTimer();
        }
    }

    async function togglePlayPause() {
        if (!playing) {
            return play();
        }
        return pause();
    }

    async function restart() {
        const info = await invoke('restart_playback');
        setInfo(info);
    }

    async function stop() {
        await invoke('stop_playback');
        stopTimer();
    }

    async function toggleLoop() {
        const position = currentSeconds();
        const nextLoop = !loopEnabled;
        if (nextLoop && !playing) {
            const played = await play();
            if (Number(played?.duration_seconds || 0) <= 0) {
                return played;
            }
        }
        const info = await invoke('set_loop', { loopStatus: nextLoop, positionSeconds: position });
        if (!nextLoop) {
            loopEnabled = false;
            stopTimer();
            setLoopButton();
            return info;
        }
        if (Number(info?.duration_seconds || 0) > 0) {
            setInfo({
                ...info,
                loop_enabled: true,
                paused: false,
                start_seconds: Number(info?.start_seconds ?? position)
            });
        } else {
            loopEnabled = true;
            setLoopButton();
            updateView(normalizePosition(position));
        }
        return info;
    }

    async function seek(value) {
        if (!duration) {
            return;
        }
        const position = Number(value) / 1000 * duration;
        const info = await invoke('seek_playback', { positionSeconds: position });
        setInfo(info);
    }

    function updateView(current) {
        if (!root) {
            if (progressCallback) {
                progressCallback(current, duration);
            }
            return;
        }
        const seekInput = root.querySelector('[data-role="seek"]');
        const time = root.querySelector('[data-role="time"]');
        if (seekInput) {
            seekInput.value = duration ? Math.round(current / duration * 1000) : 0;
        }
        if (time) {
            time.textContent = `${current.toFixed(2)} / ${duration.toFixed(2)}`;
        }
        if (progressCallback) {
            progressCallback(current, duration);
        }
    }

    function currentSeconds() {
        if (!playing || paused) {
            return startSeconds;
        }
        const elapsed = (performance.now() - startedAt) / 1000;
        const current = startSeconds + elapsed;
        if (loopEnabled && duration) {
            return current % duration;
        }
        return duration ? Math.min(duration, current) : 0;
    }

    function visibleSeconds() {
        if (!playing || paused) {
            return visualStartSeconds;
        }
        const elapsed = (performance.now() - visualStartedAt) / 1000;
        const current = visualStartSeconds + elapsed;
        return loopEnabled && duration ? current % duration : Math.min(duration, current);
    }

    function normalizePosition(value) {
        const position = Number(value || 0);
        if (!duration) {
            return 0;
        }
        return loopEnabled ? position % duration : Math.min(duration, position);
    }

    function setLoopButton() {
        const button = root?.querySelector('[data-action="loop"]');
        if (button) {
            button.setAttribute('aria-pressed', String(loopEnabled));
        }
    }

    function setVolume(value) {
        setSharedVolume(value, setStatus);
    }

    if (root) {
        root.querySelector('[data-action="play"]')?.addEventListener('click', () => play().catch(error => setStatus(error.message)));
        root.querySelector('[data-action="pause"]')?.addEventListener('click', () => pause().catch(error => setStatus(error.message)));
        root.querySelector('[data-action="restart"]')?.addEventListener('click', () => restart().catch(error => setStatus(error.message)));
        root.querySelector('[data-action="stop"]')?.addEventListener('click', () => stop().catch(error => setStatus(error.message)));
        root.querySelector('[data-action="loop"]')?.addEventListener('click', () => toggleLoop().catch(error => setStatus(error.message)));
        root.querySelector('[data-role="seek"]')?.addEventListener('change', event => seek(event.target.value).catch(error => setStatus(error.message)));
        root.querySelector('[data-role="volume"]')?.addEventListener('input', event => setVolume(event.target.value));
    }

    updateAllVolumeControls();

    return {
        play,
        pause,
        stop,
        togglePlayPause,
        setInfo,
        stopTimer,
        isPlaying: () => playing && !paused
    };
}

export function updateAllVolumeControls() {
    const decibels = sharedVolume <= 0 ? '-inf' : (20 * Math.log10(sharedVolume / 100)).toFixed(1);
    document.querySelectorAll('[data-role="volume"]').forEach(input => {
        input.value = String(sharedVolume);
    });
    document.querySelectorAll('[data-role="volume-readout"]').forEach(readout => {
        readout.textContent = `${sharedVolume.toFixed(0)}% / ${decibels} dB`;
    });
}

export function getSharedVolume() {
    return sharedVolume;
}

export function setSharedVolume(value, setStatus = () => {}) {
    sharedVolume = Math.max(0, Math.min(100, Number(value)));
    updateAllVolumeControls();
    invoke('set_volume', { volume: sharedVolume }).catch(error => setStatus(error.message));
}

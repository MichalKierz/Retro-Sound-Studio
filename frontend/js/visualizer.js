export function drawSoundVisuals(params) {
    drawOscillatorPreview(document.getElementById('waveform-canvas'), params);
    drawEnvelopes(document.getElementById('envelope-canvas'), params);
}

export function drawLiveWaveform(waveform) {
    const canvas = document.getElementById('live-waveform-canvas');
    if (!canvas) {
        return;
    }
    resizeCanvasToDisplaySize(canvas);
    const ctx = canvas.getContext('2d');
    const width = canvas.width;
    const height = canvas.height;
    ctx.clearRect(0, 0, width, height);
    fillBackground(ctx, width, height);
    drawGrid(ctx, width, height, 16, 8);
    drawZeroLine(ctx, width, height);
    const points = normalizeWaveformPoints(waveform);
    if (!points.length) {
        return;
    }
    ctx.fillStyle = 'rgba(255, 215, 106, 0.24)';
    ctx.beginPath();
    for (let index = 0; index < points.length; index += 1) {
        const point = points[index];
        const x = pointToX(index, points.length, width);
        const y = sampleToY(point.max, height);
        if (index === 0) {
            ctx.moveTo(x, y);
        } else {
            ctx.lineTo(x, y);
        }
    }
    for (let index = points.length - 1; index >= 0; index -= 1) {
        const point = points[index];
        ctx.lineTo(pointToX(index, points.length, width), sampleToY(point.min, height));
    }
    ctx.closePath();
    ctx.fill();

    ctx.strokeStyle = '#ffd76a';
    ctx.lineWidth = Math.max(1.5, window.devicePixelRatio || 1);
    drawPointLine(ctx, points, width, height, point => point.max);
    drawPointLine(ctx, points, width, height, point => point.min);

    ctx.strokeStyle = '#69d8ff';
    ctx.lineWidth = Math.max(1, window.devicePixelRatio || 1);
    drawPointLine(ctx, points, width, height, point => point.rms);
    drawPointLine(ctx, points, width, height, point => -point.rms);
}

export function liveWaveformResolution() {
    const canvas = document.getElementById('live-waveform-canvas');
    const displayWidth = canvas?.getBoundingClientRect().width || canvas?.width || 1600;
    const ratio = window.devicePixelRatio || 1;
    return Math.round(Math.max(2048, Math.min(16384, displayWidth * ratio * 4)));
}

function drawPointLine(ctx, points, width, height, valueFn) {
    ctx.beginPath();
    for (let index = 0; index < points.length; index += 1) {
        const x = pointToX(index, points.length, width);
        const y = sampleToY(valueFn(points[index]), height);
        if (index === 0) {
            ctx.moveTo(x, y);
        } else {
            ctx.lineTo(x, y);
        }
    }
    ctx.stroke();
}

function normalizeWaveformPoints(waveform) {
    if (Array.isArray(waveform)) {
        return waveform.map(value => {
            const sample = clampSample(value);
            return { min: sample, max: sample, rms: Math.abs(sample) };
        });
    }
    return (waveform?.points || []).map(point => ({
        min: clampSample(point.min),
        max: clampSample(point.max),
        rms: Math.max(0, Math.min(1, Number(point.rms || 0)))
    }));
}

function pointToX(index, count, width) {
    return count <= 1 ? 0 : index / (count - 1) * (width - 1);
}

function sampleToY(sample, height) {
    return height / 2 - clampSample(sample) * height * 0.44;
}

function clampSample(value) {
    return Math.max(-1, Math.min(1, Number(value || 0)));
}

function resizeCanvasToDisplaySize(canvas) {
    const ratio = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    const width = Math.max(720, Math.round((rect.width || canvas.width) * ratio));
    const height = Math.max(220, Math.round((rect.height || canvas.height) * ratio));
    if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
    }
}

function drawOscillatorPreview(canvas, params) {
    if (!canvas) {
        return;
    }
    const ctx = canvas.getContext('2d');
    const width = canvas.width;
    const height = canvas.height;
    ctx.clearRect(0, 0, width, height);
    fillBackground(ctx, width, height);
    drawGrid(ctx, width, height, 12, 6);
    ctx.strokeStyle = '#6cff8f';
    ctx.lineWidth = 3;
    ctx.beginPath();
    const baseCycles = 4;
    let phase = 0;
    let subPhase = 0;
    for (let x = 0; x < width; x += 1) {
        const time = x / Math.max(1, width - 1);
        const modulation = lfoModulationAt(params, time);
        const pitchScale = Math.pow(2, modulation.pitch / 12);
        const pulseWidth = clamp(effectivePulseWidth(params) + modulation.pwm * 0.4, 0.1, 0.9);
        const amp = Math.max(0, 1 - Math.min(1, Math.abs(modulation.amp)));
        phase = (phase + baseCycles * pitchScale / width) % 1;
        subPhase = (subPhase + baseCycles * 0.5 * pitchScale / width) % 1;
        const value = oscillatorValue(params.waveform, phase, pulseWidth, x, params.wavetable);
        const subValue = params.sub_waveform === 'none' ? 0 : oscillatorValue(params.sub_waveform, subPhase, 0.5, x, 'none');
        const subLevel = clamp(Number(params.sub_level || 0), 0, 1);
        const mixed = (value * (1 - subLevel) + subValue * subLevel) * amp;
        const y = height / 2 - mixed * height * 0.34;
        if (x === 0) {
            ctx.moveTo(x, y);
        } else {
            ctx.lineTo(x, y);
        }
    }
    ctx.stroke();
    drawCutoffModulation(ctx, width, height, params);
    drawFilterLine(ctx, width, height, params);
}

function drawEnvelopes(canvas, params) {
    if (!canvas) {
        return;
    }
    const ctx = canvas.getContext('2d');
    const width = canvas.width;
    const height = canvas.height;
    ctx.clearRect(0, 0, width, height);
    fillBackground(ctx, width, height);
    drawGrid(ctx, width, height, 10, 5);
    drawEnvelope(ctx, width, height, params.attack, params.decay, params.sustain, params.release, '#6cff8f');
    drawEnvelope(ctx, width, height, params.filter_attack, params.filter_decay, params.filter_sustain, params.filter_release, '#69d8ff');
}

function oscillatorValue(waveform, phase, pulseWidth, x, wavetable) {
    if (waveform === 'triangle') {
        return phase < 0.5 ? phase * 4 - 1 : 3 - phase * 4;
    }
    if (waveform === 'sawtooth') {
        return phase * 2 - 1;
    }
    if (waveform === 'sine') {
        return Math.sin(phase * Math.PI * 2);
    }
    if (waveform === 'noise') {
        return pseudoNoise(x);
    }
    if (waveform === 'wavetable') {
        return wavetableValue(phase, wavetable);
    }
    return phase < pulseWidth ? 1 : -1;
}

function effectivePulseWidth(params) {
    if (params.duty_mode === '12_5') return 0.125;
    if (params.duty_mode === '25') return 0.25;
    if (params.duty_mode === '50') return 0.5;
    if (params.duty_mode === '75') return 0.75;
    return Number(params.pulse_width || 0.5);
}

function wavetableValue(phase, name) {
    const tables = {
        gb_organ: [0, 0.45, 0.8, 0.95, 0.7, 0.35, 0.1, -0.05, -0.1, -0.35, -0.7, -0.95, -0.8, -0.45, 0, 0.2],
        gb_bell: [0, 0.9, 0.3, 0.75, -0.15, 0.4, -0.6, 0.15, -0.9, -0.2, -0.5, 0.05, -0.25, 0.35, -0.1, 0],
        gb_saw: [-1, -0.87, -0.73, -0.6, -0.47, -0.33, -0.2, -0.07, 0.07, 0.2, 0.33, 0.47, 0.6, 0.73, 0.87, 1],
        gb_pulse: [1, 1, 1, 1, 0.65, 0.2, -0.2, -0.65, -1, -1, -1, -1, -0.65, -0.2, 0.2, 0.65]
    };
    const table = tables[name] || tables.gb_organ;
    const position = phase * table.length;
    const left = Math.floor(position) % table.length;
    const right = (left + 1) % table.length;
    const mix = position - Math.floor(position);
    return table[left] * (1 - mix) + table[right] * mix;
}

function lfoModulationAt(params, time) {
    const result = { pitch: 0, cutoff: 0, pwm: 0, amp: 0 };
    applyLfo(params, 'lfo1', time, result);
    applyLfo(params, 'lfo2', time, result);
    return result;
}

function applyLfo(params, prefix, time, result) {
    const routing = params[`${prefix}_routing`];
    const depth = Number(params[`${prefix}_depth`] || 0);
    const speed = Number(params[`${prefix}_speed`] || 0);
    if (!routing || routing === 'none' || !depth || !Number.isFinite(depth)) {
        return;
    }
    const value = lfoValue(params[`${prefix}_waveform`], time * speed) * depth;
    if (routing === 'pitch') {
        result.pitch += value;
    } else if (routing === 'cutoff') {
        result.cutoff += value * 5000;
    } else if (routing === 'pwm') {
        result.pwm += value;
    } else if (routing === 'amp') {
        result.amp += value;
    }
}

function lfoValue(waveform, phase) {
    const normalized = ((phase % 1) + 1) % 1;
    if (waveform === 'square') {
        return normalized < 0.5 ? 1 : -1;
    }
    if (waveform === 'triangle') {
        return normalized < 0.5 ? 4 * normalized - 1 : 3 - 4 * normalized;
    }
    if (waveform === 'sawtooth') {
        return 2 * normalized - 1;
    }
    return Math.sin(normalized * Math.PI * 2);
}

function drawCutoffModulation(ctx, width, height, params) {
    if (params.filter_type === 'none') {
        return;
    }
    const hasCutoffLfo = params.lfo1_routing === 'cutoff' || params.lfo2_routing === 'cutoff';
    if (!hasCutoffLfo) {
        return;
    }
    ctx.strokeStyle = 'rgba(255, 215, 106, 0.72)';
    ctx.lineWidth = 2;
    ctx.beginPath();
    for (let x = 0; x < width; x += 1) {
        const time = x / Math.max(1, width - 1);
        const cutoff = clamp(Number(params.filter_cutoff || 1200) + lfoModulationAt(params, time).cutoff, 20, 20000);
        const normalizedCutoff = Math.log10(cutoff / 20) / Math.log10(20000 / 20);
        const y = height - 16 - normalizedCutoff * (height - 32);
        if (x === 0) {
            ctx.moveTo(x, y);
        } else {
            ctx.lineTo(x, y);
        }
    }
    ctx.stroke();
}

function clamp(value, min, max) {
    const numeric = Number(value);
    return Math.max(min, Math.min(max, Number.isFinite(numeric) ? numeric : min));
}

function pseudoNoise(x) {
    const value = Math.sin(x * 12.9898) * 43758.5453;
    return (value - Math.floor(value)) * 2 - 1;
}

function drawEnvelope(ctx, width, height, attack, decay, sustain, release, color) {
    const total = Math.max(attack + decay + release + 0.5, 0.5);
    const sustainTime = Math.max(total - attack - decay - release, 0.12);
    const points = [
        [0, 0],
        [attack, 1],
        [attack + decay, sustain],
        [attack + decay + sustainTime, sustain],
        [attack + decay + sustainTime + release, 0]
    ];
    const maxTime = points[points.length - 1][0] || 1;
    ctx.strokeStyle = color;
    ctx.lineWidth = 3;
    ctx.beginPath();
    points.forEach(([time, level], index) => {
        const x = 16 + (time / maxTime) * (width - 32);
        const y = height - 18 - level * (height - 36);
        if (index === 0) {
            ctx.moveTo(x, y);
        } else {
            ctx.lineTo(x, y);
        }
    });
    ctx.stroke();
}

function drawFilterLine(ctx, width, height, params) {
    if (params.filter_type === 'none') {
        return;
    }
    const cutoff = clamp(Number(params.filter_cutoff || 1200), 20, 20000);
    const normalizedCutoff = Math.log10(cutoff / 20) / Math.log10(20000 / 20);
    const x = normalizedCutoff * width;
    ctx.strokeStyle = '#ffd76a';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x, 12);
    ctx.lineTo(x, height - 12);
    ctx.stroke();
}

function fillBackground(ctx, width, height) {
    ctx.fillStyle = '#07090a';
    ctx.fillRect(0, 0, width, height);
}

function drawZeroLine(ctx, width, height) {
    ctx.strokeStyle = '#32443c';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, height / 2);
    ctx.lineTo(width, height / 2);
    ctx.stroke();
}

function drawGrid(ctx, width, height, columns, rows) {
    ctx.strokeStyle = '#1d2a27';
    ctx.lineWidth = 1;
    for (let column = 1; column < columns; column += 1) {
        const x = width * column / columns;
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, height);
        ctx.stroke();
    }
    for (let row = 1; row < rows; row += 1) {
        const y = height * row / rows;
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(width, y);
        ctx.stroke();
    }
}

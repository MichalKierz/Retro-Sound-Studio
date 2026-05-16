const numericFields = {
    pulse_width: { type: 'float', digits: 2, unit: '' },
    duty_sequence_rate: { type: 'float', digits: 2, unit: ' Hz' },
    noise_period: { type: 'int', digits: 0, unit: 'x' },
    sub_level: { type: 'float', digits: 2, unit: '' },
    filter_cutoff: { type: 'float', digits: 0, unit: ' Hz' },
    filter_resonance: { type: 'float', digits: 2, unit: '' },
    filter_env_depth: { type: 'float', digits: 2, unit: '' },
    frequency: { type: 'float', digits: 0, unit: ' Hz' },
    sweep_amount: { type: 'float', digits: 0, unit: ' st' },
    sweep_time: { type: 'float', digits: 2, unit: ' s' },
    pitch_env_amount: { type: 'float', digits: 0, unit: ' st' },
    pitch_env_decay: { type: 'float', digits: 2, unit: ' s' },
    portamento: { type: 'float', digits: 2, unit: ' s' },
    attack: { type: 'float', digits: 2, unit: ' s' },
    decay: { type: 'float', digits: 2, unit: ' s' },
    sustain: { type: 'float', digits: 2, unit: '' },
    release: { type: 'float', digits: 2, unit: ' s' },
    filter_attack: { type: 'float', digits: 2, unit: ' s' },
    filter_decay: { type: 'float', digits: 2, unit: ' s' },
    filter_sustain: { type: 'float', digits: 2, unit: '' },
    filter_release: { type: 'float', digits: 2, unit: ' s' },
    lfo1_speed: { type: 'float', digits: 2, unit: ' Hz' },
    lfo1_depth: { type: 'float', digits: 2, unit: '' },
    lfo2_speed: { type: 'float', digits: 2, unit: ' Hz' },
    lfo2_depth: { type: 'float', digits: 2, unit: '' },
    delay_time: { type: 'float', digits: 2, unit: ' s' },
    delay_feedback: { type: 'float', digits: 2, unit: '' },
    delay_mix: { type: 'float', digits: 2, unit: '' },
    bit_depth: { type: 'int', digits: 0, unit: ' bit' },
    sample_rate_reduction: { type: 'int', digits: 0, unit: 'x' },
    distortion_drive: { type: 'float', digits: 2, unit: 'x' },
    arp_speed: { type: 'float', digits: 2, unit: ' Hz' },
    retrigger_rate: { type: 'float', digits: 2, unit: ' Hz' },
    gate_length: { type: 'float', digits: 2, unit: '' },
    pan: { type: 'float', digits: 2, unit: '' },
    auto_pan_speed: { type: 'float', digits: 2, unit: ' Hz' },
    auto_pan_depth: { type: 'float', digits: 2, unit: '' }
};

const stringFields = [
    'waveform',
    'sub_waveform',
    'duty_mode',
    'duty_sequence',
    'noise_color',
    'noise_mode',
    'wavetable',
    'filter_type',
    'pitch_env_curve',
    'lfo1_waveform',
    'lfo1_routing',
    'lfo2_waveform',
    'lfo2_routing',
    'distortion_type',
    'arp_chord'
];

export function getSoundParams() {
    const params = {};
    for (const [id, config] of Object.entries(numericFields)) {
        const element = document.getElementById(id);
        const value = element ? element.value : '0';
        params[id] = config.type === 'int' ? parseInt(value, 10) : parseFloat(value);
    }
    for (const id of stringFields) {
        const element = document.getElementById(id);
        params[id] = element ? element.value : '';
    }
    params.sample_rate = 44100;
    return params;
}

export function applySoundParams(params) {
    if (!params) {
        return;
    }
    for (const id of Object.keys(numericFields)) {
        const element = document.getElementById(id);
        if (element && params[id] !== undefined) {
            element.value = params[id];
        }
    }
    for (const id of stringFields) {
        const element = document.getElementById(id);
        if (element && params[id] !== undefined) {
            element.value = params[id];
        }
    }
    updateValueLabels();
    updateControlAvailability();
}

export function bindSoundParamInputs(callback) {
    const selector = Object.keys(numericFields).concat(stringFields).map(id => `#${id}`).join(',');
    document.querySelectorAll(selector).forEach(element => {
        element.addEventListener('input', () => {
            updateValueLabels();
            updateControlAvailability();
            callback(getSoundParams());
        });
    });
}

export function updateValueLabels() {
    refreshValueLabels();
    updateControlAvailability();
}

function refreshValueLabels() {
    for (const [id, config] of Object.entries(numericFields)) {
        const element = document.getElementById(id);
        const label = document.getElementById(`${id}-val`);
        if (!element || !label) {
            continue;
        }
        const number = config.type === 'int' ? parseInt(element.value, 10) : parseFloat(element.value);
        label.textContent = `${number.toFixed(config.digits)}${config.unit}`;
    }
}

export function sanitizeFilename(value, fallback) {
    const cleaned = String(value || '')
        .replace(/[^a-z0-9_-]/gi, '_')
        .replace(/_+/g, '_')
        .replace(/^_+|_+$/g, '')
        .slice(0, 64);
    return cleaned || fallback;
}

function updateControlAvailability() {
    applyEffectiveValues();
    const params = getSoundParams();
    setRoutingOptionDisabled('lfo1_routing', 'cutoff', params.filter_type === 'none');
    setRoutingOptionDisabled('lfo1_routing', 'pwm', params.waveform !== 'square');
    setRoutingOptionDisabled('lfo2_routing', 'cutoff', params.filter_type === 'none');
    setRoutingOptionDisabled('lfo2_routing', 'pwm', params.waveform !== 'square');
    setChordOptionsDisabled(Number(params.arp_speed || 0) <= 0);
    const updated = getSoundParams();
    const squareDisabled = updated.waveform !== 'square';
    setDisabled('duty_mode', squareDisabled);
    setDisabled('pulse_width', squareDisabled || updated.duty_mode !== 'free');
    setDisabled('duty_sequence', squareDisabled);
    setDisabled('duty_sequence_rate', squareDisabled || updated.duty_sequence === 'none');
    setDisabled('noise_color', updated.waveform !== 'noise');
    setDisabled('noise_mode', updated.waveform !== 'noise');
    setDisabled('noise_period', updated.waveform !== 'noise');
    setDisabled('wavetable', updated.waveform !== 'wavetable');
    setDisabled('sub_level', updated.sub_waveform === 'none');
    const filterDisabled = updated.filter_type === 'none';
    const filterEnvelopeDisabled = filterDisabled || Math.abs(Number(updated.filter_env_depth || 0)) <= Number.EPSILON;
    ['filter_cutoff', 'filter_resonance', 'filter_env_depth'].forEach(id => setDisabled(id, filterDisabled));
    ['filter_attack', 'filter_decay', 'filter_sustain', 'filter_release'].forEach(id => setDisabled(id, filterEnvelopeDisabled));
    setDisabled('lfo1_waveform', updated.lfo1_routing === 'none');
    setDisabled('lfo1_speed', updated.lfo1_routing === 'none');
    setDisabled('lfo1_depth', updated.lfo1_routing === 'none');
    setDisabled('lfo2_waveform', updated.lfo2_routing === 'none');
    setDisabled('lfo2_speed', updated.lfo2_routing === 'none');
    setDisabled('lfo2_depth', updated.lfo2_routing === 'none');
    const sweepActive = Math.abs(Number(updated.sweep_amount || 0)) > Number.EPSILON && Number(updated.sweep_time || 0) > 0;
    const pitchEnvelopeActive = Math.abs(Number(updated.pitch_env_amount || 0)) > Number.EPSILON && Number(updated.pitch_env_decay || 0) > 0;
    const arpActive = updated.arp_chord !== 'none' && Number(updated.arp_speed || 0) > 0;
    setDisabled('sweep_amount', Number(updated.sweep_time || 0) <= 0);
    setDisabled('pitch_env_amount', Number(updated.pitch_env_decay || 0) <= 0);
    setDisabled('pitch_env_curve', !pitchEnvelopeActive);
    setDisabled('portamento', !sweepActive && !arpActive && !pitchEnvelopeActive);
    setDisabled('delay_time', Number(updated.delay_mix || 0) <= 0);
    setDisabled('delay_feedback', Number(updated.delay_mix || 0) <= 0 || Number(updated.delay_time || 0) <= 0);
    setDisabled('distortion_drive', updated.distortion_type === 'none');
    setDisabled('gate_length', Number(updated.retrigger_rate || 0) <= 0);
    setDisabled('auto_pan_depth', Number(updated.auto_pan_speed || 0) <= 0);
    refreshValueLabels();
}

function applyEffectiveValues() {
    const params = getSoundParams();
    let changed = false;
    const setValue = (id, value) => {
        const element = document.getElementById(id);
        if (element && String(element.value) !== String(value)) {
            element.value = String(value);
            changed = true;
        }
        params[id] = typeof params[id] === 'number' ? Number(value) : String(value);
    };
    const valueOf = id => Number(params[id] || 0);
    const defaultLfoDepth = routing => {
        switch (routing) {
            case 'pitch':
                return 2;
            case 'cutoff':
                return 0.45;
            case 'pwm':
                return 0.35;
            case 'amp':
                return 0.4;
            default:
                return 0;
        }
    };
    const defaultDistortionDrive = type => {
        switch (type) {
            case 'hard_clip':
                return 3;
            case 'soft_clip':
                return 2;
            case 'foldback':
                return 4;
            default:
                return 1;
        }
    };

    if (params.waveform === 'wavetable' && params.wavetable === 'none') {
        setValue('wavetable', 'gb_organ');
    }
    if (params.duty_sequence !== 'none' && valueOf('duty_sequence_rate') <= 0) {
        setValue('duty_sequence_rate', 12);
    }
    if (params.sub_waveform !== 'none' && valueOf('sub_level') <= 0) {
        setValue('sub_level', 0.35);
    }
    if (params.lfo1_routing !== 'none' && valueOf('lfo1_depth') <= 0) {
        setValue('lfo1_depth', defaultLfoDepth(params.lfo1_routing));
    }
    if (params.lfo2_routing !== 'none' && valueOf('lfo2_depth') <= 0) {
        setValue('lfo2_depth', defaultLfoDepth(params.lfo2_routing));
    }
    if (valueOf('sweep_time') > 0 && Math.abs(valueOf('sweep_amount')) <= Number.EPSILON) {
        setValue('sweep_amount', 12);
    }
    if (valueOf('pitch_env_decay') > 0 && Math.abs(valueOf('pitch_env_amount')) <= Number.EPSILON) {
        setValue('pitch_env_amount', 12);
    }
    if (valueOf('delay_mix') > 0 && valueOf('delay_time') <= 0) {
        setValue('delay_time', 0.12);
    }
    if (params.distortion_type !== 'none' && valueOf('distortion_drive') <= 1) {
        setValue('distortion_drive', defaultDistortionDrive(params.distortion_type));
    }
    if (valueOf('arp_speed') > 0 && params.arp_chord === 'none') {
        setValue('arp_chord', 'octave');
    }
    if (valueOf('auto_pan_speed') > 0 && valueOf('auto_pan_depth') <= 0) {
        setValue('auto_pan_depth', 0.65);
    }

    return changed;
}

function setDisabled(id, disabled) {
    const element = document.getElementById(id);
    if (element) {
        element.disabled = disabled;
    }
}

function setRoutingOptionDisabled(selectId, value, disabled) {
    const select = document.getElementById(selectId);
    const option = select?.querySelector(`option[value="${value}"]`);
    if (!select || !option) {
        return;
    }
    option.disabled = disabled;
    if (disabled && select.value === value) {
        select.value = 'none';
    }
}

function setChordOptionsDisabled(disabled) {
    const select = document.getElementById('arp_chord');
    if (!select) {
        return;
    }
    select.querySelectorAll('option').forEach(option => {
        if (option.value !== 'none') {
            option.disabled = disabled;
        }
    });
    if (disabled && select.value !== 'none') {
        select.value = 'none';
    }
}

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const sourcePath = path.join(repoRoot, 'frontend', 'js', 'params.js');
const source = fs.readFileSync(sourcePath, 'utf8').replaceAll('export function ', 'function ');

class FakeOption {
    constructor(value) {
        this.value = value;
        this.disabled = false;
    }
}

class FakeElement {
    constructor(id, value, options = []) {
        this.id = id;
        this.value = String(value);
        this.disabled = false;
        this.textContent = '';
        this.options = options.map(option => new FakeOption(option));
    }

    addEventListener() {}

    querySelector(selector) {
        const match = selector.match(/^option\[value="(.+)"\]$/);
        return match ? this.options.find(option => option.value === match[1]) || null : null;
    }

    querySelectorAll(selector) {
        return selector === 'option' ? this.options : [];
    }
}

const numericDefaults = {
    pulse_width: 0.35,
    duty_sequence_rate: 0,
    noise_period: 1,
    sub_level: 0,
    filter_cutoff: 20000,
    filter_resonance: 0,
    filter_env_depth: 0,
    frequency: 440,
    sweep_amount: 0,
    sweep_time: 0,
    pitch_env_amount: 0,
    pitch_env_decay: 0,
    portamento: 0,
    attack: 0.05,
    decay: 0.1,
    sustain: 0.5,
    release: 0.2,
    filter_attack: 0.05,
    filter_decay: 0.1,
    filter_sustain: 0.5,
    filter_release: 0.2,
    lfo1_speed: 5,
    lfo1_depth: 0,
    lfo2_speed: 1,
    lfo2_depth: 0,
    delay_time: 0,
    delay_feedback: 0,
    delay_mix: 0,
    bit_depth: 16,
    sample_rate_reduction: 1,
    distortion_drive: 1,
    arp_speed: 0,
    retrigger_rate: 0,
    gate_length: 1,
    pan: 0,
    auto_pan_speed: 0,
    auto_pan_depth: 0
};

const selectDefaults = {
    waveform: ['square', ['square', 'triangle', 'sawtooth', 'sine', 'noise', 'wavetable']],
    sub_waveform: ['none', ['none', 'square', 'triangle', 'sawtooth', 'sine']],
    duty_mode: ['free', ['free', '12_5', '25', '50', '75']],
    duty_sequence: ['none', ['none', 'classic_steps', 'pulse_train', 'skewed_ladder']],
    noise_color: ['white', ['white', 'pink', 'brown']],
    noise_mode: ['lfsr', ['lfsr', 'periodic', 'metallic']],
    wavetable: ['none', ['none', 'gb_organ', 'gb_bell', 'gb_saw', 'gb_pulse']],
    filter_type: ['none', ['none', 'lowpass', 'highpass', 'bandpass']],
    pitch_env_curve: ['exponential', ['exponential', 'linear']],
    lfo1_waveform: ['sine', ['sine', 'square', 'triangle', 'sawtooth']],
    lfo1_routing: ['none', ['none', 'pitch', 'cutoff', 'pwm', 'amp']],
    lfo2_waveform: ['sine', ['sine', 'square', 'triangle', 'sawtooth']],
    lfo2_routing: ['none', ['none', 'pitch', 'cutoff', 'pwm', 'amp']],
    distortion_type: ['none', ['none', 'hard_clip', 'soft_clip', 'foldback']],
    arp_chord: ['none', ['none', 'major', 'minor', 'octave', 'fifth']]
};

function createDocument() {
    const elements = new Map();
    Object.entries(numericDefaults).forEach(([id, value]) => {
        elements.set(id, new FakeElement(id, value));
        elements.set(`${id}-val`, new FakeElement(`${id}-val`, ''));
    });
    Object.entries(selectDefaults).forEach(([id, [value, options]]) => {
        elements.set(id, new FakeElement(id, value, options));
    });
    return {
        getElementById(id) {
            return elements.get(id) || null;
        },
        querySelectorAll(selector) {
            return selector
                .split(',')
                .map(part => part.trim().replace(/^#/, ''))
                .map(id => elements.get(id))
                .filter(Boolean);
        }
    };
}

function createHarness() {
    const document = createDocument();
    const factory = new Function('document', `${source}; return { getSoundParams, updateValueLabels };`);
    const api = factory(document);
    return { document, api };
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function runCase(name, action, assertion) {
    const { document, api } = createHarness();
    const set = (id, value) => {
        const element = document.getElementById(id);
        assert(element, `${name}: missing #${id}`);
        element.value = String(value);
    };
    action(set, document);
    api.updateValueLabels();
    assertion(api.getSoundParams(), document);
}

runCase('duty sequence auto-enables speed', set => {
    set('duty_sequence', 'classic_steps');
}, (params, document) => {
    assert(params.duty_sequence_rate > 0, 'Duty Sequence must assign a non-zero Duty Seq Speed.');
    assert(document.getElementById('duty_sequence_rate').disabled === false, 'Duty Seq Speed must stay editable after sequence activation.');
});

runCase('free duty starts distinct from fixed 50%', () => {}, params => {
    assert(params.duty_mode === 'free', 'Default Duty Mode should remain Free.');
    assert(Math.abs(params.pulse_width - 0.5) > Number.EPSILON, 'Free Pulse Width must not duplicate fixed 50% at startup.');
});

runCase('sub oscillator auto-enables level', set => {
    set('sub_waveform', 'square');
}, params => {
    assert(params.sub_level > 0, 'Sub Osc Waveform must assign a non-zero Sub Osc Level.');
});

runCase('LFO 1 routing auto-enables depth', set => {
    set('lfo1_routing', 'pitch');
}, (params, document) => {
    assert(params.lfo1_depth > 0, 'LFO 1 Routing must assign a non-zero LFO 1 Depth.');
    assert(document.getElementById('lfo1_waveform').disabled === false, 'LFO 1 Waveform must be enabled after active routing.');
    assert(document.getElementById('lfo1_speed').disabled === false, 'LFO 1 Speed must be enabled after active routing.');
});

runCase('LFO 2 routing auto-enables depth', set => {
    set('lfo2_routing', 'amp');
}, (params, document) => {
    assert(params.lfo2_depth > 0, 'LFO 2 Routing must assign a non-zero LFO 2 Depth.');
    assert(document.getElementById('lfo2_waveform').disabled === false, 'LFO 2 Waveform must be enabled after active routing.');
    assert(document.getElementById('lfo2_speed').disabled === false, 'LFO 2 Speed must be enabled after active routing.');
});

runCase('sweep time auto-enables amount', set => {
    set('sweep_time', 0.25);
}, params => {
    assert(Math.abs(params.sweep_amount) > 0, 'Sweep Time must assign a non-zero Sweep Amount.');
});

runCase('pitch envelope decay auto-enables amount', set => {
    set('pitch_env_decay', 0.3);
}, (params, document) => {
    assert(Math.abs(params.pitch_env_amount) > 0, 'Pitch Env Decay must assign a non-zero Pitch Env Amount.');
    assert(document.getElementById('pitch_env_curve').disabled === false, 'Pitch Env Curve must be enabled after pitch envelope activation.');
});

runCase('delay mix auto-enables time', set => {
    set('delay_mix', 0.35);
}, (params, document) => {
    assert(params.delay_time > 0, 'Delay Mix must assign a non-zero Delay Time.');
    assert(document.getElementById('delay_time').disabled === false, 'Delay Time must be enabled after delay activation.');
});

runCase('distortion type auto-enables audible drive', set => {
    set('distortion_type', 'hard_clip');
}, params => {
    assert(params.distortion_drive > 1, 'Hard Clip must assign Distortion Drive above 1.00x.');
});

runCase('arp speed auto-enables chord', set => {
    set('arp_speed', 8);
}, (params, document) => {
    assert(params.arp_chord !== 'none', 'Arp Speed must assign a non-None Arp Chord.');
    assert(document.getElementById('arp_chord').querySelector('option[value="octave"]').disabled === false, 'Arp Chord options must be enabled after arpeggiator activation.');
});

runCase('auto pan speed auto-enables depth', set => {
    set('auto_pan_speed', 4);
}, params => {
    assert(params.auto_pan_depth > 0, 'Auto Pan Speed must assign a non-zero Auto Pan Depth.');
});

console.log('Sound Creator UI audit passed');

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const presetRoot = path.join(root, 'presets');

const directories = {
    sounds: path.join(presetRoot, 'sounds'),
    rendered: path.join(presetRoot, 'rendered-sounds'),
    melodyWorkflows: path.join(presetRoot, 'melody-workflows'),
    melodyPresets: path.join(presetRoot, 'melody-presets')
};

const baseSound = {
    waveform: 'square',
    sub_waveform: 'none',
    pulse_width: 0.5,
    noise_color: 'white',
    sub_level: 0,
    filter_type: 'none',
    filter_cutoff: 20000,
    filter_resonance: 0,
    filter_env_depth: 0,
    frequency: 440,
    sweep_amount: 0,
    sweep_time: 0,
    portamento: 0,
    attack: 0.01,
    decay: 0.12,
    sustain: 0.55,
    release: 0.12,
    filter_attack: 0.01,
    filter_decay: 0.12,
    filter_sustain: 0.5,
    filter_release: 0.12,
    lfo1_waveform: 'sine',
    lfo1_speed: 5,
    lfo1_depth: 0,
    lfo1_routing: 'none',
    lfo2_waveform: 'sine',
    lfo2_speed: 1,
    lfo2_depth: 0,
    lfo2_routing: 'none',
    delay_time: 0,
    delay_feedback: 0,
    delay_mix: 0,
    bit_depth: 12,
    sample_rate_reduction: 1,
    arp_chord: 'none',
    arp_speed: 0,
    sample_rate: 44100
};

const sounds = [
    sound('lead_01', 'Bright Square Lead', 'Clean square lead for main 8-bit hooks.', { frequency: 523.25, pulse_width: 0.5, sustain: 0.62, release: 0.07, filter_type: 'lowpass', filter_cutoff: 9200, filter_resonance: 0.12, bit_depth: 12 }),
    sound('lead_02', 'Pulse Width Lead', 'Animated PWM lead with a light vibrato edge.', { frequency: 659.25, pulse_width: 0.34, sustain: 0.55, release: 0.08, lfo1_routing: 'pwm', lfo1_waveform: 'triangle', lfo1_speed: 5.5, lfo1_depth: 0.18, filter_type: 'lowpass', filter_cutoff: 7800, bit_depth: 11 }),
    sound('lead_03', 'Glass Triangle Lead', 'Softer triangle lead for high counter melodies.', { waveform: 'triangle', frequency: 783.99, attack: 0.005, decay: 0.1, sustain: 0.5, release: 0.09, filter_type: 'lowpass', filter_cutoff: 6800, delay_time: 0.08, delay_feedback: 0.12, delay_mix: 0.08, bit_depth: 13 }),
    sound('lead_04', 'Arcade Saw Lead', 'Forward saw lead for energetic chorus phrases.', { waveform: 'sawtooth', frequency: 587.33, attack: 0, decay: 0.08, sustain: 0.48, release: 0.06, filter_type: 'lowpass', filter_cutoff: 6200, filter_resonance: 0.18, lfo1_routing: 'pitch', lfo1_speed: 6.2, lfo1_depth: 0.12, bit_depth: 10, sample_rate_reduction: 2 }),
    sound('lead_05', 'Hollow Fifth Lead', 'Square lead reinforced by a fifth-style sub layer.', { frequency: 440, sub_waveform: 'square', sub_level: 0.22, pulse_width: 0.42, attack: 0.004, decay: 0.13, sustain: 0.58, release: 0.08, filter_type: 'lowpass', filter_cutoff: 7600, bit_depth: 12 }),
    sound('lead_06', 'Soft Sine Lead', 'Rounded lead for calmer melody lines.', { waveform: 'sine', frequency: 698.46, attack: 0.02, decay: 0.16, sustain: 0.7, release: 0.12, delay_time: 0.11, delay_feedback: 0.14, delay_mix: 0.1, bit_depth: 14 }),

    sound('bass_01', 'Round Pulse Bass', 'Stable low pulse bass for driving roots.', { frequency: 65.41, pulse_width: 0.42, attack: 0, decay: 0.1, sustain: 0.72, release: 0.05, filter_type: 'lowpass', filter_cutoff: 1800, filter_resonance: 0.18, bit_depth: 11 }),
    sound('bass_02', 'Rubber Square Bass', 'Short square bass with a tight filter snap.', { frequency: 82.41, pulse_width: 0.38, attack: 0, decay: 0.16, sustain: 0.38, release: 0.04, filter_type: 'lowpass', filter_cutoff: 1300, filter_resonance: 0.32, filter_env_depth: 0.36, filter_attack: 0, filter_decay: 0.12, filter_sustain: 0.18, filter_release: 0.04, bit_depth: 10 }),
    sound('bass_03', 'Deep Triangle Bass', 'Subby triangle bass for softer progressions.', { waveform: 'triangle', frequency: 55, attack: 0.006, decay: 0.12, sustain: 0.78, release: 0.08, filter_type: 'lowpass', filter_cutoff: 1200, bit_depth: 13 }),
    sound('bass_04', 'Saw Tooth Bass', 'Grainy saw bass for darker sections.', { waveform: 'sawtooth', frequency: 73.42, attack: 0, decay: 0.09, sustain: 0.48, release: 0.05, filter_type: 'lowpass', filter_cutoff: 1500, filter_resonance: 0.24, bit_depth: 9, sample_rate_reduction: 2 }),
    sound('bass_05', 'Octave Pulse Bass', 'Classic octave-friendly bass with light bit grit.', { frequency: 98, pulse_width: 0.5, sub_waveform: 'square', sub_level: 0.28, attack: 0, decay: 0.1, sustain: 0.55, release: 0.05, filter_type: 'lowpass', filter_cutoff: 2100, bit_depth: 10 }),

    sound('pluck_01', 'Coin Pluck', 'Short bright pluck for arpeggios and accents.', { frequency: 880, pulse_width: 0.28, attack: 0, decay: 0.09, sustain: 0, release: 0.035, filter_type: 'lowpass', filter_cutoff: 10500, bit_depth: 10 }),
    sound('pluck_02', 'Muted Triangle Pluck', 'Rounded short triangle pluck for chord fragments.', { waveform: 'triangle', frequency: 659.25, attack: 0, decay: 0.12, sustain: 0, release: 0.04, filter_type: 'lowpass', filter_cutoff: 6400, bit_depth: 12 }),
    sound('pluck_03', 'Echo Pulse Pluck', 'Pulse pluck with a small chip echo tail.', { frequency: 1046.5, pulse_width: 0.36, attack: 0, decay: 0.075, sustain: 0, release: 0.025, delay_time: 0.12, delay_feedback: 0.18, delay_mix: 0.12, bit_depth: 10 }),
    sound('pluck_04', 'Saw Stab Pluck', 'Sharp saw stab for syncopated backing patterns.', { waveform: 'sawtooth', frequency: 523.25, attack: 0, decay: 0.08, sustain: 0.08, release: 0.035, filter_type: 'lowpass', filter_cutoff: 5200, filter_resonance: 0.2, bit_depth: 9, sample_rate_reduction: 2 }),

    sound('arcade_sfx_01', 'Menu Blip Up', 'Short upward UI blip for arcade actions.', { frequency: 740, pulse_width: 0.22, attack: 0, decay: 0.05, sustain: 0, release: 0.02, sweep_amount: 9, sweep_time: 0.08, bit_depth: 8, sample_rate_reduction: 2 }),
    sound('arcade_sfx_02', 'Laser Drop', 'Descending laser sweep for transitions.', { waveform: 'sawtooth', frequency: 900, attack: 0, decay: 0.13, sustain: 0, release: 0.04, sweep_amount: -18, sweep_time: 0.16, filter_type: 'lowpass', filter_cutoff: 5000, bit_depth: 8, sample_rate_reduction: 3 }),
    sound('arcade_sfx_03', 'Power Chime', 'Fast power-up chime usable as a musical accent.', { waveform: 'square', frequency: 660, pulse_width: 0.5, attack: 0, decay: 0.12, sustain: 0.18, release: 0.04, arp_chord: 'major', arp_speed: 18, bit_depth: 9 }),

    sound('noise_drum_01', 'Tight Noise Kick', 'Low snapped noise kick for chip drum patterns.', { waveform: 'noise', noise_color: 'brown', frequency: 90, attack: 0, decay: 0.11, sustain: 0, release: 0.025, filter_type: 'lowpass', filter_cutoff: 950, filter_resonance: 0.08, sweep_amount: -12, sweep_time: 0.07, bit_depth: 8, sample_rate_reduction: 2 }),
    sound('noise_drum_02', 'Crisp Noise Snare', 'Bright filtered noise snare with a short body.', { waveform: 'noise', noise_color: 'white', frequency: 220, attack: 0, decay: 0.12, sustain: 0.05, release: 0.035, filter_type: 'bandpass', filter_cutoff: 2600, filter_resonance: 0.32, bit_depth: 8, sample_rate_reduction: 2 }),
    sound('noise_drum_03', 'Tick Hat Closed', 'Very short highpass hat tick.', { waveform: 'noise', noise_color: 'white', frequency: 440, attack: 0, decay: 0.035, sustain: 0, release: 0.012, filter_type: 'highpass', filter_cutoff: 6200, filter_resonance: 0.16, bit_depth: 7, sample_rate_reduction: 3 }),
    sound('noise_drum_04', 'Open Noise Hat', 'Longer highpass noise hat for offbeat motion.', { waveform: 'noise', noise_color: 'pink', frequency: 440, attack: 0, decay: 0.16, sustain: 0.05, release: 0.08, filter_type: 'highpass', filter_cutoff: 4800, filter_resonance: 0.14, bit_depth: 8, sample_rate_reduction: 2 }),

    sound('lead_07', 'Crystal Pulse Lead', 'Bright upper-register pulse lead for airy hooks.', { frequency: 739.99, pulse_width: 0.31, attack: 0.006, decay: 0.09, sustain: 0.52, release: 0.07, filter_type: 'lowpass', filter_cutoff: 9800, lfo1_routing: 'pitch', lfo1_speed: 5.8, lfo1_depth: 0.08, bit_depth: 12 }),
    sound('lead_08', 'Detuned Saw Lead', 'Wide saw lead with slow filter movement.', { waveform: 'sawtooth', frequency: 622.25, sub_waveform: 'square', sub_level: 0.14, attack: 0.004, decay: 0.11, sustain: 0.46, release: 0.08, filter_type: 'lowpass', filter_cutoff: 7000, filter_resonance: 0.22, lfo1_routing: 'cutoff', lfo1_speed: 1.6, lfo1_depth: 0.22, bit_depth: 10 }),
    sound('lead_09', 'Triangle Bell Lead', 'Bell-like triangle lead for clean arpeggio tops.', { waveform: 'triangle', frequency: 987.77, attack: 0.002, decay: 0.19, sustain: 0.18, release: 0.09, filter_type: 'lowpass', filter_cutoff: 8500, delay_time: 0.09, delay_feedback: 0.16, delay_mix: 0.1, bit_depth: 13 }),
    sound('lead_10', 'Narrow Pulse Solo', 'Nasal narrow pulse lead for solo phrases.', { frequency: 554.37, pulse_width: 0.22, attack: 0, decay: 0.1, sustain: 0.6, release: 0.06, filter_type: 'bandpass', filter_cutoff: 2800, filter_resonance: 0.38, bit_depth: 11 }),
    sound('lead_11', 'Sine Glide Lead', 'Smooth sine lead with a small portamento feel.', { waveform: 'sine', frequency: 830.61, attack: 0.018, decay: 0.12, sustain: 0.68, release: 0.1, portamento: 0.04, delay_time: 0.1, delay_feedback: 0.12, delay_mix: 0.08, bit_depth: 14 }),
    sound('lead_12', 'Bitcrush Hero Lead', 'Aggressive crushed lead for high-energy hooks.', { waveform: 'square', frequency: 698.46, pulse_width: 0.45, attack: 0, decay: 0.08, sustain: 0.5, release: 0.05, filter_type: 'lowpass', filter_cutoff: 5400, bit_depth: 7, sample_rate_reduction: 4 }),

    sound('bass_06', 'Sub Square Bass', 'Deep square sub bass for steady electronic grooves.', { frequency: 49, pulse_width: 0.5, attack: 0, decay: 0.08, sustain: 0.8, release: 0.05, filter_type: 'lowpass', filter_cutoff: 1100, filter_resonance: 0.12, bit_depth: 12 }),
    sound('bass_07', 'Acid Chip Bass', 'Resonant bass with fast envelope bite.', { waveform: 'sawtooth', frequency: 92.5, attack: 0, decay: 0.14, sustain: 0.32, release: 0.045, filter_type: 'lowpass', filter_cutoff: 1600, filter_resonance: 0.46, filter_env_depth: 0.52, filter_attack: 0, filter_decay: 0.1, filter_sustain: 0.15, filter_release: 0.04, bit_depth: 9, sample_rate_reduction: 2 }),
    sound('bass_08', 'Muted Pulse Bass', 'Short muted bass for tight staccato patterns.', { frequency: 73.42, pulse_width: 0.3, attack: 0, decay: 0.07, sustain: 0.18, release: 0.035, filter_type: 'lowpass', filter_cutoff: 1700, bit_depth: 10 }),
    sound('bass_09', 'Triangle Round Bass', 'Warm triangle bass for calmer loops.', { waveform: 'triangle', frequency: 61.74, attack: 0.005, decay: 0.15, sustain: 0.74, release: 0.08, filter_type: 'lowpass', filter_cutoff: 1300, bit_depth: 13 }),
    sound('bass_10', 'PWM Bass Drive', 'Moving PWM bass for rhythmic pressure.', { frequency: 87.31, pulse_width: 0.36, attack: 0, decay: 0.1, sustain: 0.5, release: 0.05, filter_type: 'lowpass', filter_cutoff: 1900, lfo1_routing: 'pwm', lfo1_waveform: 'triangle', lfo1_speed: 3.2, lfo1_depth: 0.14, bit_depth: 10 }),

    sound('pluck_05', 'Glass Arp Pluck', 'Clear arpeggio pluck with a small echo.', { waveform: 'triangle', frequency: 1174.66, attack: 0, decay: 0.08, sustain: 0, release: 0.035, delay_time: 0.1, delay_feedback: 0.14, delay_mix: 0.09, bit_depth: 12 }),
    sound('pluck_06', 'Dry Pulse Tick', 'Dry pulse tick for fast rhythmic ostinatos.', { frequency: 932.33, pulse_width: 0.25, attack: 0, decay: 0.045, sustain: 0, release: 0.018, filter_type: 'lowpass', filter_cutoff: 8200, bit_depth: 9 }),
    sound('pluck_07', 'Low Wood Pluck', 'Lower muted pluck for harmonic support.', { waveform: 'triangle', frequency: 392, attack: 0, decay: 0.11, sustain: 0.04, release: 0.035, filter_type: 'lowpass', filter_cutoff: 3600, bit_depth: 11 }),
    sound('pluck_08', 'Sparkle Saw Pluck', 'Bright saw pluck for sparkling high patterns.', { waveform: 'sawtooth', frequency: 1318.51, attack: 0, decay: 0.055, sustain: 0, release: 0.02, filter_type: 'lowpass', filter_cutoff: 9000, filter_resonance: 0.14, bit_depth: 8, sample_rate_reduction: 2 }),

    sound('arcade_sfx_04', 'Confirm Chirp', 'Two-stage chirp for positive UI confirmations.', { frequency: 880, pulse_width: 0.28, attack: 0, decay: 0.07, sustain: 0, release: 0.02, sweep_amount: 14, sweep_time: 0.09, arp_chord: 'fifth', arp_speed: 22, bit_depth: 8, sample_rate_reduction: 2 }),
    sound('arcade_sfx_05', 'Error Zap', 'Short downward zap for warning accents.', { waveform: 'square', frequency: 520, pulse_width: 0.2, attack: 0, decay: 0.08, sustain: 0, release: 0.02, sweep_amount: -16, sweep_time: 0.11, filter_type: 'bandpass', filter_cutoff: 1800, filter_resonance: 0.34, bit_depth: 7, sample_rate_reduction: 3 }),
    sound('arcade_sfx_06', 'Bonus Twinkle', 'Small major arpeggio sparkle for bonus pickups.', { waveform: 'triangle', frequency: 740, attack: 0, decay: 0.12, sustain: 0.08, release: 0.04, arp_chord: 'major', arp_speed: 24, delay_time: 0.08, delay_feedback: 0.12, delay_mix: 0.08, bit_depth: 10 }),

    sound('noise_drum_05', 'Soft Noise Kick', 'Softer low noise kick for lighter grooves.', { waveform: 'noise', noise_color: 'brown', frequency: 80, attack: 0, decay: 0.13, sustain: 0, release: 0.03, filter_type: 'lowpass', filter_cutoff: 780, sweep_amount: -10, sweep_time: 0.08, bit_depth: 8, sample_rate_reduction: 2 }),
    sound('noise_drum_06', 'Box Noise Snare', 'Boxy mid noise snare for retro backbeats.', { waveform: 'noise', noise_color: 'pink', frequency: 220, attack: 0, decay: 0.1, sustain: 0.08, release: 0.035, filter_type: 'bandpass', filter_cutoff: 1900, filter_resonance: 0.42, bit_depth: 8, sample_rate_reduction: 2 }),
    sound('noise_drum_07', 'Metal Hat Closed', 'Metallic highpass tick for tight hats.', { waveform: 'noise', noise_color: 'white', frequency: 440, attack: 0, decay: 0.028, sustain: 0, release: 0.01, filter_type: 'highpass', filter_cutoff: 7600, filter_resonance: 0.2, bit_depth: 6, sample_rate_reduction: 4 }),
    sound('noise_drum_08', 'Splash Noise Hat', 'Long highpass splash hat for transitions.', { waveform: 'noise', noise_color: 'white', frequency: 440, attack: 0, decay: 0.2, sustain: 0.08, release: 0.1, filter_type: 'highpass', filter_cutoff: 5200, filter_resonance: 0.16, bit_depth: 8, sample_rate_reduction: 2 })
];

const melodyPresets = [
    melodyPreset('neon_lead_phrase_a', 'Neon Lead Phrase A', 'Ascending 8-bar C minor hook.', 'Lead', [
        n(72, 0, 0.5), n(75, 0.5, 0.5), n(79, 1, 0.5), n(82, 1.5, 0.5),
        n(80, 2, 0.5), n(79, 2.5, 0.5), n(75, 3, 0.5), n(72, 3.5, 0.5),
        n(74, 4, 0.5), n(77, 4.5, 0.5), n(80, 5, 0.75), n(79, 5.75, 0.25),
        n(77, 6, 0.5), n(75, 6.5, 0.5), n(72, 7, 1)
    ]),
    melodyPreset('neon_bass_roots', 'Neon Bass Roots', 'C minor root pulse with a turnaround.', 'Bass', [
        n(36, 0, 0.75), n(36, 1, 0.5), n(43, 1.75, 0.25), n(44, 2, 0.75), n(44, 3, 0.5),
        n(39, 4, 0.75), n(39, 5, 0.5), n(41, 6, 0.5), n(43, 6.5, 0.5), n(36, 7, 1)
    ]),
    melodyPreset('neon_pluck_offbeats', 'Neon Pluck Offbeats', 'Syncopated minor chord tones.', 'Pluck', [
        n(60, 0.5, 0.25), n(63, 1.5, 0.25), n(67, 2.5, 0.25), n(70, 3.5, 0.25),
        n(58, 4.5, 0.25), n(62, 5.5, 0.25), n(65, 6.5, 0.25), n(67, 7.5, 0.25)
    ]),
    melodyPreset('neon_kick_grid', 'Neon Kick Grid', 'Four-on-floor kick layer for the neon workflow.', 'Kick', [
        n(36, 0, 0.25, 1), n(36, 2, 0.25, 1), n(36, 4, 0.25, 1), n(36, 6, 0.25, 1)
    ]),
    melodyPreset('neon_snare_backbeat', 'Neon Snare Backbeat', 'Backbeat snare layer for the neon workflow.', 'Snare', [
        n(38, 1, 0.25, 0.9), n(38, 3, 0.25, 0.9), n(38, 5, 0.25, 0.9), n(38, 7, 0.25, 0.9)
    ]),
    melodyPreset('neon_hat_offbeats', 'Neon Hat Offbeats', 'Offbeat hat layer for the neon workflow.', 'Hat', [
        n(42, 0.5, 0.125, 0.5), n(42, 1.5, 0.125, 0.45), n(42, 2.5, 0.125, 0.5), n(42, 3.5, 0.125, 0.45),
        n(42, 4.5, 0.125, 0.5), n(42, 5.5, 0.125, 0.45), n(42, 6.5, 0.125, 0.5), n(42, 7.5, 0.25, 0.65)
    ]),

    melodyPreset('dungeon_lead_motif', 'Dungeon Lead Motif', 'Low Phrygian-flavored hook.', 'Lead', [
        n(67, 0, 0.5), n(68, 0.5, 0.5), n(65, 1, 0.5), n(63, 1.5, 0.5),
        n(62, 2, 0.75), n(63, 3, 0.5), n(67, 3.5, 0.5),
        n(70, 4, 0.5), n(68, 4.5, 0.5), n(67, 5, 0.5), n(63, 5.5, 0.5),
        n(62, 6, 0.5), n(60, 6.5, 0.5), n(55, 7, 1)
    ]),
    melodyPreset('dungeon_bass_ostinato', 'Dungeon Bass Ostinato', 'Tense repeating low ostinato.', 'Bass', [
        n(31, 0, 0.5), n(31, 0.75, 0.25), n(34, 1, 0.5), n(31, 1.75, 0.25),
        n(32, 2, 0.5), n(32, 2.75, 0.25), n(35, 3, 0.5), n(32, 3.75, 0.25),
        n(31, 4, 0.5), n(31, 4.75, 0.25), n(34, 5, 0.5), n(31, 5.75, 0.25),
        n(29, 6, 0.5), n(31, 6.5, 0.5), n(32, 7, 1)
    ]),
    melodyPreset('dungeon_pluck_stabs', 'Dungeon Pluck Stabs', 'Sparse stabs that leave room for the bass.', 'Pluck', [
        n(55, 0, 0.25), n(58, 1.5, 0.25), n(60, 2, 0.25), n(63, 3.5, 0.25),
        n(55, 4, 0.25), n(58, 5.5, 0.25), n(53, 6, 0.25), n(55, 7.5, 0.25)
    ]),
    melodyPreset('dungeon_kick_steps', 'Dungeon Kick Steps', 'Half-time kick layer for dungeon pulse.', 'Kick', [
        n(36, 0, 0.25, 1), n(36, 4, 0.25, 1)
    ]),
    melodyPreset('dungeon_snare_steps', 'Dungeon Snare Steps', 'Half-time snare layer for dungeon pulse.', 'Snare', [
        n(38, 2, 0.25, 0.95), n(38, 6, 0.25, 0.95)
    ]),
    melodyPreset('dungeon_hat_steps', 'Dungeon Hat Steps', 'Sparse hat layer for dungeon pulse.', 'Hat', [
        n(42, 0.75, 0.125, 0.35), n(42, 2.75, 0.125, 0.35), n(42, 4.75, 0.125, 0.35), n(42, 7, 0.25, 0.5)
    ]),

    melodyPreset('victory_lead_fanfare', 'Victory Lead Fanfare', 'Major fanfare phrase for reward screens.', 'Lead', [
        n(72, 0, 0.5), n(76, 0.5, 0.5), n(79, 1, 0.5), n(84, 1.5, 1),
        n(83, 2.75, 0.25), n(81, 3, 0.5), n(79, 3.5, 0.5),
        n(76, 4, 0.5), n(79, 4.5, 0.5), n(84, 5, 0.75), n(88, 5.75, 0.25),
        n(86, 6, 0.5), n(84, 6.5, 0.5), n(79, 7, 1)
    ]),
    melodyPreset('victory_bass_climb', 'Victory Bass Climb', 'Major bass climb with a clear cadence.', 'Bass', [
        n(36, 0, 0.75), n(43, 1, 0.5), n(45, 2, 0.75), n(47, 3, 0.5),
        n(48, 4, 0.75), n(50, 5, 0.5), n(52, 6, 0.5), n(55, 6.5, 0.5), n(48, 7, 1)
    ]),
    melodyPreset('victory_pluck_chords', 'Victory Pluck Chords', 'Broken major chord sparkle.', 'Pluck', [
        n(60, 0, 0.25), n(64, 0.25, 0.25), n(67, 0.5, 0.25), n(72, 0.75, 0.25),
        n(65, 2, 0.25), n(69, 2.25, 0.25), n(72, 2.5, 0.25), n(77, 2.75, 0.25),
        n(67, 4, 0.25), n(71, 4.25, 0.25), n(74, 4.5, 0.25), n(79, 4.75, 0.25),
        n(72, 6, 0.25), n(76, 6.25, 0.25), n(79, 6.5, 0.25), n(84, 6.75, 0.25)
    ]),
    melodyPreset('victory_kick_drive', 'Victory Kick Drive', 'Upbeat kick drive for victory lift.', 'Kick', [
        n(36, 0, 0.25, 1), n(36, 2, 0.25, 1), n(36, 4, 0.25, 1), n(36, 6, 0.25, 1)
    ]),
    melodyPreset('victory_snare_drive', 'Victory Snare Drive', 'Bright snare backbeat for victory lift.', 'Snare', [
        n(38, 1, 0.25, 0.85), n(38, 3, 0.25, 0.85), n(38, 5, 0.25, 0.85), n(38, 7, 0.25, 0.85)
    ]),
    melodyPreset('victory_hat_drive', 'Victory Hat Drive', 'Open hat lift and closed hat motion.', 'Hat', [
        n(42, 0.5, 0.125, 0.45), n(42, 1.5, 0.125, 0.45), n(42, 2.5, 0.125, 0.45), n(46, 3.5, 0.25, 0.55),
        n(42, 4.5, 0.125, 0.45), n(42, 5.5, 0.125, 0.45), n(42, 6.5, 0.125, 0.45), n(46, 7.5, 0.25, 0.6)
    ]),

    melodyPreset('boss_lead_alarm', 'Boss Lead Alarm', 'Repeated warning motif with narrow motion.', 'Lead', [
        n(75, 0, 0.375), n(74, 0.5, 0.375), n(75, 1, 0.375), n(70, 1.5, 0.375),
        n(75, 2, 0.375), n(74, 2.5, 0.375), n(77, 3, 0.375), n(70, 3.5, 0.375),
        n(75, 4, 0.375), n(74, 4.5, 0.375), n(75, 5, 0.375), n(70, 5.5, 0.375),
        n(78, 6, 0.375), n(77, 6.5, 0.375), n(75, 7, 0.75)
    ]),
    melodyPreset('boss_bass_hits', 'Boss Bass Hits', 'Sparse low hits for warning tension.', 'Bass', [
        n(35, 0, 0.5), n(35, 1.5, 0.25), n(35, 2, 0.5), n(38, 3.5, 0.25),
        n(35, 4, 0.5), n(35, 5.5, 0.25), n(34, 6, 0.5), n(31, 7, 0.75)
    ]),
    melodyPreset('boss_pulse_ticks', 'Boss Pulse Ticks', 'Nervous repeated pulse ticks.', 'Pluck', [
        n(58, 0.25, 0.125), n(58, 0.75, 0.125), n(58, 1.25, 0.125), n(58, 1.75, 0.125),
        n(61, 2.25, 0.125), n(61, 2.75, 0.125), n(61, 3.25, 0.125), n(61, 3.75, 0.125),
        n(58, 4.25, 0.125), n(58, 4.75, 0.125), n(58, 5.25, 0.125), n(58, 5.75, 0.125),
        n(56, 6.25, 0.125), n(56, 6.75, 0.125), n(55, 7.25, 0.125), n(55, 7.75, 0.125)
    ]),
    melodyPreset('boss_kick_alarm', 'Boss Kick Alarm', 'Repeated warning kick layer.', 'Kick', [
        n(36, 0, 0.25, 1), n(36, 2, 0.25, 1), n(36, 4, 0.25, 1), n(36, 6, 0.25, 1)
    ]),
    melodyPreset('boss_snare_alarm', 'Boss Snare Alarm', 'Dense warning snare layer.', 'Snare', [
        n(38, 1, 0.25, 0.95), n(38, 3, 0.25, 0.95), n(38, 5, 0.25, 0.95), n(38, 7, 0.25, 0.95)
    ]),
    melodyPreset('boss_hat_alarm', 'Boss Hat Alarm', 'Tense hat layer between warning hits.', 'Hat', [
        n(42, 0.5, 0.125, 0.5), n(42, 1.5, 0.125, 0.5), n(42, 2.5, 0.125, 0.5), n(42, 3.5, 0.125, 0.5),
        n(42, 4.5, 0.125, 0.5), n(42, 5.5, 0.125, 0.5), n(42, 6.5, 0.125, 0.5), n(46, 7.5, 0.25, 0.65)
    ]),

    melodyPreset('skyline_lead_run', 'Skyline Lead Run', 'Fast E minor skyline hook with a rising finish.', 'Lead', [
        n(76, 0, 0.25), n(79, 0.25, 0.25), n(83, 0.5, 0.5), n(81, 1, 0.25), n(79, 1.25, 0.25), n(76, 1.5, 0.5),
        n(74, 2, 0.25), n(76, 2.25, 0.25), n(79, 2.5, 0.5), n(83, 3, 0.5), n(86, 3.5, 0.5),
        n(88, 4, 0.5), n(86, 4.5, 0.25), n(83, 4.75, 0.25), n(81, 5, 0.5), n(79, 5.5, 0.5),
        n(76, 6, 0.5), n(79, 6.5, 0.5), n(83, 7, 1)
    ]),
    melodyPreset('skyline_bass_drive', 'Skyline Bass Drive', 'E minor driving bass with octave pushes.', 'Bass', [
        n(40, 0, 0.5), n(40, 0.75, 0.25), n(47, 1, 0.5), n(40, 1.75, 0.25),
        n(43, 2, 0.5), n(43, 2.75, 0.25), n(50, 3, 0.5), n(43, 3.75, 0.25),
        n(45, 4, 0.5), n(45, 4.75, 0.25), n(52, 5, 0.5), n(45, 5.75, 0.25),
        n(47, 6, 0.5), n(45, 6.5, 0.5), n(40, 7, 1)
    ]),
    melodyPreset('skyline_arp_glass', 'Skyline Arp Glass', 'High glass arpeggio line for skyline motion.', 'Arp', [
        n(64, 0, 0.25), n(67, 0.25, 0.25), n(71, 0.5, 0.25), n(76, 0.75, 0.25),
        n(62, 2, 0.25), n(66, 2.25, 0.25), n(69, 2.5, 0.25), n(74, 2.75, 0.25),
        n(60, 4, 0.25), n(64, 4.25, 0.25), n(67, 4.5, 0.25), n(72, 4.75, 0.25),
        n(59, 6, 0.25), n(62, 6.25, 0.25), n(66, 6.5, 0.25), n(71, 6.75, 0.25)
    ]),
    melodyPreset('skyline_kick_drive', 'Skyline Kick Drive', 'Steady kick layer for skyline run.', 'Kick', [
        n(36, 0, 0.25, 1), n(36, 1.5, 0.25, 0.75), n(36, 2, 0.25, 1), n(36, 3.5, 0.25, 0.75),
        n(36, 4, 0.25, 1), n(36, 5.5, 0.25, 0.75), n(36, 6, 0.25, 1)
    ]),
    melodyPreset('skyline_snare_snap', 'Skyline Snare Snap', 'Snappy backbeat layer for skyline run.', 'Snare', [
        n(38, 1, 0.25, 0.9), n(38, 3, 0.25, 0.9), n(38, 5, 0.25, 0.9), n(38, 7, 0.25, 0.95)
    ]),
    melodyPreset('skyline_hat_ticks', 'Skyline Hat Ticks', 'Sixteenth-style hat ticks for skyline run.', 'Hat', [
        n(42, 0.5, 0.125, 0.45), n(42, 1.5, 0.125, 0.45), n(42, 2.5, 0.125, 0.45), n(42, 3.5, 0.125, 0.45),
        n(42, 4.5, 0.125, 0.45), n(42, 5.5, 0.125, 0.45), n(42, 6.5, 0.125, 0.45), n(46, 7.5, 0.25, 0.6)
    ]),

    melodyPreset('factory_lead_hook', 'Factory Lead Hook', 'Mechanical D minor hook with repeated bends.', 'Lead', [
        n(74, 0, 0.5), n(76, 0.5, 0.25), n(77, 0.75, 0.25), n(74, 1, 0.5), n(69, 1.5, 0.5),
        n(72, 2, 0.5), n(74, 2.5, 0.25), n(76, 2.75, 0.25), n(72, 3, 0.5), n(69, 3.5, 0.5),
        n(77, 4, 0.5), n(76, 4.5, 0.5), n(74, 5, 0.5), n(72, 5.5, 0.5),
        n(69, 6, 0.5), n(72, 6.5, 0.5), n(74, 7, 1)
    ]),
    melodyPreset('factory_bass_pulse', 'Factory Bass Pulse', 'D minor machine pulse bass.', 'Bass', [
        n(38, 0, 0.375), n(38, 0.5, 0.25), n(38, 1, 0.375), n(45, 1.5, 0.25),
        n(41, 2, 0.375), n(41, 2.5, 0.25), n(41, 3, 0.375), n(48, 3.5, 0.25),
        n(43, 4, 0.375), n(43, 4.5, 0.25), n(43, 5, 0.375), n(50, 5.5, 0.25),
        n(36, 6, 0.5), n(38, 6.5, 0.5), n(41, 7, 1)
    ]),
    melodyPreset('factory_stab_pattern', 'Factory Stab Pattern', 'Short industrial chord-tone stabs.', 'Stabs', [
        n(57, 0, 0.125), n(62, 0.75, 0.125), n(65, 1.5, 0.125), n(57, 2, 0.125),
        n(60, 2.75, 0.125), n(64, 3.5, 0.125), n(55, 4, 0.125), n(60, 4.75, 0.125),
        n(64, 5.5, 0.125), n(53, 6, 0.125), n(57, 6.75, 0.125), n(62, 7.5, 0.125)
    ]),
    melodyPreset('factory_kick_press', 'Factory Kick Press', 'Machine kick presses on the downbeats.', 'Kick', [
        n(36, 0, 0.25, 1), n(36, 2, 0.25, 1), n(36, 4, 0.25, 1), n(36, 6, 0.25, 1)
    ]),
    melodyPreset('factory_snare_clank', 'Factory Snare Clank', 'Clanking snare layer for factory rhythm.', 'Snare', [
        n(38, 1.5, 0.25, 0.9), n(38, 3.5, 0.25, 0.9), n(38, 5.5, 0.25, 0.9), n(38, 7.5, 0.25, 0.95)
    ]),
    melodyPreset('factory_hat_steam', 'Factory Hat Steam', 'Steam-like hat layer for factory rhythm.', 'Hat', [
        n(42, 0.25, 0.125, 0.36), n(42, 0.75, 0.125, 0.36), n(42, 1.25, 0.125, 0.36), n(42, 1.75, 0.125, 0.36),
        n(42, 2.25, 0.125, 0.36), n(42, 2.75, 0.125, 0.36), n(42, 3.25, 0.125, 0.36), n(42, 3.75, 0.125, 0.36),
        n(42, 4.25, 0.125, 0.36), n(42, 4.75, 0.125, 0.36), n(42, 5.25, 0.125, 0.36), n(42, 5.75, 0.125, 0.36),
        n(42, 6.25, 0.125, 0.36), n(42, 6.75, 0.125, 0.36), n(46, 7.25, 0.25, 0.52)
    ]),

    melodyPreset('ocean_lead_swell', 'Ocean Lead Swell', 'Gentler A minor lead phrase for water stages.', 'Lead', [
        n(69, 0, 0.75), n(72, 0.75, 0.25), n(76, 1, 0.75), n(74, 1.75, 0.25),
        n(72, 2, 0.5), n(71, 2.5, 0.5), n(69, 3, 1),
        n(67, 4, 0.75), n(69, 4.75, 0.25), n(72, 5, 0.75), n(74, 5.75, 0.25),
        n(76, 6, 0.5), n(74, 6.5, 0.5), n(72, 7, 1)
    ]),
    melodyPreset('ocean_bass_tide', 'Ocean Bass Tide', 'Slow A minor tide bass movement.', 'Bass', [
        n(33, 0, 1), n(40, 1.5, 0.5), n(36, 2, 1), n(43, 3.5, 0.5),
        n(38, 4, 1), n(45, 5.5, 0.5), n(40, 6, 0.5), n(38, 6.5, 0.5), n(33, 7, 1)
    ]),
    melodyPreset('ocean_pluck_ripples', 'Ocean Pluck Ripples', 'Soft ripple plucks for water-stage shimmer.', 'Pluck', [
        n(57, 0.5, 0.25), n(60, 1, 0.25), n(64, 1.5, 0.25), n(69, 2, 0.25),
        n(55, 2.5, 0.25), n(59, 3, 0.25), n(62, 3.5, 0.25), n(67, 4, 0.25),
        n(53, 4.5, 0.25), n(57, 5, 0.25), n(60, 5.5, 0.25), n(65, 6, 0.25),
        n(52, 6.5, 0.25), n(55, 7, 0.25), n(60, 7.5, 0.25)
    ]),
    melodyPreset('ocean_kick_soft', 'Ocean Kick Soft', 'Soft kick layer for tide groove.', 'Kick', [
        n(36, 0, 0.25, 0.9), n(36, 2.5, 0.25, 0.75), n(36, 4, 0.25, 0.9), n(36, 6.5, 0.25, 0.75)
    ]),
    melodyPreset('ocean_snare_soft', 'Ocean Snare Soft', 'Soft snare layer for tide groove.', 'Snare', [
        n(38, 2, 0.25, 0.78), n(38, 6, 0.25, 0.78)
    ]),
    melodyPreset('ocean_hat_splash', 'Ocean Hat Splash', 'Open hat splashes for tide groove.', 'Hat', [
        n(46, 1, 0.25, 0.42), n(42, 3, 0.125, 0.32), n(46, 5, 0.25, 0.42), n(42, 7, 0.125, 0.32)
    ]),

    melodyPreset('finale_lead_charge', 'Finale Lead Charge', 'Heroic final-stage charge in G minor.', 'Lead', [
        n(67, 0, 0.375), n(70, 0.5, 0.375), n(74, 1, 0.5), n(77, 1.5, 0.5),
        n(79, 2, 0.375), n(77, 2.5, 0.375), n(74, 3, 0.5), n(70, 3.5, 0.5),
        n(72, 4, 0.375), n(75, 4.5, 0.375), n(79, 5, 0.5), n(82, 5.5, 0.5),
        n(84, 6, 0.5), n(82, 6.5, 0.5), n(79, 7, 1)
    ]),
    melodyPreset('finale_bass_charge', 'Finale Bass Charge', 'G minor bass charge with strong cadence.', 'Bass', [
        n(31, 0, 0.5), n(38, 0.75, 0.25), n(43, 1, 0.5), n(38, 1.75, 0.25),
        n(34, 2, 0.5), n(41, 2.75, 0.25), n(46, 3, 0.5), n(41, 3.75, 0.25),
        n(36, 4, 0.5), n(43, 4.75, 0.25), n(48, 5, 0.5), n(43, 5.75, 0.25),
        n(38, 6, 0.5), n(36, 6.5, 0.5), n(31, 7, 1)
    ]),
    melodyPreset('finale_tick_ostinato', 'Finale Tick Ostinato', 'Urgent tick ostinato for final-stage pressure.', 'Pluck', [
        n(55, 0.25, 0.125), n(58, 0.75, 0.125), n(62, 1.25, 0.125), n(67, 1.75, 0.125),
        n(53, 2.25, 0.125), n(58, 2.75, 0.125), n(62, 3.25, 0.125), n(65, 3.75, 0.125),
        n(55, 4.25, 0.125), n(58, 4.75, 0.125), n(62, 5.25, 0.125), n(67, 5.75, 0.125),
        n(50, 6.25, 0.125), n(55, 6.75, 0.125), n(58, 7.25, 0.125), n(62, 7.75, 0.125)
    ]),
    melodyPreset('finale_kick_charge', 'Finale Kick Charge', 'Driving kick layer for final charge.', 'Kick', [
        n(36, 0, 0.25, 1), n(36, 1, 0.25, 0.75), n(36, 2, 0.25, 1), n(36, 3, 0.25, 0.75),
        n(36, 4, 0.25, 1), n(36, 5, 0.25, 0.75), n(36, 6, 0.25, 1), n(36, 7, 0.25, 0.75)
    ]),
    melodyPreset('finale_snare_charge', 'Finale Snare Charge', 'Driving snare layer for final charge.', 'Snare', [
        n(38, 1, 0.25, 0.9), n(38, 3, 0.25, 0.9), n(38, 5, 0.25, 0.9), n(38, 7, 0.25, 0.95)
    ]),
    melodyPreset('finale_hat_charge', 'Finale Hat Charge', 'Urgent hat layer for final charge.', 'Hat', [
        n(42, 0.5, 0.125, 0.5), n(42, 1.5, 0.125, 0.5), n(42, 2.5, 0.125, 0.5), n(42, 3.5, 0.125, 0.5),
        n(42, 4.5, 0.125, 0.5), n(42, 5.5, 0.125, 0.5), n(42, 6.5, 0.125, 0.5), n(46, 7.5, 0.25, 0.7)
    ])
];

const workflows = [
    workflow('neon_patrol', 'Neon Patrol', 'C minor patrol loop built from bright lead, round bass, offbeat plucks and chip drums.', 132, [
        layer(1, 'Lead', 'lead_02', 'neon_lead_phrase_a'),
        layer(2, 'Bass', 'bass_01', 'neon_bass_roots'),
        layer(3, 'Pluck', 'pluck_03', 'neon_pluck_offbeats'),
        layer(4, 'Kick', 'noise_drum_01', 'neon_kick_grid'),
        layer(5, 'Snare', 'noise_drum_02', 'neon_snare_backbeat'),
        layer(6, 'Hat', 'noise_drum_03', 'neon_hat_offbeats')
    ]),
    workflow('dungeon_pulse', 'Dungeon Pulse', 'Darker dungeon loop with tense ostinato, sparse stabs and half-time drums.', 104, [
        layer(1, 'Lead', 'lead_03', 'dungeon_lead_motif'),
        layer(2, 'Bass', 'bass_04', 'dungeon_bass_ostinato'),
        layer(3, 'Stabs', 'pluck_04', 'dungeon_pluck_stabs'),
        layer(4, 'Kick', 'noise_drum_01', 'dungeon_kick_steps'),
        layer(5, 'Snare', 'noise_drum_02', 'dungeon_snare_steps'),
        layer(6, 'Hat', 'noise_drum_04', 'dungeon_hat_steps')
    ]),
    workflow('victory_lift', 'Victory Lift', 'Major reward loop with fanfare lead, climbing bass, chord sparkle and upbeat drums.', 144, [
        layer(1, 'Lead', 'lead_01', 'victory_lead_fanfare'),
        layer(2, 'Bass', 'bass_05', 'victory_bass_climb'),
        layer(3, 'Arp', 'pluck_01', 'victory_pluck_chords'),
        layer(4, 'Kick', 'noise_drum_01', 'victory_kick_drive'),
        layer(5, 'Snare', 'noise_drum_02', 'victory_snare_drive'),
        layer(6, 'Hat', 'noise_drum_04', 'victory_hat_drive')
    ]),
    workflow('boss_warning', 'Boss Warning', 'Warning loop with alarm lead, heavy bass hits, nervous ticks and dense drum alarms.', 126, [
        layer(1, 'Alarm Lead', 'lead_04', 'boss_lead_alarm'),
        layer(2, 'Bass Hits', 'bass_02', 'boss_bass_hits'),
        layer(3, 'Pulse Ticks', 'arcade_sfx_03', 'boss_pulse_ticks'),
        layer(4, 'Kick', 'noise_drum_01', 'boss_kick_alarm'),
        layer(5, 'Snare', 'noise_drum_02', 'boss_snare_alarm'),
        layer(6, 'Hat', 'noise_drum_04', 'boss_hat_alarm')
    ]),
    workflow('skyline_run', 'Skyline Run', 'Fast E minor rooftop loop with glass arps, driving bass and crisp hats.', 150, [
        layer(1, 'Lead', 'lead_07', 'skyline_lead_run'),
        layer(2, 'Bass', 'bass_10', 'skyline_bass_drive'),
        layer(3, 'Arp', 'pluck_05', 'skyline_arp_glass'),
        layer(4, 'Kick', 'noise_drum_05', 'skyline_kick_drive'),
        layer(5, 'Snare', 'noise_drum_06', 'skyline_snare_snap'),
        layer(6, 'Hat', 'noise_drum_07', 'skyline_hat_ticks')
    ]),
    workflow('factory_shift', 'Factory Shift', 'Mechanical D minor loop with machine pulse bass and industrial stabs.', 118, [
        layer(1, 'Lead', 'lead_10', 'factory_lead_hook'),
        layer(2, 'Bass', 'bass_07', 'factory_bass_pulse'),
        layer(3, 'Stabs', 'pluck_06', 'factory_stab_pattern'),
        layer(4, 'Kick', 'noise_drum_05', 'factory_kick_press'),
        layer(5, 'Snare', 'noise_drum_06', 'factory_snare_clank'),
        layer(6, 'Hat', 'noise_drum_08', 'factory_hat_steam')
    ]),
    workflow('ocean_drift', 'Ocean Drift', 'Gentler A minor water-stage loop with soft leads, ripples and sparse drums.', 96, [
        layer(1, 'Lead', 'lead_11', 'ocean_lead_swell'),
        layer(2, 'Bass', 'bass_09', 'ocean_bass_tide'),
        layer(3, 'Ripples', 'pluck_07', 'ocean_pluck_ripples'),
        layer(4, 'Kick', 'noise_drum_05', 'ocean_kick_soft'),
        layer(5, 'Snare', 'noise_drum_06', 'ocean_snare_soft'),
        layer(6, 'Hat', 'noise_drum_08', 'ocean_hat_splash')
    ]),
    workflow('finale_charge', 'Finale Charge', 'Heroic G minor final-stage loop with urgent ticks and full driving drums.', 138, [
        layer(1, 'Lead', 'lead_12', 'finale_lead_charge'),
        layer(2, 'Bass', 'bass_06', 'finale_bass_charge'),
        layer(3, 'Ticks', 'pluck_08', 'finale_tick_ostinato'),
        layer(4, 'Kick', 'noise_drum_01', 'finale_kick_charge'),
        layer(5, 'Snare', 'noise_drum_02', 'finale_snare_charge'),
        layer(6, 'Hat', 'noise_drum_04', 'finale_hat_charge')
    ])
];

const dataMelodyPresets = melodyPresets.map(compactMelodyPreset);
const dataWorkflows = workflows.map(compactWorkflow);

for (const directory of Object.values(directories)) {
    fs.mkdirSync(directory, { recursive: true });
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
        const fullPath = path.join(directory, entry.name);
        if (entry.isFile() && entry.name !== '.gitkeep') {
            fs.rmSync(fullPath);
        }
    }
    keep(directory);
}

for (const entry of sounds) {
    writeJson(path.join(directories.sounds, `${entry.id}.json`), entry.params);
}

for (const entry of dataMelodyPresets) {
    writeJson(path.join(directories.melodyPresets, `${entry.id}.json`), entry.project);
}

const melodyById = new Map(dataMelodyPresets.map(entry => [entry.id, entry.project.layers[0].notes]));
for (const entry of dataWorkflows) {
    const project = {
        name: entry.name,
        description: entry.description,
        tempo: entry.tempo,
        sample_rate: 44100,
        layers: entry.layers.map(item => ({
            id: item.id,
            name: item.name,
            muted: false,
            volume: 0.85,
            soundFilePath: `rendered-sounds/${item.soundId}.wav`,
            soundLabel: `${item.soundId}.wav`,
            melodyPresetId: item.melodyPresetId,
            notes: cloneNotes(melodyById.get(item.melodyPresetId) || [])
        }))
    };
    writeJson(path.join(directories.melodyWorkflows, `${entry.id}.json`), project);
}

console.log(`Generated ${sounds.length} sounds, ${dataMelodyPresets.length} melody presets and ${dataWorkflows.length} melody workflows.`);

function sound(id, name, description, overrides) {
    return {
        id,
        params: {
            name,
            description,
            ...baseSound,
            ...overrides
        }
    };
}

function melodyPreset(id, name, description, layerName, notes) {
    return {
        id,
        project: {
            name,
            description,
            tempo: 120,
            sample_rate: 44100,
            layers: [{
                id: 1,
                name: layerName,
                muted: false,
                volume: 1,
                notes
            }]
        }
    };
}

function workflow(id, name, description, tempo, layers) {
    return { id, name, description, tempo, layers };
}

function layer(id, name, soundId, melodyPresetId) {
    return { id, name, soundId, melodyPresetId };
}

function compactWorkflow(entry) {
    return {
        ...entry,
        description: `Compact three-layer ${entry.name} loop built from lead, bass and one supporting pattern.`,
        layers: entry.layers.slice(0, 3).map((item, index) => ({ ...item, id: index + 1 }))
    };
}

function compactMelodyPreset(entry) {
    const project = {
        ...entry.project,
        description: compactDescription(entry.project.description),
        layers: entry.project.layers.map(layer => ({
            ...layer,
            notes: compactNotes(layer.notes)
        }))
    };
    return { ...entry, project };
}

function compactDescription(description) {
    return String(description || '')
        .replace('8-bar ', '')
        .replace('longer ', '')
        .replace('dense ', '')
        .trim();
}

function compactNotes(notes) {
    const selected = notes
        .filter(note => note.start < 4)
        .slice(0, 8)
        .map(note => ({
            ...note,
            start: roundBeat(note.start),
            duration: Math.min(note.duration, note.start < 3.5 ? 0.75 : 0.5)
        }));
    return selected.length ? selected : notes.slice(0, 4).map(note => ({ ...note }));
}

function roundBeat(value) {
    return Math.round(value * 4) / 4;
}

function n(pitch, start, duration, velocity = 0.85) {
    return { pitch, start, duration, velocity };
}

function cloneNotes(notes) {
    return notes.map(note => ({ ...note }));
}

function writeJson(file, value) {
    fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function keep(directory) {
    const file = path.join(directory, '.gitkeep');
    if (!fs.existsSync(file)) {
        fs.writeFileSync(file, '');
    }
}

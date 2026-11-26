(function () {
  var st = document.getElementById('ef-status');
  window.addEventListener('error', function (ev) {
    if (st) { st.style.color = '#fca5a5'; st.textContent = 'Error: ' + (ev.message || 'Unknown script error'); }
  });
  window.addEventListener('unhandledrejection', function (ev) {
    if (st) { st.style.color = '#fca5a5'; st.textContent = 'WASM Error: ' + (ev.reason && ev.reason.message ? ev.reason.message : String(ev.reason)); }
  });

  var MODIFIERS = ['BLOOD MOON', 'ARCANE SURGE', 'SWARM RUSH', "TITAN'S WRATH"];

  // =========================================================================
  // PHASE 3: PRO PROCEDURAL MULTI-TRACK DARK-SYNTHWAVE / CHIPTUNE ENGINE
  // Zero external audio files — DynamicsCompressor + Procedural Cathedral Reverb
  // =========================================================================
  var audioCtx = null;
  var masterBus = null;
  var reverbBus = null;
  var noiseBuffer = null;
  var isMuted = false;
  var crtEnabled = false;
  var musicActive = false;
  var currentBiomeIdx = 0;
  var bossMusicPhase = 0; // 0 = biome theme, 1 = boss phase I, 2 = boss phase II eclipse
  var seqTimer = null;
  var seqStep = 0;

  // 3 Distinct 16-step Musical Themes per Biome (Frequencies in Hz)
  // 0: Obsidian Sanctuary (D Harmonic Minor — gothic synthwave)
  // 1: Volcanic Rift (E Phrygian — fierce driving metal/synth)
  // 2: Frost Abyss (C Dorian / Lydian icy shimmer — crystalline 16-bit)
  var BIOME_THEMES = [
    {
      bpm: 124,
      bass: [73.42, 73.42, 87.31, 73.42, 110.0, 110.0, 98.0, 82.41, 73.42, 73.42, 87.31, 110.0, 116.54, 110.0, 98.0, 82.41],
      arp:  [293.66, 349.23, 440.0, 554.37, 440.0, 349.23, 293.66, 220.0, 293.66, 349.23, 466.16, 587.33, 554.37, 440.0, 349.23, 329.63],
      lead: [587.33, 0, 554.37, 587.33, 698.46, 0, 659.25, 0, 587.33, 0, 440.0, 466.16, 554.37, 0, 440.0, 0]
    },
    {
      bpm: 132,
      bass: [82.41, 82.41, 87.31, 82.41, 98.0, 87.31, 82.41, 73.42, 82.41, 82.41, 110.0, 98.0, 123.47, 110.0, 98.0, 87.31],
      arp:  [329.63, 349.23, 493.88, 659.25, 493.88, 392.0, 349.23, 329.63, 329.63, 440.0, 493.88, 659.25, 698.46, 659.25, 493.88, 392.0],
      lead: [659.25, 0, 698.46, 659.25, 493.88, 0, 523.25, 493.88, 659.25, 0, 783.99, 698.46, 659.25, 0, 493.88, 0]
    },
    {
      bpm: 118,
      bass: [65.41, 65.41, 77.78, 98.0, 87.31, 87.31, 77.78, 73.42, 65.41, 98.0, 116.54, 98.0, 87.31, 77.78, 73.42, 65.41],
      arp:  [261.63, 311.13, 392.0, 523.25, 587.33, 523.25, 392.0, 311.13, 349.23, 440.0, 523.25, 698.46, 622.25, 523.25, 466.16, 392.0],
      lead: [523.25, 0, 622.25, 587.33, 783.99, 0, 698.46, 0, 622.25, 0, 587.33, 523.25, 466.16, 0, 392.0, 0]
    }
  ];

  var BOSS_THEME = {
    bass: [55.0, 55.0, 65.41, 55.0, 73.42, 55.0, 82.41, 77.78, 55.0, 55.0, 87.31, 82.41, 73.42, 65.41, 61.74, 55.0],
    arp:  [220.0, 261.63, 329.63, 440.0, 466.16, 440.0, 329.63, 261.63, 220.0, 293.66, 349.23, 440.0, 523.25, 466.16, 440.0, 329.63],
    lead: [440.0, 466.16, 440.0, 329.63, 523.25, 0, 466.16, 440.0, 587.33, 0, 523.25, 466.16, 440.0, 329.63, 440.0, 0]
  };

  function utcSeed(offsetDays) {
    var d = new Date(Date.now() + (offsetDays || 0) * 86400000);
    var y = d.getUTCFullYear();
    var m = String(d.getUTCMonth() + 1).padStart(2, '0');
    var day = String(d.getUTCDate()).padStart(2, '0');
    return y + '-' + m + '-' + day;
  }

  function hashSeedU64(seed) {
    try {
      var mask = 0xffffffffffffffffn;
      var h = 0xcbf29ce484222325n;
      for (var i = 0; i < seed.length; i++) {
        h ^= BigInt(seed.charCodeAt(i) & 0xff);
        h = (h * 0x100000001b3n) & mask;
      }
      if (h === 0n) h = 0x9e3779b97f4a7c15n;
      var x = h;
      x ^= x >> 12n;
      x ^= (x << 25n) & mask;
      x ^= x >> 27n;
      return (x * 0x2545f4914f6cdd1dn) & mask;
    } catch (_) {
      return 0n;
    }
  }

  function defaultBiomeIdx(seed) {
    try { return Number(hashSeedU64(seed + '-biome') % 3n); } catch (_) { return 0; }
  }

  function defaultMutatorIdx(seed) {
    try { return Number(hashSeedU64(seed) % 4n); } catch (_) { return 0; }
  }

  function ensureAudioCtx() {
    if (!audioCtx) {
      var Ctx = window.AudioContext || window.webkitAudioContext;
      if (!Ctx) return null;
      audioCtx = new Ctx();
      try {
        var comp = audioCtx.createDynamicsCompressor();
        comp.threshold.value = -16;
        comp.knee.value = 18;
        comp.ratio.value = 6;
        comp.attack.value = 0.004;
        comp.release.value = 0.16;
        comp.connect(audioCtx.destination);
        masterBus = comp;

        // Procedural cathedral impulse response for ConvolverNode reverb
        if (typeof audioCtx.createConvolver === 'function') {
          var conv = audioCtx.createConvolver();
          var rate = audioCtx.sampleRate;
          var irLen = Math.floor(rate * 1.15);
          var ir = audioCtx.createBuffer(2, irLen, rate);
          for (var ch = 0; ch < 2; ch++) {
            var data = ir.getChannelData(ch);
            for (var i = 0; i < irLen; i++) {
              var decay = Math.pow(1 - i / irLen, 2.8);
              data[i] = (Math.random() * 2 - 1) * decay * 0.45;
            }
          }
          conv.buffer = ir;
          var wetGain = audioCtx.createGain();
          wetGain.gain.value = 0.22;
          conv.connect(wetGain);
          wetGain.connect(masterBus);
          reverbBus = conv;
        }
      } catch (_) {
        masterBus = audioCtx.destination;
      }
    }
    if (audioCtx.state === 'suspended') {
      try { audioCtx.resume(); } catch (_) {}
    }
    return audioCtx;
  }

  function ensureNoiseBuffer(ctx) {
    if (noiseBuffer) return noiseBuffer;
    try {
      var len = Math.floor(ctx.sampleRate * 0.5);
      var buf = ctx.createBuffer(1, len, ctx.sampleRate);
      var data = buf.getChannelData(0);
      for (var i = 0; i < len; i++) {
        data[i] = Math.random() * 2 - 1;
      }
      noiseBuffer = buf;
    } catch (_) {}
    return noiseBuffer;
  }

  function unlockAudio() {
    try {
      var ctx = ensureAudioCtx();
      if (!ctx) return;
      var buf = ctx.createBuffer(1, 1, 22050);
      var src = ctx.createBufferSource();
      src.buffer = buf;
      src.connect(ctx.destination);
      if (src.start) src.start(0);
      if (musicActive && !seqTimer) startSequencer();
    } catch (_) {}
  }

  ['pointerdown', 'touchstart', 'keydown'].forEach(function (evt) {
    window.addEventListener(evt, unlockAudio, { passive: true });
  });

  function tone(f1, f2, dur, type, vol, pitchMul, delaySec, useReverb, filterHz) {
    if (isMuted) return;
    try {
      var ctx = ensureAudioCtx();
      if (!ctx) return;
      var pm = (typeof pitchMul === 'number' && isFinite(pitchMul)) ? Math.max(0.4, Math.min(2.5, pitchMul)) : 1.0;
      var now = ctx.currentTime + (delaySec || 0);
      var osc = ctx.createOscillator();
      var g = ctx.createGain();
      osc.type = type || 'triangle';
      osc.frequency.setValueAtTime(Math.max(25, f1 * pm), now);
      osc.frequency.exponentialRampToValueAtTime(Math.max(25, f2 * pm), now + dur);
      g.gain.setValueAtTime(0.0001, now);
      g.gain.linearRampToValueAtTime(vol || 0.1, now + Math.min(0.012, dur * 0.2));
      g.gain.exponentialRampToValueAtTime(0.0001, now + dur);

      var dest = masterBus || ctx.destination;
      if (filterHz && filterHz > 0) {
        var flt = ctx.createBiquadFilter();
        flt.type = 'lowpass';
        flt.frequency.setValueAtTime(filterHz, now);
        flt.Q.value = 1.6;
        osc.connect(flt);
        flt.connect(g);
      } else {
        osc.connect(g);
      }
      g.connect(dest);
      if (useReverb && reverbBus) g.connect(reverbBus);
      osc.start(now);
      osc.stop(now + dur + 0.03);
    } catch (_) {}
  }

  function noiseBurst(dur, filterFreq, filterType, vol, delaySec, useReverb) {
    if (isMuted) return;
    try {
      var ctx = ensureAudioCtx();
      if (!ctx) return;
      var buf = ensureNoiseBuffer(ctx);
      if (!buf) return;
      var now = ctx.currentTime + (delaySec || 0);
      var src = ctx.createBufferSource();
      src.buffer = buf;
      var flt = ctx.createBiquadFilter();
      flt.type = filterType || 'bandpass';
      flt.frequency.setValueAtTime(filterFreq || 900, now);
      flt.frequency.exponentialRampToValueAtTime(Math.max(120, (filterFreq || 900) * 0.35), now + dur);
      var g = ctx.createGain();
      g.gain.setValueAtTime(0.0001, now);
      g.gain.linearRampToValueAtTime(vol || 0.06, now + 0.008);
      g.gain.exponentialRampToValueAtTime(0.0001, now + dur);
      src.connect(flt);
      flt.connect(g);
      var dest = masterBus || ctx.destination;
      g.connect(dest);
      if (useReverb && reverbBus) g.connect(reverbBus);
      src.start(now);
      src.stop(now + dur + 0.03);
    } catch (_) {}
  }

  function stepSequencer() {
    if (isMuted || !musicActive) return;
    var theme = bossMusicPhase > 0 ? BOSS_THEME : (BIOME_THEMES[currentBiomeIdx % 3] || BIOME_THEMES[0]);
    var i = seqStep % 16;
    var bar = Math.floor(seqStep / 16) % 4;
    seqStep++;

    // Track 1: Punchy Synthwave Sub + Drive Bass
    var bNote = theme.bass[i];
    if (bNote > 0) {
      tone(bNote, bNote * 0.992, 0.17, 'sawtooth', bossMusicPhase > 0 ? 0.052 : 0.040, 1.0, 0, false, 340);
      if (i % 2 === 0) {
        tone(bNote * 0.5, bNote * 0.495, 0.18, 'triangle', 0.048, 1.0, 0, false, 180);
      }
    }

    // Track 2: 16-bit Crystalline Arpeggiator
    var aNote = theme.arp[i];
    if (aNote > 0) {
      var arpWave = currentBiomeIdx === 2 && bossMusicPhase === 0 ? 'triangle' : 'square';
      tone(aNote, aNote, 0.11, arpWave, bossMusicPhase > 0 ? 0.028 : 0.022, 1.0, 0, true, 1900);
    }

    // Track 3: Heroic Melodic Lead (enters on bars 1-3 or always during Boss)
    var lNote = theme.lead[i];
    if (lNote > 0 && (bossMusicPhase > 0 || bar >= 1)) {
      tone(lNote, lNote * 1.003, 0.24, 'sawtooth', bossMusicPhase === 2 ? 0.038 : 0.028, 1.0, 0, true, 2400);
    }

    // Track 4: Synthwave Percussion (Kick + Gated Snare + Crisp Hat)
    var kickBeat = (i % 4 === 0) || (bossMusicPhase === 2 && i % 4 === 2);
    if (kickBeat) {
      tone(128, 38, 0.13, 'sine', bossMusicPhase > 0 ? 0.088 : 0.068, 1.0, 0, false, 0);
    }
    if (i === 4 || i === 12) {
      noiseBurst(0.11, 1450, 'bandpass', bossMusicPhase > 0 ? 0.048 : 0.036, 0, true);
      tone(195, 110, 0.06, 'triangle', 0.035, 1.0, 0, false, 0);
    }
    if (i % 2 === 1) {
      noiseBurst(0.032, 6500, 'highpass', 0.015, 0, false);
    }
  }

  var choirNodes = null;

  function ensureChoirDrone() {
    if (isMuted || !musicActive) return;
    var ctx = ensureAudioCtx();
    if (!ctx || choirNodes) return;
    try {
      var sub = ctx.createOscillator();
      var choir1 = ctx.createOscillator();
      var choir2 = ctx.createOscillator();
      var vowel1 = ctx.createBiquadFilter();
      var vowel2 = ctx.createBiquadFilter();
      var subGain = ctx.createGain();
      var choirGain = ctx.createGain();

      sub.type = 'sine';
      sub.frequency.value = 36.71; // D1 deep sub-bass drone
      choir1.type = 'sawtooth';
      choir1.frequency.value = 146.83; // D3
      choir2.type = 'sawtooth';
      choir2.frequency.value = 220.25; // A3 fifth detuned

      vowel1.type = 'bandpass';
      vowel1.frequency.value = 520;
      vowel1.Q.value = 4.2;
      vowel2.type = 'bandpass';
      vowel2.frequency.value = 1120;
      vowel2.Q.value = 5.0;

      subGain.gain.value = 0.032;
      choirGain.gain.value = 0.018;

      sub.connect(subGain);
      subGain.connect(masterBus || ctx.destination);

      choir1.connect(vowel1);
      choir2.connect(vowel1);
      choir1.connect(vowel2);
      choir2.connect(vowel2);
      vowel1.connect(choirGain);
      vowel2.connect(choirGain);
      choirGain.connect(masterBus || ctx.destination);
      if (reverbBus) choirGain.connect(reverbBus);

      sub.start();
      choir1.start();
      choir2.start();
      choirNodes = { sub: sub, choir1: choir1, choir2: choir2, vowel1: vowel1, vowel2: vowel2, choirGain: choirGain, subGain: subGain };
    } catch (_) {}
  }

  function updateChoirIntensity(combo, overdrive) {
    if (!choirNodes || !audioCtx) return;
    try {
      var intensity = overdrive ? 1.0 : Math.min(1.0, (combo || 0) / 40.0);
      var now = audioCtx.currentTime;
      choirNodes.vowel1.frequency.setTargetAtTime(520 + intensity * 340, now, 0.15);
      choirNodes.vowel2.frequency.setTargetAtTime(1120 + intensity * 580, now, 0.15);
      choirNodes.choirGain.gain.setTargetAtTime(isMuted ? 0.0001 : (0.016 + intensity * 0.022), now, 0.15);
      choirNodes.subGain.gain.setTargetAtTime(isMuted ? 0.0001 : (0.028 + intensity * 0.020), now, 0.15);
    } catch (_) {}
  }

  function stopChoirDrone() {
    if (!choirNodes) return;
    try {
      choirNodes.sub.stop();
      choirNodes.choir1.stop();
      choirNodes.choir2.stop();
    } catch (_) {}
    choirNodes = null;
  }

  function startSequencer() {
    stopSequencer();
    if (!musicActive) return;
    ensureChoirDrone();
    var bpm = bossMusicPhase === 2 ? 152 : (bossMusicPhase === 1 ? 140 : (BIOME_THEMES[currentBiomeIdx % 3] || BIOME_THEMES[0]).bpm);
    var stepMs = Math.max(85, Math.round((60000 / bpm) / 2)); // 8th-note steps
    seqTimer = setInterval(stepSequencer, stepMs);
  }

  function stopSequencer() {
    if (seqTimer) {
      clearInterval(seqTimer);
      seqTimer = null;
    }
    stopChoirDrone();
  }

  function setBossMusic(phase) {
    var nextPhase = Math.max(0, Math.min(2, phase | 0));
    if (bossMusicPhase !== nextPhase) {
      bossMusicPhase = nextPhase;
      if (musicActive) startSequencer();
    }
  }

  function playVoiceBlip(speakerKind) {
    if (isMuted) return;
    if (speakerKind === 0) {
      tone(210, 260, 0.065, 'triangle', 0.075, 1.0, 0, true, 1200);
      tone(245, 295, 0.06, 'triangle', 0.065, 1.0, 0.07, true, 1200);
    } else if (speakerKind === 1) {
      tone(340, 420, 0.06, 'sine', 0.07, 1.0, 0, true, 2200);
      tone(420, 510, 0.055, 'triangle', 0.06, 1.0, 0.065, true, 2200);
    } else if (speakerKind === 2) {
      tone(290, 210, 0.05, 'sawtooth', 0.055, 1.0, 0, true, 1400);
      tone(320, 250, 0.05, 'sawtooth', 0.05, 1.0, 0.055, true, 1400);
    } else if (speakerKind === 3) {
      tone(110, 88, 0.11, 'sawtooth', 0.085, 1.0, 0, true, 620);
      tone(98, 78, 0.12, 'sawtooth', 0.08, 1.0, 0.09, true, 540);
    } else if (speakerKind === 4) {
      tone(82, 58, 0.14, 'sawtooth', 0.095, 1.0, 0, true, 420);
      tone(73, 52, 0.14, 'square', 0.07, 1.0, 0.10, true, 360);
    } else {
      tone(523, 659, 0.09, 'sine', 0.075, 1.0, 0, true, 3200);
      tone(659, 587, 0.10, 'triangle', 0.065, 1.0, 0.08, true, 3000);
    }
  }

  function playSfxPitched(kind, pitchMul) {
    if (isMuted) return;
    var jitter = 0.94 + Math.random() * 0.12;
    var pm = (pitchMul || 1.0) * jitter;
    if (kind === 0) {
      tone(440, 659, 0.085, 'triangle', 0.08, pitchMul, 0, true, 2800);
      tone(659, 880, 0.09, 'sine', 0.055, pitchMul, 0.04, true, 3200);
    } else if (kind === 1) {
      tone(540, 185, 0.085, 'sawtooth', 0.045, pm, 0, false, 2000);
      noiseBurst(0.045, 1900, 'highpass', 0.025, 0, false);
    } else if (kind === 2) {
      tone(210, 68, 0.075, 'square', 0.046, pm, 0, false, 900);
      noiseBurst(0.038, 650, 'bandpass', 0.028, 0, false);
    } else if (kind === 3) {
      tone(1320, 360, 0.14, 'sawtooth', 0.062, pm, 0, true, 3400);
      noiseBurst(0.11, 2900, 'highpass', 0.052, 0, true);
    } else if (kind === 4) {
      tone(165, 38, 0.28, 'sawtooth', 0.115, pm, 0, true, 620);
      noiseBurst(0.24, 480, 'lowpass', 0.095, 0, true);
    } else if (kind === 5) {
      tone(523.25, 523.25, 0.14, 'triangle', 0.09, pitchMul, 0, true, 3200);
      tone(659.25, 659.25, 0.16, 'triangle', 0.09, pitchMul, 0.065, true, 3200);
      tone(783.99, 1046.5, 0.28, 'sine', 0.105, pitchMul, 0.13, true, 3600);
    } else if (kind === 6) {
      tone(110, 73.4, 0.56, 'sawtooth', 0.125, pitchMul, 0, true, 780);
      tone(55, 46, 0.60, 'triangle', 0.135, pitchMul, 0.02, true, 420);
      noiseBurst(0.35, 340, 'lowpass', 0.055, 0.04, true);
    } else if (kind === 7) {
      tone(175, 54, 0.19, 'sawtooth', 0.115, pm, 0, false, 700);
      noiseBurst(0.09, 420, 'bandpass', 0.062, 0, false);
    } else if (kind === 8) {
      tone(880, 1320, 0.065, 'sine', 0.05, pitchMul, 0, true, 0);
    } else if (kind === 9) {
      tone(392, 587, 0.17, 'triangle', 0.088, pitchMul, 0, true, 2800);
      tone(587, 880, 0.24, 'sine', 0.085, pitchMul, 0.075, true, 3400);
    } else if (kind === 10) {
      noiseBurst(0.15, 1650, 'bandpass', 0.075, 0, true);
      tone(360, 115, 0.14, 'triangle', 0.07, pm, 0, false, 1400);
    } else if (kind === 11) {
      tone(660, 130, 0.36, 'sine', 0.115, 1.0, 0, true, 0);
      tone(1320, 1760, 0.28, 'triangle', 0.082, 1.0, 0.05, true, 0);
      tone(987.77, 1480, 0.25, 'sine', 0.065, 1.0, 0.11, true, 0);
    } else if (kind === 12) {
      // SSS Triumphant Cathedral Brass Fanfare
      tone(523.25, 523.25, 0.16, 'sawtooth', 0.08, 1.0, 0, true, 2600);
      tone(659.25, 659.25, 0.16, 'sawtooth', 0.08, 1.0, 0.11, true, 2600);
      tone(783.99, 783.99, 0.18, 'sawtooth', 0.088, 1.0, 0.22, true, 2800);
      tone(1046.5, 1046.5, 0.62, 'triangle', 0.12, 1.0, 0.35, true, 3400);
      tone(523.25, 523.25, 0.62, 'sawtooth', 0.075, 1.0, 0.35, true, 1800);
    } else if (kind === 13) {
      tone(220, 196, 0.30, 'sawtooth', 0.08, 1.0, 0, true, 900);
      tone(164.81, 146.83, 0.42, 'triangle', 0.085, 1.0, 0.18, true, 700);
      tone(110, 92.5, 0.55, 'sawtooth', 0.09, 1.0, 0.32, true, 520);
    } else if (kind === 14) {
      tone(55, 110, 0.65, 'sawtooth', 0.09, 1.0, 0, true, 900);
      tone(110, 220, 0.65, 'triangle', 0.08, 1.0, 0.08, true, 1600);
      noiseBurst(0.32, 620, 'lowpass', 0.075, 0.48, true);
      tone(220, 220, 0.55, 'sawtooth', 0.085, 1.0, 0.50, true, 2400);
      tone(329.63, 329.63, 0.55, 'triangle', 0.08, 1.0, 0.50, true, 2800);
    }
  }

  var initReady = 0;
  var submitState = 0;
  var shareState = 0;
  var currentRunId = '';
  var lastStatusMsg = '';
  var defaultTomorrow = utcSeed(1);
  var initData = {
    seed: utcSeed(0),
    mutatorOverride: -1,
    username: '',
    streak: 0,
    personalBest: 0,
    myRank: 0,
    totalPlayers: 0,
    bossKillsToday: 0,
    siegeTarget: 25,
    siegeFeed: [],
    factionScores: [0, 0, 0],
    yesterdayChampion: null,
    leaderboard: [],
    topGhost: null,
    canVoteMutator: true,
    tomorrowBiomeIndex: defaultBiomeIdx(defaultTomorrow),
    tomorrowLeadingMutator: defaultMutatorIdx(defaultTomorrow),
    tomorrowMutatorVotes: [0, 0, 0, 0],
    tomorrowTotalVotes: 0,
    myTomorrowVote: -1,
    medalsMask: 0,
    bestEndlessWave: 0,
    selectedAura: 0,
    yesterdayWinningFaction: -1,
    localStreak: 0,
    localStreakDate: ''
  };

  function loadLocalProfile() {
    try {
      var raw = localStorage.getItem('ef_profile_v1');
      if (!raw) return;
      var p = JSON.parse(raw);
      if (p && typeof p === 'object') {
        initData.medalsMask = (initData.medalsMask | (Number(p.medalsMask) || 0)) & 63;
        initData.bestEndlessWave = Math.max(initData.bestEndlessWave, Number(p.bestEndlessWave) || 0);
        initData.selectedAura = Math.max(0, Math.min(3, Number(p.selectedAura) || 0));
        if (typeof p.muted === 'boolean') isMuted = p.muted;
        if (typeof p.crt === 'boolean') crtEnabled = p.crt;
        var today = utcSeed(0);
        var yesterday = utcSeed(-1);
        if (p.localStreakDate === today || p.localStreakDate === yesterday) {
          initData.localStreak = Math.max(0, Number(p.localStreak) || 0);
          initData.localStreakDate = p.localStreakDate;
          if (initData.streak === 0) initData.streak = initData.localStreak;
        }
        if (p.localVoteDate === today && typeof p.localVoteIdx === 'number' && p.localVoteIdx >= 0 && p.localVoteIdx <= 3) {
          initData.myTomorrowVote = p.localVoteIdx;
        }
      }
    } catch (_) {}
  }

  function saveLocalProfile() {
    try {
      localStorage.setItem('ef_profile_v1', JSON.stringify({
        medalsMask: initData.medalsMask & 63,
        bestEndlessWave: initData.bestEndlessWave,
        selectedAura: initData.selectedAura,
        muted: isMuted,
        crt: crtEnabled,
        localStreak: initData.localStreak,
        localStreakDate: initData.localStreakDate,
        localVoteDate: utcSeed(0),
        localVoteIdx: initData.myTomorrowVote
      }));
    } catch (_) {}
  }

  function bumpLocalStreak() {
    var today = utcSeed(0);
    var yesterday = utcSeed(-1);
    if (initData.localStreakDate === today) {
      initData.localStreak = Math.max(1, initData.localStreak);
    } else if (initData.localStreakDate === yesterday) {
      initData.localStreak = Math.max(1, initData.localStreak + 1);
    } else {
      initData.localStreak = 1;
    }
    initData.localStreakDate = today;
    if (initData.streak < initData.localStreak) {
      initData.streak = initData.localStreak;
    }
    saveLocalProfile();
  }

  loadLocalProfile();

  function applyInitPayload(d) {
    if (!d || typeof d !== 'object') return;
    initData.seed = typeof d.seed === 'string' ? d.seed : initData.seed;
    initData.mutatorOverride = typeof d.mutatorOverride === 'number' ? d.mutatorOverride : -1;
    initData.username = typeof d.username === 'string' ? d.username : '';
    var srvStreak = Number(d.streak) || 0;
    initData.streak = initData.username ? srvStreak : Math.max(srvStreak, initData.localStreak);
    initData.personalBest = Number(d.personalBest) || 0;
    initData.myRank = Number(d.myRank) || 0;
    initData.totalPlayers = Number(d.totalPlayers) || 0;
    initData.bossKillsToday = Number(d.bossKillsToday) || 0;
    initData.siegeTarget = Number(d.siegeTarget) || 25;
    if (Array.isArray(d.siegeFeed)) {
      initData.siegeFeed = d.siegeFeed.slice(0, 6);
    }
    if (Array.isArray(d.factionScores) && d.factionScores.length >= 3) {
      initData.factionScores = [
        Number(d.factionScores[0]) || 0,
        Number(d.factionScores[1]) || 0,
        Number(d.factionScores[2]) || 0
      ];
    }
    initData.yesterdayChampion = (d.yesterdayChampion && typeof d.yesterdayChampion.username === 'string')
      ? d.yesterdayChampion
      : null;
    initData.leaderboard = Array.isArray(d.leaderboard) ? d.leaderboard : [];
    initData.topGhost = d.topGhost && Array.isArray(d.topGhost.points) ? d.topGhost : null;
    initData.canVoteMutator = typeof d.canVoteMutator === 'boolean' ? d.canVoteMutator : Boolean(initData.username);
    if (typeof d.tomorrowBiomeIndex === 'number') initData.tomorrowBiomeIndex = d.tomorrowBiomeIndex | 0;
    if (typeof d.tomorrowLeadingMutator === 'number') initData.tomorrowLeadingMutator = d.tomorrowLeadingMutator | 0;
    if (Array.isArray(d.tomorrowMutatorVotes) && d.tomorrowMutatorVotes.length >= 4) {
      initData.tomorrowMutatorVotes = [
        Number(d.tomorrowMutatorVotes[0]) || 0,
        Number(d.tomorrowMutatorVotes[1]) || 0,
        Number(d.tomorrowMutatorVotes[2]) || 0,
        Number(d.tomorrowMutatorVotes[3]) || 0
      ];
    }
    if (typeof d.tomorrowTotalVotes === 'number') initData.tomorrowTotalVotes = d.tomorrowTotalVotes | 0;
    if (typeof d.myTomorrowVote === 'number' && d.myTomorrowVote >= 0) {
      initData.myTomorrowVote = d.myTomorrowVote | 0;
    }
    initData.medalsMask = (initData.medalsMask | (Number(d.medalsMask) || 0)) & 63;
    initData.bestEndlessWave = Math.max(initData.bestEndlessWave, Number(d.bestEndlessWave) || 0);
    if (typeof d.yesterdayWinningFaction === 'number') {
      initData.yesterdayWinningFaction = d.yesterdayWinningFaction;
    }
    saveLocalProfile();
  }

  function fetchInit() {
    return fetch('/api/init', { cache: 'no-store' })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (json) {
        if (json) applyInitPayload(json);
        initReady = 1;
      })
      .catch(function () {
        initReady = 1;
      });
  }

  function computeRank(score, bossSlain) {
    if (bossSlain && score >= 14000) return 'SSS';
    if (bossSlain && score >= 9500) return 'SS';
    if (score >= 7000) return 'S';
    if (score >= 4500) return 'A';
    if (score >= 2500) return 'B';
    if (score >= 1200) return 'C';
    if (score >= 500) return 'D';
    return 'E';
  }

  function readUtf8(ptr, len) {
    if (!ptr || !len || len <= 0) return '';
    try {
      if (typeof UTF8ToString === 'function') return UTF8ToString(ptr, len);
      var bytes = new Uint8Array(wasm_memory.buffer, ptr, len);
      return new TextDecoder('utf-8').decode(bytes);
    } catch (_) {
      return '';
    }
  }

  function readGhostPoints(ptr, count) {
    var n = Math.min(200, Math.max(0, count | 0));
    if (!ptr || n === 0) return [];
    try {
      var arr = new Int16Array(wasm_memory.buffer, ptr, n * 2);
      var pts = [];
      for (var i = 0; i < n; i++) {
        pts.push([arr[i * 2], arr[i * 2 + 1]]);
      }
      return pts;
    } catch (_) {
      return [];
    }
  }

  var textEncoder = typeof TextEncoder !== 'undefined' ? new TextEncoder() : null;
  function writeUtf8(str, outPtr, maxLen) {
    if (!outPtr || !maxLen || maxLen <= 0 || !str) return 0;
    try {
      var dst = new Uint8Array(wasm_memory.buffer, outPtr, maxLen);
      var bytes;
      if (textEncoder) {
        bytes = textEncoder.encode(str);
      } else {
        bytes = [];
        for (var i = 0; i < str.length; i++) bytes.push(str.charCodeAt(i) & 0x7f);
      }
      var len = Math.min(maxLen, bytes.length);
      for (var j = 0; j < len; j++) dst[j] = bytes[j];
      return len;
    } catch (_) {
      return 0;
    }
  }

  fetchInit();

  miniquad_add_plugin({
    register_plugin: function (importObject) {
      var env = importObject.env;

      env.emberfall_get_day_number = function () {
        var d = new Date();
        return d.getUTCFullYear() * 10000 + (d.getUTCMonth() + 1) * 100 + d.getUTCDate();
      };

      env.emberfall_play_sfx = function (kind) {
        playSfxPitched(kind, 1.0);
      };

      env.emberfall_play_sfx_pitched = function (kind, pitchMul) {
        playSfxPitched(kind, pitchMul);
      };

      env.emberfall_play_voice_blip = function (speakerKind) {
        playVoiceBlip(speakerKind | 0);
      };

      env.emberfall_set_biome_music = function (biomeIdx) {
        currentBiomeIdx = Math.max(0, Math.min(2, biomeIdx | 0));
        if (musicActive && bossMusicPhase === 0) startSequencer();
      };

      env.emberfall_set_crt = function (enabled) {
        crtEnabled = enabled !== 0;
        saveLocalProfile();
      };

      env.emberfall_set_muted = function (muted) {
        isMuted = muted !== 0;
        saveLocalProfile();
      };

      env.emberfall_toggle_drone = function (active) {
        musicActive = active !== 0;
        if (musicActive) {
          seqStep = 0;
          startSequencer();
        } else {
          bossMusicPhase = 0;
          stopSequencer();
        }
      };

      env.emberfall_set_boss_music = function (active) {
        setBossMusic(active | 0);
      };

      env.emberfall_on_run_start = function () {
        currentRunId = '';
        submitState = 0;
        shareState = 0;
        fetch('/api/start-run', { method: 'POST' })
          .then(function (r) { return r.ok ? r.json() : null; })
          .then(function (json) {
            if (json && json.runId) currentRunId = String(json.runId);
          })
          .catch(function () {});
      };

      env.emberfall_on_run_complete = function (score, kills, bossSlain, level, heroClass, buildPtr, buildLen, ghostPtr, ghostCount, isEndless, endlessWave, medalsMask) {
        var buildSummary = readUtf8(buildPtr, buildLen);
        var ghostPoints = readGhostPoints(ghostPtr, ghostCount);
        var endlessFlag = isEndless === 1;
        var waveVal = Math.max(0, endlessWave | 0);
        var maskVal = (medalsMask | 0) & 63;

        initData.medalsMask = (initData.medalsMask | maskVal) & 63;
        if (endlessFlag && waveVal > initData.bestEndlessWave) {
          initData.bestEndlessWave = waveVal;
        }
        bumpLocalStreak();

        submitState = 1;
        lastStatusMsg = 'Submitting run...';

        fetch('/api/submit-run', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            runId: currentRunId || undefined,
            score: score,
            kills: kills,
            bossSlain: bossSlain === 1,
            bossReached: bossSlain === 1 || score >= 1200,
            level: level,
            heroClass: heroClass | 0,
            buildSummary: buildSummary,
            ghostPoints: ghostPoints,
            isEndless: endlessFlag,
            endlessWave: waveVal,
            medalsMask: maskVal
          })
        })
          .then(function (r) {
            return r.json().catch(function () { return {}; }).then(function (json) {
              return { ok: r.ok, status: r.status, json: json };
            });
          })
          .then(function (res) {
            if (res.ok && res.json && res.json.ok) {
              submitState = 2;
              initData.myRank = Number(res.json.myRank) || initData.myRank;
              initData.totalPlayers = Number(res.json.totalPlayers) || initData.totalPlayers;
              initData.streak = Math.max(Number(res.json.streak) || 0, initData.localStreak);
              initData.bossKillsToday = Number(res.json.bossKillsToday) || initData.bossKillsToday;
              if (Array.isArray(res.json.siegeFeed)) {
                initData.siegeFeed = res.json.siegeFeed.slice(0, 6);
              }
              if (Array.isArray(res.json.factionScores) && res.json.factionScores.length >= 3) {
                initData.factionScores = [
                  Number(res.json.factionScores[0]) || 0,
                  Number(res.json.factionScores[1]) || 0,
                  Number(res.json.factionScores[2]) || 0
                ];
              }
              initData.canVoteMutator = Boolean(res.json.canVoteMutator);
              if (Array.isArray(res.json.tomorrowMutatorVotes) && res.json.tomorrowMutatorVotes.length >= 4) {
                initData.tomorrowMutatorVotes = [
                  Number(res.json.tomorrowMutatorVotes[0]) || 0,
                  Number(res.json.tomorrowMutatorVotes[1]) || 0,
                  Number(res.json.tomorrowMutatorVotes[2]) || 0,
                  Number(res.json.tomorrowMutatorVotes[3]) || 0
                ];
              }
              if (typeof res.json.tomorrowLeadingMutator === 'number') {
                initData.tomorrowLeadingMutator = res.json.tomorrowLeadingMutator | 0;
              }
              if (typeof res.json.tomorrowTotalVotes === 'number') {
                initData.tomorrowTotalVotes = res.json.tomorrowTotalVotes | 0;
              }
              if (typeof res.json.myTomorrowVote === 'number' && res.json.myTomorrowVote >= 0) {
                initData.myTomorrowVote = res.json.myTomorrowVote | 0;
              }
              initData.medalsMask = (initData.medalsMask | (Number(res.json.medalsMask) || 0)) & 63;
              initData.bestEndlessWave = Math.max(initData.bestEndlessWave, Number(res.json.bestEndlessWave) || 0);
              saveLocalProfile();
              if (score > initData.personalBest) initData.personalBest = score;
              lastStatusMsg = res.json.isNewBest ? 'New Daily Best!' : 'Score Submitted!';
              fetchInit();
            } else {
              submitState = 3;
              if (score > initData.personalBest) initData.personalBest = score;
              lastStatusMsg = (res.json && res.json.error) || 'Guest run saved locally';
            }
          })
          .catch(function () {
            submitState = 3;
            if (score > initData.personalBest) initData.personalBest = score;
            lastStatusMsg = 'Offline run saved locally';
          });
      };

      env.emberfall_share_comment = function (score, kills, bossSlain, level, heroClass, buildPtr, buildLen) {
        var buildEmojis = readUtf8(buildPtr, buildLen) || '⚔️🔥';
        var ov = initData.mutatorOverride;
        var mutatorName = (ov >= 0 && ov <= 3) ? MODIFIERS[ov] : '';
        shareState = 1;
        lastStatusMsg = 'Posting to Reddit...';

        fetch('/api/share-comment', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            score: score,
            kills: kills,
            bossSlain: bossSlain === 1,
            level: level || 1,
            heroClass: heroClass | 0,
            streak: initData.streak,
            mutatorName: mutatorName,
            buildEmojis: buildEmojis,
            rank: computeRank(score, bossSlain === 1),
            myRank: initData.myRank,
            chronicle: lastChronicle
          })
        })
          .then(function (r) {
            return r.json().catch(function () { return {}; }).then(function (json) {
              return { ok: r.ok, status: r.status, json: json };
            });
          })
          .then(function (res) {
            if (res.ok && res.json && res.json.ok) {
              shareState = 2;
              lastStatusMsg = 'Shared to Reddit comments!';
            } else {
              shareState = 3;
              lastStatusMsg = (res.json && res.json.error) || 'Play on Reddit to post comments';
            }
          })
          .catch(function () {
            shareState = 3;
            lastStatusMsg = 'Play on Reddit to post comments';
          });
      };

      env.emberfall_vote_mutator = function (mutatorIdx) {
        var idx = Math.max(0, Math.min(3, mutatorIdx | 0));
        initData.myTomorrowVote = idx;
        saveLocalProfile();
        lastStatusMsg = 'Submitting community vote...';
        fetch('/api/vote-mutator', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mutatorIndex: idx })
        })
          .then(function (r) {
            return r.json().catch(function () { return {}; }).then(function (json) {
              return { ok: r.ok, status: r.status, json: json };
            });
          })
          .then(function (res) {
            if (res.ok && res.json && res.json.ok) {
              if (Array.isArray(res.json.tomorrowMutatorVotes) && res.json.tomorrowMutatorVotes.length >= 4) {
                initData.tomorrowMutatorVotes = [
                  Number(res.json.tomorrowMutatorVotes[0]) || 0,
                  Number(res.json.tomorrowMutatorVotes[1]) || 0,
                  Number(res.json.tomorrowMutatorVotes[2]) || 0,
                  Number(res.json.tomorrowMutatorVotes[3]) || 0
                ];
              }
              if (typeof res.json.tomorrowLeadingMutator === 'number') {
                initData.tomorrowLeadingMutator = res.json.tomorrowLeadingMutator | 0;
              }
              if (typeof res.json.tomorrowTotalVotes === 'number') {
                initData.tomorrowTotalVotes = res.json.tomorrowTotalVotes | 0;
              }
              lastStatusMsg = 'Voted for tomorrow: ' + (MODIFIERS[idx] || 'Mutator');
            } else {
              lastStatusMsg = (res.json && res.json.error) || 'Sign in on Reddit to vote live';
            }
          })
          .catch(function () {
            lastStatusMsg = 'Vote saved locally (Reddit required for live poll)';
          });
      };

      env.emberfall_set_selected_aura = function (auraIdx) {
        initData.selectedAura = Math.max(0, Math.min(3, auraIdx | 0));
        saveLocalProfile();
      };

      env.emberfall_get_gamepad_axis = function (axisIdx) {
        try {
          var gps = navigator.getGamepads ? navigator.getGamepads() : null;
          if (!gps) return 0;
          for (var i = 0; i < gps.length; i++) {
            var gp = gps[i];
            if (gp && gp.connected && gp.axes && gp.axes.length > axisIdx) {
              var v = gp.axes[axisIdx] || 0;
              if (Math.abs(v) > 0.16) return Math.round(v * 1000);
            }
          }
        } catch (_) {}
        return 0;
      };

      env.emberfall_get_gamepad_btn = function (btnIdx) {
        try {
          var gps = navigator.getGamepads ? navigator.getGamepads() : null;
          if (!gps) return 0;
          for (var i = 0; i < gps.length; i++) {
            var gp = gps[i];
            if (gp && gp.connected && gp.buttons && gp.buttons.length > btnIdx) {
              var b = gp.buttons[btnIdx];
              if (b && (b.pressed || b.value > 0.5)) return 1;
            }
          }
        } catch (_) {}
        return 0;
      };

      env.emberfall_poll_init_ready = function () {
        var ld = document.getElementById('ef-loader');
        if (ld) ld.remove();
        return initReady;
      };

      env.emberfall_get_init_field = function (fieldId) {
        switch (fieldId) {
          case 0: return initData.streak | 0;
          case 1: return initData.personalBest | 0;
          case 2: return initData.myRank | 0;
          case 3: return initData.totalPlayers | 0;
          case 4: return initData.bossKillsToday | 0;
          case 5: return initData.siegeTarget | 0;
          case 6: return initData.mutatorOverride | 0;
          case 7: return initData.canVoteMutator ? 1 : 0;
          case 8: return initData.leaderboard.length | 0;
          case 9: return (initData.topGhost && initData.topGhost.points) ? initData.topGhost.points.length : 0;
          case 10: return (initData.yesterdayChampion && initData.yesterdayChampion.score) ? (initData.yesterdayChampion.score | 0) : 0;
          case 11: return initData.medalsMask | 0;
          case 12: return initData.bestEndlessWave | 0;
          case 13: return initData.selectedAura | 0;
          case 14: return initData.yesterdayWinningFaction | 0;
          case 15: return initData.tomorrowBiomeIndex | 0;
          case 16: return initData.tomorrowLeadingMutator | 0;
          case 17: return initData.myTomorrowVote | 0;
          case 18: return initData.tomorrowTotalVotes | 0;
          case 19: return initData.siegeFeed ? initData.siegeFeed.length : 0;
          case 20: {
            var now = new Date();
            var nextMid = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() + 1);
            return Math.max(0, Math.floor((nextMid - now.getTime()) / 1000)) | 0;
          }
          case 21: return isMuted ? 1 : 0;
          case 22: return crtEnabled ? 1 : 0;
          default: return 0;
        }
      };

      env.emberfall_get_tomorrow_vote = function (index) {
        var idx = index | 0;
        if (!initData.tomorrowMutatorVotes || idx < 0 || idx >= 4) return 0;
        return (Number(initData.tomorrowMutatorVotes[idx]) || 0) >>> 0;
      };

      env.emberfall_copy_str_field = function (fieldId, index, outPtr, maxLen) {
        var str = '';
        if (fieldId === 0) str = initData.username || '';
        else if (fieldId === 1) {
          var row = initData.leaderboard[index | 0];
          str = row && row.username ? row.username : '';
        } else if (fieldId === 2) {
          str = (initData.topGhost && initData.topGhost.username) ? initData.topGhost.username : '';
        } else if (fieldId === 3) {
          str = lastStatusMsg || '';
        } else if (fieldId === 4) {
          str = (initData.yesterdayChampion && initData.yesterdayChampion.username) ? initData.yesterdayChampion.username : '';
        } else if (fieldId === 5) {
          var sf = initData.siegeFeed && initData.siegeFeed[index | 0];
          if (sf && sf.username) {
            str = 'u/' + sf.username + (sf.slain ? ' [SLAIN]' : ' [STRIKE]');
          }
        }
        return writeUtf8(str, outPtr, maxLen);
      };

      env.emberfall_get_lb_score = function (index) {
        var row = initData.leaderboard[index | 0];
        return row ? (Number(row.score) || 0) >>> 0 : 0;
      };

      env.emberfall_get_faction_score = function (index) {
        var idx = index | 0;
        if (!initData.factionScores || idx < 0 || idx >= 3) return 0;
        return (Number(initData.factionScores[idx]) || 0) >>> 0;
      };

      env.emberfall_get_ghost_x = function (index) {
        if (!initData.topGhost || !initData.topGhost.points) return 0;
        var pt = initData.topGhost.points[index | 0];
        return pt ? (pt[0] | 0) : 0;
      };

      env.emberfall_get_ghost_y = function (index) {
        if (!initData.topGhost || !initData.topGhost.points) return 0;
        var pt = initData.topGhost.points[index | 0];
        return pt ? (pt[1] | 0) : 0;
      };

      env.emberfall_get_share_state = function () {
        return shareState | 0;
      };

      env.emberfall_get_submit_state = function () {
        return submitState | 0;
      };

      env.emberfall_sync_hud = function (
        phase, biome, heroClass, hp, maxHp, ember, overdriveSec,
        xp, xpNext, lv, score, kills, combo, timeRem,
        bossHp, bossMaxHp, bossTrail, ghostDelta, muted, crt,
        rankPtr, rankLen, chronPtr, chronLen
      ) {
        try {
          updateHudDom(
            phase, biome, heroClass, hp, maxHp, ember, overdriveSec,
            xp, xpNext, lv, score, kills, combo, timeRem,
            bossHp, bossMaxHp, bossTrail, ghostDelta, muted, crt,
            readUtf8(rankPtr, rankLen), readUtf8(chronPtr, chronLen)
          );
        } catch (e) {
          if (typeof console !== 'undefined' && console.error) console.error(e);
        }
      };

      env.emberfall_sync_choices = function (c0, c1, c2, r0, r1, r2, b0, b1, b2) {
        lastChoices = [c0, c1, c2];
        lastRarities = [r0, r1, r2];
        lastBountiesDone = [b0 !== 0, b1 !== 0, b2 !== 0];
      };

      env.emberfall_sync_skills = function (dashCd, runicCd, cutinSec) {
        try {
          updateSkillsDom(dashCd, runicCd, cutinSec);
        } catch (e) {
          if (typeof console !== 'undefined' && console.error) console.error(e);
        }
      };
    },
    version: 1,
    name: 'emberfall_bridge'
  });

  // =========================================================================
  // v0.2.0 DIEGETIC PERIMETER HUD + RUBY VESSELS + ACTIVE SKILLS + CUT-IN
  // =========================================================================
  var BIOME_CLASSES = ['biome-obsidian', 'biome-volcanic', 'biome-frost'];
  var BIOME_BOSSES = ['MALAKOR, VOID LICH', 'IGNIS REX, MAGMA TITAN', 'SKADI, GLACIAL EMPRESS'];
  var CHAMP_PORTRAITS = ['./portrait_knight.webp', './portrait_mage.webp', './portrait_rogue.webp'];
  var CHAMP_NAMES = ['AURELIUS', 'SERAPHINA', 'NYX'];
  var UPGRADE_META = [
    { name: 'Arcane Slash', desc: 'Sweeping spectral blade arc (+Damage & Area)', synergy: '◆ SYNERGY: +Overclock Haste -> Crimson Reaper' },
    { name: 'Ember Bolts', desc: 'Seeking firebolts pierce the nearest threats', synergy: '◆ SYNERGY: +Chain Lightning -> Storm Sovereign' },
    { name: 'Chain Lightning', desc: 'Arcing storm bolts jump across the horde', synergy: '◆ SYNERGY: +Ember Bolts -> Storm Sovereign' },
    { name: 'Runic Orbit', desc: 'Spinning obsidian blades shred close foes', synergy: '◆ SYNERGY: +Nova Dash -> Inferno Vortex' },
    { name: 'Nova Dash', desc: 'Dash triggers an explosive shockwave on arrival', synergy: '◆ SYNERGY: +Runic Orbit -> Inferno Vortex' },
    { name: 'Overclock Haste', desc: 'Accelerates all weapon cooldowns & cast speed', synergy: '◆ SYNERGY: +Arcane Slash -> Crimson Reaper' },
    { name: 'Soul Magnet', desc: 'Pulls XP gems & relics from afar (+20% XP)', synergy: '◆ PASSIVE: +20% Faster Ascension' },
    { name: 'Vampiric Ward', desc: '+25 Max HP, +35 Heal & life-steal ward', synergy: '◆ PASSIVE: Blood Sustenance' }
  ];
  var RARITY_META = [
    { cls: 'rarity-rare', label: 'RARE BOON' },
    { cls: 'rarity-epic', label: 'EPIC RUNIC' },
    { cls: 'rarity-mythic', label: 'MYTHIC AWAKENING' }
  ];

  var selectedHeroClass = 0;
  var selectedEndless = 0;
  var lastPhase = -1;
  var lastHeroClass = 0;
  var lastChoices = [0, 1, 2];
  var lastRarities = [0, 0, 0];
  var lastBountiesDone = [false, false, false];
  var lastChronicle = '';
  var cutinVisible = false;
  var hudState = {
    hpPct: 1, emberPct: 0, overdrive: false, combo: 0, wavePhase: 0, biome: 0
  };
  var onboardingHidden = false;
  var onboardingStartTs = 0;

  function setVisible(id, show) {
    var el = document.getElementById(id);
    if (!el) return;
    if (show) el.classList.remove('is-hidden');
    else el.classList.add('is-hidden');
  }

  function callCmd(fnName, a, b) {
    try {
      if (typeof wasm_exports !== 'undefined' && wasm_exports && typeof wasm_exports[fnName] === 'function') {
        wasm_exports[fnName](a || 0, b || 0);
      }
    } catch (_) {}
  }

  function drawLiquidOrb(canvasId, pct, colTop, colBot, glowCol, phaseOff) {
    var cvs = document.getElementById(canvasId);
    if (!cvs || !cvs.getContext) return;
    var ctx = cvs.getContext('2d');
    var w = cvs.width;
    var h = cvs.height;
    var cx = w * 0.5;
    var cy = h * 0.5;
    var r = w * 0.45;
    ctx.clearRect(0, 0, w, h);

    ctx.save();
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.clip();

    ctx.fillStyle = 'rgba(9, 10, 15, 0.92)';
    ctx.fillRect(0, 0, w, h);

    var levelY = cy + r - pct * (r * 2);
    var grad = ctx.createLinearGradient(0, levelY - 10, 0, cy + r);
    grad.addColorStop(0, colTop);
    grad.addColorStop(1, colBot);

    ctx.beginPath();
    ctx.moveTo(0, h);
    for (var x = 0; x <= w; x += 4) {
      var wave = Math.sin(x * 0.08 + hudState.wavePhase + phaseOff) * (pct > 0.03 && pct < 0.98 ? 4.2 : 0.8);
      ctx.lineTo(x, levelY + wave);
    }
    ctx.lineTo(w, h);
    ctx.closePath();
    ctx.fillStyle = grad;
    ctx.fill();

    // Specular glass highlight
    var glass = ctx.createRadialGradient(cx - r * 0.32, cy - r * 0.35, 2, cx, cy, r);
    glass.addColorStop(0, 'rgba(255,255,255,0.34)');
    glass.addColorStop(0.45, 'rgba(255,255,255,0.05)');
    glass.addColorStop(1, 'rgba(0,0,0,0.45)');
    ctx.fillStyle = glass;
    ctx.fillRect(0, 0, w, h);
    ctx.restore();

    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.lineWidth = 3;
    ctx.strokeStyle = glowCol;
    ctx.stroke();
  }

  function drawCarnageThermometer() {
    var cvs = document.getElementById('carnage-canvas');
    if (!cvs || !cvs.getContext) return;
    var ctx = cvs.getContext('2d');
    var w = cvs.width;
    var h = cvs.height;
    ctx.clearRect(0, 0, w, h);

    var pct = hudState.overdrive ? 1.0 : Math.min(1.0, hudState.combo / 50.0);
    var fillH = Math.max(0, (h - 8) * pct);
    var grad = ctx.createLinearGradient(0, h, 0, 0);
    if (hudState.overdrive) {
      grad.addColorStop(0, '#FF4D2D');
      grad.addColorStop(0.5, '#E5B85C');
      grad.addColorStop(1, '#FCE194');
    } else {
      grad.addColorStop(0, '#9D4EDD');
      grad.addColorStop(0.6, '#FF4D2D');
      grad.addColorStop(1, '#E5B85C');
    }
    ctx.fillStyle = grad;
    ctx.fillRect(4, h - 4 - fillH, w - 8, fillH);
  }

  function drawCutinLightning(cutinSec) {
    var cvs = document.getElementById('cutin-canvas');
    if (!cvs || !cvs.getContext) return;
    var w = window.innerWidth || 800;
    var h = window.innerHeight || 600;
    if (cvs.width !== w || cvs.height !== h) {
      cvs.width = w;
      cvs.height = h;
    }
    var ctx = cvs.getContext('2d');
    ctx.clearRect(0, 0, w, h);
    var alpha = Math.min(1, cutinSec / 0.35);
    var cx = w * 0.5;
    var cy = h * 0.5;
    var colors = ['#38E8FF', '#FCE194', '#FF4D2D', '#C084FC'];
    for (var b = 0; b < 12; b++) {
      var ang = (b / 12) * Math.PI * 2 + cutinSec * 6.5;
      var len = Math.max(w, h) * 0.65;
      var tx = cx + Math.cos(ang) * len;
      var ty = cy + Math.sin(ang) * len;
      var pts = [[cx, cy]];
      var segs = 8;
      for (var s = 1; s <= segs; s++) {
        var t = s / segs;
        var jx = (s < segs) ? (Math.random() - 0.5) * 48 : 0;
        var jy = (s < segs) ? (Math.random() - 0.5) * 48 : 0;
        pts.push([cx + (tx - cx) * t + jx, cy + (ty - cy) * t + jy]);
      }
      ctx.save();
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      // Outer plasma halo
      ctx.globalAlpha = alpha * 0.34;
      ctx.strokeStyle = colors[b % colors.length];
      ctx.lineWidth = (b % 2 === 0) ? 9.0 : 5.5;
      ctx.beginPath();
      ctx.moveTo(pts[0][0], pts[0][1]);
      for (var p1 = 1; p1 < pts.length; p1++) ctx.lineTo(pts[p1][0], pts[p1][1]);
      ctx.stroke();
      // Crisp white-hot core
      ctx.globalAlpha = alpha * 0.92;
      ctx.strokeStyle = (b % 2 === 0) ? '#FFFFFF' : colors[b % colors.length];
      ctx.lineWidth = (b % 2 === 0) ? 2.4 : 1.6;
      ctx.beginPath();
      ctx.moveTo(pts[0][0], pts[0][1]);
      for (var p2 = 1; p2 < pts.length; p2++) ctx.lineTo(pts[p2][0], pts[p2][1]);
      ctx.stroke();
      ctx.restore();
    }
  }

  function updateSkillsDom(dashCd, runicCd, cutinSec) {
    var ringDash = document.getElementById('ring-dash');
    if (ringDash) {
      var dPct = Math.max(0, Math.min(1, (dashCd || 0) / 1.35));
      ringDash.setAttribute('stroke-dashoffset', String(Math.round(113 * dPct)));
    }
    var btnDash = document.getElementById('btn-skill-dash');
    if (btnDash) {
      if ((dashCd || 0) <= 0.02) btnDash.classList.add('ready');
      else btnDash.classList.remove('ready');
    }

    var ringRunic = document.getElementById('ring-runic');
    if (ringRunic) {
      var rPct = Math.max(0, Math.min(1, (runicCd || 0) / 4.5));
      ringRunic.setAttribute('stroke-dashoffset', String(Math.round(113 * rPct)));
    }
    var btnRunic = document.getElementById('btn-skill-runic');
    if (btnRunic) {
      if ((runicCd || 0) <= 0.02) btnRunic.classList.add('ready');
      else btnRunic.classList.remove('ready');
    }

    var showCutin = (cutinSec || 0) > 0.02 && lastPhase >= 2 && lastPhase <= 4;
    if (showCutin) {
      if (!cutinVisible) {
        cutinVisible = true;
        var cp = document.getElementById('cutin-portrait');
        if (cp) cp.setAttribute('src', CHAMP_PORTRAITS[lastHeroClass] || CHAMP_PORTRAITS[0]);
        var cc = document.getElementById('cutin-champ');
        if (cc) cc.textContent = (CHAMP_NAMES[lastHeroClass] || CHAMP_NAMES[0]) + ' AWAKENED';
        setVisible('electric-cutin', true);
      }
      drawCutinLightning(cutinSec);
    } else if (cutinVisible) {
      cutinVisible = false;
      setVisible('electric-cutin', false);
    }
  }

  function animOrbsLoop() {
    hudState.wavePhase += hudState.overdrive ? 0.14 : 0.07;
    drawLiquidOrb('orb-hp', hudState.hpPct, '#EF4444', '#7F1D1D', '#E5B85C', 0);
    drawLiquidOrb(
      'orb-ember',
      hudState.emberPct,
      hudState.overdrive ? '#FCE194' : '#FF7A2D',
      hudState.overdrive ? '#FF4D2D' : '#9D4EDD',
      hudState.emberPct >= 0.99 || hudState.overdrive ? '#FCE194' : 'rgba(229,184,92,0.55)',
      2.1
    );
    drawCarnageThermometer();
    requestAnimationFrame(animOrbsLoop);
  }
  requestAnimationFrame(animOrbsLoop);

  function renderUpgradeModal() {
    var grid = document.getElementById('upgrade-grid');
    if (!grid) return;
    grid.innerHTML = '';
    for (var i = 0; i < 3; i++) {
      (function (idx) {
        var uIdx = lastChoices[idx] || 0;
        var rIdx = Math.max(0, Math.min(2, lastRarities[idx] || 0));
        var meta = UPGRADE_META[uIdx] || UPGRADE_META[0];
        var rar = RARITY_META[rIdx] || RARITY_META[0];
        var btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'upgrade-card ' + rar.cls;
        btn.innerHTML =
          '<div class="rarity-tag">' + rar.label + ' [' + (idx + 1) + ']</div>' +
          '<div class="up-title">' + meta.name + '</div>' +
          '<div class="up-desc">' + meta.desc + '</div>' +
          '<div class="up-synergy">' + meta.synergy + '</div>';
        btn.addEventListener('click', function () {
          callCmd('ef_cmd_pick_upgrade', idx);
        });
        grid.appendChild(btn);
      })(i);
    }
  }

  function refreshChronicleVotesAndFeed() {
    var voteShort = ['BLOOD', 'ARCANE', 'SWARM', 'TITAN'];
    var vBtns = document.querySelectorAll('#chronicle-votes .vote-card');
    vBtns.forEach(function (vb, idx) {
      var cnt = (initData.tomorrowMutatorVotes && initData.tomorrowMutatorVotes[idx]) || 0;
      vb.textContent = cnt > 0 ? voteShort[idx] + ' (' + cnt + ')' : voteShort[idx];
      if (initData.myTomorrowVote === idx) vb.classList.add('voted');
      else vb.classList.remove('voted');
    });
    var sfEl = document.getElementById('chronicle-siege-feed');
    if (sfEl) {
      if (initData.siegeFeed && initData.siegeFeed.length > 0) {
        sfEl.textContent = 'LIVE SIEGE: ' + initData.siegeFeed.slice(0, 3).map(function (e) {
          return 'u/' + e.username + (e.slain ? ' [SLAIN]' : ' [STRIKE]');
        }).join(' · ');
      } else {
        sfEl.textContent = 'COMMUNITY SIEGE: ' + initData.bossKillsToday + ' / ' + initData.siegeTarget + ' OVERLORDS SLAIN';
      }
    }
  }

  function updateHudDom(
    phase, biome, heroClass, hp, maxHp, ember, overdriveSec,
    xp, xpNext, lv, score, kills, combo, timeRem,
    bossHp, bossMaxHp, bossTrail, ghostDelta, muted, crtFlags,
    rankStr, chronStr
  ) {
    var crt = (crtFlags & 1) !== 0;
    var onboardingDoneByInput = (crtFlags & 2) !== 0;
    var bCls = BIOME_CLASSES[biome % 3] || 'biome-obsidian';
    if (!document.body.classList.contains(bCls)) {
      document.body.classList.remove('biome-obsidian', 'biome-volcanic', 'biome-frost');
      document.body.classList.add(bCls);
    }

    var hc = ((heroClass % 3) + 3) % 3;
    lastHeroClass = hc;
    hudState.hpPct = Math.max(0, Math.min(1, hp / Math.max(1, maxHp)));
    hudState.emberPct = overdriveSec > 0 ? Math.max(0.15, overdriveSec / 6.0) : Math.max(0, Math.min(1, ember / 100));
    hudState.overdrive = overdriveSec > 0;
    hudState.combo = combo;
    hudState.biome = biome;
    if (chronStr) lastChronicle = chronStr;
    updateChoirIntensity(combo, overdriveSec > 0);

    if (phase !== lastPhase) {
      lastPhase = phase;
      setVisible('modal-title', phase === 0);
      setVisible('btn-skip-intro', phase === 1);
      setVisible('hud-layer', phase === 2 || phase === 3 || phase === 4);
      setVisible('modal-pause', phase === 3);
      setVisible('modal-levelup', phase === 4);
      setVisible('modal-chronicle', phase === 5);
      if (phase < 2 || phase > 4) {
        cutinVisible = false;
        setVisible('electric-cutin', false);
      }

      if (phase === 0) {
        var sub = document.getElementById('title-seed-sub');
        if (sub) sub.textContent = 'DAILY SANCTUARY · #' + (initData.seed || utcSeed(0));
        var tBoss = document.getElementById('title-boss-line');
        if (tBoss) tBoss.textContent = BIOME_BOSSES[biome % 3] || BIOME_BOSSES[0];
        var tMut = document.getElementById('title-mutator-line');
        var mIdx = initData.mutatorOverride >= 0 ? initData.mutatorOverride : defaultMutatorIdx(initData.seed || utcSeed(0));
        if (tMut) tMut.textContent = 'ACTIVE MUTATOR: ' + (MODIFIERS[mIdx] || MODIFIERS[0]) + ' · 60 FPS';
      } else if (phase === 2) {
        if (!onboardingStartTs) onboardingStartTs = Date.now();
      } else if (phase === 3) {
        var pb = document.getElementById('pause-bounties');
        if (pb) {
          pb.innerHTML = [0, 1, 2].map(function (bi) {
            return '<li>' + (lastBountiesDone[bi] ? '[DONE] ' : '[ACTIVE] ') + 'Daily Bounty #' + (bi + 1) + ' (+750 PTS)</li>';
          }).join('');
        }
      } else if (phase === 4) {
        renderUpgradeModal();
      } else if (phase === 5) {
        var cs = document.getElementById('chronicle-score');
        if (cs) cs.textContent = Number(score || 0).toLocaleString('en-US') + ' PTS';
        var cst = document.getElementById('chronicle-stats');
        if (cst) cst.textContent = kills + ' Slain · Lv.' + lv + ' · ' + Math.max(1, initData.streak) + '-Day Streak';
        var cr = document.getElementById('chronicle-rank');
        if (cr) cr.textContent = rankStr || computeRank(score, bossHp <= 0 && bossMaxHp > 0);
        var ct = document.getElementById('chronicle-text');
        if (ct && lastChronicle) ct.textContent = '"' + lastChronicle + '"';
        refreshChronicleVotesAndFeed();
      }
    }

    if (phase >= 2 && phase <= 4) {
      var hPort = document.getElementById('hud-portrait');
      if (hPort && hPort.getAttribute('data-hc') !== String(hc)) {
        hPort.setAttribute('data-hc', String(hc));
        hPort.setAttribute('src', CHAMP_PORTRAITS[hc] || CHAMP_PORTRAITS[0]);
      }
      var hName = document.getElementById('hud-champ-name');
      if (hName) hName.textContent = CHAMP_NAMES[hc] || CHAMP_NAMES[0];

      var vessels = document.getElementById('hp-vessels');
      if (vessels && vessels.children) {
        var vialsFloat = hudState.hpPct * 5.0;
        for (var vi = 0; vi < vessels.children.length; vi++) {
          var vEl = vessels.children[vi];
          var cls = 'hp-vial';
          if (vialsFloat >= vi + 0.72) cls = 'hp-vial full';
          else if (vialsFloat >= vi + 0.25) cls = 'hp-vial half';
          if (vEl.className !== cls) vEl.className = cls;
        }
      }

      var hLv = document.getElementById('hud-lv');
      if (hLv) hLv.textContent = 'LV.' + lv;
      var hSc = document.getElementById('hud-score');
      if (hSc) hSc.textContent = Number(score || 0).toLocaleString('en-US') + ' PTS';
      var xpRect = document.getElementById('hud-xp-rect');
      if (xpRect) xpRect.setAttribute('width', String(Math.round((xp / Math.max(1, xpNext)) * 100)));

      var secs = Math.max(0, Math.floor(timeRem));
      var tmEl = document.getElementById('hud-timer');
      if (tmEl) tmEl.textContent = String(Math.floor(secs / 60)).padStart(2, '0') + ':' + String(secs % 60).padStart(2, '0');
      var kEl = document.getElementById('hud-kills');
      if (kEl) kEl.textContent = kills + ' SLAIN';

      var mBtn = document.getElementById('btn-mute');
      if (mBtn) mBtn.textContent = muted ? 'MUTE' : 'AUDIO';
      var pmBtn = document.getElementById('btn-pause-mute');
      if (pmBtn) pmBtn.textContent = muted ? 'AUDIO: OFF' : 'AUDIO: ON';
      var pcBtn = document.getElementById('btn-pause-crt');
      if (pcBtn) pcBtn.textContent = crt ? 'CRT: ON' : 'CRT: OFF';

      var hpLbl = document.getElementById('orb-hp-label');
      if (hpLbl) hpLbl.textContent = Math.ceil(hp) + ' / ' + Math.ceil(maxHp);
      var emLbl = document.getElementById('orb-ember-label');
      if (emLbl) emLbl.textContent = overdriveSec > 0 ? overdriveSec.toFixed(1) + 's BURN' : Math.floor(ember) + '% EMBER';
      var emBtn = document.getElementById('btn-overdrive-orb');
      var ultReady = ember >= 99 || overdriveSec > 0;
      if (emBtn) {
        if (ultReady) emBtn.classList.add('ready');
        else emBtn.classList.remove('ready');
      }
      var ringUlt = document.getElementById('ring-ult');
      if (ringUlt) {
        ringUlt.setAttribute('stroke-dashoffset', String(Math.round(113 * (1 - hudState.emberPct))));
      }
      var btnSkillUlt = document.getElementById('btn-skill-ult');
      if (btnSkillUlt) {
        if (ultReady) btnSkillUlt.classList.add('overdrive-ready');
        else btnSkillUlt.classList.remove('overdrive-ready');
      }

      var cBadge = document.getElementById('carnage-badge');
      if (cBadge) {
        cBadge.textContent = 'x' + Math.max(1, combo);
        if (overdriveSec > 0 || combo >= 25) cBadge.classList.add('overdrive');
        else cBadge.classList.remove('overdrive');
      }

      var hasBoss = bossMaxHp > 0 && bossHp > 0;
      setVisible('boss-bar-wrap', hasBoss);
      if (hasBoss) {
        var bn = document.getElementById('boss-bar-name');
        if (bn) bn.textContent = BIOME_BOSSES[biome % 3] || BIOME_BOSSES[0];
        var bhp = document.getElementById('boss-bar-hp');
        var pct = Math.max(0, Math.min(1, bossHp / bossMaxHp));
        if (bhp) bhp.textContent = Math.ceil(pct * 100) + '%';
        var bFill = document.getElementById('boss-bar-fill');
        if (bFill) bFill.setAttribute('width', String(Math.round(394 * pct)));
        var bTrail = document.getElementById('boss-bar-trail');
        if (bTrail) bTrail.setAttribute('width', String(Math.round(394 * Math.max(pct, Math.min(1, bossTrail)))));
      }

      var gdEl = document.getElementById('hud-ghost-delta');
      if (gdEl) {
        if (initData.topGhost && initData.leaderboard && initData.leaderboard.length > 0) {
          gdEl.classList.remove('is-hidden');
          gdEl.textContent = (ghostDelta >= 0 ? '+' : '') + ghostDelta + ' PTS VS #1';
          if (ghostDelta < 0) gdEl.classList.add('behind');
          else gdEl.classList.remove('behind');
        } else {
          gdEl.classList.add('is-hidden');
        }
      }

      if (!onboardingHidden && onboardingStartTs > 0) {
        var ob = document.getElementById('onboarding-hint');
        if (ob) {
          if (!onboardingDoneByInput && Date.now() - onboardingStartTs < 8500) {
            ob.classList.remove('is-hidden');
          } else {
            ob.classList.add('faded');
            onboardingHidden = true;
          }
        }
      }
    }

    if (phase === 5) {
      var stEl = document.getElementById('chronicle-status');
      if (stEl) stEl.textContent = lastStatusMsg || '';
      refreshChronicleVotesAndFeed();
    }
  }

  // Bind DOM controls
  var classCards = document.querySelectorAll('#title-class-grid .class-card');
  classCards.forEach(function (card) {
    card.addEventListener('click', function () {
      classCards.forEach(function (c) { c.classList.remove('selected'); });
      card.classList.add('selected');
      selectedHeroClass = Number(card.getAttribute('data-class')) || 0;
      callCmd('ef_cmd_select_hero', selectedHeroClass);
    });
  });

  var modeDaily = document.getElementById('mode-daily');
  var modeEndless = document.getElementById('mode-endless');
  if (modeDaily) modeDaily.addEventListener('click', function () {
    selectedEndless = 0;
    modeDaily.classList.add('selected');
    if (modeEndless) modeEndless.classList.remove('selected');
  });
  if (modeEndless) modeEndless.addEventListener('click', function () {
    selectedEndless = 1;
    modeEndless.classList.add('selected');
    if (modeDaily) modeDaily.classList.remove('selected');
  });

  var btnEnterAbyss = document.getElementById('btn-enter-abyss');
  if (btnEnterAbyss) btnEnterAbyss.addEventListener('click', function () {
    unlockAudio();
    callCmd('ef_cmd_start_run', selectedHeroClass, selectedEndless);
  });

  var btnSkipIntro = document.getElementById('btn-skip-intro');
  if (btnSkipIntro) btnSkipIntro.addEventListener('click', function () {
    callCmd('skip_intro');
    callCmd('ef_cmd_skip_intro');
  });

  var btnSkillDash = document.getElementById('btn-skill-dash');
  if (btnSkillDash) btnSkillDash.addEventListener('click', function () {
    unlockAudio();
    callCmd('ef_cmd_dash');
  });

  var btnSkillRunic = document.getElementById('btn-skill-runic');
  if (btnSkillRunic) btnSkillRunic.addEventListener('click', function () {
    unlockAudio();
    callCmd('ef_cmd_runic_strike');
  });

  var btnSkillUlt = document.getElementById('btn-skill-ult');
  if (btnSkillUlt) btnSkillUlt.addEventListener('click', function () {
    unlockAudio();
    callCmd('trigger_overdrive');
    callCmd('ef_cmd_trigger_overdrive');
  });

  var btnOverdrive = document.getElementById('btn-overdrive-orb');
  if (btnOverdrive) btnOverdrive.addEventListener('click', function () {
    unlockAudio();
    callCmd('trigger_overdrive');
    callCmd('ef_cmd_trigger_overdrive');
  });

  var btnMute = document.getElementById('btn-mute');
  if (btnMute) btnMute.addEventListener('click', function () { callCmd('ef_cmd_toggle_mute'); });

  var btnPause = document.getElementById('btn-pause');
  if (btnPause) btnPause.addEventListener('click', function () { callCmd('ef_cmd_toggle_pause'); });

  var btnResume = document.getElementById('btn-resume');
  if (btnResume) btnResume.addEventListener('click', function () { callCmd('ef_cmd_toggle_pause'); });

  var btnPauseResume = document.getElementById('btn-pause-resume');
  if (btnPauseResume) btnPauseResume.addEventListener('click', function () { callCmd('ef_cmd_toggle_pause'); });

  var btnPauseMute = document.getElementById('btn-pause-mute');
  if (btnPauseMute) btnPauseMute.addEventListener('click', function () { callCmd('ef_cmd_toggle_mute'); });

  var btnPauseCrt = document.getElementById('btn-pause-crt');
  if (btnPauseCrt) btnPauseCrt.addEventListener('click', function () { callCmd('ef_cmd_toggle_crt'); });

  var btnShare = document.getElementById('btn-share-reddit');
  if (btnShare) btnShare.addEventListener('click', function () { callCmd('ef_cmd_share'); });

  var btnReplay = document.getElementById('btn-replay-daily');
  if (btnReplay) btnReplay.addEventListener('click', function () { callCmd('ef_cmd_replay'); });

  var btnEndless = document.getElementById('btn-play-endless');
  if (btnEndless) btnEndless.addEventListener('click', function () { callCmd('ef_cmd_endless'); });

  var voteBtns = document.querySelectorAll('#chronicle-votes .vote-card');
  voteBtns.forEach(function (vb) {
    vb.addEventListener('click', function () {
      voteBtns.forEach(function (v) { v.classList.remove('voted'); });
      vb.classList.add('voted');
      callCmd('ef_cmd_vote', Number(vb.getAttribute('data-mut')) || 0);
    });
  });

  load('./emberfall.wasm?v=20261126v20');
})();

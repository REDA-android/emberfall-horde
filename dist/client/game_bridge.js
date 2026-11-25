(function () {
  var st = document.getElementById('ef-status');
  window.addEventListener('error', function (ev) {
    if (st) { st.style.color = '#fca5a5'; st.textContent = 'Error: ' + (ev.message || 'Unknown script error'); }
  });
  window.addEventListener('unhandledrejection', function (ev) {
    if (st) { st.style.color = '#fca5a5'; st.textContent = 'WASM Error: ' + (ev.reason && ev.reason.message ? ev.reason.message : String(ev.reason)); }
  });

  var MODIFIERS = ['BLOOD MOON', 'ARCANE SURGE', 'SWARM RUSH', "TITAN'S WRATH"];
  var audioCtx = null;
  var noiseBuffer = null;
  var isMuted = false;
  var droneNodes = null;
  var droneWanted = false;
  var bossMusicTimer = null;
  var bossStep = 0;
  var BOSS_NOTES = [110, 130.81, 146.83, 164.81, 155.56, 146.83, 130.81, 98.0];

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
    }
    if (audioCtx.state === 'suspended') {
      try { audioCtx.resume(); } catch (_) {}
    }
    return audioCtx;
  }

  function ensureNoiseBuffer(ctx) {
    if (noiseBuffer) return noiseBuffer;
    try {
      var len = Math.floor(ctx.sampleRate * 0.45);
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
      if (droneWanted && !droneNodes) startDrone();
    } catch (_) {}
  }

  ['pointerdown', 'touchstart', 'keydown'].forEach(function (evt) {
    window.addEventListener(evt, unlockAudio, { passive: true });
  });

  function tone(f1, f2, dur, type, vol, pitchMul, delaySec) {
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
      osc.connect(g);
      g.connect(ctx.destination);
      osc.start(now);
      osc.stop(now + dur + 0.02);
    } catch (_) {}
  }

  function noiseBurst(dur, filterFreq, filterType, vol, delaySec) {
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
      g.gain.linearRampToValueAtTime(vol || 0.06, now + 0.01);
      g.gain.exponentialRampToValueAtTime(0.0001, now + dur);
      src.connect(flt);
      flt.connect(g);
      g.connect(ctx.destination);
      src.start(now);
      src.stop(now + dur + 0.02);
    } catch (_) {}
  }

  function setBossMusic(active) {
    if (bossMusicTimer) {
      clearInterval(bossMusicTimer);
      bossMusicTimer = null;
    }
    if (active) {
      bossStep = 0;
      bossMusicTimer = setInterval(function () {
        if (isMuted || !droneWanted) return;
        var n = BOSS_NOTES[bossStep % BOSS_NOTES.length];
        bossStep++;
        tone(n, n * 0.985, 0.16, 'sawtooth', 0.042, 1.0);
        if (bossStep % 4 === 1) {
          tone(55, 42, 0.15, 'triangle', 0.062, 1.0);
        }
      }, 214);
    }
  }

  function startDrone() {
    try {
      var ctx = ensureAudioCtx();
      if (!ctx || droneNodes) return;
      var filter = ctx.createBiquadFilter();
      filter.type = 'lowpass';
      filter.frequency.value = 180;
      var gain = ctx.createGain();
      gain.gain.setValueAtTime(0.0001, ctx.currentTime);
      var targetGain = isMuted ? 0.0001 : 0.030;
      gain.gain.setTargetAtTime(targetGain, ctx.currentTime, 0.35);

      var osc1 = ctx.createOscillator();
      osc1.type = 'sawtooth';
      osc1.frequency.value = 55;
      var osc2 = ctx.createOscillator();
      osc2.type = 'triangle';
      osc2.frequency.value = 82.41;

      osc1.connect(filter);
      osc2.connect(filter);
      filter.connect(gain);
      gain.connect(ctx.destination);
      osc1.start();
      osc2.start();
      droneNodes = { osc1: osc1, osc2: osc2, gain: gain };
    } catch (_) {}
  }

  function stopDrone() {
    if (!droneNodes || !audioCtx) return;
    try {
      var nodes = droneNodes;
      droneNodes = null;
      nodes.gain.gain.setTargetAtTime(0.0001, audioCtx.currentTime, 0.15);
      setTimeout(function () {
        try { nodes.osc1.stop(); nodes.osc2.stop(); } catch (_) {}
      }, 400);
    } catch (_) {
      droneNodes = null;
    }
  }

  function playSfxPitched(kind, pitchMul) {
    if (isMuted) return;
    if (kind === 0) {
      // Crisp UI select chime
      tone(392, 587, 0.09, 'triangle', 0.085, pitchMul, 0);
      tone(587, 784, 0.09, 'sine', 0.06, pitchMul, 0.04);
    } else if (kind === 1) {
      // Weapon shot / slash
      tone(540, 190, 0.085, 'sawtooth', 0.048, pitchMul, 0);
      noiseBurst(0.045, 1800, 'highpass', 0.025, 0);
    } else if (kind === 2) {
      // Impact thud
      tone(220, 72, 0.075, 'square', 0.048, pitchMul, 0);
      noiseBurst(0.04, 650, 'bandpass', 0.03, 0);
    } else if (kind === 3) {
      // Chain lightning / Void storm crackle
      tone(1260, 380, 0.14, 'sawtooth', 0.065, pitchMul, 0);
      noiseBurst(0.11, 2800, 'highpass', 0.055, 0);
    } else if (kind === 4) {
      // Barrel / Supernova explosion
      tone(180, 44, 0.24, 'sawtooth', 0.105, pitchMul, 0);
      noiseBurst(0.22, 520, 'lowpass', 0.085, 0);
    } else if (kind === 5) {
      // Level-Up / Bounty / Victory fanfare
      tone(523.25, 523.25, 0.14, 'triangle', 0.095, pitchMul, 0);
      tone(659.25, 659.25, 0.16, 'triangle', 0.095, pitchMul, 0.065);
      tone(783.99, 1046.5, 0.26, 'sine', 0.105, pitchMul, 0.13);
    } else if (kind === 6) {
      // Boss alert war horn
      tone(110, 73.4, 0.48, 'sawtooth', 0.12, pitchMul, 0);
      tone(55, 46, 0.52, 'triangle', 0.13, pitchMul, 0.02);
      noiseBurst(0.32, 340, 'lowpass', 0.05, 0.04);
    } else if (kind === 7) {
      // Hero hurt
      tone(175, 58, 0.18, 'sawtooth', 0.11, pitchMul, 0);
      noiseBurst(0.09, 420, 'bandpass', 0.06, 0);
    } else if (kind === 8) {
      // XP gem sparkle
      tone(880, 1320, 0.065, 'sine', 0.055, pitchMul, 0);
    } else if (kind === 9) {
      // Runic chest / Blood shrine harmonic chime
      tone(392, 587, 0.17, 'triangle', 0.09, pitchMul, 0);
      tone(587, 880, 0.22, 'sine', 0.085, pitchMul, 0.075);
    } else if (kind === 10) {
      // Inferno Dash whoosh
      noiseBurst(0.15, 1600, 'bandpass', 0.075, 0);
      tone(340, 120, 0.14, 'triangle', 0.07, pitchMul, 0);
    } else if (kind === 11) {
      // Witch-Time Perfect Dodge (time-warp dive + crystal shimmer)
      tone(660, 135, 0.34, 'sine', 0.11, 1.0, 0);
      tone(1320, 1760, 0.24, 'triangle', 0.08, 1.0, 0.05);
      tone(987.77, 1480, 0.22, 'sine', 0.06, 1.0, 0.11);
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

      env.emberfall_set_muted = function (muted) {
        isMuted = muted !== 0;
        saveLocalProfile();
        if (droneNodes && audioCtx) {
          try {
            droneNodes.gain.gain.setTargetAtTime(isMuted ? 0.0001 : 0.030, audioCtx.currentTime, 0.08);
          } catch (_) {}
        }
      };

      env.emberfall_toggle_drone = function (active) {
        droneWanted = active !== 0;
        if (droneWanted) startDrone();
        else {
          setBossMusic(false);
          stopDrone();
        }
      };

      env.emberfall_set_boss_music = function (active) {
        setBossMusic(active !== 0);
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
            myRank: initData.myRank
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
    },
    version: 1,
    name: 'emberfall_bridge'
  });

  load('./emberfall.wasm?v=20261125v7');
})();

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
  var isMuted = false;
  var droneNodes = null;
  var droneWanted = false;
  var bossMusicTimer = null;
  var bossStep = 0;
  var BOSS_NOTES = [110, 130.81, 146.83, 164.81, 155.56, 146.83, 130.81, 98.0];

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
        tone(n, n * 0.98, 0.16, 'sawtooth', 0.045, 1.0);
        if (bossStep % 4 === 1) {
          tone(55, 42, 0.14, 'triangle', 0.06, 1.0);
        }
      }, 214);
    }
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

  function tone(f1, f2, dur, type, vol, pitchMul) {
    if (isMuted) return;
    try {
      var ctx = ensureAudioCtx();
      if (!ctx) return;
      var pm = (typeof pitchMul === 'number' && isFinite(pitchMul)) ? Math.max(0.4, Math.min(2.5, pitchMul)) : 1.0;
      var now = ctx.currentTime;
      var osc = ctx.createOscillator();
      var g = ctx.createGain();
      osc.type = type || 'triangle';
      osc.frequency.setValueAtTime(Math.max(25, f1 * pm), now);
      osc.frequency.exponentialRampToValueAtTime(Math.max(25, f2 * pm), now + dur);
      g.gain.setValueAtTime(0.0001, now);
      g.gain.linearRampToValueAtTime(vol || 0.1, now + 0.01);
      g.gain.exponentialRampToValueAtTime(0.0001, now + dur);
      osc.connect(g);
      g.connect(ctx.destination);
      osc.start(now);
      osc.stop(now + dur + 0.02);
    } catch (_) {}
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
      var targetGain = isMuted ? 0.0001 : 0.032;
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
    if (kind === 0) tone(330, 660, 0.16, 'triangle', 0.11, pitchMul);
    else if (kind === 1) tone(520, 180, 0.09, 'sawtooth', 0.05, pitchMul);
    else if (kind === 2) tone(240, 80, 0.08, 'square', 0.05, pitchMul);
    else if (kind === 3) tone(1180, 420, 0.14, 'sawtooth', 0.07, pitchMul);
    else if (kind === 4) tone(220, 60, 0.22, 'sawtooth', 0.11, pitchMul);
    else if (kind === 5) {
      tone(523, 784, 0.2, 'triangle', 0.12, pitchMul);
      setTimeout(function () { tone(784, 1046, 0.25, 'triangle', 0.11, pitchMul); }, 90);
    } else if (kind === 6) tone(110, 55, 0.45, 'sawtooth', 0.14, pitchMul);
    else if (kind === 7) tone(160, 65, 0.18, 'sawtooth', 0.12, pitchMul);
    else if (kind === 8) tone(880, 1320, 0.07, 'sine', 0.06, pitchMul);
    else if (kind === 9) {
      tone(392, 587, 0.18, 'triangle', 0.10, pitchMul);
      setTimeout(function () { tone(587, 880, 0.22, 'sine', 0.09, pitchMul); }, 80);
    }
  }

  var initReady = 0;
  var submitState = 0;
  var shareState = 0;
  var currentRunId = '';
  var lastStatusMsg = '';
  var initData = {
    seed: '',
    mutatorOverride: -1,
    username: '',
    streak: 0,
    personalBest: 0,
    myRank: 0,
    totalPlayers: 0,
    bossKillsToday: 0,
    siegeTarget: 25,
    factionScores: [0, 0, 0],
    yesterdayChampion: null,
    leaderboard: [],
    topGhost: null,
    canVoteMutator: false,
    medalsMask: 0,
    bestEndlessWave: 0,
    selectedAura: 0,
    yesterdayWinningFaction: -1
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
      }
    } catch (_) {}
  }

  function saveLocalProfile() {
    try {
      localStorage.setItem('ef_profile_v1', JSON.stringify({
        medalsMask: initData.medalsMask & 63,
        bestEndlessWave: initData.bestEndlessWave,
        selectedAura: initData.selectedAura
      }));
    } catch (_) {}
  }

  loadLocalProfile();

  function applyInitPayload(d) {
    if (!d || typeof d !== 'object') return;
    initData.seed = typeof d.seed === 'string' ? d.seed : '';
    initData.mutatorOverride = typeof d.mutatorOverride === 'number' ? d.mutatorOverride : -1;
    initData.username = typeof d.username === 'string' ? d.username : '';
    initData.streak = Number(d.streak) || 0;
    initData.personalBest = Number(d.personalBest) || 0;
    initData.myRank = Number(d.myRank) || 0;
    initData.totalPlayers = Number(d.totalPlayers) || 0;
    initData.bossKillsToday = Number(d.bossKillsToday) || 0;
    initData.siegeTarget = Number(d.siegeTarget) || 25;
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
    initData.canVoteMutator = Boolean(d.canVoteMutator);
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
        if (droneNodes && audioCtx) {
          try {
            droneNodes.gain.gain.setTargetAtTime(isMuted ? 0.0001 : 0.032, audioCtx.currentTime, 0.08);
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
        saveLocalProfile();

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
              initData.streak = Number(res.json.streak) || initData.streak;
              initData.bossKillsToday = Number(res.json.bossKillsToday) || initData.bossKillsToday;
              if (Array.isArray(res.json.factionScores) && res.json.factionScores.length >= 3) {
                initData.factionScores = [
                  Number(res.json.factionScores[0]) || 0,
                  Number(res.json.factionScores[1]) || 0,
                  Number(res.json.factionScores[2]) || 0
                ];
              }
              initData.canVoteMutator = Boolean(res.json.canVoteMutator);
              initData.medalsMask = (initData.medalsMask | (Number(res.json.medalsMask) || 0)) & 63;
              initData.bestEndlessWave = Math.max(initData.bestEndlessWave, Number(res.json.bestEndlessWave) || 0);
              saveLocalProfile();
              if (score > initData.personalBest) initData.personalBest = score;
              lastStatusMsg = res.json.isNewBest ? 'New Daily Best!' : 'Score Submitted!';
              fetchInit();
            } else {
              submitState = 3;
              lastStatusMsg = (res.json && res.json.error) || ('Submit failed (' + res.status + ')');
            }
          })
          .catch(function () {
            submitState = 3;
            lastStatusMsg = 'Network error submitting run';
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
              lastStatusMsg = 'Shared to comments!';
            } else {
              shareState = 3;
              lastStatusMsg = (res.json && res.json.error) || ('Share failed (' + res.status + ')');
            }
          })
          .catch(function () {
            shareState = 3;
            lastStatusMsg = 'Network error sharing comment';
          });
      };

      env.emberfall_vote_mutator = function (mutatorIdx) {
        lastStatusMsg = 'Submitting vote...';
        fetch('/api/vote-mutator', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ mutatorIndex: mutatorIdx | 0 })
        })
          .then(function (r) {
            return r.json().catch(function () { return {}; }).then(function (json) {
              return { ok: r.ok, status: r.status, json: json };
            });
          })
          .then(function (res) {
            if (res.ok && res.json && res.json.ok) {
              lastStatusMsg = 'Voted: ' + (MODIFIERS[mutatorIdx] || 'Mutator');
            } else {
              lastStatusMsg = (res.json && res.json.error) || ('Vote failed (' + res.status + ')');
            }
          })
          .catch(function () {
            lastStatusMsg = 'Network error voting';
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
          default: return 0;
        }
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

  load('./emberfall.wasm?v=20261121d');
})();

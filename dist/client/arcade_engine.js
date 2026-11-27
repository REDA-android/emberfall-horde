(function () {
  "use strict";

  var W = 960;
  var H = 540;
  var WORLD_W = 2640;
  var LANE_MIN_Y = 382;
  var LANE_MAX_Y = 450;
  var WATER_Y = 468;
  var GOAL_KOS = 25;

  var canvas = document.getElementById("arcade-canvas");
  var ctx = canvas ? canvas.getContext("2d") : null;
  var thermoCanvas = document.getElementById("thermo-canvas");
  var thermoCtx = thermoCanvas ? thermoCanvas.getContext("2d") : null;

  // =========================================================================
  // 16-BIT CHIPTUNE & ARCADE SFX ENGINE (WebAudio API, Zero External Files)
  // =========================================================================
  var audioCtx = null;
  var masterGain = null;
  var noiseBuf = null;
  var isMuted = false;
  var crtEnabled = true;
  var seqTimer = null;
  var seqStep = 0;

  var STREET_BASS = [110, 110, 130.81, 146.83, 164.81, 146.83, 130.81, 98, 110, 110, 164.81, 146.83, 130.81, 123.47, 98, 87.31];
  var STREET_LEAD = [440, 0, 523.25, 587.33, 659.25, 0, 587.33, 523.25, 440, 0, 392, 440, 523.25, 0, 493.88, 0];
  var BOSS_BASS = [82.41, 82.41, 98, 82.41, 110, 98, 87.31, 82.41, 82.41, 82.41, 123.47, 110, 98, 87.31, 73.42, 82.41];
  var BOSS_LEAD = [329.63, 349.23, 329.63, 0, 440, 392, 349.23, 329.63, 493.88, 0, 440, 392, 349.23, 329.63, 293.66, 0];

  function ensureAudio() {
    if (!audioCtx) {
      var Ctx = window.AudioContext || window.webkitAudioContext;
      if (!Ctx) return null;
      audioCtx = new Ctx();
      masterGain = audioCtx.createGain();
      masterGain.gain.value = 0.72;
      masterGain.connect(audioCtx.destination);
    }
    if (audioCtx.state === "suspended") {
      try { audioCtx.resume(); } catch (_) {}
    }
    return audioCtx;
  }

  function getNoise(ac) {
    if (noiseBuf) return noiseBuf;
    var len = Math.floor(ac.sampleRate * 0.35);
    var b = ac.createBuffer(1, len, ac.sampleRate);
    var d = b.getChannelData(0);
    for (var i = 0; i < len; i++) d[i] = Math.random() * 2 - 1;
    noiseBuf = b;
    return b;
  }

  function tone(f1, f2, dur, type, vol, delay) {
    if (isMuted) return;
    try {
      var ac = ensureAudio();
      if (!ac) return;
      var t0 = ac.currentTime + (delay || 0);
      var osc = ac.createOscillator();
      var g = ac.createGain();
      osc.type = type || "square";
      osc.frequency.setValueAtTime(Math.max(30, f1), t0);
      osc.frequency.exponentialRampToValueAtTime(Math.max(30, f2), t0 + dur);
      g.gain.setValueAtTime(0.0001, t0);
      g.gain.linearRampToValueAtTime(vol || 0.08, t0 + 0.008);
      g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
      osc.connect(g);
      g.connect(masterGain);
      osc.start(t0);
      osc.stop(t0 + dur + 0.02);
    } catch (_) {}
  }

  function noise(dur, freq, type, vol, delay) {
    if (isMuted) return;
    try {
      var ac = ensureAudio();
      if (!ac) return;
      var t0 = ac.currentTime + (delay || 0);
      var src = ac.createBufferSource();
      src.buffer = getNoise(ac);
      var flt = ac.createBiquadFilter();
      flt.type = type || "bandpass";
      flt.frequency.setValueAtTime(freq || 900, t0);
      var g = ac.createGain();
      g.gain.setValueAtTime(0.0001, t0);
      g.gain.linearRampToValueAtTime(vol || 0.07, t0 + 0.005);
      g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
      src.connect(flt);
      flt.connect(g);
      g.connect(masterGain);
      src.start(t0);
      src.stop(t0 + dur + 0.02);
    } catch (_) {}
  }

  function sfx(name) {
    if (isMuted) return;
    if (name === "punch") {
      tone(210, 68, 0.08, "square", 0.11, 0);
      noise(0.06, 820, "bandpass", 0.09, 0);
    } else if (name === "heavy") {
      tone(150, 42, 0.16, "sawtooth", 0.15, 0);
      noise(0.14, 520, "lowpass", 0.14, 0);
    } else if (name === "kick") {
      tone(340, 120, 0.11, "triangle", 0.10, 0);
      noise(0.07, 1500, "highpass", 0.06, 0);
    } else if (name === "jump") {
      tone(220, 520, 0.12, "square", 0.07, 0);
    } else if (name === "parry") {
      tone(880, 1480, 0.11, "triangle", 0.12, 0);
      tone(1320, 1760, 0.12, "sine", 0.09, 0.05);
    } else if (name === "reload") {
      // Iconic mechanical Arm Reload clicks + triumphant fanfare
      noise(0.05, 2200, "highpass", 0.12, 0);
      tone(190, 260, 0.07, "sawtooth", 0.11, 0.02);
      noise(0.06, 2600, "highpass", 0.13, 0.11);
      tone(280, 390, 0.08, "sawtooth", 0.12, 0.12);
      tone(523.25, 523.25, 0.16, "square", 0.11, 0.22);
      tone(659.25, 659.25, 0.16, "square", 0.11, 0.30);
      tone(783.99, 1046.5, 0.34, "sawtooth", 0.13, 0.38);
    } else if (name === "splash") {
      noise(0.26, 650, "bandpass", 0.13, 0);
      tone(310, 95, 0.18, "sine", 0.09, 0.02);
    } else if (name === "glass") {
      noise(0.16, 4200, "highpass", 0.12, 0);
      tone(1450, 780, 0.11, "triangle", 0.07, 0);
    } else if (name === "pickup") {
      tone(523.25, 659.25, 0.09, "square", 0.08, 0);
      tone(659.25, 1046.5, 0.14, "square", 0.09, 0.07);
    } else if (name === "hurt") {
      tone(180, 58, 0.19, "sawtooth", 0.13, 0);
      noise(0.11, 450, "lowpass", 0.09, 0);
    } else if (name === "victory") {
      tone(523.25, 523.25, 0.14, "square", 0.10, 0);
      tone(659.25, 659.25, 0.14, "square", 0.10, 0.12);
      tone(783.99, 783.99, 0.16, "square", 0.11, 0.24);
      tone(1046.5, 1046.5, 0.45, "sawtooth", 0.12, 0.36);
    } else if (name === "go") {
      tone(880, 880, 0.07, "square", 0.07, 0);
      tone(1174.6, 1174.6, 0.11, "square", 0.08, 0.09);
    }
  }

  function startMusic() {
    if (seqTimer) return;
    seqTimer = setInterval(function () {
      if (isMuted || state.phase !== "playing") return;
      var intense = state.bossActive || state.hero.armReloadTimer > 0;
      var bass = intense ? BOSS_BASS : STREET_BASS;
      var lead = intense ? BOSS_LEAD : STREET_LEAD;
      var i = seqStep % 16;
      seqStep++;
      var bn = bass[i];
      if (bn > 0) tone(bn, bn * 0.99, 0.13, "triangle", intense ? 0.065 : 0.05, 0);
      var ln = lead[i];
      if (ln > 0 && (intense || seqStep % 32 >= 16)) {
        tone(ln, ln, 0.12, "square", intense ? 0.036 : 0.025, 0);
      }
      if (i % 4 === 0) tone(122, 42, 0.09, "sine", 0.08, 0);
      if (i === 4 || i === 12) noise(0.06, 1600, "bandpass", 0.04, 0);
    }, 145);
  }

  // =========================================================================
  // SPRITE LOADER (16-bit Pixel Art Transparent WebPs)
  // =========================================================================
  var sprites = {};
  var SPRITE_URLS = {
    heroIdle: "./arcade/hero_idle.webp",
    heroPunch: "./arcade/hero_punch.webp",
    heroKick: "./arcade/hero_kick.webp",
    enemySuit: "./arcade/enemy_suit.webp",
    bossKingpin: "./arcade/boss_kingpin.webp"
  };
  Object.keys(SPRITE_URLS).forEach(function (k) {
    var img = new Image();
    img.src = SPRITE_URLS[k];
    sprites[k] = img;
  });

  // =========================================================================
  // REDDIT DEVVIT BACKEND & LOCAL PROFILE
  // =========================================================================
  var apiState = {
    seed: new Date().toISOString().slice(0, 10),
    username: "",
    streak: 1,
    bestScore: 0,
    runId: "",
    submitted: false
  };

  function loadLocal() {
    try {
      var raw = localStorage.getItem("henry_arcade_v3");
      if (!raw) return;
      var d = JSON.parse(raw);
      if (d) {
        apiState.bestScore = Number(d.bestScore) || 0;
        apiState.streak = Math.max(1, Number(d.streak) || 1);
        if (typeof d.muted === "boolean") isMuted = d.muted;
        if (typeof d.crt === "boolean") crtEnabled = d.crt;
      }
    } catch (_) {}
  }

  function saveLocal() {
    try {
      localStorage.setItem("henry_arcade_v3", JSON.stringify({
        bestScore: apiState.bestScore,
        streak: apiState.streak,
        muted: isMuted,
        crt: crtEnabled
      }));
    } catch (_) {}
  }

  function fetchInit() {
    fetch("/api/init", { cache: "no-store" })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (d) {
        if (!d) return;
        if (d.seed) apiState.seed = d.seed;
        if (d.username) apiState.username = d.username;
        if (typeof d.streak === "number") apiState.streak = Math.max(1, d.streak);
        if (typeof d.personalBest === "number") apiState.bestScore = Math.max(apiState.bestScore, d.personalBest);
        var tag = document.getElementById("title-daily-tag");
        if (tag) tag.textContent = "16-BIT HARBOR BEAT'EM UP · DAILY #" + apiState.seed;
      })
      .catch(function () {});
  }

  loadLocal();
  fetchInit();

  // =========================================================================
  // GAME STATE & WORLD DEFINITIONS (Exact Reference Street Architecture)
  // =========================================================================
  var BUILDINGS = [
    { x: 0, w: 195, h: 215, col: "#92b59c", roof: "#4a5a6a", style: "barber", label: "barbershop", windows: 1 },
    { x: 195, w: 220, h: 315, col: "#e7b944", roof: "#4c5c6e", style: "townhouse", doorCol: "#a06d2c", windows: 2 },
    { x: 415, w: 225, h: 315, col: "#255aa8", roof: "#3a4d66", style: "townhouse", doorCol: "#1a3c7c", quoins: true, windows: 2 },
    { x: 640, w: 205, h: 268, col: "#e04248", roof: "#475769", style: "townhouse", doorCol: "#991b22", windows: 2 },
    { x: 845, w: 230, h: 218, col: "#dce4ee", roof: "#556475", style: "fancy", label: "FANCY", windows: 0 },
    { x: 1075, w: 215, h: 315, col: "#2f74c0", roof: "#3c4e64", style: "townhouse", doorCol: "#6d747d", quoins: true, windows: 2 },
    { x: 1290, w: 265, h: 318, col: "#ea5b5c", roof: "#455568", style: "townhouse", doorCol: "#9e2a2b", windows: 3 },
    { x: 1555, w: 200, h: 240, col: "#f0d24b", roof: "#4b5b6c", style: "townhouse", doorCol: "#7c5218", windows: 2 },
    { x: 1755, w: 215, h: 285, col: "#1d5aa6", roof: "#36465c", style: "townhouse", doorCol: "#133166", quoins: true, windows: 2 },
    { x: 1970, w: 230, h: 220, col: "#8fb79f", roof: "#485869", style: "barber", label: "barbershop", windows: 1 },
    { x: 2200, w: 220, h: 310, col: "#d9464a", roof: "#425062", style: "townhouse", doorCol: "#7f1d1d", windows: 2 },
    { x: 2420, w: 240, h: 260, col: "#2c3448", roof: "#1e2436", style: "fancy", label: "SYNDICATE HQ", windows: 0 }
  ];

  var LAMPS_X = [358, 780, 1215, 1680, 2110];

  var SKIN_PERKS = [
    { name: "GOLDEN TITAN", speedMul: 1.0, dmgMul: 1.0, aura: "#ffe24a" },
    { name: "ROYAL RANGER", speedMul: 1.15, dmgMul: 1.0, aura: "#38e8ff" },
    { name: "CRIMSON BRAWLER", speedMul: 1.0, dmgMul: 1.2, aura: "#ff5f2e" }
  ];

  var state = {
    phase: "title", // title | playing | arm_reload | paused | ended
    endless: false,
    skinIdx: 0,
    time: 0,
    camX: 0,
    shake: 0,
    hitstop: 0,
    kos: 0,
    score: 0,
    combo: 0,
    maxCombo: 0,
    comboTimer: 0,
    thermo: 15, // 0..100
    armReloadShownOnce: false,
    bossActive: false,
    bossSlain: false,
    limoX: -400,
    limoPhase: 0, // 0 none, 1 arriving, 2 leaving
    goTimer: 0,
    zoneLockX: 1050,
    spawnTimer: 0.9,
    victory: false,
    hero: {
      x: 260,
      y: 416,
      z: 0,
      vz: 0,
      vx: 0,
      vy: 0,
      facing: 1,
      hp: 6, // 3 hearts = 6 half-hearts
      maxHp: 6,
      invuln: 0,
      attackType: "none", // none | punch | kick
      attackTimer: 0,
      attackHitDone: false,
      comboStep: 0,
      comboWindow: 0,
      oCd: 0,
      kCd: 0,
      armReloadTimer: 0,
      weapon: null, // null | { kind: "pipe", uses: 3 }
      walkPhase: 0
    },
    props: [],
    enemies: [],
    projectiles: [],
    pickups: [],
    particles: [],
    popups: [],
    shatteredWindows: {}
  };

  var keys = {};
  var touchInput = { left: false, right: false };

  function resetWorld(endlessMode) {
    state.endless = Boolean(endlessMode);
    state.time = 0;
    state.camX = 0;
    state.shake = 0;
    state.hitstop = 0;
    state.kos = 0;
    state.score = 0;
    state.combo = 0;
    state.maxCombo = 0;
    state.comboTimer = 0;
    state.thermo = 18;
    state.armReloadShownOnce = false;
    state.bossActive = false;
    state.bossSlain = false;
    state.limoX = -400;
    state.limoPhase = 0;
    state.goTimer = 0;
    state.spawnTimer = 0.7;
    state.victory = false;
    state.shatteredWindows = {};
    apiState.submitted = false;

    state.hero = {
      x: 340,
      y: 418,
      z: 0,
      vz: 0,
      vx: 0,
      vy: 0,
      facing: 1,
      hp: 6,
      maxHp: 6,
      invuln: 0,
      attackType: "none",
      attackTimer: 0,
      attackHitDone: false,
      comboStep: 0,
      comboWindow: 0,
      oCd: 0,
      kCd: 0,
      armReloadTimer: 0,
      weapon: null,
      walkPhase: 0
    };

    // Destructible & Jumpable Iconic Street Props (Phone Booths, White Hatchback Car, Crates)
    state.props = [
      { id: "booth1", kind: "booth", x: 585, y: 382, w: 54, h: 102, hp: 3, maxHp: 3, hitFlash: 0, dropped: false },
      { id: "car1", kind: "car", x: 930, y: 404, w: 148, h: 56, hp: 6, maxHp: 6, hitFlash: 0, dropped: false },
      { id: "crate1", kind: "crate", x: 1380, y: 394, w: 46, h: 42, hp: 2, maxHp: 2, hitFlash: 0, dropped: false },
      { id: "booth2", kind: "booth", x: 1715, y: 382, w: 54, h: 102, hp: 3, maxHp: 3, hitFlash: 0, dropped: false },
      { id: "car2", kind: "car", x: 2050, y: 404, w: 148, h: 56, hp: 6, maxHp: 6, hitFlash: 0, dropped: false }
    ];

    state.enemies = [];
    state.projectiles = [];
    state.pickups = [];
    state.particles = [];
    state.popups = [];

    // Initial street squad right on screen (instant action!)
    spawnEnemy("brawler", 620, 422);
    spawnEnemy("thrower", 815, 410);
  }

  function spawnEnemy(kind, x, y) {
    var isBoss = kind === "boss";
    var maxHp = isBoss ? 22 : (kind === "heavy" ? 6 : (kind === "thrower" ? 3 : 4));
    state.enemies.push({
      kind: kind,
      x: x,
      y: y,
      z: 0,
      vz: 0,
      vx: 0,
      vy: 0,
      facing: -1,
      hp: maxHp,
      maxHp: maxHp,
      hitFlash: 0,
      stunTimer: 0,
      attackCd: isBoss ? 1.4 : (1.1 + Math.random() * 1.1),
      walkPhase: Math.random() * 6,
      dying: false,
      splashKO: false,
      dieTimer: 0,
      glassesLost: false,
      trailHp: maxHp
    });
  }

  function addPopup(x, y, text, color, size) {
    state.popups.push({
      x: x,
      y: y,
      text: text,
      color: color || "#ffe680",
      size: size || 20,
      life: 0.9,
      maxLife: 0.9
    });
  }

  function spawnParticles(x, y, count, color, speed, gravity) {
    for (var i = 0; i < count; i++) {
      var ang = Math.random() * Math.PI * 2;
      var spd = (0.35 + Math.random() * 0.85) * speed;
      state.particles.push({
        x: x,
        y: y,
        vx: Math.cos(ang) * spd,
        vy: Math.sin(ang) * spd - 70,
        size: 3 + Math.random() * 5,
        color: color,
        life: 0.35 + Math.random() * 0.35,
        grav: gravity !== undefined ? gravity : 520
      });
    }
  }

  // =========================================================================
  // COMBAT ACTIONS: [O] PUNCH, [K] JUMP-KICK / THROW, [L] ARM RELOAD, SPACE JUMP
  // =========================================================================
  function heroJump() {
    if (state.phase !== "playing") return;
    var h = state.hero;
    if (h.z <= 1 || isStandingOnProp(h)) {
      h.vz = 470;
      sfx("jump");
      spawnParticles(h.x, h.y, 6, "#e2e8f0", 90, 220);
    }
  }

  function heroPunch() {
    if (state.phase !== "playing") return;
    var h = state.hero;
    if (h.oCd > 0) return;
    h.oCd = h.armReloadTimer > 0 ? 0.16 : 0.21;
    h.attackType = "punch";
    h.attackTimer = 0.20;
    h.attackHitDone = false;
    h.comboStep = h.comboWindow > 0 ? (h.comboStep + 1) % 3 : 0;
    h.comboWindow = 0.65;
    h.vx += h.facing * (h.armReloadTimer > 0 ? 150 : 65);
    sfx(h.comboStep === 2 || h.armReloadTimer > 0 ? "heavy" : "punch");
    performMeleeHit(false);
  }

  function heroKick() {
    if (state.phase !== "playing") return;
    var h = state.hero;
    if (h.kCd > 0) return;
    // If holding a picked-up street pipe, throw it!
    if (h.weapon) {
      h.kCd = 0.35;
      state.projectiles.push({
        kind: "pipe",
        owner: "hero",
        x: h.x + h.facing * 28,
        y: h.y - h.z - 42,
        vx: h.facing * 540,
        vy: 0,
        rot: 0,
        life: 1.5
      });
      h.weapon = null;
      sfx("kick");
      addPopup(h.x, h.y - h.z - 88, "PIPE THROW!", "#38e8ff", 18);
      return;
    }

    h.kCd = 0.55;
    h.attackType = "kick";
    h.attackTimer = 0.32;
    h.attackHitDone = false;
    if (h.z < 18) h.vz = 290;
    h.vx = h.facing * 360 * SKIN_PERKS[state.skinIdx].speedMul;
    sfx("kick");
    performMeleeHit(true);
  }

  function triggerArmReload(forceShowCutin) {
    if (state.phase !== "playing") return;
    var h = state.hero;
    if (!forceShowCutin && state.thermo < 100 && h.armReloadTimer <= 0) return;
    state.thermo = 0;
    h.armReloadTimer = 8.0;
    h.invuln = 8.0;
    state.shake = 12;
    sfx("reload");
    spawnParticles(h.x, h.y - h.z - 45, 28, "#ffe24a", 320, 180);
    spawnParticles(h.x, h.y - h.z - 45, 18, "#ff5f2e", 260, 140);

    if (!state.armReloadShownOnce || forceShowCutin) {
      state.armReloadShownOnce = true;
      state.phase = "arm_reload";
      setVisible("arm-reload-screen", true);
    } else {
      addPopup(h.x, h.y - h.z - 96, "ARM RELOAD!", "#ffe24a", 26);
    }
  }

  function performMeleeHit(isKick) {
    var h = state.hero;
    var superMode = h.armReloadTimer > 0;
    var reach = isKick ? 112 : (superMode ? 124 : (h.weapon ? 116 : 94));
    var hitX = h.x + h.facing * (reach * 0.55);
    var dmg = (isKick ? 2 : (h.comboStep === 2 ? 2.5 : 1.5)) * SKIN_PERKS[state.skinIdx].dmgMul;
    if (h.weapon) dmg += 1.5;
    if (superMode) dmg = 6;

    // 1. Deflect Flying Mustache Boomerangs!
    for (var pi = 0; pi < state.projectiles.length; pi++) {
      var p = state.projectiles[pi];
      if (p.owner === "enemy" && Math.abs(p.x - hitX) < 76 && Math.abs(p.y - (h.y - h.z - 46)) < 68) {
        p.owner = "hero";
        p.vx = h.facing * 520;
        p.vy = -20;
        p.life = 2.0;
        state.thermo = Math.min(100, state.thermo + 22);
        state.score += 250;
        state.hitstop = 0.05;
        state.shake = 8;
        sfx("parry");
        addPopup(p.x, p.y - 24, "PARRY! +250", "#38e8ff", 22);
        spawnParticles(p.x, p.y, 14, "#ffe24a", 240, 180);
      }
    }

    // 2. Hit Street Props (Phone Booth, Parked White Car, Crates)
    for (var pri = 0; pri < state.props.length; pri++) {
      var pr = state.props[pri];
      if (pr.hp <= 0) continue;
      var pcx = pr.x + pr.w * 0.5;
      if (Math.abs(pcx - hitX) < pr.w * 0.5 + 52 && Math.abs(pr.y - h.y) < 55) {
        pr.hp = Math.max(0, pr.hp - (superMode ? 2 : 1));
        pr.hitFlash = 0.16;
        state.shake = Math.max(state.shake, 6);
        state.thermo = Math.min(100, state.thermo + 14);
        state.score += 150;
        sfx(pr.kind === "car" ? "glass" : "heavy");
        spawnParticles(pcx, pr.y - 32, 12, pr.kind === "booth" ? "#facc15" : "#e2e8f0", 220, 480);
        addPopup(pcx, pr.y - 68, pr.kind === "car" ? "CRASH! +150" : "SMASH!", "#ffe680", 20);
        if (pr.hp <= 0 && !pr.dropped) {
          pr.dropped = true;
          if (pr.kind === "booth") {
            state.pickups.push({ kind: "pizza", x: pcx, y: pr.y + 12, z: 32, vz: 180 });
          } else if (pr.kind === "car") {
            state.pickups.push({ kind: "gold_mustache", x: pcx - 20, y: pr.y + 16, z: 36, vz: 210 });
            state.pickups.push({ kind: "gold_mustache", x: pcx + 24, y: pr.y + 18, z: 42, vz: 230 });
            addPopup(pcx, pr.y - 96, "BONUS CAR SMASHED! +500", "#ffe24a", 21);
            state.score += 500;
          } else {
            state.pickups.push({ kind: "pipe", x: pcx, y: pr.y + 10, z: 24, vz: 160 });
          }
        }
      }
    }

    // 3. Hit Enemies & Boss
    var landed = false;
    for (var ei = 0; ei < state.enemies.length; ei++) {
      var e = state.enemies[ei];
      if (e.dying) continue;
      var dx = e.x - h.x;
      var inFront = dx * h.facing > -26 && Math.abs(dx) <= reach;
      var inLane = Math.abs(e.y - h.y) <= 44;
      if (inFront && inLane) {
        landed = true;
        damageEnemy(e, dmg, h.facing, isKick || h.comboStep === 2 || superMode, superMode);
      }
    }

    if (landed && h.weapon) {
      h.weapon.uses--;
      if (h.weapon.uses <= 0) {
        h.weapon = null;
        addPopup(h.x, h.y - h.z - 86, "PIPE BROKE!", "#fca5a5", 16);
      }
    }
  }

  function damageEnemy(e, dmg, dir, heavy, superMode) {
    e.hp -= dmg;
    e.hitFlash = 0.16;
    e.stunTimer = e.kind === "boss" ? 0.22 : (heavy ? 0.55 : 0.34);
    e.vx = dir * (heavy ? (superMode ? 480 : 320) : 140);
    if (heavy && e.kind !== "boss") e.vz = 220;

    state.combo++;
    state.maxCombo = Math.max(state.maxCombo, state.combo);
    state.comboTimer = 2.6;
    state.hitstop = heavy ? 0.055 : 0.028;
    state.shake = Math.max(state.shake, heavy ? 9 : 4);
    if (state.hero.armReloadTimer <= 0) {
      var mult = state.hero.hp <= 1 ? 1.5 : 1.0;
      state.thermo = Math.min(100, state.thermo + (heavy ? 12 : 7) * mult);
    }

    // Knock purple glasses off goon on first big hit!
    if (!e.glassesLost && e.kind !== "boss") {
      e.glassesLost = true;
      spawnParticles(e.x, e.y - e.z - 64, 5, "#c084fc", 180, 420);
    }

    var words = superMode ? ["SUPER PUNCH!", "ARM RELOAD!", "MEGA POW!"] : (heavy ? ["POW!", "BAM!", "WHAM!"] : ["SMACK!", "HIT!"]);
    var word = words[Math.floor(Math.random() * words.length)];
    addPopup(e.x, e.y - e.z - 76, word, superMode ? "#ffe24a" : (heavy ? "#ff7a18" : "#ffffff"), heavy ? 22 : 17);
    spawnParticles(e.x, e.y - e.z - 46, heavy ? 14 : 7, superMode ? "#ffe24a" : "#ff5f2e", 240, 340);

    // Check if goon hits FANCY storefront glass!
    if (e.x > 855 && e.x < 1070 && heavy && !state.shatteredWindows.fancy1) {
      state.shatteredWindows.fancy1 = true;
      sfx("glass");
      spawnParticles(960, 340, 22, "#bae6fd", 280, 500);
      addPopup(960, 300, "CRASH! +300", "#bae6fd", 22);
      state.score += 300;
    }

    if (e.hp <= 0) {
      koEnemy(e, dir, heavy || superMode);
    }
  }

  function koEnemy(e, dir, dramatic) {
    if (e.dying) return;
    e.dying = true;
    e.dieTimer = 0.85;
    e.vx = dir * (dramatic ? 380 : 210);
    e.vz = dramatic ? 320 : 180;

    // 45% chance on heavy KOs near waterfront to splash into harbor!
    if (dramatic && e.y >= 412 && e.kind !== "boss") {
      e.splashKO = true;
      e.vy = 135;
    }

    var comboMult = state.thermo >= 80 ? 3 : (state.thermo >= 40 ? 2 : 1);
    var pts = (e.kind === "boss" ? 5000 : (e.kind === "heavy" ? 300 : 150)) * comboMult;
    state.score += pts;
    addPopup(e.x, e.y - 96, "+" + pts, "#ffe680", 19);

    if (e.kind === "boss") {
      state.bossSlain = true;
      state.bossActive = false;
      sfx("victory");
      spawnParticles(e.x, e.y - 60, 42, "#ffe24a", 360, 280);
      if (!state.endless) {
        setTimeout(function () {
          if (state.phase === "playing") finishRun(true);
        }, 1150);
      } else {
        addPopup(state.hero.x, 220, "BOSS CRUSHED! ENDLESS CONTINUES!", "#ffe24a", 24);
      }
      return;
    }

    state.kos++;

    // Drop golden mustache or pizza occasionally
    if (state.kos % 5 === 0) {
      state.pickups.push({ kind: "gold_mustache", x: e.x, y: e.y, z: 22, vz: 190 });
    } else if (state.hero.hp <= 3 && Math.random() < 0.35) {
      state.pickups.push({ kind: "pizza", x: e.x, y: e.y, z: 22, vz: 180 });
    }

    // Trigger iconic first-time ARM RELOAD Power-Up at 8 KOs (just like the reference video!)
    if (state.kos === 8 && !state.armReloadShownOnce) {
      setTimeout(function () {
        if (state.phase === "playing") triggerArmReload(true);
      }, 320);
    }

    // Encourage player to advance along the street with flashing GO ->
    if (state.kos === 6 || state.kos === 14 || state.kos === 20) {
      state.goTimer = 2.8;
      sfx("go");
    }

    // Trigger Harbor Syndicate Kingpin Boss at 25 KOs!
    if (!state.bossActive && !state.bossSlain && state.kos >= GOAL_KOS) {
      summonKingpinBoss();
    }
  }

  function summonKingpinBoss() {
    state.bossActive = true;
    state.limoPhase = 1;
    state.limoX = Math.min(WORLD_W - 220, state.camX + W + 180);
    sfx("go");
    addPopup(state.camX + W * 0.5, 220, "WARNING: DON MUSTACHIO ARRIVES!", "#ff5f2e", 26);
    var bx = Math.min(WORLD_W - 120, state.camX + W - 110);
    spawnEnemy("boss", bx, 420);
  }

  function isStandingOnProp(h) {
    for (var i = 0; i < state.props.length; i++) {
      var pr = state.props[i];
      if (pr.hp <= 0) continue;
      if (h.x > pr.x + 4 && h.x < pr.x + pr.w - 4 && Math.abs(h.y - pr.y) < 34) {
        var topZ = pr.kind === "booth" ? 86 : (pr.kind === "car" ? 44 : 36);
        if (Math.abs(h.z - topZ) < 10) return topZ;
      }
    }
    return 0;
  }

  // =========================================================================
  // UPDATE LOOP (60 FPS FIXED-TIMESTEP FRIENDLY)
  // =========================================================================
  function update(dt) {
    state.time += dt;

    if (state.phase === "title") {
      // Attract mode: gentle harbor breeze & hero breathing on the quay!
      state.camX = 120 + Math.sin(state.time * 0.35) * 110;
      state.hero.x = state.camX + 340;
      state.hero.y = 420;
      state.hero.walkPhase += dt * 4;
      updateParticles(dt);
      return;
    }

    if (state.phase !== "playing") return;

    if (state.hitstop > 0) {
      state.hitstop = Math.max(0, state.hitstop - dt);
      state.shake = Math.max(0, state.shake - dt * 18);
      return;
    }

    state.shake = Math.max(0, state.shake - dt * 24);
    if (state.goTimer > 0) state.goTimer = Math.max(0, state.goTimer - dt);
    if (state.comboTimer > 0) {
      state.comboTimer -= dt;
      if (state.comboTimer <= 0) state.combo = 0;
    }

    var h = state.hero;
    h.oCd = Math.max(0, h.oCd - dt);
    h.kCd = Math.max(0, h.kCd - dt);
    h.comboWindow = Math.max(0, h.comboWindow - dt);
    h.invuln = Math.max(0, h.invuln - dt);
    if (h.armReloadTimer > 0) {
      h.armReloadTimer = Math.max(0, h.armReloadTimer - dt);
      if (Math.random() < 0.45) {
        spawnParticles(h.x + (Math.random() - 0.5) * 34, h.y - h.z - 45, 1, "#ffe24a", 75, -80);
      }
    }
    if (h.attackTimer > 0) {
      h.attackTimer = Math.max(0, h.attackTimer - dt);
      if (h.attackTimer <= 0) h.attackType = "none";
    }

    // Movement input
    var mx = 0;
    var my = 0;
    if (keys.ArrowLeft || keys.KeyA || touchInput.left) mx -= 1;
    if (keys.ArrowRight || keys.KeyD || touchInput.right) mx += 1;
    if (keys.ArrowUp || keys.KeyW) my -= 1;
    if (keys.ArrowDown || keys.KeyS) my += 1;

    var spd = 265 * SKIN_PERKS[state.skinIdx].speedMul * (h.armReloadTimer > 0 ? 1.2 : 1.0);
    if (mx !== 0) {
      h.facing = mx > 0 ? 1 : -1;
      h.walkPhase += dt * 11;
    } else if (my !== 0) {
      h.walkPhase += dt * 9;
    }

    h.x += (mx * spd + h.vx) * dt;
    h.y += (my * 165 + h.vy) * dt;
    h.vx *= Math.pow(0.001, dt);
    h.vy *= Math.pow(0.001, dt);
    h.x = Math.max(44, Math.min(WORLD_W - 54, h.x));
    h.y = Math.max(LANE_MIN_Y, Math.min(LANE_MAX_Y, h.y));

    // Jump & Prop Roof Landing Physics
    var prevZ = h.z;
    h.z += h.vz * dt;
    h.vz -= 1280 * dt;

    var floorZ = 0;
    for (var pi = 0; pi < state.props.length; pi++) {
      var pr = state.props[pi];
      if (pr.hp <= 0) continue;
      if (h.x > pr.x + 6 && h.x < pr.x + pr.w - 6 && Math.abs(h.y - pr.y) < 34) {
        var topZ = pr.kind === "booth" ? 86 : (pr.kind === "car" ? 44 : 36);
        if (prevZ >= topZ - 10 && h.vz <= 0) {
          floorZ = Math.max(floorZ, topZ);
        }
      }
    }
    if (h.z <= floorZ) {
      h.z = floorZ;
      h.vz = 0;
    }

    // Smooth camera follow
    var targetCam = Math.max(0, Math.min(WORLD_W - W, h.x - W * 0.42));
    state.camX += (targetCam - state.camX) * Math.min(1, dt * 6.5);

    // Syndicate Limousine animation
    if (state.limoPhase === 1) {
      var stopX = Math.min(WORLD_W - 260, state.camX + W - 260);
      state.limoX -= 320 * dt;
      if (state.limoX <= stopX) {
        state.limoX = stopX;
        state.limoPhase = 2;
      }
    } else if (state.limoPhase === 2) {
      state.limoX += 380 * dt;
      if (state.limoX > state.camX + W + 340) state.limoPhase = 0;
    }

    // Spawn Enemies (up to cap, continuous in Endless or until Boss)
    var aliveGoons = state.enemies.filter(function (e) { return !e.dying && e.kind !== "boss"; }).length;
    var needMore = state.endless || (!state.bossActive && !state.bossSlain && state.kos + aliveGoons < GOAL_KOS);
    if (needMore && aliveGoons < 4) {
      state.spawnTimer -= dt;
      if (state.spawnTimer <= 0) {
        state.spawnTimer = state.endless ? 1.35 : 1.75;
        var fromRight = Math.random() < 0.72;
        var sx = fromRight ? Math.min(WORLD_W - 40, state.camX + W + 45) : Math.max(40, state.camX - 45);
        var sy = LANE_MIN_Y + Math.random() * (LANE_MAX_Y - LANE_MIN_Y);
        var roll = Math.random();
        var ek = roll < 0.44 ? "brawler" : (roll < 0.78 ? "thrower" : "heavy");
        spawnEnemy(ek, sx, sy);
      }
    }

    // Update Props
    for (var pIdx = 0; pIdx < state.props.length; pIdx++) {
      state.props[pIdx].hitFlash = Math.max(0, state.props[pIdx].hitFlash - dt);
    }

    // Update Enemies
    for (var ei = state.enemies.length - 1; ei >= 0; ei--) {
      var e = state.enemies[ei];
      e.hitFlash = Math.max(0, e.hitFlash - dt);
      if (e.trailHp > e.hp) e.trailHp = Math.max(e.hp, e.trailHp - dt * 4.5);

      if (e.dying) {
        e.dieTimer -= dt;
        e.x += e.vx * dt;
        e.y += e.vy * dt;
        e.z += e.vz * dt;
        e.vz -= 1050 * dt;
        if (e.splashKO && e.y >= WATER_Y + 8) {
          sfx("splash");
          spawnParticles(e.x, WATER_Y + 6, 20, "#ffffff", 260, 480);
          spawnParticles(e.x, WATER_Y + 6, 14, "#38bdf8", 210, 460);
          addPopup(e.x, WATER_Y - 22, "PLOUF!", "#38e8ff", 24);
          state.enemies.splice(ei, 1);
          continue;
        }
        if (e.z < 0) { e.z = 0; e.vz = 0; e.vx *= 0.6; }
        if (e.dieTimer <= 0) state.enemies.splice(ei, 1);
        continue;
      }

      if (e.stunTimer > 0) {
        e.stunTimer -= dt;
        e.x += e.vx * dt;
        e.vx *= Math.pow(0.01, dt);
        e.z = Math.max(0, e.z + e.vz * dt);
        if (e.z > 0) e.vz -= 1150 * dt;
        continue;
      }

      var dx = h.x - e.x;
      var dy = h.y - e.y;
      e.facing = dx >= 0 ? 1 : -1;
      var dist = Math.hypot(dx, dy);

      var moveSpd = e.kind === "boss" ? 128 : (e.kind === "heavy" ? 148 : (e.kind === "thrower" ? 96 : 118));
      var stopDist = e.kind === "thrower" ? 235 : (e.kind === "boss" ? 86 : 64);

      if (Math.abs(dx) > stopDist) {
        e.x += (dx > 0 ? 1 : -1) * moveSpd * dt;
        e.walkPhase += dt * 8;
      }
      if (Math.abs(dy) > 10) {
        e.y += (dy > 0 ? 1 : -1) * 78 * dt;
      }
      e.x = Math.max(36, Math.min(WORLD_W - 36, e.x));
      e.y = Math.max(LANE_MIN_Y, Math.min(LANE_MAX_Y, e.y));

      e.attackCd -= dt;
      if (e.attackCd <= 0) {
        if (e.kind === "thrower" && dist < 460) {
          e.attackCd = 2.15;
          state.projectiles.push({
            kind: "mustache",
            owner: "enemy",
            x: e.x + e.facing * 26,
            y: e.y - 48,
            vx: e.facing * 290,
            vy: 0,
            rot: 0,
            life: 2.6
          });
          sfx("kick");
        } else if (e.kind === "boss") {
          e.attackCd = e.hp < e.maxHp * 0.5 ? 1.35 : 1.85;
          if (dist < 130) {
            if (h.z < 36) hurtHero(1, e.facing);
            state.shake = 10;
            sfx("heavy");
            addPopup(e.x, e.y - 120, "KINGPIN SLAM!", "#ff5f2e", 21);
          } else {
            // Triple flying mustache volley!
            for (var v = -1; v <= 1; v++) {
              state.projectiles.push({
                kind: "mustache",
                owner: "enemy",
                x: e.x + e.facing * 32,
                y: e.y - 54 + v * 16,
                vx: e.facing * 320,
                vy: v * 28,
                rot: 0,
                life: 2.6
              });
            }
            sfx("kick");
          }
        } else if (dist < 76 && h.z < 34) {
          e.attackCd = 1.3;
          hurtHero(1, e.facing);
        }
      }
    }

    // Update Projectiles (Flying Mustache Boomerangs + Thrown Pipes)
    for (var qi = state.projectiles.length - 1; qi >= 0; qi--) {
      var q = state.projectiles[qi];
      q.life -= dt;
      q.x += q.vx * dt;
      q.y += q.vy * dt;
      q.rot += dt * 14;
      if (q.life <= 0 || q.x < state.camX - 120 || q.x > state.camX + W + 120) {
        state.projectiles.splice(qi, 1);
        continue;
      }
      if (q.owner === "enemy") {
        if (Math.abs(q.x - h.x) < 34 && Math.abs(q.y - (h.y - h.z - 46)) < 42) {
          hurtHero(1, q.vx >= 0 ? 1 : -1);
          state.projectiles.splice(qi, 1);
        }
      } else {
        for (var tei = 0; tei < state.enemies.length; tei++) {
          var te = state.enemies[tei];
          if (te.dying) continue;
          if (Math.abs(q.x - te.x) < 44 && Math.abs(q.y - (te.y - te.z - 48)) < 58) {
            damageEnemy(te, q.kind === "pipe" ? 4.5 : 4, q.vx >= 0 ? 1 : -1, true, true);
            if (q.kind !== "pipe") {
              state.projectiles.splice(qi, 1);
              break;
            }
          }
        }
      }
    }

    // Update Pickups (Pizza Heal, Steel Pipe, Golden Mustache)
    for (var ki = state.pickups.length - 1; ki >= 0; ki--) {
      var pk = state.pickups[ki];
      pk.z = Math.max(0, pk.z + pk.vz * dt);
      if (pk.z > 0) pk.vz -= 900 * dt;
      if (Math.abs(pk.x - h.x) < 42 && Math.abs(pk.y - h.y) < 40 && h.z < 36) {
        sfx("pickup");
        if (pk.kind === "pizza") {
          h.hp = Math.min(h.maxHp, h.hp + 2);
          addPopup(pk.x, pk.y - 44, "PIZZA! +1 HEART", "#4ade80", 19);
        } else if (pk.kind === "pipe") {
          h.weapon = { kind: "pipe", uses: 3 };
          addPopup(pk.x, pk.y - 44, "STEEL PIPE!", "#38e8ff", 19);
        } else {
          state.score += 250;
          state.thermo = Math.min(100, state.thermo + 25);
          addPopup(pk.x, pk.y - 44, "GOLD MUSTACHE! +250", "#ffe24a", 19);
        }
        state.pickups.splice(ki, 1);
      }
    }

    updateParticles(dt);
  }

  function hurtHero(amount, knockDir) {
    var h = state.hero;
    if (h.invuln > 0 || h.armReloadTimer > 0) return;
    h.hp = Math.max(0, h.hp - amount);
    h.invuln = 1.05;
    h.vx = knockDir * 240;
    state.combo = 0;
    state.shake = 10;
    sfx("hurt");
    spawnParticles(h.x, h.y - h.z - 46, 12, "#ef4444", 200, 320);
    if (h.hp <= 0) {
      finishRun(false);
    }
  }

  function updateParticles(dt) {
    for (var i = state.particles.length - 1; i >= 0; i--) {
      var p = state.particles[i];
      p.life -= dt;
      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.vy += p.grav * dt;
      if (p.life <= 0) state.particles.splice(i, 1);
    }
    for (var j = state.popups.length - 1; j >= 0; j--) {
      var pop = state.popups[j];
      pop.life -= dt;
      pop.y -= 42 * dt;
      if (pop.life <= 0) state.popups.splice(j, 1);
    }
  }

  // =========================================================================
  // 16-BIT HARBOR STREET RENDERER (EXACT 1:1 VISUAL DNA OF REFERENCE VIDEO)
  // =========================================================================
  function render() {
    if (!ctx) return;
    ctx.imageSmoothingEnabled = false;
    ctx.save();

    if (state.shake > 0.2) {
      ctx.translate((Math.random() - 0.5) * state.shake, (Math.random() - 0.5) * state.shake);
    }

    // 1. Dynamic Sky (Day -> Sunset at 12+ KOs -> Neon Night at Boss!)
    var isNight = state.bossActive;
    var isSunset = !isNight && state.kos >= 12;
    var skyGrad = ctx.createLinearGradient(0, 0, 0, 290);
    if (isNight) {
      skyGrad.addColorStop(0, "#0b1433");
      skyGrad.addColorStop(1, "#261f54");
    } else if (isSunset) {
      skyGrad.addColorStop(0, "#23408e");
      skyGrad.addColorStop(0.55, "#d95b43");
      skyGrad.addColorStop(1, "#f6b94c");
    } else {
      skyGrad.addColorStop(0, "#104db5");
      skyGrad.addColorStop(0.65, "#2b7de0");
      skyGrad.addColorStop(1, "#7cc2f7");
    }
    ctx.fillStyle = skyGrad;
    ctx.fillRect(0, 0, W, 320);

    // 2. Fluffy 16-bit Pixel Clouds (Far Parallax 0.15x)
    drawClouds(state.camX * 0.15, isNight);

    // 3. Distant Medieval Harbor Castle Fortress (Far Parallax 0.25x, right horizon like reference video)
    drawDistantCastle(state.camX * 0.25, isNight);

    // 4. Midground Colorful Harbor Rowhouses (1.0x world scroll)
    ctx.save();
    ctx.translate(-Math.round(state.camX), 0);

    for (var bi = 0; bi < BUILDINGS.length; bi++) {
      var b = BUILDINGS[bi];
      if (b.x + b.w < state.camX - 40 || b.x > state.camX + W + 40) continue;
      drawBuilding(b, isNight || isSunset);
    }

    // Victorian Black Wrought-Iron Streetlamps
    for (var li = 0; li < LAMPS_X.length; li++) {
      var lx = LAMPS_X[li];
      if (lx < state.camX - 60 || lx > state.camX + W + 60) continue;
      drawStreetLamp(lx, 376, isNight || isSunset);
    }

    // 5. Paved Stone Harbor Quay Sidewalk (y = 374..468)
    drawQuaySidewalk(state.camX);

    // Syndicate Boss Limousine if active
    if (state.limoPhase > 0) {
      drawLimo(state.limoX, 396);
    }

    // 6. Depth-Sorted Entities (Props, Pickups, Hero, Enemies)
    var drawList = [];
    for (var pri = 0; pri < state.props.length; pri++) {
      drawList.push({ y: state.props[pri].y, fn: drawPropFn(state.props[pri]) });
    }
    for (var pki = 0; pki < state.pickups.length; pki++) {
      drawList.push({ y: state.pickups[pki].y, fn: drawPickupFn(state.pickups[pki]) });
    }
    drawList.push({ y: state.hero.y + 1, fn: drawHero });
    for (var ei = 0; ei < state.enemies.length; ei++) {
      drawList.push({ y: state.enemies[ei].y, fn: drawEnemyFn(state.enemies[ei]) });
    }
    drawList.sort(function (a, b) { return a.y - b.y; });
    for (var di = 0; di < drawList.length; di++) drawList[di].fn();

    // 7. Flying Projectiles (Mustache Boomerangs & Thrown Pipes)
    for (var qi = 0; qi < state.projectiles.length; qi++) {
      drawProjectile(state.projectiles[qi]);
    }

    // 8. Iron Mooring Bollards along front quay edge
    drawMooringBollards(state.camX);

    // 9. Particles & Comic Onomatopoeia Popups
    for (var pai = 0; pai < state.particles.length; pai++) {
      var pa = state.particles[pai];
      ctx.fillStyle = pa.color;
      ctx.fillRect(Math.round(pa.x), Math.round(pa.y), Math.round(pa.size), Math.round(pa.size));
    }
    for (var poi = 0; poi < state.popups.length; poi++) {
      var pop = state.popups[poi];
      ctx.save();
      ctx.globalAlpha = Math.min(1, pop.life / 0.35);
      ctx.font = "900 " + pop.size + "px 'Trebuchet MS', Impact, sans-serif";
      ctx.textAlign = "center";
      ctx.lineWidth = 5;
      ctx.strokeStyle = "#0a0c12";
      ctx.strokeText(pop.text, Math.round(pop.x), Math.round(pop.y));
      ctx.fillStyle = pop.color;
      ctx.fillText(pop.text, Math.round(pop.x), Math.round(pop.y));
      ctx.restore();
    }

    ctx.restore(); // end world camera transform

    // 10. Animated Royal Blue Rolling Harbor Water Waves (Bottom Foreground y = 468..540)
    drawHarborWater();

    // 11. Giant Arcade Combo Banner
    if (state.combo >= 4 && state.phase === "playing") {
      ctx.save();
      ctx.font = "900 28px Impact, 'Trebuchet MS', sans-serif";
      ctx.textAlign = "right";
      var label = state.combo + " HITS!" + (state.combo >= 15 ? " UNSTOPPABLE!" : "");
      ctx.lineWidth = 6;
      ctx.strokeStyle = "#090d16";
      ctx.strokeText(label, W - 26, 132);
      ctx.fillStyle = state.combo >= 15 ? "#ffe24a" : "#ffffff";
      ctx.fillText(label, W - 26, 132);
      ctx.restore();
    }

    // 12. Low-HP Urgent Red Edge Flash (1 half-heart left)
    if (state.phase === "playing" && state.hero.hp === 1) {
      var pulse = 0.16 + Math.sin(state.time * 10) * 0.1;
      ctx.strokeStyle = "rgba(225, 29, 72, " + pulse.toFixed(2) + ")";
      ctx.lineWidth = 14;
      ctx.strokeRect(7, 7, W - 14, H - 14);
    }

    ctx.restore();
  }

  function drawClouds(offsetX, isNight) {
    ctx.fillStyle = isNight ? "rgba(148, 163, 184, 0.22)" : "#f8fbff";
    var spots = [110, 380, 620, 880, 1180];
    for (var i = 0; i < spots.length; i++) {
      var cx = ((spots[i] - offsetX - state.time * 6) % 1250 + 1250) % 1250 - 120;
      var cy = 48 + (i % 3) * 28;
      ctx.fillRect(cx, cy, 86, 26);
      ctx.fillRect(cx + 16, cy - 14, 52, 18);
      ctx.fillRect(cx - 14, cy + 8, 114, 18);
      ctx.fillStyle = isNight ? "rgba(100,116,139,0.2)" : "#dbeafe";
      ctx.fillRect(cx - 10, cy + 22, 104, 6);
      ctx.fillStyle = isNight ? "rgba(148, 163, 184, 0.22)" : "#f8fbff";
    }
  }

  function drawDistantCastle(offsetX, isNight) {
    var baseX = Math.round(700 - offsetX);
    ctx.fillStyle = isNight ? "#1b233c" : "#8b7a6b";
    ctx.fillRect(baseX, 82, 230, 150);
    ctx.fillRect(baseX + 36, 46, 54, 60);
    ctx.fillRect(baseX + 140, 58, 64, 50);
    // Battlements
    for (var m = 0; m < 10; m++) {
      ctx.fillRect(baseX + m * 23, 72, 13, 12);
    }
    ctx.fillStyle = isNight ? "#facc15" : "#4a3f38";
    ctx.fillRect(baseX + 54, 64, 8, 14);
    ctx.fillRect(baseX + 162, 74, 8, 14);
    ctx.fillRect(baseX + 106, 108, 10, 16);
  }

  function drawBuilding(b, litWindows) {
    var topY = 376 - b.h;
    // Slate pitched roof
    var roofH = 74;
    ctx.fillStyle = b.roof;
    ctx.fillRect(b.x, topY, b.w, roofH);
    ctx.strokeStyle = "#18202c";
    ctx.lineWidth = 3;
    ctx.strokeRect(b.x, topY, b.w, roofH);
    // Roof shingle horizontal pixel lines
    ctx.fillStyle = "rgba(255,255,255,0.12)";
    for (var ry = topY + 12; ry < topY + roofH; ry += 12) {
      ctx.fillRect(b.x + 3, ry, b.w - 6, 2);
    }
    // Brick Chimneys
    ctx.fillStyle = "#9c4a35";
    ctx.fillRect(b.x + 18, topY - 22, 26, 28);
    ctx.strokeRect(b.x + 18, topY - 22, 26, 28);
    ctx.fillStyle = "#c88a55";
    ctx.fillRect(b.x + 22, topY - 30, 7, 10);
    ctx.fillRect(b.x + 33, topY - 30, 7, 10);

    // Main Facade Wall
    var wallY = topY + roofH;
    var wallH = 376 - wallY;
    ctx.fillStyle = b.col;
    ctx.fillRect(b.x, wallY, b.w, wallH);
    ctx.strokeStyle = "#141821";
    ctx.lineWidth = 3;
    ctx.strokeRect(b.x, wallY, b.w, wallH);

    // Stone corner quoins (just like the blue townhouse in frame_03s!)
    if (b.quoins) {
      ctx.fillStyle = "#c4bfb5";
      for (var qy = wallY + 4; qy < 368; qy += 18) {
        var qw = (qy % 36 === 0) ? 14 : 9;
        ctx.fillRect(b.x + 2, qy, qw, 12);
        ctx.fillRect(b.x + b.w - qw - 2, qy, qw, 12);
      }
    }

    // Foundation stone course
    ctx.fillStyle = "#9aa0a8";
    ctx.fillRect(b.x + 2, 356, b.w - 4, 20);

    if (b.style === "barber") {
      // Iconic Barbershop storefront
      ctx.fillStyle = "#2b2024";
      ctx.fillRect(b.x + 28, 286, 126, 90);
      // Striped red/white awning
      for (var s = 0; s < 10; s++) {
        ctx.fillStyle = s % 2 === 0 ? "#dc2626" : "#f8fafc";
        ctx.fillRect(b.x + 28 + s * 12.6, 286, 13, 18);
      }
      // Sign
      ctx.fillStyle = "#1e293b";
      ctx.fillRect(b.x + 28, 266, 126, 22);
      ctx.strokeStyle = "#f8fafc";
      ctx.lineWidth = 2;
      ctx.strokeRect(b.x + 28, 266, 126, 22);
      ctx.fillStyle = "#ffffff";
      ctx.font = "bold 13px 'Trebuchet MS', sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(b.label, b.x + 91, 281);
      // Spinning Barber Pole!
      var px = b.x + 160;
      ctx.fillStyle = "#ffffff";
      ctx.fillRect(px, 314, 12, 46);
      ctx.fillStyle = "#dc2626";
      var shift = Math.floor(state.time * 24) % 12;
      for (var by = 314 + shift; by < 358; by += 12) {
        ctx.fillRect(px, by, 12, 5);
      }
      ctx.strokeStyle = "#141821";
      ctx.strokeRect(px, 314, 12, 46);
    } else if (b.style === "fancy") {
      // Iconic FANCY storefront with shatterable showroom window
      ctx.fillStyle = "#1f2937";
      ctx.fillRect(b.x + 12, 236, b.w - 24, 34);
      ctx.strokeStyle = "#0f172a";
      ctx.strokeRect(b.x + 12, 236, b.w - 24, 34);
      ctx.fillStyle = "#f8fafc";
      ctx.font = "900 22px Impact, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(b.label, b.x + b.w * 0.5, 261);

      var shattered = state.shatteredWindows.fancy1 && b.label === "FANCY";
      ctx.fillStyle = shattered ? "#1e293b" : "#93c5fd";
      ctx.fillRect(b.x + 18, 278, b.w - 36, 94);
      if (!shattered) {
        ctx.fillStyle = "rgba(255,255,255,0.35)";
        ctx.beginPath();
        ctx.moveTo(b.x + 28, 372);
        ctx.lineTo(b.x + 88, 278);
        ctx.lineTo(b.x + 114, 278);
        ctx.lineTo(b.x + 54, 372);
        ctx.closePath();
        ctx.fill();
      } else {
        ctx.strokeStyle = "#e0f2fe";
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(b.x + 18, 278); ctx.lineTo(b.x + 110, 326); ctx.lineTo(b.x + b.w - 18, 290);
        ctx.moveTo(b.x + 110, 326); ctx.lineTo(b.x + 80, 372);
        ctx.stroke();
      }
      ctx.strokeStyle = "#141821";
      ctx.lineWidth = 3;
      ctx.strokeRect(b.x + 18, 278, b.w - 36, 94);
    } else {
      // Classic 6-pane mullioned windows + paneled door
      var cols = b.windows || 2;
      var spacing = b.w / (cols + 1);
      for (var row = 0; row < 2; row++) {
        var wy = wallY + 26 + row * 82;
        if (wy > 310) continue;
        for (var col = 0; col < cols; col++) {
          var wx = b.x + spacing * (col + 1) - 20;
          // Skip bottom center window where door goes
          if (row === 1 && cols === 2 && col === 0) {
            // Draw paneled front door
            ctx.fillStyle = b.doorCol || "#6d28d9";
            ctx.fillRect(wx + 2, 300, 38, 76);
            ctx.strokeStyle = "#141821";
            ctx.lineWidth = 2.5;
            ctx.strokeRect(wx + 2, 300, 38, 76);
            ctx.fillStyle = "#facc15";
            ctx.fillRect(wx + 31, 338, 5, 5);
            continue;
          }
          ctx.fillStyle = litWindows ? "#fde047" : "#334155";
          ctx.fillRect(wx, wy, 40, 52);
          ctx.strokeStyle = "#f8fafc";
          ctx.lineWidth = 3;
          ctx.strokeRect(wx, wy, 40, 52);
          ctx.lineWidth = 2;
          ctx.beginPath();
          ctx.moveTo(wx + 20, wy); ctx.lineTo(wx + 20, wy + 52);
          ctx.moveTo(wx, wy + 18); ctx.lineTo(wx + 40, wy + 18);
          ctx.moveTo(wx, wy + 35); ctx.lineTo(wx + 40, wy + 35);
          ctx.stroke();
          // Window sill
          ctx.fillStyle = "#cbd5e1";
          ctx.fillRect(wx - 4, wy + 52, 48, 6);
        }
      }
    }
  }

  function drawStreetLamp(x, groundY, glowing) {
    ctx.fillStyle = "#181c24";
    ctx.fillRect(x - 5, groundY - 12, 10, 12);
    ctx.fillRect(x - 2, groundY - 98, 4, 88);
    ctx.fillRect(x - 9, groundY - 114, 18, 16);
    ctx.fillStyle = glowing ? "#fef08a" : "#cbd5e1";
    ctx.fillRect(x - 6, groundY - 111, 12, 10);
    if (glowing) {
      var g = ctx.createRadialGradient(x, groundY - 106, 4, x, groundY - 106, 62);
      g.addColorStop(0, "rgba(254, 240, 138, 0.5)");
      g.addColorStop(1, "rgba(254, 240, 138, 0)");
      ctx.fillStyle = g;
      ctx.beginPath();
      ctx.arc(x, groundY - 106, 62, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  function drawQuaySidewalk(camX) {
    var startX = Math.floor(camX / 48) * 48 - 48;
    var endX = camX + W + 96;
    // Paved sidewalk stone plane
    ctx.fillStyle = "#adb6c2";
    ctx.fillRect(startX, 376, endX - startX, 74);
    ctx.fillStyle = "#949eaa";
    ctx.fillRect(startX, 376, endX - startX, 8);

    // Paver grid lines
    ctx.strokeStyle = "#6c7684";
    ctx.lineWidth = 2;
    for (var py = 392; py < 450; py += 18) {
      ctx.beginPath();
      ctx.moveTo(startX, py);
      ctx.lineTo(endX, py);
      ctx.stroke();
    }
    for (var px = startX; px < endX; px += 48) {
      ctx.beginPath();
      ctx.moveTo(px, 376);
      ctx.lineTo(px - 14, 450);
      ctx.stroke();
    }

    // Thick Stone Quay Wall above harbor water
    ctx.fillStyle = "#7c8594";
    ctx.fillRect(startX, 450, endX - startX, 20);
    ctx.strokeStyle = "#2b323d";
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.moveTo(startX, 450); ctx.lineTo(endX, 450);
    ctx.stroke();
    for (var bx = startX; bx < endX; bx += 42) {
      ctx.strokeRect(bx, 450, 42, 20);
    }
  }

  function drawMooringBollards(camX) {
    var start = Math.floor(camX / 240) * 240;
    for (var x = start; x < camX + W + 240; x += 240) {
      var bx = x + 110;
      ctx.fillStyle = "#5c3428";
      ctx.fillRect(bx - 8, 424, 16, 26);
      ctx.fillStyle = "#7c4636";
      ctx.fillRect(bx - 10, 420, 20, 6);
      ctx.strokeStyle = "#181012";
      ctx.lineWidth = 2;
      ctx.strokeRect(bx - 8, 424, 16, 26);
    }
  }

  function drawHarborWater() {
    ctx.fillStyle = "#124ca6";
    ctx.fillRect(0, WATER_Y, W, H - WATER_Y);

    // Multi-layer rolling 16-bit white/cyan harbor waves
    for (var layer = 0; layer < 4; layer++) {
      var baseY = WATER_Y + 10 + layer * 17;
      var speed = (layer % 2 === 0 ? 1 : -1) * (24 + layer * 9);
      ctx.strokeStyle = layer % 2 === 0 ? "#e0f2fe" : "#38bdf8";
      ctx.lineWidth = layer === 0 ? 3.5 : 2.5;
      ctx.beginPath();
      for (var x = 0; x <= W; x += 10) {
        var wy = baseY + Math.sin((x + state.camX * 0.7 + state.time * speed) * 0.045 + layer * 1.4) * 5;
        if (x === 0) ctx.moveTo(x, wy);
        else ctx.lineTo(x, wy);
      }
      ctx.stroke();
    }
  }

  function drawLimo(x, y) {
    ctx.save();
    ctx.fillStyle = "#111827";
    ctx.fillRect(x, y - 48, 220, 44);
    ctx.fillRect(x + 36, y - 74, 148, 28);
    ctx.fillStyle = "#38bdf8";
    ctx.fillRect(x + 46, y - 68, 40, 18);
    ctx.fillRect(x + 94, y - 68, 42, 18);
    ctx.fillRect(x + 144, y - 68, 32, 18);
    ctx.fillStyle = "#facc15";
    ctx.fillRect(x + 2, y - 34, 12, 10);
    ctx.fillStyle = "#1f2937";
    ctx.beginPath(); ctx.arc(x + 42, y - 4, 16, 0, Math.PI * 2); ctx.fill();
    ctx.beginPath(); ctx.arc(x + 178, y - 4, 16, 0, Math.PI * 2); ctx.fill();
    ctx.fillStyle = "#cbd5e1";
    ctx.beginPath(); ctx.arc(x + 42, y - 4, 7, 0, Math.PI * 2); ctx.fill();
    ctx.beginPath(); ctx.arc(x + 178, y - 4, 7, 0, Math.PI * 2); ctx.fill();
    ctx.restore();
  }

  function drawPropFn(pr) {
    return function () {
      if (pr.hp <= 0 && pr.kind === "crate") return;
      ctx.save();
      if (pr.hitFlash > 0) ctx.translate((Math.random() - 0.5) * 6, 0);

      if (pr.kind === "booth") {
        // Iconic Yellow Telephone Booth (exact reference frame_03s!)
        var by = pr.y - pr.h;
        ctx.fillStyle = pr.hp <= 0 ? "#856a12" : "#f5c828";
        ctx.fillRect(pr.x, by, pr.w, pr.h);
        ctx.strokeStyle = "#141821";
        ctx.lineWidth = 3;
        ctx.strokeRect(pr.x, by, pr.w, pr.h);
        // Header sign
        ctx.fillStyle = "#1e293b";
        ctx.fillRect(pr.x + 4, by + 5, pr.w - 8, 12);
        ctx.fillStyle = "#fef08a";
        ctx.font = "bold 7.5px monospace";
        ctx.textAlign = "center";
        ctx.fillText("TELEPHONE", pr.x + pr.w * 0.5, by + 14);
        // Window panes
        ctx.fillStyle = pr.hp <= 0 ? "#1f2937" : "#7dd3fc";
        ctx.fillRect(pr.x + 8, by + 22, pr.w - 16, 52);
        ctx.strokeRect(pr.x + 8, by + 22, pr.w - 16, 52);
      } else if (pr.kind === "car") {
        // Parked Retro White Hatchback Car (destructible Street Fighter II style!)
        var cy = pr.y - pr.h;
        var dmgStage = pr.hp <= 0 ? 2 : (pr.hp <= 3 ? 1 : 0);
        ctx.fillStyle = dmgStage === 2 ? "#94a3b8" : "#e2e8f0";
        ctx.fillRect(pr.x, cy + 20, pr.w, 34);
        ctx.fillRect(pr.x + 28, cy, 86, 24);
        ctx.strokeStyle = "#141821";
        ctx.lineWidth = 3;
        ctx.strokeRect(pr.x, cy + 20, pr.w, 34);
        ctx.strokeRect(pr.x + 28, cy, 86, 24);
        ctx.fillStyle = dmgStage >= 1 ? "#334155" : "#64748b";
        ctx.fillRect(pr.x + 34, cy + 4, 34, 16);
        ctx.fillRect(pr.x + 74, cy + 4, 34, 16);
        // Wheels
        ctx.fillStyle = "#111827";
        ctx.beginPath(); ctx.arc(pr.x + 32, pr.y - 2, 14, 0, Math.PI * 2); ctx.fill();
        ctx.beginPath(); ctx.arc(pr.x + 116, pr.y - 2, 14, 0, Math.PI * 2); ctx.fill();
        ctx.fillStyle = "#cbd5e1";
        ctx.beginPath(); ctx.arc(pr.x + 32, pr.y - 2, 6, 0, Math.PI * 2); ctx.fill();
        ctx.beginPath(); ctx.arc(pr.x + 116, pr.y - 2, 6, 0, Math.PI * 2); ctx.fill();
        // Smoke if damaged
        if (dmgStage >= 1 && Math.random() < 0.25) {
          spawnParticles(pr.x + 18, cy + 16, 1, "#64748b", 35, -60);
        }
      } else {
        // Wooden Street Crate
        ctx.fillStyle = "#a16207";
        ctx.fillRect(pr.x, pr.y - pr.h, pr.w, pr.h);
        ctx.strokeStyle = "#1c1917";
        ctx.lineWidth = 3;
        ctx.strokeRect(pr.x, pr.y - pr.h, pr.w, pr.h);
        ctx.beginPath();
        ctx.moveTo(pr.x, pr.y - pr.h); ctx.lineTo(pr.x + pr.w, pr.y);
        ctx.stroke();
      }
      ctx.restore();
    };
  }

  function drawPickupFn(pk) {
    return function () {
      var bob = Math.sin(state.time * 7 + pk.x) * 4;
      var py = pk.y - pk.z - 18 + bob;
      ctx.save();
      ctx.fillStyle = "rgba(0,0,0,0.35)";
      ctx.beginPath(); ctx.ellipse(pk.x, pk.y, 16, 5, 0, 0, Math.PI * 2); ctx.fill();

      if (pk.kind === "pizza") {
        ctx.fillStyle = "#f59e0b";
        ctx.beginPath();
        ctx.moveTo(pk.x - 14, py - 10);
        ctx.lineTo(pk.x + 14, py - 10);
        ctx.lineTo(pk.x, py + 14);
        ctx.closePath();
        ctx.fill();
        ctx.strokeStyle = "#141821"; ctx.lineWidth = 2; ctx.stroke();
        ctx.fillStyle = "#dc2626";
        ctx.beginPath(); ctx.arc(pk.x - 3, py - 2, 3, 0, Math.PI * 2); ctx.fill();
        ctx.beginPath(); ctx.arc(pk.x + 4, py - 4, 2.5, 0, Math.PI * 2); ctx.fill();
      } else if (pk.kind === "pipe") {
        ctx.fillStyle = "#cbd5e1";
        ctx.strokeStyle = "#0f172a";
        ctx.lineWidth = 2;
        ctx.fillRect(pk.x - 20, py - 4, 40, 8);
        ctx.strokeRect(pk.x - 20, py - 4, 40, 8);
      } else {
        drawMustacheShape(pk.x, py, 1.0, "#facc15", "#78350f");
      }
      ctx.restore();
    };
  }

  function drawHero() {
    var h = state.hero;
    // Ground shadow
    ctx.fillStyle = "rgba(0,0,0,0.38)";
    var shScale = Math.max(0.45, 1 - h.z / 260);
    ctx.beginPath();
    ctx.ellipse(h.x, h.y + 2, 28 * shScale, 9 * shScale, 0, 0, Math.PI * 2);
    ctx.fill();

    // Blink on damage invuln only during active combat (never on title screen or Arm Reload)
    if (state.phase === "playing" && h.invuln > 0 && h.armReloadTimer <= 0 && Math.floor(state.time * 18) % 2 === 0) return;

    var img = sprites.heroIdle;
    var drawH = 112;
    if (h.attackType === "punch") {
      img = sprites.heroPunch;
      drawH = 116;
    } else if (h.attackType === "kick" || h.z > 24) {
      img = sprites.heroKick;
      drawH = 110;
    }

    var bob = h.z <= 1 ? Math.abs(Math.sin(h.walkPhase)) * -6 : 0;
    ctx.save();
    ctx.translate(Math.round(h.x), Math.round(h.y - h.z + bob));

    // Arm Reload Blazing Golden Aura
    if (h.armReloadTimer > 0) {
      var aura = ctx.createRadialGradient(0, -54, 12, 0, -54, 72);
      aura.addColorStop(0, "rgba(255, 226, 74, 0.72)");
      aura.addColorStop(0.55, "rgba(255, 95, 46, 0.35)");
      aura.addColorStop(1, "rgba(255, 95, 46, 0)");
      ctx.fillStyle = aura;
      ctx.beginPath();
      ctx.arc(0, -54, 72, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.scale(h.facing, 1);

    if (img && img.complete && img.naturalWidth > 0) {
      var drawW = Math.round(drawH * (img.naturalWidth / img.naturalHeight));
      ctx.drawImage(img, -Math.round(drawW * 0.45), -drawH, drawW, drawH);
    } else {
      ctx.fillStyle = "#f59e0b";
      ctx.fillRect(-20, -96, 40, 96);
    }

    // Held Steel Pipe indicator
    if (h.weapon) {
      ctx.fillStyle = "#e2e8f0";
      ctx.strokeStyle = "#0f172a";
      ctx.lineWidth = 2;
      ctx.fillRect(16, -64, 34, 7);
      ctx.strokeRect(16, -64, 34, 7);
    }

    ctx.restore();
  }

  function drawEnemyFn(e) {
    return function () {
      ctx.fillStyle = "rgba(0,0,0,0.35)";
      ctx.beginPath();
      ctx.ellipse(e.x, e.y + 2, e.kind === "boss" ? 38 : 26, 9, 0, 0, Math.PI * 2);
      ctx.fill();

      var img = e.kind === "boss" ? sprites.bossKingpin : sprites.enemySuit;
      var drawH = e.kind === "boss" ? 148 : (e.kind === "heavy" ? 116 : 104);
      var bob = e.dying ? 0 : Math.abs(Math.sin(e.walkPhase)) * -5;

      ctx.save();
      ctx.translate(Math.round(e.x), Math.round(e.y - e.z + bob));
      if (e.dying) {
        ctx.rotate((0.85 - e.dieTimer) * (e.vx >= 0 ? 7 : -7));
      }
      // Enemy sprites face left natively, so flip when facing right!
      ctx.scale(e.facing > 0 ? -1 : 1, 1);

      if (e.hitFlash > 0) {
        ctx.filter = "brightness(2.4)";
      } else if (e.kind === "heavy") {
        ctx.filter = "hue-rotate(-32deg) saturate(1.3)";
      } else if (e.kind === "thrower") {
        ctx.filter = "hue-rotate(155deg) saturate(1.15)";
      }

      if (img && img.complete && img.naturalWidth > 0) {
        var drawW = Math.round(drawH * (img.naturalWidth / img.naturalHeight));
        ctx.drawImage(img, -Math.round(drawW * 0.5), -drawH, drawW, drawH);
      } else {
        ctx.fillStyle = "#78350f";
        ctx.fillRect(-18, -90, 36, 90);
      }
      ctx.filter = "none";
      ctx.restore();

      // Tiny overhead HP pips for damaged non-boss goons
      if (!e.dying && e.kind !== "boss" && e.hp < e.maxHp) {
        var bw = 42;
        ctx.fillStyle = "#141821";
        ctx.fillRect(e.x - bw * 0.5, e.y - drawH - 12, bw, 6);
        ctx.fillStyle = "#ef4444";
        ctx.fillRect(e.x - bw * 0.5 + 1, e.y - drawH - 11, (bw - 2) * Math.max(0, e.hp / e.maxHp), 4);
      }
    };
  }

  function drawMustacheShape(x, y, scale, fillCol, strokeCol) {
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(scale, scale);
    ctx.fillStyle = fillCol || "#7c3e18";
    ctx.strokeStyle = strokeCol || "#181012";
    ctx.lineWidth = 2.4;
    ctx.beginPath();
    ctx.moveTo(0, -2);
    ctx.bezierCurveTo(-5, -10, -16, -10, -22, -2);
    ctx.bezierCurveTo(-25, 2, -27, 4, -30, 1);
    ctx.bezierCurveTo(-27, 10, -16, 12, -7, 7);
    ctx.lineTo(0, 2);
    ctx.lineTo(7, 7);
    ctx.bezierCurveTo(16, 12, 27, 10, 30, 1);
    ctx.bezierCurveTo(27, 4, 25, 2, 22, -2);
    ctx.bezierCurveTo(16, -10, 5, -10, 0, -2);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
    ctx.restore();
  }

  function drawProjectile(q) {
    ctx.save();
    ctx.translate(Math.round(q.x), Math.round(q.y));
    ctx.rotate(q.rot);
    if (q.kind === "pipe") {
      ctx.fillStyle = "#e2e8f0";
      ctx.strokeStyle = "#0f172a";
      ctx.lineWidth = 2.5;
      ctx.fillRect(-20, -4, 40, 8);
      ctx.strokeRect(-20, -4, 40, 8);
    } else {
      // Iconic Spinning Mustache Boomerang!
      var fill = q.owner === "hero" ? "#38e8ff" : "#7c3e18";
      var stroke = q.owner === "hero" ? "#ffe680" : "#141821";
      drawMustacheShape(0, 0, 0.9, fill, stroke);
    }
    ctx.restore();
  }

  // =========================================================================
  // VINTAGE BRASS MERCURY THERMOMETER & DOM HUD SYNC
  // =========================================================================
  function drawThermometer() {
    if (!thermoCtx) return;
    var c = thermoCtx;
    var tw = thermoCanvas.width;
    var th = thermoCanvas.height;
    c.clearRect(0, 0, tw, th);

    var pct = state.hero.armReloadTimer > 0
      ? Math.max(0.1, state.hero.armReloadTimer / 8.0)
      : Math.max(0, Math.min(1, state.thermo / 100));

    // Top ornate brass medallion circle (matches frame_03s!)
    c.fillStyle = "#5c1124";
    c.strokeStyle = "#f5b82e";
    c.lineWidth = 3;
    c.beginPath(); c.arc(27, 22, 18, 0, Math.PI * 2); c.fill(); c.stroke();
    if (pct >= 0.99 || state.hero.armReloadTimer > 0) {
      c.fillStyle = "#ffe24a";
      c.beginPath(); c.arc(27, 22, 11, 0, Math.PI * 2); c.fill();
    }

    // Outer brass tube body
    c.fillStyle = "#d99b26";
    c.strokeStyle = "#3a1a08";
    c.lineWidth = 2.5;
    c.beginPath(); c.roundRect(14, 44, 26, 150, 12); c.fill(); c.stroke();

    // Inner glass channel
    c.fillStyle = "#dbeafe";
    c.fillRect(20, 54, 14, 130);

    // Rising Mercury Column (Crimson -> Gold when Arm Reload ready!)
    var fillH = Math.round(126 * pct);
    var grad = c.createLinearGradient(0, 184, 0, 54);
    if (pct >= 0.99 || state.hero.armReloadTimer > 0) {
      grad.addColorStop(0, "#ff5f2e");
      grad.addColorStop(1, "#ffe24a");
    } else {
      grad.addColorStop(0, "#2563eb");
      grad.addColorStop(0.55, "#38bdf8");
      grad.addColorStop(1, "#e11d48");
    }
    c.fillStyle = grad;
    c.fillRect(21, 182 - fillH, 12, fillH);

    // Graduation ticks
    c.strokeStyle = "#1e293b";
    c.lineWidth = 1.5;
    for (var ty = 64; ty <= 174; ty += 14) {
      c.beginPath(); c.moveTo(20, ty); c.lineTo(28, ty); c.stroke();
    }

    // Bottom ornate crimson/gold bulb
    c.fillStyle = "#b81d33";
    c.strokeStyle = "#f5b82e";
    c.lineWidth = 3;
    c.beginPath(); c.arc(27, 198, 17, 0, Math.PI * 2); c.fill(); c.stroke();
    c.fillStyle = pct >= 0.99 ? "#ffe24a" : "#fb7185";
    c.beginPath(); c.arc(27, 198, 9, 0, Math.PI * 2); c.fill();
  }

  function setVisible(id, show) {
    var el = document.getElementById(id);
    if (!el) return;
    if (show) el.classList.remove("is-hidden");
    else el.classList.add("is-hidden");
  }

  function syncHud() {
    drawThermometer();

    var h = state.hero;
    var heartsRow = document.getElementById("hearts-row");
    if (heartsRow && heartsRow.children) {
      for (var i = 0; i < 3; i++) {
        var heartEl = heartsRow.children[i];
        var rem = h.hp - i * 2;
        heartEl.classList.remove("empty", "half");
        if (rem <= 0) heartEl.classList.add("empty");
        else if (rem === 1) heartEl.classList.add("half");
      }
    }

    // Skill badges [O] [K] [L]
    var rO = document.getElementById("ring-o");
    if (rO) rO.setAttribute("stroke-dashoffset", String(Math.round(113 * Math.min(1, h.oCd / 0.21))));
    var rK = document.getElementById("ring-k");
    if (rK) rK.setAttribute("stroke-dashoffset", String(Math.round(113 * Math.min(1, h.kCd / 0.55))));
    var rL = document.getElementById("ring-l");
    var ultPct = h.armReloadTimer > 0 ? h.armReloadTimer / 8.0 : state.thermo / 100;
    if (rL) rL.setAttribute("stroke-dashoffset", String(Math.round(113 * (1 - ultPct))));

    var btnL = document.getElementById("btn-skill-l");
    var readyTag = document.getElementById("thermo-ready-tag");
    var ultReady = state.thermo >= 100 || h.armReloadTimer > 0;
    if (btnL) {
      if (ultReady) btnL.classList.add("ready-glow");
      else btnL.classList.remove("ready-glow");
    }
    if (readyTag) {
      if (ultReady) readyTag.classList.remove("is-hidden");
      else readyTag.classList.add("is-hidden");
    }

    // Counter [000/025] & Score
    var koEl = document.getElementById("hud-ko-counter");
    if (koEl) {
      var kStr = String(state.kos).padStart(3, "0");
      koEl.textContent = state.endless ? kStr + " KO" : kStr + "/" + String(GOAL_KOS).padStart(3, "0");
    }
    var scEl = document.getElementById("hud-score-pill");
    if (scEl) scEl.textContent = "SCORE " + String(state.score).padStart(5, "0");

    // Flashing GO ->
    setVisible("go-banner", state.goTimer > 0 && state.phase === "playing");

    // Boss HP Bar
    var boss = null;
    for (var ei = 0; ei < state.enemies.length; ei++) {
      if (state.enemies[ei].kind === "boss" && !state.enemies[ei].dying) {
        boss = state.enemies[ei];
        break;
      }
    }
    setVisible("boss-hud-bar", Boolean(boss));
    if (boss) {
      var bpct = Math.max(0, Math.min(1, boss.hp / boss.maxHp));
      var tpct = Math.max(bpct, Math.min(1, boss.trailHp / boss.maxHp));
      var pctEl = document.getElementById("boss-hud-pct");
      if (pctEl) pctEl.textContent = Math.ceil(bpct * 100) + "%";
      var fillEl = document.getElementById("boss-hp-fill");
      if (fillEl) fillEl.setAttribute("width", String(Math.round(238 * bpct)));
      var trailEl = document.getElementById("boss-hp-trail");
      if (trailEl) trailEl.setAttribute("width", String(Math.round(238 * tpct)));
    }
  }

  // =========================================================================
  // GAME FLOW & REDDIT SUBMISSIONS
  // =========================================================================
  function startRun(endless) {
    ensureAudio();
    startMusic();
    resetWorld(endless);
    state.phase = "playing";
    setVisible("arcade-title-screen", false);
    setVisible("arcade-pause-modal", false);
    setVisible("arcade-end-modal", false);
    setVisible("arm-reload-screen", false);
    setVisible("arcade-hud", true);
    var tc = document.getElementById("touch-controls");
    if (tc) tc.classList.add("active");
    sfx("go");
    addPopup(state.hero.x, 260, "FIGHT!", "#ffe24a", 34);

    fetch("/api/start-run", { method: "POST" })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (d) { if (d && d.runId) apiState.runId = String(d.runId); })
      .catch(function () {});
  }

  function computeRank(score, victory) {
    if (victory && score >= 14000) return "SSS";
    if (victory && score >= 9500) return "SS";
    if (score >= 7000) return "S";
    if (score >= 4500) return "A";
    if (score >= 2500) return "B";
    if (score >= 1200) return "C";
    return "D";
  }

  function finishRun(victory) {
    state.phase = "ended";
    state.victory = victory;
    var tc = document.getElementById("touch-controls");
    if (tc) tc.classList.remove("active");
    setVisible("arcade-end-modal", true);

    if (state.score > apiState.bestScore) {
      apiState.bestScore = state.score;
      saveLocal();
    }

    var rank = computeRank(state.score, victory);
    var tag = document.getElementById("end-outcome-tag");
    if (tag) tag.textContent = victory ? "STAGE CLEAR! HARBOR SAVED" : "KNOCKED OUT!";
    var sc = document.getElementById("end-score-val");
    if (sc) sc.textContent = state.score.toLocaleString("en-US") + " PTS";
    var rk = document.getElementById("end-rank-val");
    if (rk) rk.textContent = rank;
    var sum = document.getElementById("end-summary-box");
    var chronicle = victory
      ? "Cleared the Harbor Quay with " + state.kos + " KOs, max combo " + state.maxCombo + "x, and crushed Don Mustachio with ARM RELOAD!"
      : "Fought through " + state.kos + " suited goons on the Harbor Quay (max combo " + state.maxCombo + "x, " + state.score + " PTS).";
    if (sum) sum.textContent = chronicle;

    if (!apiState.submitted) {
      apiState.submitted = true;
      fetch("/api/submit-run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          runId: apiState.runId || undefined,
          score: state.score,
          kills: state.kos,
          bossSlain: state.bossSlain,
          bossReached: state.bossActive || state.bossSlain,
          level: Math.max(1, Math.floor(state.kos / 4) + 1),
          heroClass: state.skinIdx,
          buildSummary: SKIN_PERKS[state.skinIdx].name + " · ARM RELOAD",
          isEndless: state.endless
        })
      }).catch(function () {});
    }
  }

  function togglePause() {
    if (state.phase === "playing") {
      state.phase = "paused";
      setVisible("arcade-pause-modal", true);
      var ps = document.getElementById("pause-stats-line");
      if (ps) ps.textContent = "KOs: " + state.kos + " / " + GOAL_KOS + " · Score: " + state.score + " · Max Combo: " + state.maxCombo + "x";
    } else if (state.phase === "paused") {
      state.phase = "playing";
      setVisible("arcade-pause-modal", false);
    }
  }

  function applyCrtVisual() {
    var el = document.getElementById("crt-scanlines");
    if (el) {
      if (crtEnabled) el.classList.remove("crt-off");
      else el.classList.add("crt-off");
    }
    var btn = document.getElementById("btn-toggle-crt");
    if (btn) btn.textContent = crtEnabled ? "CRT: ON" : "CRT: OFF";
  }

  function applyAudioVisual() {
    var btn = document.getElementById("btn-toggle-audio");
    if (btn) btn.textContent = isMuted ? "AUDIO: OFF" : "AUDIO: ON";
  }

  // =========================================================================
  // INPUT & DOM BINDINGS
  // =========================================================================
  window.addEventListener("keydown", function (ev) {
    keys[ev.code] = true;
    if (state.phase === "arm_reload" && (ev.code === "Space" || ev.code === "Enter" || ev.code === "KeyO" || ev.code === "KeyL")) {
      ev.preventDefault();
      dismissArmReload();
      return;
    }
    if (state.phase === "title" && (ev.code === "Enter" || ev.code === "Space")) {
      ev.preventDefault();
      startRun(state.endless);
      return;
    }
    if (ev.code === "Escape" || ev.code === "KeyP") {
      togglePause();
      return;
    }
    if (state.phase !== "playing") return;
    if (ev.code === "Space") { ev.preventDefault(); heroJump(); }
    else if (ev.code === "KeyO" || ev.code === "KeyJ") heroPunch();
    else if (ev.code === "KeyK") heroKick();
    else if (ev.code === "KeyL" || ev.code === "KeyQ") triggerArmReload(false);
  });

  window.addEventListener("keyup", function (ev) {
    keys[ev.code] = false;
  });

  function dismissArmReload() {
    if (state.phase !== "arm_reload") return;
    setVisible("arm-reload-screen", false);
    state.phase = "playing";
    addPopup(state.hero.x, state.hero.y - 95, "INVINCIBLE SUPER PUNCHES!", "#ffe24a", 23);
  }

  var armScreen = document.getElementById("arm-reload-screen");
  if (armScreen) armScreen.addEventListener("click", dismissArmReload);
  var armBtn = document.getElementById("btn-arm-continue");
  if (armBtn) armBtn.addEventListener("click", function (e) { e.stopPropagation(); dismissArmReload(); });

  var skinBtns = document.querySelectorAll("#skin-selector .skin-btn");
  skinBtns.forEach(function (btn) {
    btn.addEventListener("click", function () {
      skinBtns.forEach(function (b) { b.classList.remove("selected"); });
      btn.classList.add("selected");
      state.skinIdx = Number(btn.getAttribute("data-skin")) || 0;
    });
  });

  var modeStory = document.getElementById("btn-mode-story");
  var modeEndless = document.getElementById("btn-mode-endless");
  if (modeStory) modeStory.addEventListener("click", function () {
    state.endless = false;
    modeStory.classList.add("selected");
    if (modeEndless) modeEndless.classList.remove("selected");
  });
  if (modeEndless) modeEndless.addEventListener("click", function () {
    state.endless = true;
    modeEndless.classList.add("selected");
    if (modeStory) modeStory.classList.remove("selected");
  });

  var btnStart = document.getElementById("btn-press-start");
  if (btnStart) btnStart.addEventListener("click", function () { startRun(state.endless); });

  var btnSkillO = document.getElementById("btn-skill-o");
  if (btnSkillO) btnSkillO.addEventListener("click", heroPunch);
  var btnSkillK = document.getElementById("btn-skill-k");
  if (btnSkillK) btnSkillK.addEventListener("click", heroKick);
  var btnSkillL = document.getElementById("btn-skill-l");
  if (btnSkillL) btnSkillL.addEventListener("click", function () { triggerArmReload(false); });

  var btnPause = document.getElementById("btn-pause");
  if (btnPause) btnPause.addEventListener("click", togglePause);
  var btnResumeTop = document.getElementById("btn-pause-resume-top");
  if (btnResumeTop) btnResumeTop.addEventListener("click", togglePause);
  var btnResumeBot = document.getElementById("btn-pause-resume");
  if (btnResumeBot) btnResumeBot.addEventListener("click", togglePause);
  var btnRestart = document.getElementById("btn-pause-restart");
  if (btnRestart) btnRestart.addEventListener("click", function () { startRun(state.endless); });

  var btnAudio = document.getElementById("btn-toggle-audio");
  if (btnAudio) btnAudio.addEventListener("click", function () {
    isMuted = !isMuted;
    applyAudioVisual();
    saveLocal();
  });
  var btnCrt = document.getElementById("btn-toggle-crt");
  if (btnCrt) btnCrt.addEventListener("click", function () {
    crtEnabled = !crtEnabled;
    applyCrtVisual();
    saveLocal();
  });

  var btnEndReplay = document.getElementById("btn-end-replay");
  if (btnEndReplay) btnEndReplay.addEventListener("click", function () { startRun(false); });
  var btnEndEndless = document.getElementById("btn-end-endless");
  if (btnEndEndless) btnEndEndless.addEventListener("click", function () { startRun(true); });
  var btnEndShare = document.getElementById("btn-end-share");
  if (btnEndShare) btnEndShare.addEventListener("click", function () {
    var st = document.getElementById("end-status-msg");
    if (st) st.textContent = "Posting arcade chronicle to Reddit...";
    fetch("/api/share-comment", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        score: state.score,
        kills: state.kos,
        bossSlain: state.bossSlain,
        level: Math.max(1, Math.floor(state.kos / 4) + 1),
        heroClass: state.skinIdx,
        streak: apiState.streak,
        buildEmojis: "🥊⚡👨🏻",
        rank: computeRank(state.score, state.victory),
        chronicle: "HENRY'S ARCADE STREETS · " + state.kos + " KOs · " + state.maxCombo + "x Combo · ARM RELOAD!"
      })
    })
      .then(function (r) { return r.json().catch(function () { return {}; }); })
      .then(function (d) {
        if (st) st.textContent = d && d.ok ? "Shared to Reddit comments!" : "Play inside Reddit to post comments!";
      })
      .catch(function () {
        if (st) st.textContent = "Play inside Reddit to post comments!";
      });
  });

  // Mobile Touch Controls Bindings
  function bindHold(id, prop) {
    var el = document.getElementById(id);
    if (!el) return;
    el.addEventListener("pointerdown", function (e) { e.preventDefault(); touchInput[prop] = true; });
    el.addEventListener("pointerup", function () { touchInput[prop] = false; });
    el.addEventListener("pointerleave", function () { touchInput[prop] = false; });
    el.addEventListener("pointercancel", function () { touchInput[prop] = false; });
  }
  bindHold("touch-left", "left");
  bindHold("touch-right", "right");
  var tJump = document.getElementById("touch-jump");
  if (tJump) tJump.addEventListener("pointerdown", function (e) { e.preventDefault(); heroJump(); });
  var tO = document.getElementById("touch-o");
  if (tO) tO.addEventListener("pointerdown", function (e) { e.preventDefault(); heroPunch(); });
  var tK = document.getElementById("touch-k");
  if (tK) tK.addEventListener("pointerdown", function (e) { e.preventDefault(); heroKick(); });
  var tL = document.getElementById("touch-l");
  if (tL) tL.addEventListener("pointerdown", function (e) { e.preventDefault(); triggerArmReload(false); });

  // Verification hook for automated Chrome CDP testing
  window.__ARCADE_DEBUG__ = {
    getState: function () { return state; },
    startStory: function () { startRun(false); },
    punch: heroPunch,
    kick: heroKick,
    jump: heroJump,
    showArmReload: function () { triggerArmReload(true); },
    dismissArmReload: dismissArmReload,
    summonBoss: summonKingpinBoss
  };

  applyCrtVisual();
  applyAudioVisual();
  resetWorld(false);
  state.phase = "title";

  var lastTs = performance.now();
  function loop(now) {
    var dt = Math.min(0.05, Math.max(0.001, (now - lastTs) / 1000));
    lastTs = now;
    update(dt);
    render();
    syncHud();
    requestAnimationFrame(loop);
  }
  requestAnimationFrame(loop);
})();

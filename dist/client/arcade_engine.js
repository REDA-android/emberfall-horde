(function () {
  "use strict";

  var W = 960;
  var H = 540;
  var WORLD_W = 3840; // 4 seamless 960px 16-bit SNES panoramas: [A, B, A, B]
  var LANE_MIN_Y = 386;
  var LANE_MAX_Y = 434;
  var WATER_Y = 472;
  var GOAL_KOS = 25;

  var canvas = document.getElementById("arcade-canvas");
  var ctx = canvas ? canvas.getContext("2d") : null;
  var thermoCanvas = document.getElementById("thermo-canvas");
  var thermoCtx = thermoCanvas ? thermoCanvas.getContext("2d") : null;

  // =========================================================================
  // 16-BIT SNES CHIPTUNE & ARCADE SFX SYNTHESIZER (Zero External Audio URLs)
  // =========================================================================
  var audioCtx = null;
  var masterGain = null;
  var noiseBuf = null;
  var isMuted = false;
  var crtEnabled = true;
  var seqTimer = null;
  var seqStep = 0;
  var musicVol = 0.70;
  var sfxVol = 0.85;

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
    var len = Math.floor(ac.sampleRate * 0.38);
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
    if (isMuted || sfxVol <= 0.01) return;
    if (masterGain && audioCtx) {
      masterGain.gain.setTargetAtTime(0.72 * Math.max(0.15, sfxVol), audioCtx.currentTime, 0.02);
    }
    if (name === "slash") {
      noise(0.08, 2400, "bandpass", 0.10, 0);
      tone(520, 170, 0.10, "sawtooth", 0.10, 0);
    } else if (name === "punch") {
      tone(220, 68, 0.08, "square", 0.11, 0);
      noise(0.06, 860, "bandpass", 0.09, 0);
    } else if (name === "heavy") {
      tone(155, 42, 0.16, "sawtooth", 0.15, 0);
      noise(0.14, 520, "lowpass", 0.14, 0);
    } else if (name === "whirlwind") {
      noise(0.18, 1900, "bandpass", 0.12, 0);
      tone(330, 880, 0.22, "sawtooth", 0.12, 0);
    } else if (name === "jump") {
      tone(240, 560, 0.12, "square", 0.07, 0);
    } else if (name === "thunder") {
      noise(0.36, 420, "lowpass", 0.20, 0);
      tone(180, 44, 0.34, "sawtooth", 0.16, 0);
      tone(523.25, 1046.5, 0.28, "square", 0.11, 0.12);
    } else if (name === "stache") {
      tone(659.25, 880, 0.07, "square", 0.07, 0);
      tone(880, 1174.6, 0.09, "triangle", 0.07, 0.05);
    } else if (name === "canister") {
      tone(440, 659.25, 0.09, "square", 0.09, 0);
      tone(659.25, 987.77, 0.12, "square", 0.10, 0.07);
      tone(987.77, 1318.5, 0.18, "sawtooth", 0.10, 0.14);
    } else if (name === "heal") {
      tone(523.25, 659.25, 0.10, "triangle", 0.10, 0);
      tone(659.25, 783.99, 0.11, "triangle", 0.10, 0.08);
      tone(783.99, 1046.5, 0.18, "sine", 0.11, 0.16);
    } else if (name === "parry") {
      tone(880, 1480, 0.11, "triangle", 0.12, 0);
      tone(1320, 1760, 0.12, "sine", 0.09, 0.05);
    } else if (name === "reload") {
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
    } else if (name === "hurt") {
      tone(180, 58, 0.19, "sawtooth", 0.13, 0);
      noise(0.11, 450, "lowpass", 0.09, 0);
    } else if (name === "boss_rage") {
      tone(110, 62, 0.42, "sawtooth", 0.15, 0);
      tone(164.81, 82.41, 0.38, "square", 0.11, 0.08);
    } else if (name === "victory") {
      tone(523.25, 523.25, 0.14, "square", 0.10, 0);
      tone(659.25, 659.25, 0.14, "square", 0.10, 0.12);
      tone(783.99, 783.99, 0.16, "square", 0.11, 0.24);
      tone(1046.5, 1046.5, 0.45, "sawtooth", 0.12, 0.36);
    } else if (name === "gameover") {
      tone(330, 294, 0.22, "sawtooth", 0.10, 0);
      tone(262, 220, 0.26, "sawtooth", 0.10, 0.2);
      tone(196, 130, 0.48, "triangle", 0.11, 0.42);
    } else if (name === "go") {
      tone(880, 880, 0.07, "square", 0.07, 0);
      tone(1174.6, 1174.6, 0.11, "square", 0.08, 0.09);
    }
  }

  function startMusic() {
    if (seqTimer) return;
    seqTimer = setInterval(function () {
      if (isMuted || (state.phase !== "playing" && state.phase !== "intro")) return;
      var intense = state.bossActive || state.hero.armReloadTimer > 0;
      var bass = intense ? BOSS_BASS : STREET_BASS;
      var lead = intense ? BOSS_LEAD : STREET_LEAD;
      var i = seqStep % 16;
      seqStep++;
      if (musicVol <= 0.01) return;
      var bn = bass[i];
      if (bn > 0) tone(bn, bn * 0.99, 0.13, "triangle", (intense ? 0.068 : 0.052) * musicVol, 0);
      var ln = lead[i];
      if (ln > 0 && (intense || seqStep % 32 >= 16)) {
        tone(ln, ln, 0.12, "square", (intense ? 0.038 : 0.026) * musicVol, 0);
      }
      if (i % 4 === 0) tone(122, 42, 0.09, "sine", 0.082 * musicVol, 0);
      if (i === 4 || i === 12) noise(0.06, 1600, "bandpass", 0.042 * musicVol, 0);
    }, 142);
  }

  // =========================================================================
  // 16-BIT PIXEL-ART TEXTURE & SPRITE LOADER (Zero External Assets)
  // =========================================================================
  var sprites = {};
  var SPRITE_URLS = {
    bgStreetA: "./arcade/bg_street_a.webp",
    bgStreetB: "./arcade/bg_street_b.webp",
    cutBarberTv: "./arcade/cut_barber_tv.webp",
    cutShattered: "./arcade/cut_shattered_shop.webp",
    cutCliff: "./arcade/cut_sunset_cliff.webp",
    heroGoldIdle: "./arcade/hero_gold_idle.webp",
    heroCombSlash: "./arcade/hero_comb_slash.webp",
    heroKick: "./arcade/hero_kick.webp",
    enemyScissor: "./arcade/enemy_scissor.webp",
    enemySuit: "./arcade/enemy_suit.webp",
    bossKingpin: "./arcade/boss_kingpin.webp",
    propBooth: "./arcade/prop_booth.webp",
    propCar: "./arcade/prop_car.webp",
    pickupCanister: "./arcade/pickup_canister.webp",
    pickupCroissant: "./arcade/pickup_croissant.webp",
    pickupMustache: "./arcade/pickup_mustache.webp"
  };
  Object.keys(SPRITE_URLS).forEach(function (k) {
    var img = new Image();
    img.src = SPRITE_URLS[k];
    sprites[k] = img;
  });

  function imgReady(im) {
    return Boolean(im && im.complete && im.naturalWidth > 0);
  }

  // =========================================================================
  // REDDIT DEVVIT PERSISTENCE & LEADERBOARD SYNC
  // =========================================================================
  var STAGE_DEFS = [
    { idx: 0, title: "LEVEL 1 · HARBOR ISLAND", endless: false, startKos: 0, lighting: "day" },
    { idx: 1, title: "LEVEL 2 · SUNSET BOULEVARD", endless: false, startKos: 12, lighting: "sunset" },
    { idx: 2, title: "LEVEL 3 · SYNDICATE NIGHT", endless: false, startKos: 22, lighting: "night" },
    { idx: 3, title: "ENDLESS HARBOR SURVIVAL", endless: true, startKos: 0, lighting: "day" }
  ];

  var SKIN_PERKS = [
    { name: "GOLDEN TITAN", speedMul: 1.0, dmgMul: 1.0, thermoMul: 1.0, special: "whirlwind", aura: "#ffe24a", filter: "none" },
    { name: "ROYAL RANGER", speedMul: 1.15, dmgMul: 1.0, thermoMul: 1.0, special: "blitz", aura: "#38e8ff", filter: "hue-rotate(175deg) saturate(1.2)" },
    { name: "CRIMSON FIST", speedMul: 1.0, dmgMul: 1.2, thermoMul: 1.0, special: "uppercut", aura: "#ff5f2e", filter: "hue-rotate(-25deg) saturate(1.35)" },
    { name: "SECRET AGENT", speedMul: 1.12, dmgMul: 1.15, thermoMul: 1.2, special: "whirlwind", aura: "#f8fafc", filter: "grayscale(0.65) contrast(1.22)" }
  ];

  var apiState = {
    seed: new Date().toISOString().slice(0, 10),
    username: "",
    streak: 1,
    bestScore: 0,
    secretUnlocked: false,
    leaderboard: [],
    runId: "",
    submitted: false
  };

  function loadLocal() {
    try {
      var raw = localStorage.getItem("mustache_protocol_v7") || localStorage.getItem("henry_arcade_v5");
      if (!raw) return;
      var d = JSON.parse(raw);
      if (d) {
        apiState.bestScore = Number(d.bestScore) || 0;
        apiState.streak = Math.max(1, Number(d.streak) || 1);
        apiState.secretUnlocked = Boolean(d.secretUnlocked) || apiState.bestScore >= 5000;
        if (typeof d.muted === "boolean") isMuted = d.muted;
        if (typeof d.crt === "boolean") crtEnabled = d.crt;
        if (typeof d.musicVol === "number") musicVol = Math.max(0, Math.min(1, d.musicVol));
        if (typeof d.sfxVol === "number") sfxVol = Math.max(0, Math.min(1, d.sfxVol));
      }
    } catch (_) {}
  }

  function saveLocal() {
    try {
      localStorage.setItem("mustache_protocol_v7", JSON.stringify({
        bestScore: apiState.bestScore,
        streak: apiState.streak,
        secretUnlocked: apiState.secretUnlocked,
        muted: isMuted,
        crt: crtEnabled,
        musicVol: musicVol,
        sfxVol: sfxVol
      }));
    } catch (_) {}
  }

  function refreshSecretUnlockUi() {
    var btn = document.getElementById("skin-secret-btn");
    var role = document.getElementById("skin-secret-role");
    if (!btn) return;
    if (apiState.secretUnlocked) {
      btn.classList.remove("locked");
      if (role) role.textContent = "+20% Energy [K]";
    } else {
      btn.classList.add("locked");
      if (role) role.textContent = "5K PTS Unlock";
    }
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
        if (Array.isArray(d.leaderboard)) apiState.leaderboard = d.leaderboard.slice(0, 5);
        if (apiState.bestScore >= 5000) apiState.secretUnlocked = true;
        refreshSecretUnlockUi();
      })
      .catch(function () {});
  }

  loadLocal();
  refreshSecretUnlockUi();
  fetchInit();

  // =========================================================================
  // ARCHITECTURAL ROOFTOP & WINDOW LEDGES (Matches v2_01.png High-Jumping!)
  // =========================================================================
  // Each 1920px cycle ([bg_street_a (0..960), bg_street_b (960..1920)]) has real 16-bit roofs & window sills:
  // - Barbershop roof (x=0..205, z=156)
  // - Townhouse 2nd-floor window sills (x=215..835, z=98)
  // - Fancy Garage showroom roof (x=740..960, z=152) -> holds the glowing Blue Energy Canister just like v2_01.png!
  // - Cottage & stone house ledges in street_b (x=1225..1495, z=118)
  function buildLedges() {
    var list = [];
    for (var cycle = 0; cycle < 2; cycle++) {
      var ox = cycle * 1920;
      list.push({ x0: ox + 8, x1: ox + 206, z: 156 });    // Barbershop flat cornice
      list.push({ x0: ox + 225, x1: ox + 392, z: 96 });   // Yellow house window sill
      list.push({ x0: ox + 418, x1: ox + 585, z: 96 });   // Royal-blue house flower sill
      list.push({ x0: ox + 602, x1: ox + 732, z: 104 });  // Crimson house upper window sill (v2_01.png jump!)
      list.push({ x0: ox + 742, x1: ox + 956, z: 152 });  // FANCY GARAGE flat roof!
      list.push({ x0: ox + 1230, x1: ox + 1495, z: 112 }); // Yellow gabled cottage roof
      list.push({ x0: ox + 1515, x1: ox + 1700, z: 96 }); // Blue house flower sill B
    }
    return list;
  }
  var WORLD_LEDGES = buildLedges();

  var state = {
    phase: "title", // title | intro | playing | arm_reload | paused | ended
    stageIdx: 0,
    endless: false,
    skinIdx: 0,
    time: 0,
    introTimer: 0,
    thunderPlayed: false,
    camX: 0,
    camZoom: 1.0,
    zoomTimer: 0,
    shake: 0,
    hitstop: 0,
    mustaches: 0,
    kos: 0,
    score: 0,
    combo: 0,
    maxCombo: 0,
    comboTimer: 0,
    thermo: 24,
    parries: 0,
    reloadsUsed: 0,
    armReloadShownOnce: false,
    bossActive: false,
    bossSlain: false,
    goTimer: 0,
    spawnTimer: 0.9,
    victory: false,
    hero: {
      x: 240,
      y: 412,
      z: 0,
      vz: 0,
      vx: 0,
      vy: 0,
      jumpsUsed: 0,
      facing: 1,
      hp: 6,
      maxHp: 6,
      invuln: 0,
      attackType: "none",
      attackTimer: 0,
      slashArcTimer: 0,
      comboStep: 0,
      comboWindow: 0,
      oCd: 0,
      kCd: 0,
      armReloadTimer: 0,
      walkPhase: 0,
      afterimages: []
    },
    props: [],
    collectibles: [],
    enemies: [],
    projectiles: [],
    particles: [],
    popups: [],
    debris: []
  };

  var keys = {};
  var touchInput = { left: false, right: false };

  function spawnStreetCollectibles() {
    var c = [];
    // Trail 1: Sidewalk mustache line in front of Yellow & Blue houses (v2_01 / v2_04!)
    var sidewalkXs = [360, 412, 464, 620, 670, 1080, 1130, 1180, 1230, 1610, 1660, 2120, 2170, 2220, 2680, 2730, 3150, 3200];
    for (var i = 0; i < sidewalkXs.length; i++) {
      c.push({ kind: "mustache", x: sidewalkXs[i], y: 410, z: 18, bobOff: i * 0.7 });
    }
    // Trail 2: Arcing jump mustaches over Cars & up onto FANCY GARAGE / Barbershop roofs (exact v2_01.png!)
    var aerialArcs = [
      { x: 112, z: 172 }, { x: 168, z: 172 },
      { x: 642, z: 122 }, { x: 696, z: 146 },
      { x: 790, z: 68 },  { x: 842, z: 108 }, { x: 892, z: 138 }, // Arc right over the Silver Hatchback Car up to Fancy Garage!
      { x: 1315, z: 132 }, { x: 1375, z: 132 },
      { x: 2710, z: 68 }, { x: 2762, z: 108 }, { x: 2814, z: 138 }
    ];
    for (var j = 0; j < aerialArcs.length; j++) {
      c.push({ kind: "mustache", x: aerialArcs[j].x, y: 404, z: aerialArcs[j].z, bobOff: j * 0.9 });
    }
    // Rooftop Glowing Blue Energy Canisters (exact v2_01.png spot on right edge of FANCY GARAGE roof!)
    c.push({ kind: "canister", x: 924, y: 398, z: 156, bobOff: 0 });
    c.push({ kind: "canister", x: 1435, y: 398, z: 116, bobOff: 1.2 });
    c.push({ kind: "canister", x: 2844, y: 398, z: 156, bobOff: 2.4 });
    // Golden Healing Croissants ("Restores 1 heart" — exact v2_04.png!)
    c.push({ kind: "croissant", x: 545, y: 402, z: 110, bobOff: 0.4 });
    c.push({ kind: "croissant", x: 1760, y: 412, z: 20, bobOff: 1.5 });
    c.push({ kind: "croissant", x: 2980, y: 412, z: 20, bobOff: 2.2 });
    return c;
  }

  function resetWorld(stageIdx) {
    var stg = STAGE_DEFS[stageIdx] || STAGE_DEFS[0];
    state.stageIdx = stg.idx;
    state.endless = stg.endless;
    state.time = 0;
    state.introTimer = 0;
    state.thunderPlayed = false;
    state.camX = 0;
    state.camZoom = 1.0;
    state.zoomTimer = 0;
    state.shake = 0;
    state.hitstop = 0;
    state.mustaches = 3; // Starts at 003 like v2_01.png!
    state.kos = stg.startKos;
    state.score = 0;
    state.combo = 0;
    state.maxCombo = 0;
    state.comboTimer = 0;
    state.thermo = 26;
    state.parries = 0;
    state.reloadsUsed = 0;
    state.armReloadShownOnce = false;
    state.bossActive = false;
    state.bossSlain = false;
    state.goTimer = 0;
    state.spawnTimer = 0.85;
    state.victory = false;
    apiState.submitted = false;

    state.hero = {
      x: 235,
      y: 412,
      z: 0,
      vz: 0,
      vx: 0,
      vy: 0,
      jumpsUsed: 0,
      facing: 1,
      hp: 6,
      maxHp: 6,
      invuln: 0,
      attackType: "none",
      attackTimer: 0,
      slashArcTimer: 0,
      comboStep: 0,
      comboWindow: 0,
      oCd: 0,
      kCd: 0,
      armReloadTimer: 0,
      walkPhase: 0,
      afterimages: []
    };

    // 16-Bit Pixel-Art Street Props: Yellow Telephone Booth & Silver 2-Door Hatchback Car (exact v2_01.png placement!)
    state.props = [
      { id: "booth1", kind: "booth", x: 518, y: 394, w: 68, h: 112, topZ: 102, hp: 4, maxHp: 4, hitFlash: 0, dropped: false },
      { id: "car1", kind: "car", x: 760, y: 418, w: 172, h: 78, topZ: 62, hp: 6, maxHp: 6, hitFlash: 0, dropped: false },
      { id: "booth2", kind: "booth", x: 1685, y: 394, w: 68, h: 112, topZ: 102, hp: 4, maxHp: 4, hitFlash: 0, dropped: false },
      { id: "car2", kind: "car", x: 1995, y: 418, w: 172, h: 78, topZ: 62, hp: 6, maxHp: 6, hitFlash: 0, dropped: false },
      { id: "booth3", kind: "booth", x: 2438, y: 394, w: 68, h: 112, topZ: 102, hp: 4, maxHp: 4, hitFlash: 0, dropped: false },
      { id: "car3", kind: "car", x: 2680, y: 418, w: 172, h: 78, topZ: 62, hp: 6, maxHp: 6, hitFlash: 0, dropped: false }
    ];

    state.collectibles = spawnStreetCollectibles();
    state.enemies = [];
    state.projectiles = [];
    state.particles = [];
    state.popups = [];
    state.debris = [];

    spawnEnemy("scissor_brown", 545, 414);
    spawnEnemy("scissor_olive", 740, 404);
  }

  function spawnEnemy(kind, x, y) {
    var isBoss = kind === "boss";
    var maxHp = isBoss ? 28 : (kind === "heavy" ? 6 : 3.5);
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
      attackCd: isBoss ? 1.4 : (1.05 + Math.random() * 0.9),
      walkPhase: Math.random() * 6,
      dying: false,
      splashKO: false,
      dieTimer: 0,
      glassesLost: false,
      trailHp: maxHp,
      phase2: false,
      telegraphTimer: 0,
      counterWindow: 0
    });
  }

  function addPopup(x, y, text, color, size) {
    state.popups.push({
      x: x,
      y: y,
      text: text,
      color: color || "#ffe680",
      size: size || 20,
      life: 1.05,
      maxLife: 1.05
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
  // COMBAT & HIGH-JUMP ACTIONS: [SPACE] ROOF LEAP, [O] COMB SLASH, [K] 5s WHIRLWIND, [L] ARM RELOAD
  // =========================================================================
  function currentSupportZ(h, prevZ) {
    var best = 0;
    // 1. Street Props (Phone Booth & Hatchback Car)
    for (var i = 0; i < state.props.length; i++) {
      var pr = state.props[i];
      if (pr.hp <= 0) continue;
      if (h.x > pr.x + 6 && h.x < pr.x + pr.w - 6 && Math.abs(h.y - pr.y) < 34) {
        if (prevZ >= pr.topZ - 14 && h.vz <= 0) {
          best = Math.max(best, pr.topZ);
        }
      }
    }
    // 2. Building Window Sills & Shop Roofs (Barbershop & Fancy Garage!)
    for (var li = 0; li < WORLD_LEDGES.length; li++) {
      var lg = WORLD_LEDGES[li];
      if (h.x >= lg.x0 && h.x <= lg.x1 && h.y <= 414) {
        if (prevZ >= lg.z - 14 && h.vz <= 0) {
          best = Math.max(best, lg.z);
        }
      }
    }
    return best;
  }

  function heroJump() {
    if (state.phase === "intro") { finishIntro(); return; }
    if (state.phase !== "playing") return;
    var h = state.hero;
    if (h.jumpsUsed < 2) {
      h.vz = h.jumpsUsed === 0 ? 605 : 515;
      h.jumpsUsed++;
      sfx("jump");
      spawnParticles(h.x, h.y - h.z, 7, "#ffe680", 110, 240);
    }
  }

  function heroPunch() {
    if (state.phase === "intro") { finishIntro(); return; }
    if (state.phase !== "playing") return;
    var h = state.hero;
    if (h.oCd > 0) return;

    var superMode = h.armReloadTimer > 0;
    h.oCd = superMode ? 0.15 : 0.19;
    h.comboStep = h.comboWindow > 0 ? (h.comboStep + 1) % 3 : 0;
    h.comboWindow = 0.72;
    // In Arm Reload, Henry throws blazing super punches; normally he alternates Golden Comb-Sword slashes & armored hooks!
    h.attackType = superMode ? "super_punch" : "comb_slash";
    h.attackTimer = 0.24;
    h.slashArcTimer = 0.20;
    h.vx += h.facing * (superMode ? 165 : 72);
    sfx(superMode ? "heavy" : (h.comboStep === 2 ? "heavy" : "slash"));
    performMeleeHit("slash");
  }

  function heroKick() {
    if (state.phase === "intro") { finishIntro(); return; }
    if (state.phase !== "playing") return;
    var h = state.hero;
    if (h.kCd > 0) return;

    // Exact 5.0s Cooldown Special Skill (shows cyan ring + "5s" -> "1s" inside badge like v2_04.png!)
    var perk = SKIN_PERKS[state.skinIdx] || SKIN_PERKS[0];
    h.kCd = 5.0;
    h.attackType = "whirlwind";
    h.attackTimer = 0.42;
    h.slashArcTimer = 0.38;

    if (perk.special === "uppercut") {
      h.vz = 540;
      h.vx = h.facing * 260;
      sfx("whirlwind");
      spawnParticles(h.x + h.facing * 24, h.y - h.z - 44, 18, "#ff5f2e", 280, 140);
      addPopup(h.x, h.y - h.z - 98, "GOLDEN UPPERCUT!", "#ff5f2e", 21);
    } else {
      if (h.z < 20) h.vz = 240;
      h.vx = h.facing * 480;
      sfx("whirlwind");
      for (var a = 0; a < 3; a++) {
        h.afterimages.push({ x: h.x - h.facing * a * 20, y: h.y, z: h.z, facing: h.facing, life: 0.28 });
      }
      spawnParticles(h.x + h.facing * 30, h.y - h.z - 44, 18, "#38e8ff", 270, 150);
      addPopup(h.x, h.y - h.z - 96, "COMB WHIRLWIND!", "#38e8ff", 21);
    }
    performMeleeHit("whirlwind");
  }

  function triggerArmReload(forceShowCutin) {
    if (state.phase !== "playing") return;
    var h = state.hero;
    if (!forceShowCutin && state.thermo < 100 && h.armReloadTimer <= 0) return;
    state.thermo = 0;
    state.reloadsUsed++;
    h.armReloadTimer = 8.0;
    h.invuln = 8.0;
    state.shake = 14;
    sfx("reload");
    spawnParticles(h.x, h.y - h.z - 45, 32, "#ffe24a", 340, 180);
    spawnParticles(h.x, h.y - h.z - 45, 20, "#ff5f2e", 270, 140);

    if (!state.armReloadShownOnce || forceShowCutin) {
      state.armReloadShownOnce = true;
      state.phase = "arm_reload";
      setVisible("arm-reload-screen", true);
    } else {
      state.zoomTimer = 0.50;
      addPopup(h.x, h.y - h.z - 98, "ARM RELOAD!", "#ffe24a", 27);
    }
  }

  function performMeleeHit(moveKind) {
    var h = state.hero;
    var perk = SKIN_PERKS[state.skinIdx] || SKIN_PERKS[0];
    var superMode = h.armReloadTimer > 0;
    var isWhirl = moveKind === "whirlwind";
    var reach = isWhirl ? 150 : (superMode ? 136 : 122);
    var hitX = h.x + h.facing * (reach * 0.52);
    var dmg = (isWhirl ? 3.4 : (h.comboStep === 2 ? 2.4 : 1.5)) * perk.dmgMul;
    if (superMode) dmg = 6.5;

    // 1. Deflect Enemy Scissor-Blades / Mustache Boomerangs!
    for (var pi = 0; pi < state.projectiles.length; pi++) {
      var p = state.projectiles[pi];
      if (p.owner === "enemy" && Math.abs(p.x - hitX) < 88 && Math.abs(p.y - (h.y - h.z - 46)) < 76) {
        p.owner = "hero";
        p.vx = h.facing * 560;
        p.vy = -14;
        p.life = 2.2;
        state.parries++;
        state.thermo = Math.min(100, state.thermo + 22 * perk.thermoMul);
        state.score += 250;
        state.hitstop = 0.05;
        state.shake = 8;
        sfx("parry");
        addPopup(p.x, p.y - 24, "PARRY! +250", "#38e8ff", 21);
        spawnParticles(p.x, p.y, 14, "#ffe24a", 240, 180);
      }
    }

    // 2. Smash Street Props (Phone Booth -> drops Healing Croissant! Car -> drops Mustaches + Canister!)
    for (var pri = 0; pri < state.props.length; pri++) {
      var pr = state.props[pri];
      if (pr.hp <= 0) continue;
      var pcx = pr.x + pr.w * 0.5;
      if (Math.abs(pcx - hitX) < pr.w * 0.5 + 54 && Math.abs(pr.y - h.y) < 52 && h.z < 42) {
        pr.hp = Math.max(0, pr.hp - (superMode || isWhirl ? 2 : 1));
        pr.hitFlash = 0.16;
        state.shake = Math.max(state.shake, 6);
        state.thermo = Math.min(100, state.thermo + 12 * perk.thermoMul);
        state.score += 150;
        sfx("glass");
        spawnParticles(pcx, pr.y - 36, 12, pr.kind === "booth" ? "#facc15" : "#e2e8f0", 220, 480);
        addPopup(pcx, pr.y - 78, pr.kind === "car" ? "CAR SMASH! +150" : "CRASH! +150", "#ffe680", 19);
        if (pr.hp <= 0 && !pr.dropped) {
          pr.dropped = true;
          if (pr.kind === "booth") {
            state.collectibles.push({ kind: "croissant", x: pcx, y: pr.y + 8, z: 18, bobOff: 0 });
          } else {
            state.collectibles.push({ kind: "canister", x: pcx, y: pr.y + 8, z: 20, bobOff: 0 });
            state.collectibles.push({ kind: "mustache", x: pcx - 28, y: pr.y + 10, z: 22, bobOff: 0.5 });
            state.collectibles.push({ kind: "mustache", x: pcx + 28, y: pr.y + 10, z: 22, bobOff: 1.1 });
            state.score += 400;
          }
        }
      }
    }

    // 3. Slice Scissor-Claw Agents & Boss!
    for (var ei = 0; ei < state.enemies.length; ei++) {
      var e = state.enemies[ei];
      if (e.dying) continue;
      var dx = e.x - h.x;
      var inFront = isWhirl ? (Math.abs(dx) <= reach) : (dx * h.facing > -32 && Math.abs(dx) <= reach);
      var inLane = Math.abs(e.y - h.y) <= 46 && Math.abs(e.z - h.z) <= 68;
      if (inFront && inLane) {
        damageEnemy(e, dmg, dx >= 0 ? 1 : -1, isWhirl || h.comboStep === 2 || superMode, superMode);
      }
    }
  }

  function damageEnemy(e, dmg, dir, heavy, superMode) {
    var perk = SKIN_PERKS[state.skinIdx] || SKIN_PERKS[0];

    if (e.kind === "boss" && e.counterWindow > 0) {
      e.counterWindow = 0;
      dmg *= 2.2;
      e.stunTimer = 1.6;
      state.score += 600;
      sfx("parry");
      addPopup(e.x, e.y - 148, "COUNTER STUN! +600", "#38e8ff", 24);
    }

    e.hp -= dmg;
    e.hitFlash = 0.16;
    e.stunTimer = Math.max(e.stunTimer, e.kind === "boss" ? 0.22 : (heavy ? 0.68 : 0.34));
    e.vx = dir * (heavy ? (superMode ? 500 : 320) : 145);
    if (heavy && e.kind !== "boss") e.vz = 230;

    state.combo++;
    state.maxCombo = Math.max(state.maxCombo, state.combo);
    state.comboTimer = 2.8;
    state.hitstop = heavy ? 0.048 : 0.024;
    state.shake = Math.max(state.shake, heavy ? 9 : 4);
    if (state.hero.armReloadTimer <= 0) {
      state.thermo = Math.min(100, state.thermo + (heavy ? 13 : 8) * perk.thermoMul);
    }

    // Pop purple goggles off Scissor-Agent on first heavy hit!
    if (!e.glassesLost && e.kind !== "boss") {
      e.glassesLost = true;
      state.debris.push({
        x: e.x,
        y: e.y - e.z - 72,
        vx: -dir * 130 + (Math.random() - 0.5) * 70,
        vy: -250,
        rot: 0,
        life: 1.05
      });
    }

    if (e.kind === "boss" && !e.phase2 && e.hp > 0 && e.hp <= e.maxHp * 0.5) {
      e.phase2 = true;
      state.shake = 14;
      sfx("boss_rage");
      spawnParticles(e.x, e.y - 70, 26, "#c084fc", 300, 180);
      addPopup(e.x, e.y - 144, "PHASE II: CYBER OVERDRIVE!", "#c084fc", 24);
    }

    var words = superMode ? ["SUPER PUNCH!", "ARM RELOAD!", "MEGA POW!"] : (heavy ? ["SLASH!", "POW!", "WHAM!"] : ["SNIP!", "HIT!"]);
    var word = words[Math.floor(Math.random() * words.length)];
    addPopup(e.x, e.y - e.z - 82, word, superMode ? "#ffe24a" : (heavy ? "#ff9f1c" : "#ffffff"), heavy ? 21 : 17);
    spawnParticles(e.x, e.y - e.z - 48, heavy ? 14 : 7, "#ffe24a", 240, 340);

    if (e.hp <= 0) {
      koEnemy(e, dir, heavy || superMode);
    }
  }

  function koEnemy(e, dir, dramatic) {
    if (e.dying) return;
    e.dying = true;
    e.dieTimer = 0.85;
    e.vx = dir * (dramatic ? 400 : 220);
    e.vz = dramatic ? 330 : 190;

    if (dramatic && e.y >= 408 && e.kind !== "boss") {
      e.splashKO = true;
      e.vy = 155;
    }

    var pts = e.kind === "boss" ? 5000 : (e.kind === "heavy" ? 350 : 200);
    state.score += pts;

    if (e.kind === "boss") {
      state.bossSlain = true;
      state.bossActive = false;
      apiState.secretUnlocked = true;
      saveLocal();
      refreshSecretUnlockUi();
      sfx("victory");
      spawnParticles(e.x, e.y - 60, 46, "#ffe24a", 360, 280);
      if (!state.endless) {
        setTimeout(function () {
          if (state.phase === "playing") finishRun(true);
        }, 1150);
      } else {
        addPopup(state.hero.x, 220, "BOSS DEFEATED! ENDLESS CONTINUES!", "#ffe24a", 24);
      }
      return;
    }

    state.kos++;
    // Defeated Scissor-Agents drop stolen Mustaches or Healing Croissants!
    if (state.hero.hp <= 4 && state.kos % 4 === 0) {
      state.collectibles.push({ kind: "croissant", x: e.x, y: e.y, z: 18, bobOff: 0 });
    } else {
      state.collectibles.push({ kind: "mustache", x: e.x, y: e.y, z: 20, bobOff: 0 });
    }

    if (state.kos === 7 && !state.armReloadShownOnce) {
      setTimeout(function () {
        if (state.phase === "playing") triggerArmReload(true);
      }, 280);
    }

    if (state.kos === 6 || state.kos === 12 || state.kos === 19) {
      state.goTimer = 2.8;
      sfx("go");
    }

    if (!state.bossActive && !state.bossSlain && !state.endless && state.kos >= GOAL_KOS) {
      summonKingpinBoss();
    }
  }

  function summonKingpinBoss() {
    state.bossActive = true;
    sfx("boss_rage");
    addPopup(state.camX + W * 0.5, 210, "FINAL BOSS · CYBER-BARON VEX!", "#ff5f2e", 26);
    var bx = Math.min(WORLD_W - 120, state.camX + W - 110);
    spawnEnemy("boss", bx, 414);
  }

  // =========================================================================
  // 4-SCENE ILLUSTRATED 16-BIT STORY CUTSCENE (Matches v1_02..v1_07.png!)
  // =========================================================================
  function beginIntroCinematic() {
    state.phase = "intro";
    state.introTimer = 0;
    state.thunderPlayed = false;
    setVisible("btn-skip-intro", true);
    setVisible("arcade-hud", false);
  }

  function finishIntro() {
    if (state.phase !== "intro") return;
    state.phase = "playing";
    setVisible("btn-skip-intro", false);
    setVisible("arcade-hud", true);
    var tc = document.getElementById("touch-controls");
    if (tc) tc.classList.add("active");
    sfx("go");
    var stg = STAGE_DEFS[state.stageIdx] || STAGE_DEFS[0];
    addPopup(state.hero.x + 110, 250, stg.title + " — FIGHT!", "#ffe24a", 27);
  }

  function updateIntro(dt) {
    state.introTimer += dt;
    if (state.introTimer >= 4.6 && !state.thunderPlayed) {
      state.thunderPlayed = true;
      state.shake = 18;
      sfx("thunder");
    }
    state.shake = Math.max(0, state.shake - dt * 20);
    if (state.introTimer >= 8.4) {
      finishIntro();
    }
  }

  // =========================================================================
  // GAMEPAD POLLING & 60 FPS GAME UPDATE LOOP
  // =========================================================================
  var gpPrev = {};
  var gpAxisX = 0;
  var gpAxisY = 0;
  function pollGamepad() {
    gpAxisX = 0;
    gpAxisY = 0;
    try {
      var pads = navigator.getGamepads ? navigator.getGamepads() : null;
      if (!pads) return;
      for (var i = 0; i < pads.length; i++) {
        var gp = pads[i];
        if (!gp || !gp.connected) continue;
        if (gp.axes && gp.axes.length >= 2) {
          if (Math.abs(gp.axes[0]) > 0.25) gpAxisX = gp.axes[0];
          if (Math.abs(gp.axes[1]) > 0.25) gpAxisY = gp.axes[1];
        }
        var btns = gp.buttons || [];
        var pressed = function (idx) { return Boolean(btns[idx] && (btns[idx].pressed || btns[idx].value > 0.5)); };
        if (pressed(14)) gpAxisX = -1;
        if (pressed(15)) gpAxisX = 1;
        if (pressed(12)) gpAxisY = -1;
        if (pressed(13)) gpAxisY = 1;

        var edge = function (idx) {
          var cur = pressed(idx);
          var was = Boolean(gpPrev[idx]);
          gpPrev[idx] = cur;
          return cur && !was;
        };
        if (edge(9)) togglePause();
        if (state.phase === "playing") {
          if (edge(0)) heroJump();
          if (edge(2) || edge(1)) heroPunch();
          if (edge(3)) heroKick();
          if (edge(4) || edge(5)) triggerArmReload(false);
        } else if (state.phase === "arm_reload" && (edge(0) || edge(2))) {
          dismissArmReload();
        } else if (state.phase === "title" && (edge(0) || edge(9))) {
          startRun(state.stageIdx, false);
        }
        break;
      }
    } catch (_) {}
  }

  function update(dt) {
    state.time += dt;
    pollGamepad();

    if (state.phase === "title") {
      state.camX = 0; // Frame the iconic Barbershop + Yellow + Blue + Red houses just like v1_01.png!
      return;
    }

    if (state.phase === "intro") {
      updateIntro(dt);
      return;
    }

    if (state.phase !== "playing") return;

    if (state.hitstop > 0) {
      state.hitstop = Math.max(0, state.hitstop - dt);
      state.shake = Math.max(0, state.shake - dt * 18);
      return;
    }

    state.shake = Math.max(0, state.shake - dt * 24);
    if (state.zoomTimer > 0) {
      state.zoomTimer = Math.max(0, state.zoomTimer - dt);
      state.camZoom += (1.14 - state.camZoom) * Math.min(1, dt * 14);
    } else {
      state.camZoom += (1.0 - state.camZoom) * Math.min(1, dt * 9);
    }

    if (state.goTimer > 0) state.goTimer = Math.max(0, state.goTimer - dt);
    if (state.comboTimer > 0) {
      state.comboTimer -= dt;
      if (state.comboTimer <= 0) state.combo = 0;
    }

    var h = state.hero;
    var perk = SKIN_PERKS[state.skinIdx] || SKIN_PERKS[0];
    h.oCd = Math.max(0, h.oCd - dt);
    h.kCd = Math.max(0, h.kCd - dt);
    h.comboWindow = Math.max(0, h.comboWindow - dt);
    h.invuln = Math.max(0, h.invuln - dt);
    h.slashArcTimer = Math.max(0, h.slashArcTimer - dt);

    for (var ai = h.afterimages.length - 1; ai >= 0; ai--) {
      h.afterimages[ai].life -= dt;
      if (h.afterimages[ai].life <= 0) h.afterimages.splice(ai, 1);
    }

    if (h.armReloadTimer > 0) {
      h.armReloadTimer = Math.max(0, h.armReloadTimer - dt);
      if (Math.random() < 0.48) {
        spawnParticles(h.x + (Math.random() - 0.5) * 34, h.y - h.z - 45, 1, "#ffe24a", 75, -80);
      }
    }
    if (h.attackTimer > 0) {
      h.attackTimer = Math.max(0, h.attackTimer - dt);
      if (h.attackTimer <= 0) h.attackType = "none";
    }

    var mx = gpAxisX;
    var my = gpAxisY;
    if (keys.ArrowLeft || keys.KeyA || touchInput.left) mx = -1;
    if (keys.ArrowRight || keys.KeyD || touchInput.right) mx = 1;
    if (keys.ArrowUp || keys.KeyW) my = -1;
    if (keys.ArrowDown || keys.KeyS) my = 1;

    var spd = 285 * perk.speedMul * (h.armReloadTimer > 0 ? 1.2 : 1.0);
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

    var prevZ = h.z;
    h.z += h.vz * dt;
    h.vz -= 1360 * dt;

    var floorZ = currentSupportZ(h, prevZ);
    if (h.z <= floorZ) {
      h.z = floorZ;
      h.vz = 0;
      h.jumpsUsed = 0;
    }

    var targetCam = Math.max(0, Math.min(WORLD_W - W, h.x - W * 0.42));
    state.camX += (targetCam - state.camX) * Math.min(1, dt * 6.8);

    // Collect Floating Mustaches, Rooftop Blue Energy Canisters & Healing Croissants!
    for (var ci = state.collectibles.length - 1; ci >= 0; ci--) {
      var col = state.collectibles[ci];
      if (Math.abs(col.x - h.x) < 46 && Math.abs((col.z || 0) - (h.z + 26)) < 54) {
        if (col.kind === "mustache") {
          state.mustaches++;
          state.score += 75;
          if (h.armReloadTimer <= 0) state.thermo = Math.min(100, state.thermo + 5 * perk.thermoMul);
          sfx("stache");
          spawnParticles(col.x, col.y - col.z, 6, "#ffe680", 120, 200);
        } else if (col.kind === "canister") {
          state.score += 300;
          state.thermo = Math.min(100, state.thermo + 38 * perk.thermoMul);
          sfx("canister");
          spawnParticles(col.x, col.y - col.z, 16, "#38e8ff", 220, 180);
          addPopup(col.x, col.y - col.z - 28, "+ENERGY!", "#38e8ff", 20);
        } else if (col.kind === "croissant") {
          h.hp = Math.min(h.maxHp, h.hp + 2);
          state.score += 200;
          sfx("heal");
          spawnParticles(col.x, col.y - col.z, 14, "#ffe24a", 180, 200);
          // Exact word-for-word yellow popup from v2_04.png!
          addPopup(h.x, h.y - h.z - 54, "Restores 1 heart", "#facc15", 22);
        }
        state.collectibles.splice(ci, 1);
      }
    }

    // Spawn Scissor-Claw Suit Agents
    var aliveGoons = state.enemies.filter(function (e) { return !e.dying && e.kind !== "boss"; }).length;
    var needMore = state.endless || (!state.bossActive && !state.bossSlain && state.kos + aliveGoons < GOAL_KOS);
    if (needMore && aliveGoons < 4) {
      state.spawnTimer -= dt;
      if (state.spawnTimer <= 0) {
        state.spawnTimer = state.endless ? 1.3 : 1.65;
        var fromRight = Math.random() < 0.75;
        var sx = fromRight ? Math.min(WORLD_W - 40, state.camX + W + 45) : Math.max(40, state.camX - 45);
        var sy = LANE_MIN_Y + Math.random() * (LANE_MAX_Y - LANE_MIN_Y);
        var roll = Math.random();
        var ek = roll < 0.48 ? "scissor_brown" : (roll < 0.84 ? "scissor_olive" : "heavy");
        spawnEnemy(ek, sx, sy);
      }
    }

    for (var pIdx = 0; pIdx < state.props.length; pIdx++) {
      state.props[pIdx].hitFlash = Math.max(0, state.props[pIdx].hitFlash - dt);
    }

    // Update Scissor-Claw Agents & Boss
    for (var ei = state.enemies.length - 1; ei >= 0; ei--) {
      var e = state.enemies[ei];
      e.hitFlash = Math.max(0, e.hitFlash - dt);
      e.counterWindow = Math.max(0, e.counterWindow - dt);
      if (e.trailHp > e.hp) e.trailHp = Math.max(e.hp, e.trailHp - dt * 4.5);

      if (e.dying) {
        e.dieTimer -= dt;
        e.x += e.vx * dt;
        e.y += e.vy * dt;
        e.z += e.vz * dt;
        e.vz -= 1050 * dt;
        if (e.splashKO && e.y >= WATER_Y + 6) {
          sfx("splash");
          spawnParticles(e.x, WATER_Y + 6, 22, "#ffffff", 270, 480);
          spawnParticles(e.x, WATER_Y + 6, 14, "#38bdf8", 210, 460);
          addPopup(e.x, WATER_Y - 20, "SPLASH!", "#38e8ff", 22);
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

      if (e.kind === "boss" && e.telegraphTimer > 0) {
        e.telegraphTimer -= dt;
        if (e.telegraphTimer <= 0) {
          state.shake = 14;
          sfx("heavy");
          spawnParticles(e.x, e.y, 26, "#ff5f2e", 300, 320);
          addPopup(e.x, e.y - 134, "SEISMIC SLAM!", "#ff5f2e", 24);
          if (Math.abs(h.x - e.x) < 190 && h.z < 38) {
            hurtHero(1, e.x > h.x ? -1 : 1);
          } else {
            e.counterWindow = 1.6;
            addPopup(e.x, e.y - 160, "COUNTER NOW!", "#38e8ff", 20);
          }
        }
        continue;
      }

      var dx = h.x - e.x;
      var dy = h.y - e.y;
      e.facing = dx >= 0 ? 1 : -1;
      var dist = Math.hypot(dx, dy);

      var moveSpd = e.kind === "boss" ? (e.phase2 ? 156 : 130) : (e.kind === "scissor_olive" ? 136 : 118);
      var stopDist = e.kind === "boss" ? 90 : 66;

      if (Math.abs(dx) > stopDist) {
        e.x += (dx > 0 ? 1 : -1) * moveSpd * dt;
        e.walkPhase += dt * 8.5;
      }
      if (Math.abs(dy) > 10) {
        e.y += (dy > 0 ? 1 : -1) * 78 * dt;
      }
      e.x = Math.max(36, Math.min(WORLD_W - 36, e.x));
      e.y = Math.max(LANE_MIN_Y, Math.min(LANE_MAX_Y, e.y));

      e.attackCd -= dt;
      if (e.attackCd <= 0) {
        if (e.kind === "scissor_olive" && dist > 140 && dist < 430) {
          e.attackCd = 2.25;
          state.projectiles.push({
            owner: "enemy",
            x: e.x + e.facing * 28,
            y: e.y - 48,
            vx: e.facing * 295,
            vy: 0,
            rot: 0,
            life: 2.5
          });
          sfx("slash");
        } else if (e.kind === "boss") {
          e.attackCd = e.phase2 ? 1.45 : 1.9;
          if (dist < 175) {
            e.telegraphTimer = e.phase2 ? 0.62 : 0.78;
            addPopup(e.x, e.y - 138, "JUMP! SLAM INCOMING!", "#ffe24a", 18);
          } else {
            for (var v = -1; v <= 1; v++) {
              state.projectiles.push({
                owner: "enemy",
                x: e.x + e.facing * 32,
                y: e.y - 54 + v * 16,
                vx: e.facing * (e.phase2 ? 355 : 315),
                vy: v * 26,
                rot: 0,
                life: 2.5
              });
            }
            sfx("slash");
          }
        } else if (dist < 78 && h.z < 36) {
          e.attackCd = 1.28;
          sfx("slash");
          hurtHero(1, e.facing);
        }
      }
    }

    // Update Projectiles
    for (var qi = state.projectiles.length - 1; qi >= 0; qi--) {
      var q = state.projectiles[qi];
      q.life -= dt;
      q.x += q.vx * dt;
      q.y += q.vy * dt;
      q.rot += dt * 15;
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
          if (Math.abs(q.x - te.x) < 46 && Math.abs(q.y - (te.y - te.z - 48)) < 58) {
            damageEnemy(te, 4.2, q.vx >= 0 ? 1 : -1, true, true);
            state.projectiles.splice(qi, 1);
            break;
          }
        }
      }
    }

    updateParticles(dt);
  }

  function hurtHero(amount, knockDir) {
    var h = state.hero;
    if (h.invuln > 0 || h.armReloadTimer > 0) return;
    h.hp = Math.max(0, h.hp - amount);
    h.invuln = 1.05;
    h.vx = knockDir * 235;
    state.combo = 0;
    state.shake = 10;
    sfx("hurt");
    spawnParticles(h.x, h.y - h.z - 46, 12, "#ef4444", 200, 320);
    if (h.hp <= 0) {
      sfx("gameover");
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
    for (var d = state.debris.length - 1; d >= 0; d--) {
      var db = state.debris[d];
      db.life -= dt;
      db.x += db.vx * dt;
      db.y += db.vy * dt;
      db.vy += 680 * dt;
      db.rot += dt * 15;
      if (db.life <= 0) state.debris.splice(d, 1);
    }
    for (var j = state.popups.length - 1; j >= 0; j--) {
      var pop = state.popups[j];
      pop.life -= dt;
      pop.y -= 36 * dt;
      if (pop.life <= 0) state.popups.splice(j, 1);
    }
  }

  // =========================================================================
  // 100% 16-BIT PIXEL-ART RENDERER (PANORAMAS, CUTSCENES, ROLLING SURF, SPRITES)
  // =========================================================================
  function drawAnimatedPixelSurf(camX) {
    var bgA = sprites.bgStreetA;
    if (!imgReady(bgA)) return;
    // Slice the frothy pixel-art ocean surf (y = 472..540) into 4 undulating wave bands!
    var bands = [
      { sy: 472, sh: 16, spd: 26, amp: 5 },
      { sy: 488, sh: 17, spd: -38, amp: 7 },
      { sy: 505, sh: 17, spd: 46, amp: 8 },
      { sy: 522, sh: 18, spd: -56, amp: 9 }
    ];
    for (var b = 0; b < bands.length; b++) {
      var bd = bands[b];
      var drift = Math.round(camX + state.time * bd.spd + Math.sin(state.time * 2.4 + b * 1.3) * bd.amp);
      var offX = ((drift % W) + W) % W;
      ctx.drawImage(bgA, 0, bd.sy, W, bd.sh, -offX, bd.sy, W, bd.sh);
      ctx.drawImage(bgA, 0, bd.sy, W, bd.sh, W - offX, bd.sy, W, bd.sh);
    }
  }

  function drawHarborPanorama(camX) {
    var bgA = sprites.bgStreetA;
    var bgB = sprites.bgStreetB;
    var startTile = Math.floor(camX / W);
    for (var t = startTile; t <= startTile + 1; t++) {
      if (t < 0 || t > 3) continue;
      var img = (t % 2 === 0) ? bgA : bgB;
      var dx = Math.round(t * W - camX);
      if (imgReady(img)) {
        ctx.drawImage(img, dx, 0, W, H);
      } else {
        ctx.fillStyle = "#1a58b8";
        ctx.fillRect(dx, 0, W, H);
      }
    }
    drawAnimatedPixelSurf(camX);
  }

  function drawTitleMonument() {
    // Stone "WELCOME TO HARBOR ISLAND" Monument on left quay matching v1_01.png!
    var mx = 16;
    var my = 268;
    ctx.save();
    ctx.fillStyle = "#5c606b";
    ctx.strokeStyle = "#0e111a";
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.arc(mx + 74, my + 28, 52, Math.PI, 0);
    ctx.fill();
    ctx.stroke();
    ctx.fillRect(mx, my + 26, 148, 128);
    ctx.strokeRect(mx, my + 26, 148, 128);
    ctx.fillStyle = "#351b1e";
    ctx.fillRect(mx + 12, my + 36, 124, 96);
    ctx.strokeStyle = "#8a7866";
    ctx.lineWidth = 2;
    ctx.strokeRect(mx + 12, my + 36, 124, 96);
    // Golden crests & pixel text
    ctx.fillStyle = "#d99b26";
    ctx.font = "900 10px 'Trebuchet MS', sans-serif";
    ctx.textAlign = "center";
    ctx.fillText("WELCOME TO", mx + 74, my + 78);
    ctx.font = "900 11.5px 'Trebuchet MS', sans-serif";
    ctx.fillStyle = "#ffe680";
    ctx.fillText("HARBOR ISLAND", mx + 74, my + 95);
    ctx.font = "bold 8px 'Trebuchet MS', sans-serif";
    ctx.fillStyle = "#cbd5e1";
    ctx.fillText("Home of the Golden Blade", mx + 74, my + 112);
    // Flowerbed at base
    ctx.fillStyle = "#14532d";
    ctx.fillRect(mx - 4, my + 136, 156, 22);
    var flCols = ["#ef4444", "#facc15", "#3b82f6", "#fb7185"];
    for (var f = 0; f < 14; f++) {
      ctx.fillStyle = flCols[f % flCols.length];
      ctx.fillRect(mx + 4 + f * 10, my + 140 + (f % 2) * 5, 5, 5);
    }
    ctx.restore();
  }

  function renderIntroCutscene() {
    var t = state.introTimer;
    ctx.save();
    if (state.shake > 0.2) {
      ctx.translate((Math.random() - 0.5) * state.shake, (Math.random() - 0.5) * state.shake);
    }

    if (t < 2.5) {
      // SCENE 1: Barbershop Interior Watching TV "BREAKING NEWS" (v1_03.png!)
      if (imgReady(sprites.cutBarberTv)) {
        ctx.drawImage(sprites.cutBarberTv, 0, 0, W, H);
      }
      // Subtle red alert pulse on the TV screen
      var pulse = 0.12 + Math.abs(Math.sin(t * 9)) * 0.12;
      ctx.fillStyle = "rgba(239, 68, 68, " + pulse.toFixed(2) + ")";
      ctx.fillRect(560, 56, 178, 132);
      drawCutsceneBubble(480, 486, "BREAKING NEWS: CYBER-AGENTS ARE STEALING EVERY MUSTACHE!");
    } else if (t < 4.6) {
      // SCENE 2: Shattered Barbershop Exterior & Floating Golden Comb-Sword (v1_05.png!)
      if (imgReady(sprites.cutShattered)) {
        ctx.drawImage(sprites.cutShattered, 0, 0, W, H);
      }
      // Twinkling pixel sparkles around the Golden Comb-Sword
      for (var s = 0; s < 6; s++) {
        var sx = 540 + Math.cos(t * 4 + s * 1.1) * 85;
        var sy = 240 + Math.sin(t * 5 + s * 1.4) * 65;
        ctx.fillStyle = s % 2 === 0 ? "#ffffff" : "#ffe24a";
        ctx.fillRect(Math.round(sx), Math.round(sy), 6, 6);
      }
      drawCutsceneBubble(480, 486, "AY! HE ZAPPED MY BARBERSHOP! SUMMON THE GOLDEN COMB-BLADE!");
    } else if (t < 6.3) {
      // SCENE 3: Giant Blazing Golden Lightning Transformation Pillar! (v1_06.png!)
      if (imgReady(sprites.cutShattered)) {
        ctx.drawImage(sprites.cutShattered, 0, 0, W, H);
      }
      ctx.fillStyle = "rgba(10, 8, 24, 0.45)";
      ctx.fillRect(0, 0, W, H);
      var bx = 548;
      var bw = 116 + Math.sin(t * 45) * 18;
      var beamGrad = ctx.createLinearGradient(bx - bw, 0, bx + bw, 0);
      beamGrad.addColorStop(0, "rgba(255, 160, 20, 0)");
      beamGrad.addColorStop(0.25, "rgba(255, 196, 36, 0.85)");
      beamGrad.addColorStop(0.5, "#ffffff");
      beamGrad.addColorStop(0.75, "rgba(255, 196, 36, 0.85)");
      beamGrad.addColorStop(1, "rgba(255, 160, 20, 0)");
      ctx.fillStyle = beamGrad;
      ctx.fillRect(bx - bw, 0, bw * 2, 470);
      // Ground impact burst & square pixel sparks
      ctx.fillStyle = "#fff7ae";
      ctx.fillRect(bx - 120, 448, 240, 22);
      for (var p = 0; p < 22; p++) {
        var px = bx + Math.sin(p * 17.3 + t * 8) * 170;
        var py = 450 - ((t * 320 + p * 43) % 420);
        ctx.fillStyle = p % 2 === 0 ? "#ffe24a" : "#ffffff";
        ctx.fillRect(Math.round(px), Math.round(py), 8, 8);
      }
    } else {
      // SCENE 4: Heroic Sunset Cliff Stage Card "LEVEL 1 · HARBOR ISLAND" + LOADING % (Exact v1_07.png!)
      if (imgReady(sprites.cutCliff)) {
        ctx.drawImage(sprites.cutCliff, 0, 0, W, H);
      }
      var heroIm = sprites.heroCombSlash;
      if (imgReady(heroIm)) {
        var hh = 216;
        var hw = Math.round(hh * (heroIm.naturalWidth / heroIm.naturalHeight));
        ctx.drawImage(heroIm, Math.round(W * 0.5 - hw * 0.40), 104, hw, hh);
      }
      var stg = STAGE_DEFS[state.stageIdx] || STAGE_DEFS[0];
      ctx.font = "900 22px 'Trebuchet MS', sans-serif";
      ctx.textAlign = "center";
      ctx.fillStyle = "#06281e";
      ctx.fillText(stg.title, W * 0.5 + 2, 446);
      ctx.fillStyle = "#34f5a6";
      ctx.fillText(stg.title, W * 0.5, 444);

      // Bottom-Right White Mustache + Progress Bar + LOADING 0% -> 100% (v1_07.png!)
      var loadPct = Math.min(100, Math.floor(((t - 6.3) / 1.95) * 100));
      var rx = W - 130;
      if (imgReady(sprites.pickupMustache)) {
        ctx.save();
        ctx.filter = "brightness(5) grayscale(1)";
        ctx.drawImage(sprites.pickupMustache, rx - 18, 464, 36, 19);
        ctx.restore();
      }
      ctx.fillStyle = "rgba(0, 0, 0, 0.65)";
      ctx.fillRect(rx - 68, 492, 136, 6);
      ctx.fillStyle = "#ffe24a";
      ctx.fillRect(rx - 68, 492, Math.round(1.36 * loadPct), 6);
      ctx.font = "900 11px 'Trebuchet MS', sans-serif";
      ctx.fillStyle = "#cbd5e1";
      ctx.fillText("LOADING " + loadPct + "%", rx, 514);
    }

    ctx.restore();
  }

  function drawCutsceneBubble(cx, cy, text) {
    ctx.save();
    ctx.font = "900 15px 'Trebuchet MS', sans-serif";
    var tw = Math.min(860, ctx.measureText(text).width + 38);
    var bx = Math.round(cx - tw * 0.5);
    var by = Math.round(cy - 24);
    ctx.fillStyle = "rgba(255, 252, 240, 0.96)";
    ctx.strokeStyle = "#0f172a";
    ctx.lineWidth = 4;
    ctx.beginPath();
    ctx.roundRect(bx, by, tw, 40, 8);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = "#0f172a";
    ctx.textAlign = "center";
    ctx.fillText(text, cx, by + 26);
    ctx.restore();
  }

  function render() {
    if (!ctx) return;
    ctx.imageSmoothingEnabled = false;

    if (state.phase === "intro") {
      renderIntroCutscene();
      return;
    }

    ctx.save();

    if (state.camZoom > 1.002) {
      var focusX = Math.max(120, Math.min(W - 120, state.hero.x - state.camX));
      var focusY = Math.max(140, Math.min(H - 120, state.hero.y - state.hero.z - 56));
      ctx.translate(focusX, focusY);
      ctx.scale(state.camZoom, state.camZoom);
      ctx.translate(-focusX, -focusY);
    }

    if (state.shake > 0.2) {
      ctx.translate((Math.random() - 0.5) * state.shake, (Math.random() - 0.5) * state.shake);
    }

    var camX = Math.round(state.camX);
    drawHarborPanorama(camX);

    if (state.phase === "title") {
      drawTitleMonument();
      // Twilight vignette tint matching v1_01.png!
      ctx.fillStyle = "rgba(8, 14, 34, 0.46)";
      ctx.fillRect(0, 0, W, H);
      ctx.restore();
      return;
    }

    // Dynamic Stage Atmospheric Tint (Sunset / Syndicate Night)
    var isNight = state.bossActive || state.stageIdx === 2;
    var isSunset = !isNight && (state.stageIdx === 1 || (!state.endless && state.kos >= 12));
    if (isSunset) {
      ctx.fillStyle = "rgba(245, 130, 32, 0.16)";
      ctx.fillRect(0, 0, W, WATER_Y);
    } else if (isNight) {
      ctx.fillStyle = "rgba(10, 18, 54, 0.36)";
      ctx.fillRect(0, 0, W, H);
    }

    ctx.save();
    ctx.translate(-camX, 0);

    // Draw Collectible Floating Mustaches, Rooftop Energy Canisters & Croissants
    for (var ci = 0; ci < state.collectibles.length; ci++) {
      var col = state.collectibles[ci];
      if (col.x < camX - 80 || col.x > camX + W + 80) continue;
      drawCollectible(col);
    }

    // Depth-sorted Street Props, Hero & Scissor-Claw Agents
    var drawList = [];
    for (var pri = 0; pri < state.props.length; pri++) {
      var pr = state.props[pri];
      if (pr.x + pr.w < camX - 80 || pr.x > camX + W + 80) continue;
      drawList.push({ y: pr.y, fn: drawPropFn(pr) });
    }
    drawList.push({ y: state.hero.y + 1, fn: drawHero });
    for (var ei = 0; ei < state.enemies.length; ei++) {
      drawList.push({ y: state.enemies[ei].y, fn: drawEnemyFn(state.enemies[ei]) });
    }
    drawList.sort(function (a, b) { return a.y - b.y; });
    for (var di = 0; di < drawList.length; di++) drawList[di].fn();

    // Spinning Projectiles
    for (var qi = 0; qi < state.projectiles.length; qi++) {
      drawProjectile(state.projectiles[qi]);
    }

    // Popped-off Purple Goggles Debris
    for (var dbi = 0; dbi < state.debris.length; dbi++) {
      var db = state.debris[dbi];
      ctx.save();
      ctx.translate(Math.round(db.x), Math.round(db.y));
      ctx.rotate(db.rot);
      ctx.fillStyle = "#18181b";
      ctx.fillRect(-10, -4, 20, 4);
      ctx.fillStyle = "#c084fc";
      ctx.fillRect(-9, -3, 7, 5);
      ctx.fillRect(2, -3, 7, 5);
      ctx.restore();
    }

    // Square 16-Bit Pixel Sparks
    for (var pai = 0; pai < state.particles.length; pai++) {
      var pa = state.particles[pai];
      ctx.fillStyle = pa.color;
      ctx.fillRect(Math.round(pa.x), Math.round(pa.y), Math.round(pa.size), Math.round(pa.size));
    }

    // Crisp Pixel-Outlined Popups (including "Restores 1 heart"!)
    for (var poi = 0; poi < state.popups.length; poi++) {
      var pop = state.popups[poi];
      ctx.save();
      ctx.globalAlpha = Math.min(1, pop.life / 0.35);
      ctx.font = "900 " + pop.size + "px 'Trebuchet MS', Impact, sans-serif";
      ctx.textAlign = "center";
      ctx.lineWidth = 5;
      ctx.strokeStyle = "#090b12";
      ctx.strokeText(pop.text, Math.round(pop.x), Math.round(pop.y));
      ctx.fillStyle = pop.color;
      ctx.fillText(pop.text, Math.round(pop.x), Math.round(pop.y));
      ctx.restore();
    }

    ctx.restore();

    if (state.combo >= 4 && state.phase === "playing") {
      ctx.save();
      ctx.font = "900 26px Impact, 'Trebuchet MS', sans-serif";
      ctx.textAlign = "right";
      var label = state.combo + " HIT COMBO!";
      ctx.lineWidth = 5;
      ctx.strokeStyle = "#090d16";
      ctx.strokeText(label, W - 22, 118);
      ctx.fillStyle = state.combo >= 12 ? "#ffe24a" : "#ffffff";
      ctx.fillText(label, W - 22, 118);
      ctx.restore();
    }

    if (state.phase === "playing" && state.hero.armReloadTimer > 0) {
      var gPulse = 0.24 + Math.sin(state.time * 12) * 0.12;
      ctx.strokeStyle = "rgba(255, 226, 74, " + gPulse.toFixed(2) + ")";
      ctx.lineWidth = 10;
      ctx.strokeRect(5, 5, W - 10, H - 10);
    }

    ctx.restore();
  }

  function drawCollectible(col) {
    var bob = Math.sin(state.time * 5.2 + (col.bobOff || 0)) * 4;
    var cx = Math.round(col.x);
    var cy = Math.round(col.y - (col.z || 0) + bob);
    ctx.save();
    if (col.kind === "mustache") {
      // Subtle warm golden aura ring like v2_04.png!
      ctx.strokeStyle = "rgba(255, 196, 56, 0.36)";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.ellipse(cx, cy, 24, 11, 0, 0, Math.PI * 2);
      ctx.stroke();
      if (imgReady(sprites.pickupMustache)) {
        ctx.drawImage(sprites.pickupMustache, cx - 24, cy - 13, 48, 25);
      }
    } else if (col.kind === "canister") {
      if (imgReady(sprites.pickupCanister)) {
        ctx.drawImage(sprites.pickupCanister, cx - 16, cy - 36, 32, 42);
      }
    } else if (col.kind === "croissant") {
      if (imgReady(sprites.pickupCroissant)) {
        ctx.drawImage(sprites.pickupCroissant, cx - 20, cy - 18, 40, 31);
      }
    }
    ctx.restore();
  }

  function drawPropFn(pr) {
    return function () {
      ctx.save();
      if (pr.hitFlash > 0) ctx.translate((Math.random() - 0.5) * 6, 0);
      if (pr.hp <= 0) ctx.filter = "brightness(0.55) contrast(1.2)";
      var img = pr.kind === "booth" ? sprites.propBooth : sprites.propCar;
      if (imgReady(img)) {
        ctx.drawImage(img, Math.round(pr.x), Math.round(pr.y - pr.h), pr.w, pr.h);
      }
      ctx.filter = "none";
      ctx.restore();
    };
  }

  function drawHero() {
    var h = state.hero;
    var perk = SKIN_PERKS[state.skinIdx] || SKIN_PERKS[0];

    for (var ai = 0; ai < h.afterimages.length; ai++) {
      var af = h.afterimages[ai];
      var aImg = sprites.heroCombSlash;
      if (imgReady(aImg)) {
        ctx.save();
        ctx.globalAlpha = af.life * 1.6;
        ctx.translate(Math.round(af.x), Math.round(af.y - af.z));
        ctx.scale(af.facing, 1);
        ctx.filter = "brightness(1.8) hue-rotate(155deg)";
        var aw = Math.round(118 * (aImg.naturalWidth / aImg.naturalHeight));
        ctx.drawImage(aImg, -Math.round(aw * 0.35), -118, aw, 118);
        ctx.restore();
      }
    }

    // Ground shadow
    ctx.fillStyle = "rgba(0, 0, 0, 0.36)";
    var shScale = Math.max(0.4, 1 - h.z / 280);
    ctx.beginPath();
    ctx.ellipse(h.x, h.y + 3, 28 * shScale, 8 * shScale, 0, 0, Math.PI * 2);
    ctx.fill();

    if (state.phase === "playing" && h.invuln > 0 && h.armReloadTimer <= 0 && Math.floor(state.time * 18) % 2 === 0) return;

    var img = sprites.heroGoldIdle;
    var drawH = 114;
    var anchorX = 0.48;
    if (h.attackType === "comb_slash" || h.attackType === "whirlwind") {
      img = sprites.heroCombSlash;
      drawH = 118;
      anchorX = 0.34;
    } else if (h.attackType === "super_punch") {
      img = sprites.heroGoldIdle;
      drawH = 120;
    }

    var bob = h.z <= 1 ? Math.abs(Math.sin(h.walkPhase)) * -5 : 0;
    ctx.save();
    ctx.translate(Math.round(h.x), Math.round(h.y - h.z + bob));

    if (h.armReloadTimer > 0) {
      var aura = ctx.createRadialGradient(0, -56, 12, 0, -56, 76);
      aura.addColorStop(0, "rgba(255, 226, 74, 0.78)");
      aura.addColorStop(0.55, "rgba(255, 95, 46, 0.38)");
      aura.addColorStop(1, "rgba(255, 95, 46, 0)");
      ctx.fillStyle = aura;
      ctx.beginPath();
      ctx.arc(0, -56, 76, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.scale(h.facing, 1);
    if (perk.filter && perk.filter !== "none") {
      ctx.filter = perk.filter;
    }

    if (imgReady(img)) {
      var drawW = Math.round(drawH * (img.naturalWidth / img.naturalHeight));
      ctx.drawImage(img, -Math.round(drawW * anchorX), -drawH, drawW, drawH);
    }
    ctx.filter = "none";

    // Gleaming Golden Comb-Sword Slash Arc!
    if (h.slashArcTimer > 0) {
      ctx.strokeStyle = h.attackType === "whirlwind" ? "#38e8ff" : "#ffe652";
      ctx.lineWidth = 6;
      ctx.beginPath();
      ctx.arc(28, -56, 56, -0.85, 0.95);
      ctx.stroke();
    }

    ctx.restore();
  }

  function drawEnemyFn(e) {
    return function () {
      if (e.kind === "boss" && e.telegraphTimer > 0) {
        ctx.save();
        var tPulse = 0.25 + Math.sin(state.time * 28) * 0.15;
        ctx.fillStyle = "rgba(239, 68, 68, " + tPulse.toFixed(2) + ")";
        ctx.strokeStyle = "#ffe24a";
        ctx.lineWidth = 3;
        ctx.beginPath();
        ctx.ellipse(e.x, e.y, 185, 40, 0, 0, Math.PI * 2);
        ctx.fill();
        ctx.stroke();
        ctx.restore();
      }

      ctx.fillStyle = "rgba(0, 0, 0, 0.35)";
      ctx.beginPath();
      ctx.ellipse(e.x, e.y + 3, e.kind === "boss" ? 38 : 24, 8, 0, 0, Math.PI * 2);
      ctx.fill();

      var img = e.kind === "boss"
        ? sprites.bossKingpin
        : (e.kind === "heavy" ? sprites.enemySuit : sprites.enemyScissor);
      var drawH = e.kind === "boss" ? 148 : (e.kind === "heavy" ? 114 : 106);
      var bob = e.dying ? 0 : Math.abs(Math.sin(e.walkPhase)) * -5;

      ctx.save();
      ctx.translate(Math.round(e.x), Math.round(e.y - e.z + bob));
      if (e.dying) {
        ctx.rotate((state.time * 12) * (e.vx >= 0 ? 1 : -1));
      }
      // Note: enemyScissor faces RIGHT in its raw sprite; enemySuit & bossKingpin face LEFT
      var rawFacesRight = (img === sprites.enemyScissor);
      var scaleX = rawFacesRight ? (e.facing >= 0 ? 1 : -1) : (e.facing > 0 ? -1 : 1);
      ctx.scale(scaleX, 1);

      if (e.hitFlash > 0) {
        ctx.filter = "brightness(2.4)";
      } else if (e.kind === "boss" && e.phase2) {
        ctx.filter = "saturate(1.6) drop-shadow(0 0 8px #c084fc)";
      } else if (e.kind === "scissor_olive") {
        // Olive-green tailored suit variant matching v2_04.png!
        ctx.filter = "hue-rotate(64deg) saturate(0.95)";
      }

      if (imgReady(img)) {
        var drawW = Math.round(drawH * (img.naturalWidth / img.naturalHeight));
        ctx.drawImage(img, -Math.round(drawW * 0.46), -drawH, drawW, drawH);
      }
      ctx.filter = "none";
      ctx.restore();

      if (!e.dying && e.stunTimer > 0.1) {
        for (var st = 0; st < 3; st++) {
          var sa = state.time * 7 + (st * Math.PI * 2) / 3;
          var sx = e.x + Math.cos(sa) * 22;
          var sy = e.y - e.z - drawH - 6 + Math.sin(sa) * 5;
          ctx.fillStyle = "#ffe24a";
          ctx.fillRect(Math.round(sx) - 3, Math.round(sy) - 3, 6, 6);
        }
      }
    };
  }

  function drawProjectile(q) {
    ctx.save();
    ctx.translate(Math.round(q.x), Math.round(q.y));
    ctx.rotate(q.rot);
    if (imgReady(sprites.pickupMustache)) {
      if (q.owner === "hero") ctx.filter = "hue-rotate(155deg) brightness(1.5)";
      ctx.drawImage(sprites.pickupMustache, -22, -11, 44, 22);
      ctx.filter = "none";
    }
    ctx.restore();
  }

  // =========================================================================
  // ORNATE CONNECTED BRASS & RUBY MERCURY THERMOMETER (Exact v2_01/v2_04!)
  // =========================================================================
  function drawThermometer() {
    if (!thermoCtx) return;
    var c = thermoCtx;
    var tw = thermoCanvas.width;
    var th = thermoCanvas.height;
    c.clearRect(0, 0, tw, th);

    var pct = state.hero.armReloadTimer > 0
      ? Math.max(0.08, state.hero.armReloadTimer / 8.0)
      : Math.max(0, Math.min(1, state.thermo / 100));

    // 1. Top Burgundy-Gold Circular Medallion (v2_01/v2_04!)
    c.fillStyle = "#7a142c";
    c.strokeStyle = "#e6a825";
    c.lineWidth = 3.5;
    c.beginPath();
    c.arc(30, 24, 20, 0, Math.PI * 2);
    c.fill();
    c.stroke();
    c.strokeStyle = "#260710";
    c.lineWidth = 1.5;
    c.beginPath();
    c.arc(30, 24, 22, 0, Math.PI * 2);
    c.stroke();
    if (pct >= 0.99 || state.hero.armReloadTimer > 0) {
      c.fillStyle = "#ffe24a";
      c.beginPath();
      c.arc(30, 24, 11, 0, Math.PI * 2);
      c.fill();
    }

    // 2. Ornate Crimson & Brass Vertical Thermometer Body
    c.fillStyle = "#8c1630";
    c.strokeStyle = "#e6a825";
    c.lineWidth = 3;
    c.beginPath();
    c.roundRect(14, 46, 32, 146, 14);
    c.fill();
    c.stroke();

    // Inner gold bezel & white segmented glass tube (matching v2_04.png!)
    c.fillStyle = "#d99b26";
    c.fillRect(19, 54, 22, 130);
    c.fillStyle = "#e8f1fc";
    c.fillRect(22, 58, 16, 122);

    // Rising Sky-Blue / Blazing Gold Energy Fill (v2_04.png shows sky-blue energy in the glass tube!)
    var fillH = Math.round(120 * pct);
    var grad = c.createLinearGradient(0, 180, 0, 58);
    if (pct >= 0.99 || state.hero.armReloadTimer > 0) {
      grad.addColorStop(0, "#ff5f2e");
      grad.addColorStop(1, "#ffe24a");
    } else {
      grad.addColorStop(0, "#1d74d8");
      grad.addColorStop(1, "#54c8ff");
    }
    c.fillStyle = grad;
    c.fillRect(23, 179 - fillH, 14, fillH);

    // Horizontal dark pixel tick marks + center vertical glass highlight line
    c.strokeStyle = "#334155";
    c.lineWidth = 1.5;
    for (var ty = 70; ty <= 170; ty += 14) {
      c.beginPath();
      c.moveTo(22, ty);
      c.lineTo(38, ty);
      c.stroke();
    }
    c.strokeStyle = "rgba(30, 41, 59, 0.55)";
    c.beginPath();
    c.moveTo(30, 58);
    c.lineTo(30, 179);
    c.stroke();

    // 3. Ornate Winged Brass & Ruby Bottom Bulb (v2_01/v2_04!)
    c.fillStyle = "#e6a825";
    c.strokeStyle = "#260710";
    c.lineWidth = 2;
    c.beginPath();
    c.moveTo(8, 194);
    c.lineTo(30, 224);
    c.lineTo(52, 194);
    c.closePath();
    c.fill();
    c.stroke();

    c.fillStyle = "#9e1836";
    c.strokeStyle = "#e6a825";
    c.lineWidth = 3;
    c.beginPath();
    c.arc(30, 196, 16, 0, Math.PI * 2);
    c.fill();
    c.stroke();
    c.fillStyle = pct >= 0.99 ? "#ffe24a" : "#38bdf8";
    c.beginPath();
    c.arc(30, 196, 8, 0, Math.PI * 2);
    c.fill();
  }

  function setVisible(id, show) {
    var el = document.getElementById(id);
    if (!el) return;
    if (show) el.classList.remove("is-hidden");
    else el.classList.add("is-hidden");
  }

  function updateSkillBadge(btnId, ringId, cdTextId, glyphId, remSec, maxSec) {
    var btn = document.getElementById(btnId);
    var ring = document.getElementById(ringId);
    var cdTxt = document.getElementById(cdTextId);
    var glyph = document.getElementById(glyphId);
    if (remSec > 0.05) {
      var frac = Math.max(0, Math.min(1, remSec / maxSec));
      if (ring) ring.setAttribute("stroke-dashoffset", String(Math.round(119 * (1 - frac))));
      if (btn) btn.classList.add("on-cd");
      if (cdTxt) {
        cdTxt.textContent = Math.ceil(remSec) + "s";
        cdTxt.classList.remove("is-hidden");
      }
      if (glyph) glyph.classList.add("is-hidden");
    } else {
      if (ring) ring.setAttribute("stroke-dashoffset", "119");
      if (btn) btn.classList.remove("on-cd");
      if (cdTxt) cdTxt.classList.add("is-hidden");
      if (glyph) glyph.classList.remove("is-hidden");
    }
  }

  function syncHud() {
    if (state.phase !== "playing" && state.phase !== "paused" && state.phase !== "arm_reload") return;
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

    updateSkillBadge("btn-skill-o", "ring-o", "cd-text-o", "glyph-o", 0, 1);
    updateSkillBadge("btn-skill-k", "ring-k", "cd-text-k", "glyph-k", h.kCd, 5.0);

    var rL = document.getElementById("ring-l");
    var cdL = document.getElementById("cd-text-l");
    var glL = document.getElementById("glyph-l");
    var btnL = document.getElementById("btn-skill-l");
    var readyTag = document.getElementById("thermo-ready-tag");
    var ultReady = state.thermo >= 100 || h.armReloadTimer > 0;

    if (h.armReloadTimer > 0) {
      if (rL) rL.setAttribute("stroke-dashoffset", String(Math.round(119 * (1 - h.armReloadTimer / 8.0))));
      if (cdL) { cdL.textContent = Math.ceil(h.armReloadTimer) + "s"; cdL.classList.remove("is-hidden"); }
      if (glL) glL.classList.add("is-hidden");
    } else {
      if (rL) rL.setAttribute("stroke-dashoffset", String(Math.round(119 * (1 - state.thermo / 100))));
      if (cdL) cdL.classList.add("is-hidden");
      if (glL) glL.classList.remove("is-hidden");
    }

    if (btnL) {
      if (ultReady) btnL.classList.add("ready-glow");
      else btnL.classList.remove("ready-glow");
    }
    if (readyTag) {
      if (state.thermo >= 100 && h.armReloadTimer <= 0) readyTag.classList.remove("is-hidden");
      else readyTag.classList.add("is-hidden");
    }

    var mEl = document.getElementById("hud-mustache-counter");
    if (mEl) mEl.textContent = String(state.mustaches).padStart(3, "0");

    var scEl = document.getElementById("hud-score-pill");
    if (scEl) {
      var kPart = state.endless ? (state.kos + " KOs") : ("KOs " + String(state.kos).padStart(2, "0") + "/" + GOAL_KOS);
      scEl.textContent = kPart + " · " + String(state.score).padStart(5, "0") + " PTS";
    }

    setVisible("go-banner", state.goTimer > 0 && state.phase === "playing");

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
      var bn = document.getElementById("boss-hud-name");
      if (bn) bn.textContent = boss.phase2 ? "CYBER-BARON VEX · OVERDRIVE!" : "CYBER-BARON VEX";
      var pctEl = document.getElementById("boss-hud-pct");
      if (pctEl) pctEl.textContent = Math.ceil(bpct * 100) + "%";
      var fillEl = document.getElementById("boss-hp-fill");
      if (fillEl) fillEl.setAttribute("width", String(Math.round(238 * bpct)));
      var trailEl = document.getElementById("boss-hp-trail");
      if (trailEl) trailEl.setAttribute("width", String(Math.round(238 * tpct)));
    }
  }

  // =========================================================================
  // CAMPAIGN FLOW, MODALS & REDDIT SCORE SUBMISSION
  // =========================================================================
  function startRun(stageIdx, skipIntroFlag) {
    ensureAudio();
    startMusic();
    resetWorld(stageIdx);
    setVisible("arcade-title-screen", false);
    setVisible("arcade-level-modal", false);
    setVisible("arcade-pause-modal", false);
    setVisible("arcade-end-modal", false);
    setVisible("arm-reload-screen", false);

    if (skipIntroFlag) {
      state.phase = "intro";
      finishIntro();
      if (stageIdx === 2) summonKingpinBoss();
    } else {
      beginIntroCinematic();
    }

    fetch("/api/start-run", { method: "POST" })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (d) { if (d && d.runId) apiState.runId = String(d.runId); })
      .catch(function () {});
  }

  function returnToTitle() {
    state.phase = "title";
    state.camX = 0;
    var tc = document.getElementById("touch-controls");
    if (tc) tc.classList.remove("active");
    setVisible("arcade-hud", false);
    setVisible("arcade-pause-modal", false);
    setVisible("arcade-end-modal", false);
    setVisible("arcade-level-modal", false);
    setVisible("btn-skip-intro", false);
    setVisible("arcade-title-screen", true);
  }

  function computeRank(score, victory) {
    if (victory && score >= 12000) return "SSS";
    if (victory && score >= 8500) return "SS";
    if (score >= 6000) return "S";
    if (score >= 3800) return "A";
    if (score >= 2000) return "B";
    return "C";
  }

  function buildMedals() {
    var m = [];
    if (state.mustaches >= 15) m.push("MUSTACHE GUARDIAN (" + state.mustaches + ")");
    if (state.reloadsUsed >= 1) m.push("ARM RELOAD MASTER");
    if (state.parries >= 1) m.push("SCISSOR PARRY ACE");
    if (state.maxCombo >= 12) m.push("COMBO x" + state.maxCombo);
    if (state.bossSlain) m.push("HARBOR SAVIOR");
    if (m.length === 0) m.push("PROTOCOL AGENT");
    return m;
  }

  function finishRun(victory) {
    state.phase = "ended";
    state.victory = victory;
    var tc = document.getElementById("touch-controls");
    if (tc) tc.classList.remove("active");
    setVisible("arcade-end-modal", true);

    if (state.score > apiState.bestScore) apiState.bestScore = state.score;
    if (state.score >= 5000 || victory) apiState.secretUnlocked = true;
    saveLocal();
    refreshSecretUnlockUi();

    var rank = computeRank(state.score, victory);
    var tag = document.getElementById("end-outcome-tag");
    if (tag) tag.textContent = victory ? "PROTOCOL COMPLETE! HARBOR SAVED" : "MISSION FAILED!";
    var sc = document.getElementById("end-score-val");
    if (sc) sc.textContent = state.score.toLocaleString("en-US") + " PTS";
    var rk = document.getElementById("end-rank-val");
    if (rk) rk.textContent = rank;
    var sum = document.getElementById("end-summary-box");
    if (sum) {
      sum.textContent = victory
        ? "Recovered " + state.mustaches + " stolen mustaches, defeated " + state.kos + " scissor-agents, and crushed Cyber-Baron Vex!"
        : "Recovered " + state.mustaches + " mustaches and defeated " + state.kos + " scissor-agents on Harbor Island (" + state.score + " PTS).";
    }

    var mStrip = document.getElementById("end-medals-strip");
    if (mStrip) {
      mStrip.innerHTML = buildMedals().map(function (lbl) {
        return '<span class="medal-chip">' + lbl + '</span>';
      }).join("");
    }

    var lbBox = document.getElementById("end-lb-list");
    if (lbBox) {
      var rows = (apiState.leaderboard && apiState.leaderboard.length > 0)
        ? apiState.leaderboard.slice(0, 3)
        : [{ username: apiState.username || "You", score: state.score }];
      lbBox.innerHTML = rows.map(function (r, idx) {
        var u = String(r.username || "Player").replace(/[<>&"']/g, "");
        return '<div class="end-lb-row"><span>#' + (idx + 1) + ' u/' + u + '</span><strong>' + Number(r.score || 0).toLocaleString("en-US") + ' PTS</strong></div>';
      }).join("");
    }

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
          level: state.stageIdx + 1,
          heroClass: state.skinIdx % 3,
          buildSummary: "MUSTACHE PROTOCOL · " + state.mustaches + " STACHES",
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
      if (ps) ps.textContent = "Mustaches: " + String(state.mustaches).padStart(3, "0") + " · KOs: " + state.kos + "/" + GOAL_KOS + " · Score: " + state.score;
    } else if (state.phase === "paused") {
      state.phase = "playing";
      setVisible("arcade-pause-modal", false);
    } else if (state.phase === "title") {
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
  // INPUT & UI EVENT LISTENERS
  // =========================================================================
  window.addEventListener("keydown", function (ev) {
    keys[ev.code] = true;
    if (state.phase === "intro" && (ev.code === "Space" || ev.code === "Enter" || ev.code === "Escape")) {
      ev.preventDefault();
      finishIntro();
      return;
    }
    if (state.phase === "arm_reload" && (ev.code === "Space" || ev.code === "Enter" || ev.code === "KeyO" || ev.code === "KeyL")) {
      ev.preventDefault();
      dismissArmReload();
      return;
    }
    if (state.phase === "title" && (ev.code === "Enter" || ev.code === "Space")) {
      ev.preventDefault();
      startRun(state.stageIdx, false);
      return;
    }
    if (ev.code === "Escape" || ev.code === "KeyP") {
      togglePause();
      return;
    }
    if (state.phase !== "playing") return;
    if (ev.code === "Space" || ev.code === "KeyW" || ev.code === "ArrowUp") {
      if (ev.code === "Space") { ev.preventDefault(); heroJump(); }
    }
    if (ev.code === "KeyO" || ev.code === "KeyJ") heroPunch();
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

  var btnSkipIntro = document.getElementById("btn-skip-intro");
  if (btnSkipIntro) btnSkipIntro.addEventListener("click", finishIntro);

  var armScreen = document.getElementById("arm-reload-screen");
  if (armScreen) armScreen.addEventListener("click", dismissArmReload);
  var armBtn = document.getElementById("btn-arm-continue");
  if (armBtn) armBtn.addEventListener("click", function (e) { e.stopPropagation(); dismissArmReload(); });

  var btnStart = document.getElementById("btn-press-start");
  if (btnStart) btnStart.addEventListener("click", function () { startRun(state.stageIdx, false); });

  var btnOpenLevels = document.getElementById("btn-open-levels");
  if (btnOpenLevels) btnOpenLevels.addEventListener("click", function () { setVisible("arcade-level-modal", true); });
  var btnCloseLevels = document.getElementById("btn-close-levels");
  if (btnCloseLevels) btnCloseLevels.addEventListener("click", function () { setVisible("arcade-level-modal", false); });
  var btnLaunchSelected = document.getElementById("btn-launch-selected");
  if (btnLaunchSelected) btnLaunchSelected.addEventListener("click", function () { startRun(state.stageIdx, false); });

  var btnTitleGear = document.getElementById("btn-title-settings");
  if (btnTitleGear) btnTitleGear.addEventListener("click", function () { setVisible("arcade-pause-modal", true); });

  var levelBtns = document.querySelectorAll("#level-selector .level-card-btn");
  levelBtns.forEach(function (btn) {
    btn.addEventListener("click", function () {
      var idx = Number(btn.getAttribute("data-stage")) || 0;
      levelBtns.forEach(function (b) { b.classList.remove("selected"); });
      btn.classList.add("selected");
      state.stageIdx = idx;
      var lbl = document.getElementById("selected-level-label");
      if (lbl) lbl.textContent = idx === 3 ? "ENDLESS MODE" : ("LEVEL " + (idx + 1) + " SELECTED");
    });
  });

  var skinBtns = document.querySelectorAll("#skin-selector .skin-btn");
  skinBtns.forEach(function (btn) {
    btn.addEventListener("click", function () {
      var idx = Number(btn.getAttribute("data-skin")) || 0;
      if (idx === 3 && !apiState.secretUnlocked) return;
      skinBtns.forEach(function (b) { b.classList.remove("selected"); });
      btn.classList.add("selected");
      state.skinIdx = idx;
    });
  });

  var btnSkillO = document.getElementById("btn-skill-o");
  if (btnSkillO) btnSkillO.addEventListener("click", heroPunch);
  var btnSkillK = document.getElementById("btn-skill-k");
  if (btnSkillK) btnSkillK.addEventListener("click", heroKick);
  var btnSkillL = document.getElementById("btn-skill-l");
  if (btnSkillL) btnSkillL.addEventListener("click", function () { triggerArmReload(false); });

  var btnPause = document.getElementById("btn-pause");
  if (btnPause) btnPause.addEventListener("click", togglePause);
  var btnResumeTop = document.getElementById("btn-pause-resume-top");
  if (btnResumeTop) btnResumeTop.addEventListener("click", function () {
    if (state.phase === "title") setVisible("arcade-pause-modal", false);
    else togglePause();
  });
  var btnResumeBot = document.getElementById("btn-pause-resume");
  if (btnResumeBot) btnResumeBot.addEventListener("click", function () {
    if (state.phase === "title") setVisible("arcade-pause-modal", false);
    else togglePause();
  });
  var btnRestart = document.getElementById("btn-pause-restart");
  if (btnRestart) btnRestart.addEventListener("click", returnToTitle);

  var btnAudio = document.getElementById("btn-toggle-audio");
  if (btnAudio) btnAudio.addEventListener("click", function () {
    isMuted = !isMuted;
    applyAudioVisual();
    saveLocal();
  });
  var slMusic = document.getElementById("slider-music");
  if (slMusic) {
    slMusic.value = String(Math.round(musicVol * 100));
    slMusic.addEventListener("input", function () {
      musicVol = Math.max(0, Math.min(1, Number(slMusic.value) / 100));
      saveLocal();
    });
  }
  var slSfx = document.getElementById("slider-sfx");
  if (slSfx) {
    slSfx.value = String(Math.round(sfxVol * 100));
    slSfx.addEventListener("input", function () {
      sfxVol = Math.max(0, Math.min(1, Number(slSfx.value) / 100));
      saveLocal();
    });
  }
  var btnCrt = document.getElementById("btn-toggle-crt");
  if (btnCrt) btnCrt.addEventListener("click", function () {
    crtEnabled = !crtEnabled;
    applyCrtVisual();
    saveLocal();
  });

  var btnEndReplay = document.getElementById("btn-end-replay");
  if (btnEndReplay) btnEndReplay.addEventListener("click", function () { startRun(state.stageIdx, true); });
  var btnEndEndless = document.getElementById("btn-end-endless");
  if (btnEndEndless) btnEndEndless.addEventListener("click", function () { startRun(3, true); });
  var btnEndShare = document.getElementById("btn-end-share");
  if (btnEndShare) btnEndShare.addEventListener("click", function () {
    var st = document.getElementById("end-status-msg");
    if (st) st.textContent = "Posting Mustache Protocol chronicle to Reddit...";
    fetch("/api/share-comment", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        score: state.score,
        kills: state.kos,
        bossSlain: state.bossSlain,
        level: state.stageIdx + 1,
        heroClass: state.skinIdx % 3,
        streak: apiState.streak,
        buildEmojis: "⚔️〰️🥊",
        rank: computeRank(state.score, state.victory),
        chronicle: "MUSTACHE PROTOCOL · " + state.mustaches + " Mustaches · " + state.kos + " KOs · " + buildMedals().join(" · ")
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

  // Headless QA & Gameplay Verification Hooks
  window.__ARCADE_DEBUG__ = {
    getState: function () { return state; },
    startCampaign: function (skipIntro) { startRun(0, Boolean(skipIntro)); },
    setIntroTime: function (t) { state.phase = "intro"; state.introTimer = t; },
    finishIntro: finishIntro,
    punch: heroPunch,
    kick: heroKick,
    jump: heroJump,
    showArmReload: function () { triggerArmReload(true); },
    dismissArmReload: dismissArmReload,
    summonBoss: summonKingpinBoss,
    finishVictory: function () { finishRun(true); }
  };

  applyCrtVisual();
  applyAudioVisual();
  resetWorld(0);
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

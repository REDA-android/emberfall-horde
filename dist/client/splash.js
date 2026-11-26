(function () {
  "use strict";

  var MODIFIERS = [
    { title: "BLOOD MOON", desc: "+25% Crit Chance · Relentless Swarms" },
    { title: "ARCANE SURGE", desc: "+20% Attack Speed · Double XP Gems" },
    { title: "SWARM RUSH", desc: "Massive Hordes · +30% Score Multiplier" },
    { title: "TITAN'S WRATH", desc: "Enraged Overlord · +50% Boss Bounty" }
  ];

  var BIOMES = [
    {
      key: "obsidian",
      eyebrow: "THE OBSIDIAN SANCTUARY",
      short: "OBSIDIAN",
      cls: "biome-obsidian",
      themeColor: "#140D24",
      boss: "MALAKOR, VOID LICH",
      bossTip: "Phase II Eclipse · Dual Void Slams & Soul Obelisk Shields",
      runeRgb: "157,78,221",
      emberRgb: "229,184,92"
    },
    {
      key: "volcanic",
      eyebrow: "THE VOLCANIC RIFT",
      short: "VOLCANIC",
      cls: "biome-volcanic",
      themeColor: "#240A07",
      boss: "IGNIS REX, MAGMA TITAN",
      bossTip: "3-Ring Meteor Barrage · Magma Geysers · Shatter Runic Pillars",
      runeRgb: "255,77,45",
      emberRgb: "255,183,3"
    },
    {
      key: "frost",
      eyebrow: "THE FROST ABYSS",
      short: "FROST",
      cls: "biome-frost",
      themeColor: "#071D30",
      boss: "SKADI, GLACIAL EMPRESS",
      bossTip: "14-Way Cryo Shatter Ring · Blizzard Squalls · Witch-Time Dodge",
      runeRgb: "56,232,255",
      emberRgb: "186,230,253"
    }
  ];

  var BLESSINGS = [
    "BLESSING: SOLAR AEGIS (+25 HP)",
    "BLESSING: ARCANE OVERCHARGE (+18% SPD)",
    "BLESSING: SHADOW VELOCITY (+15% CRIT)"
  ];

  var BOUNTY_POOL = [
    { kind: "WitchTimeDodges", label: "2 Witch-Time Dodges" },
    { kind: "ReachLevel", label: "Reach Level 6" },
    { kind: "SlayDailyBoss", label: "Slay Today's Boss" },
    { kind: "DetonateBarrels", label: "Detonate 3 Barrels" },
    { kind: "ClaimShrines", label: "Claim 1 Blood Shrine" },
    { kind: "ReachCombo", label: "Reach 30x Carnage" },
    { kind: "OpenChests", label: "Open 2 Runic Chests" },
    { kind: "SlayHorde", label: "Slay 75 Foes" }
  ];

  var activeBiomeIdx = 0;

  function utcSeed(offsetDays) {
    var d = new Date(Date.now() + (offsetDays || 0) * 86400000);
    var y = d.getUTCFullYear();
    var m = String(d.getUTCMonth() + 1).padStart(2, "0");
    var day = String(d.getUTCDate()).padStart(2, "0");
    return y + "-" + m + "-" + day;
  }

  function createRng(seed) {
    try {
      var mask = 0xffffffffffffffffn;
      var state = 0xcbf29ce484222325n;
      for (var i = 0; i < seed.length; i++) {
        state ^= BigInt(seed.charCodeAt(i) & 0xff);
        state = (state * 0x100000001b3n) & mask;
      }
      if (state === 0n) state = 0x9e3779b97f4a7c15n;
      return function () {
        var x = state;
        x ^= x >> 12n;
        x ^= (x << 25n) & mask;
        x ^= x >> 27n;
        state = x;
        return (x * 0x2545f4914f6cdd1dn) & mask;
      };
    } catch (_) {
      var s = 1337;
      return function () {
        s = (s * 1664525 + 1013904223) >>> 0;
        return BigInt(s);
      };
    }
  }

  function hashSeed(seed) {
    try { return createRng(seed)(); } catch (_) { return 0n; }
  }

  function defaultMutator(seed) {
    try { return Number(hashSeed(seed) % 4n); } catch (_) { return 0; }
  }

  function defaultBiomeIdx(seed) {
    try { return Number(hashSeed(seed + "-biome") % 3n); } catch (_) { return 0; }
  }

  function defaultBounties(seed) {
    try {
      var next = createRng("bounties:" + seed);
      var pool = BOUNTY_POOL.slice();
      for (var i = pool.length - 1; i >= 1; i--) {
        var j = Number(next() % BigInt(i + 1));
        var tmp = pool[i];
        pool[i] = pool[j];
        pool[j] = tmp;
      }
      return [pool[0], pool[1], pool[2]];
    } catch (_) {
      return BOUNTY_POOL.slice(0, 3);
    }
  }

  function esc(s) {
    return String(s).replace(/[&<>"']/g, function (c) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c];
    });
  }

  // Strict Reddit Devvit Expanded Mode + Telemetry trigger
  function requestExpandedGame(event) {
    if (typeof globalThis.requestExpandedMode === "function") {
      try {
        globalThis.requestExpandedMode(event, "game");
        return;
      } catch (_) {}
    }
    var d = globalThis.devvit;
    if (d && d.entrypoints && d.entrypoints.game) {
      try {
        var url = new URL(d.entrypoints.game);
        if (url.pathname.indexOf("/api/") === 0 && d.token) {
          url.searchParams.set("devvit-token", d.token);
        }
        var clickPayload = { event: "click", definition: "strict", elemTrackId: "enter-btn" };
        parent.postMessage({
          type: "devvit-internal",
          scope: 0,
          telemetry: { event: "click", click: clickPayload },
          analytics: clickPayload
        }, "*");
        parent.postMessage({
          type: "devvit-internal",
          scope: 0,
          immersiveMode: {
            entryUrl: String(url),
            immersiveMode: 2
          }
        }, "*");
        return;
      } catch (_) {}
    }
    window.location.href = "./index.html";
  }

  var btn = document.getElementById("enter-btn");
  if (btn) btn.addEventListener("click", requestExpandedGame);

  var drawerPlayBtn = document.getElementById("drawer-play-btn");
  if (drawerPlayBtn) drawerPlayBtn.addEventListener("click", requestExpandedGame);

  var guideDrawer = document.getElementById("guide-drawer");
  var helpBtn = document.getElementById("help-btn");
  var closeGuideBtn = document.getElementById("close-guide-btn");
  function setDrawerOpen(open) {
    if (!guideDrawer) return;
    if (open) guideDrawer.classList.add("open");
    else guideDrawer.classList.remove("open");
  }
  if (helpBtn) helpBtn.addEventListener("click", function () { setDrawerOpen(true); });
  if (closeGuideBtn) closeGuideBtn.addEventListener("click", function () { setDrawerOpen(false); });

  function applyBiomeTheme(bIdx) {
    activeBiomeIdx = ((bIdx % 3) + 3) % 3;
    var b = BIOMES[activeBiomeIdx] || BIOMES[0];
    document.documentElement.setAttribute("data-biome", b.key);
    document.body.classList.remove("biome-obsidian", "biome-volcanic", "biome-frost");
    document.body.classList.add(b.cls);
    var meta = document.getElementById("meta-theme");
    if (meta) meta.setAttribute("content", b.themeColor);
    var eye = document.getElementById("biome-eyebrow");
    if (eye) eye.textContent = b.eyebrow;
  }

  function updateCountdown() {
    var now = new Date();
    var nextMidnight = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() + 1);
    var remSec = Math.max(0, Math.floor((nextMidnight - now.getTime()) / 1000));
    var hh = String(Math.floor(remSec / 3600)).padStart(2, "0");
    var mm = String(Math.floor((remSec % 3600) / 60)).padStart(2, "0");
    var ss = String(remSec % 60).padStart(2, "0");
    var el = document.getElementById("utc-countdown");
    if (el) el.textContent = hh + ":" + mm + ":" + ss;
  }

  function renderData(data) {
    var seed = (data && data.seed) ? String(data.seed) : utcSeed(0);
    var seedEl = document.getElementById("seed-pill");
    if (seedEl) seedEl.textContent = "DAY #" + seed;

    var bIdx = (data && typeof data.biomeIndex === "number") ? ((data.biomeIndex % 3 + 3) % 3) : defaultBiomeIdx(seed);
    applyBiomeTheme(bIdx);

    var biomeInfo = BIOMES[bIdx] || BIOMES[0];
    var bossName = (data && data.bossName) ? String(data.bossName) : biomeInfo.boss;
    var ov = (data && typeof data.mutatorOverride === "number") ? data.mutatorOverride : -1;
    var modIdx = (ov >= 0 && ov <= 3) ? ov : defaultMutator(seed);
    var mod = MODIFIERS[modIdx] || MODIFIERS[0];

    var mutBadge = document.getElementById("mutator-badge");
    if (mutBadge) mutBadge.textContent = mod.title + (ov >= 0 ? " ★ VOTED" : "");
    var bossHead = document.getElementById("boss-headline");
    if (bossHead) bossHead.textContent = bossName;
    var mutDescEl = document.getElementById("mutator-desc");
    if (mutDescEl) mutDescEl.textContent = biomeInfo.bossTip + " · " + mod.desc;

    var blessIdx = (data && typeof data.yesterdayWinningFaction === "number" && data.yesterdayWinningFaction >= 0)
      ? (data.yesterdayWinningFaction % 3)
      : bIdx;
    var blessEl = document.getElementById("blessing-line");
    if (blessEl) blessEl.textContent = BLESSINGS[blessIdx] || BLESSINGS[0];

    var bounties = (data && Array.isArray(data.dailyBounties) && data.dailyBounties.length === 3)
      ? data.dailyBounties
      : defaultBounties(seed);
    var chipsEl = document.getElementById("bounty-chips");
    if (chipsEl) {
      chipsEl.innerHTML = bounties.map(function (b) {
        return '<span class="b-chip">◆ ' + esc(b.label) + "</span>";
      }).join("");
    }

    var dTitle = document.getElementById("drawer-title");
    if (dTitle) dTitle.textContent = "SANCTUARY #" + seed + " · " + biomeInfo.short;
    var dBoss = document.getElementById("drawer-boss");
    if (dBoss) dBoss.textContent = bossName + " — " + biomeInfo.bossTip;
    var dMut = document.getElementById("drawer-mutator");
    if (dMut) dMut.textContent = "Active Mutator: " + mod.title + " (" + mod.desc + ")";
    var dBounties = document.getElementById("drawer-bounties");
    if (dBounties) {
      dBounties.innerHTML = bounties.map(function (b) {
        return "<li><strong>" + esc(b.label) + "</strong> (+750 PTS)</li>";
      }).join("");
    }

    var kills = (data && Number(data.bossKillsToday)) || 0;
    var target = (data && Number(data.siegeTarget)) || 25;
    var pct = Math.min(100, Math.round((kills / Math.max(1, target)) * 100));
    var siegeCountEl = document.getElementById("siege-count");
    if (siegeCountEl) siegeCountEl.textContent = kills + " / " + target + " SLAIN";
    var siegeFillEl = document.getElementById("siege-fill");
    if (siegeFillEl) siegeFillEl.style.width = pct + "%";

    var fs = (data && Array.isArray(data.factionScores)) ? data.factionScores : [0, 0, 0];
    var fTotal = (Number(fs[0]) || 0) + (Number(fs[1]) || 0) + (Number(fs[2]) || 0);
    var fp = (data && Array.isArray(data.factionPercents) && data.factionPercents.length >= 3)
      ? data.factionPercents
      : (fTotal > 0
        ? [
            Math.round((Number(fs[0] || 0) / fTotal) * 100),
            Math.round((Number(fs[1] || 0) / fTotal) * 100),
            Math.max(0, 100 - Math.round((Number(fs[0] || 0) / fTotal) * 100) - Math.round((Number(fs[1] || 0) / fTotal) * 100))
          ]
        : [34, 33, 33]);
    var seg0 = document.getElementById("f-seg-0");
    var seg1 = document.getElementById("f-seg-1");
    var seg2 = document.getElementById("f-seg-2");
    if (seg0) seg0.style.width = Math.max(6, fp[0]) + "%";
    if (seg1) seg1.style.width = Math.max(6, fp[1]) + "%";
    if (seg2) seg2.style.width = Math.max(6, fp[2]) + "%";

    var factionEl = document.getElementById("faction-line");
    if (factionEl) {
      factionEl.innerHTML =
        '<span class="faction-pill"><i class="f-dot solar"></i>SOLAR <strong>' + fp[0] + "%</strong></span>" +
        '<span class="faction-pill"><i class="f-dot arcane"></i>ARCANE <strong>' + fp[1] + "%</strong></span>" +
        '<span class="faction-pill"><i class="f-dot shadow"></i>SHADOW <strong>' + fp[2] + "%</strong></span>";
    }

    var feedEl = document.getElementById("siege-feed");
    if (feedEl) {
      var feed = (data && Array.isArray(data.siegeFeed)) ? data.siegeFeed : [];
      if (feed.length > 0) {
        var items = feed.slice(0, 3).map(function (entry) {
          return "<strong>u/" + esc(entry.username) + "</strong> " + (entry.slain ? "[SLAIN]" : "[STRIKE]");
        });
        feedEl.innerHTML = "LIVE SIEGE: " + items.join(" · ");
      } else {
        feedEl.textContent = "Strike " + bossName + " to appear in the live Subreddit Siege Feed!";
      }
    }

    var ydayEl = document.getElementById("yday-champ");
    if (ydayEl) {
      if (data && data.yesterdayChampion && data.yesterdayChampion.username) {
        ydayEl.textContent = "J-1 CHAMP: u/" + data.yesterdayChampion.username;
      } else {
        ydayEl.textContent = "90s Ranked";
      }
    }

    var lb = (data && Array.isArray(data.leaderboard)) ? data.leaderboard.slice(0, 3) : [];
    var lbEl = document.getElementById("lb-list");
    if (lbEl) {
      if (lb.length === 0) {
        lbEl.innerHTML = '<div class="lb-row"><span class="lb-user">No runs yet today — claim #1 &amp; set the Champion Ghost!</span></div>';
      } else {
        lbEl.innerHTML = lb.map(function (entry, idx) {
          return '<div class="lb-row">' +
            '<div class="lb-left"><span class="lb-pos">#' + (idx + 1) + '</span><span class="lb-user">u/' + esc(entry.username) + "</span></div>" +
            '<div class="lb-right"><span class="rank-chip">' + esc(entry.rank || "E") + '</span><span class="score-num">' + Number(entry.score || 0).toLocaleString("en-US") + "</span></div>" +
            "</div>";
        }).join("");
      }
    }

    var tomorrowSeed = (data && data.tomorrowSeed) ? String(data.tomorrowSeed) : utcSeed(1);
    var tMutIdx = (data && typeof data.tomorrowLeadingMutator === "number" && data.tomorrowLeadingMutator >= 0)
      ? (data.tomorrowLeadingMutator % 4)
      : defaultMutator(tomorrowSeed);
    var tInfoEl = document.getElementById("tomorrow-info");
    if (tInfoEl) {
      tInfoEl.textContent = "INSTANT 60 FPS WASM · NEXT VOTE: " + MODIFIERS[tMutIdx].title;
    }

    var meRankEl = document.getElementById("me-rank");
    var meStreakEl = document.getElementById("me-streak");
    if (data && data.username) {
      var rankTxt = data.myRank > 0
        ? "u/" + esc(data.username) + " · <strong>#" + data.myRank + "/" + data.totalPlayers + "</strong>"
        : "u/" + esc(data.username) + " · Ready for Ranked";
      if (meRankEl) meRankEl.innerHTML = rankTxt;
      if (meStreakEl) meStreakEl.textContent = (Number(data.streak) || 0) + "-Day Streak";
    } else {
      if (meRankEl) meRankEl.textContent = "Guest Mode · Sign in on Reddit for Ranked & Votes";
      var localStreak = 0;
      try {
        var rawP = localStorage.getItem("ef_profile_v1");
        if (rawP) localStreak = Number(JSON.parse(rawP).localStreak) || 0;
      } catch (_) {}
      if (meStreakEl) meStreakEl.textContent = localStreak + "-Day Streak";
    }
  }

  // Live 60 FPS Sanctuary Astrolabe + Rising Embers + Biome Lightning Canvas
  (function initSanctuaryCanvas() {
    var cvs = document.getElementById("splash-bg-fx");
    if (!cvs || !cvs.getContext) return;
    var ctx = cvs.getContext("2d");
    var motes = [];
    for (var i = 0; i < 48; i++) {
      motes.push({
        x: Math.random(),
        y: Math.random(),
        s: 1.0 + Math.random() * 2.4,
        vx: (Math.random() - 0.5) * 0.002,
        vy: -0.003 - Math.random() * 0.0055,
        p: Math.random() * 6.28
      });
    }
    var angle = 0;
    var flash = 0;
    function tick() {
      var w = window.innerWidth || 480;
      var h = window.innerHeight || 720;
      if (cvs.width !== w || cvs.height !== h) {
        cvs.width = w;
        cvs.height = h;
      }
      ctx.clearRect(0, 0, w, h);
      angle += 0.0038;

      var b = BIOMES[activeBiomeIdx] || BIOMES[0];
      var cx = w * 0.5;
      var cy = h * 0.34;
      var rad = Math.min(w, h) * 0.34;

      if (Math.random() < 0.008) flash = 0.16;
      if (flash > 0.005) {
        ctx.fillStyle = "rgba(" + b.runeRgb + "," + flash.toFixed(3) + ")";
        ctx.fillRect(0, 0, w, h);
        flash *= 0.88;
      }

      // Volumetric sanctuary core glow
      var core = ctx.createRadialGradient(cx, cy, 8, cx, cy, rad * 1.15);
      core.addColorStop(0, "rgba(" + b.runeRgb + ",0.18)");
      core.addColorStop(1, "rgba(9,10,15,0)");
      ctx.fillStyle = core;
      ctx.fillRect(0, 0, w, h);

      ctx.save();
      ctx.translate(cx, cy);
      ctx.rotate(angle);
      ctx.strokeStyle = "rgba(" + b.runeRgb + ",0.20)";
      ctx.lineWidth = 1.4;
      ctx.beginPath();
      ctx.arc(0, 0, rad, 0, Math.PI * 2);
      ctx.stroke();

      ctx.setLineDash([12, 10]);
      ctx.strokeStyle = "rgba(" + b.emberRgb + ",0.22)";
      ctx.beginPath();
      ctx.arc(0, 0, rad * 0.78, 0, Math.PI * 2);
      ctx.stroke();
      ctx.setLineDash([]);

      for (var k = 0; k < 8; k++) {
        var a0 = (k * Math.PI) / 4;
        ctx.beginPath();
        ctx.moveTo(Math.cos(a0) * rad * 0.78, Math.sin(a0) * rad * 0.78);
        ctx.lineTo(Math.cos(a0) * rad, Math.sin(a0) * rad);
        ctx.stroke();
      }
      ctx.restore();

      for (var j = 0; j < motes.length; j++) {
        var m = motes[j];
        m.p += 0.045;
        m.x += m.vx + Math.sin(m.p) * 0.0008;
        m.y += activeBiomeIdx === 2 ? -m.vy * 0.65 : m.vy;
        if (m.y < -0.05) { m.y = 1.05; m.x = Math.random(); }
        if (m.y > 1.05) { m.y = -0.05; m.x = Math.random(); }
        if (m.x < 0) m.x += 1;
        if (m.x > 1) m.x -= 1;
        var alpha = 0.25 + 0.52 * Math.abs(Math.sin(m.p));
        var rgb = m.s > 2.2 ? b.emberRgb : b.runeRgb;
        ctx.beginPath();
        ctx.fillStyle = "rgba(" + rgb + "," + alpha.toFixed(2) + ")";
        ctx.arc(m.x * w, m.y * h, m.s, 0, Math.PI * 2);
        ctx.fill();
      }
      requestAnimationFrame(tick);
    }
    requestAnimationFrame(tick);
  })();

  renderData(null);
  updateCountdown();
  setInterval(updateCountdown, 1000);

  fetch("/api/init", { cache: "no-store" })
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(function (json) { if (json) renderData(json); })
    .catch(function () {});
})();

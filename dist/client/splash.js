(function () {
  var MODIFIERS = [
    { title: 'BLOOD MOON', desc: '+25% Crit Chance · Faster Hordes' },
    { title: 'ARCANE SURGE', desc: '+20% Attack Speed · Double XP Gems' },
    { title: 'SWARM RUSH', desc: 'Massive Swarms · +30% Score Multiplier' },
    { title: "TITAN'S WRATH", desc: 'Enraged Daily Boss · +50% Boss Bounty' }
  ];
  var BIOMES = [
    'OBSIDIAN SANCTUARY',
    'VOLCANIC RIFT',
    'FROST ABYSS'
  ];
  var BIOME_BOSSES = [
    'MALAKOR, VOID LICH',
    'IGNIS REX, MAGMA TITAN',
    'SKADI, GLACIAL EMPRESS'
  ];
  var BLESSINGS = [
    '☀️ SOLAR AEGIS (+25 HP & +15% Area)',
    '🔮 ARCANE OVERCHARGE (+18% Atk Spd & +15% XP)',
    '🗡️ SHADOW VELOCITY (+15% Move Spd & +15% Crit)'
  ];

  function utcSeed() {
    var d = new Date();
    var y = d.getUTCFullYear();
    var m = String(d.getUTCMonth() + 1).padStart(2, '0');
    var day = String(d.getUTCDate()).padStart(2, '0');
    return y + '-' + m + '-' + day;
  }

  function hashSeed(seed) {
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
      x = (x * 0x2545f4914f6cdd1dn) & mask;
      return x;
    } catch (_) {
      return 0n;
    }
  }

  function defaultMutator(seed) {
    try { return Number(hashSeed(seed) % 4n); } catch (_) { return 0; }
  }

  function defaultBiomeIdx(seed) {
    try { return Number(hashSeed(seed + '-biome') % 3n); } catch (_) { return 0; }
  }

  function esc(s) {
    return String(s).replace(/[&<>"']/g, function (c) {
      return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c];
    });
  }

  function requestExpandedGame(event) {
    if (typeof globalThis.requestExpandedMode === 'function') {
      try {
        globalThis.requestExpandedMode(event, 'game');
        return;
      } catch (_) {}
    }
    var d = globalThis.devvit;
    if (d && d.entrypoints && d.entrypoints.game) {
      try {
        var url = new URL(d.entrypoints.game);
        if (url.pathname.indexOf('/api/') === 0 && d.token) {
          url.searchParams.set('devvit-token', d.token);
        }
        var clickPayload = { event: 'click', definition: 'strict', elemTrackId: 'enter-btn' };
        parent.postMessage({
          type: 'devvit-internal',
          scope: 0,
          telemetry: { event: 'click', click: clickPayload },
          analytics: clickPayload
        }, '*');
        parent.postMessage({
          type: 'devvit-internal',
          scope: 0,
          immersiveMode: {
            entryUrl: String(url),
            immersiveMode: 2
          }
        }, '*');
        return;
      } catch (_) {}
    }
    window.location.href = './index.html';
  }

  var btn = document.getElementById('enter-btn');
  if (btn) {
    btn.addEventListener('click', function (ev) {
      requestExpandedGame(ev);
    });
  }

  function renderData(data) {
    var seed = (data && data.seed) ? String(data.seed) : utcSeed();
    var seedEl = document.getElementById('seed-pill');
    if (seedEl) seedEl.textContent = 'DUNGEON #' + seed;

    var bIdx = (data && typeof data.biomeIndex === 'number') ? (data.biomeIndex % 3) : defaultBiomeIdx(seed);
    var biomeName = (data && data.biomeName) ? String(data.biomeName) : (BIOMES[bIdx] || BIOMES[0]);
    var bossName = BIOME_BOSSES[bIdx] || BIOME_BOSSES[0];
    var ov = (data && typeof data.mutatorOverride === 'number') ? data.mutatorOverride : -1;
    var modIdx = (ov >= 0 && ov <= 3) ? ov : defaultMutator(seed);
    var mod = MODIFIERS[modIdx] || MODIFIERS[0];
    var mutNameEl = document.getElementById('mutator-name');
    if (mutNameEl) mutNameEl.textContent = biomeName + ' · ' + mod.title + (ov >= 0 ? ' ★ VOTED' : '');
    var mutDescEl = document.getElementById('mutator-desc');
    if (mutDescEl) mutDescEl.textContent = '💀 Boss: ' + bossName + ' · ' + mod.desc;

    var blessIdx = (data && typeof data.yesterdayWinningFaction === 'number' && data.yesterdayWinningFaction >= 0)
      ? (data.yesterdayWinningFaction % 3)
      : bIdx;
    var bountyEl = document.getElementById('bounty-line');
    if (bountyEl) bountyEl.textContent = '✨ Blessing: ' + (BLESSINGS[blessIdx] || BLESSINGS[0]);

    var kills = (data && Number(data.bossKillsToday)) || 0;
    var target = (data && Number(data.siegeTarget)) || 25;
    var pct = Math.min(100, Math.round((kills / Math.max(1, target)) * 100));
    var siegeCountEl = document.getElementById('siege-count');
    if (siegeCountEl) siegeCountEl.textContent = kills + ' / ' + target + ' Bosses Slain';
    var siegeFillEl = document.getElementById('siege-fill');
    if (siegeFillEl) siegeFillEl.style.width = pct + '%';

    var fs = (data && Array.isArray(data.factionScores)) ? data.factionScores : [0, 0, 0];
    var fline = '☀️ Solar ' + Number(fs[0] || 0).toLocaleString('en-US') +
      ' · 🔮 Arcane ' + Number(fs[1] || 0).toLocaleString('en-US') +
      ' · 🗡️ Shadow ' + Number(fs[2] || 0).toLocaleString('en-US');
    if (data && data.yesterdayChampion && data.yesterdayChampion.username) {
      fline += ' · 🏆 J-1: u/' + data.yesterdayChampion.username;
    }
    var factionEl = document.getElementById('faction-line');
    if (factionEl) factionEl.textContent = fline;

    var lb = (data && Array.isArray(data.leaderboard)) ? data.leaderboard.slice(0, 3) : [];
    var lbEl = document.getElementById('lb-list');
    if (lbEl) {
      if (lb.length === 0) {
        lbEl.innerHTML = '<div class="lb-row"><span class="lb-user" style="color:#94a3b8;">No runs yet today — claim #1!</span></div>';
      } else {
        lbEl.innerHTML = lb.map(function (entry, idx) {
          var pos = idx === 0 ? '👑' : '#' + (idx + 1);
          return '<div class="lb-row">' +
            '<div class="lb-left"><span class="lb-pos">' + pos + '</span><span class="lb-user">u/' + esc(entry.username) + '</span></div>' +
            '<div class="lb-right"><span class="tier">' + esc(entry.rank || 'E') + '</span><span class="score">' + Number(entry.score || 0).toLocaleString('en-US') + '</span></div>' +
            '</div>';
        }).join('');
      }
    }

    if (data && data.username) {
      var rankTxt = data.myRank > 0
        ? 'u/' + esc(data.username) + ' · <strong>#' + data.myRank + '/' + data.totalPlayers + '</strong>'
        : 'u/' + esc(data.username) + ' · Unranked';
      var meRankEl = document.getElementById('me-rank');
      if (meRankEl) meRankEl.innerHTML = rankTxt;
      var meStreakEl = document.getElementById('me-streak');
      if (meStreakEl) meStreakEl.textContent = '🔥 ' + (Number(data.streak) || 0) + 'd Streak';
    }
  }

  renderData(null);
  fetch('/api/init', { cache: 'no-store' })
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(function (json) { if (json) renderData(json); })
    .catch(function () {});
})();

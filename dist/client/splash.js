(function () {
  var MODIFIERS = [
    { title: 'BLOOD MOON', desc: '+25% Crit Chance · Faster Hordes' },
    { title: 'ARCANE SURGE', desc: '+20% Attack Speed · Double XP Gems' },
    { title: 'SWARM RUSH', desc: 'Massive Swarms · +30% Score Multiplier' },
    { title: "TITAN'S WRATH", desc: 'Enraged Daily Boss · +50% Boss Bounty' }
  ];
  var BIOMES = [
    {
      name: 'OBSIDIAN SANCTUARY',
      short: 'OBSIDIAN',
      cls: 'biome-obsidian',
      themeColor: '#1a0d2e',
      sigilColor: '#c084fc',
      boss: 'MALAKOR, VOID LICH',
      bossTip: 'Dual Void Slams + Soul Obelisk Shields in Phase II Eclipse.'
    },
    {
      name: 'VOLCANIC RIFT',
      short: 'VOLCANIC',
      cls: 'biome-volcanic',
      themeColor: '#260b07',
      sigilColor: '#fb923c',
      boss: 'IGNIS REX, MAGMA TITAN',
      bossTip: '3-Ring Staggered Meteor Barrage + Magma Geyser Eruptions.'
    },
    {
      name: 'FROST ABYSS',
      short: 'FROST',
      cls: 'biome-frost',
      themeColor: '#071b2c',
      sigilColor: '#38bdf8',
      boss: 'SKADI, GLACIAL EMPRESS',
      bossTip: '14-Way Cryo Shatter Ring + Blizzard Squalls.'
    }
  ];
  var BLESSINGS = [
    '☀️ SOLAR AEGIS (+25 HP & +15% Area)',
    '🔮 ARCANE OVERCHARGE (+18% Atk Spd & +15% XP)',
    '🗡️ SHADOW VELOCITY (+15% Move Spd & +15% Crit)'
  ];
  var CLASS_EMOJI = ['☀️', '🔮', '🗡️'];
  var BOUNTY_POOL = [
    { kind: 'WitchTimeDodges', label: '2 Witch-Time Dodges' },
    { kind: 'ReachLevel', label: 'Reach Level 6' },
    { kind: 'SlayDailyBoss', label: "Slay Today's Boss" },
    { kind: 'DetonateBarrels', label: 'Detonate 3 Barrels' },
    { kind: 'ClaimShrines', label: 'Claim 1 Blood Shrine' },
    { kind: 'ReachCombo', label: 'Reach 30x Combo' },
    { kind: 'OpenChests', label: 'Open 2 Runic Chests' },
    { kind: 'SlayHorde', label: 'Slay 75 Enemies' }
  ];

  function utcSeed(offsetDays) {
    var d = new Date(Date.now() + (offsetDays || 0) * 86400000);
    var y = d.getUTCFullYear();
    var m = String(d.getUTCMonth() + 1).padStart(2, '0');
    var day = String(d.getUTCDate()).padStart(2, '0');
    return y + '-' + m + '-' + day;
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
    try { return Number(hashSeed(seed + '-biome') % 3n); } catch (_) { return 0; }
  }

  function defaultBounties(seed) {
    try {
      var next = createRng('bounties:' + seed);
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

  var drawerPlayBtn = document.getElementById('drawer-play-btn');
  if (drawerPlayBtn) {
    drawerPlayBtn.addEventListener('click', function (ev) {
      requestExpandedGame(ev);
    });
  }

  var guideDrawer = document.getElementById('guide-drawer');
  var helpBtn = document.getElementById('help-btn');
  var closeGuideBtn = document.getElementById('close-guide-btn');
  function setDrawerOpen(open) {
    if (!guideDrawer) return;
    if (open) guideDrawer.classList.add('open');
    else guideDrawer.classList.remove('open');
  }
  if (helpBtn) {
    helpBtn.addEventListener('click', function () { setDrawerOpen(true); });
  }
  if (closeGuideBtn) {
    closeGuideBtn.addEventListener('click', function () { setDrawerOpen(false); });
  }

  function applyBiomeTheme(bIdx) {
    var b = BIOMES[bIdx % 3] || BIOMES[0];
    document.body.classList.remove('biome-obsidian', 'biome-volcanic', 'biome-frost');
    document.body.classList.add(b.cls);
    var meta = document.getElementById('meta-theme');
    if (meta) meta.setAttribute('content', b.themeColor);
    var sigil = document.getElementById('sigil-path');
    if (sigil) sigil.setAttribute('fill', b.sigilColor);
    var tag = document.getElementById('biome-tag');
    if (tag) tag.textContent = b.short;
  }

  function updateCountdown() {
    var now = new Date();
    var nextMidnight = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() + 1);
    var remSec = Math.max(0, Math.floor((nextMidnight - now.getTime()) / 1000));
    var hh = String(Math.floor(remSec / 3600)).padStart(2, '0');
    var mm = String(Math.floor((remSec % 3600) / 60)).padStart(2, '0');
    var ss = String(remSec % 60).padStart(2, '0');
    var el = document.getElementById('utc-countdown');
    if (el) el.textContent = '⏳ ' + hh + ':' + mm + ':' + ss;
  }

  function renderData(data) {
    var seed = (data && data.seed) ? String(data.seed) : utcSeed(0);
    var seedEl = document.getElementById('seed-pill');
    if (seedEl) seedEl.textContent = '#' + seed;

    var bIdx = (data && typeof data.biomeIndex === 'number') ? ((data.biomeIndex % 3 + 3) % 3) : defaultBiomeIdx(seed);
    applyBiomeTheme(bIdx);

    var biomeInfo = BIOMES[bIdx] || BIOMES[0];
    var biomeName = (data && data.biomeName) ? String(data.biomeName) : biomeInfo.name;
    var bossName = (data && data.bossName) ? String(data.bossName) : biomeInfo.boss;
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
    var blessEl = document.getElementById('blessing-line');
    if (blessEl) blessEl.textContent = '✨ Blessing: ' + (BLESSINGS[blessIdx] || BLESSINGS[0]);

    var bounties = (data && Array.isArray(data.dailyBounties) && data.dailyBounties.length === 3)
      ? data.dailyBounties
      : defaultBounties(seed);
    var chipsEl = document.getElementById('bounty-chips');
    if (chipsEl) {
      chipsEl.innerHTML = bounties.map(function (b) {
        return '<span class="b-chip">🎯 ' + esc(b.label) + '</span>';
      }).join('');
    }

    // Update Guide Drawer contents
    var dTitle = document.getElementById('drawer-title');
    if (dTitle) dTitle.textContent = 'DUNGEON #' + seed + ' · ' + biomeInfo.short;
    var dBoss = document.getElementById('drawer-boss');
    if (dBoss) dBoss.textContent = bossName + ' — ' + biomeInfo.bossTip;
    var dMut = document.getElementById('drawer-mutator');
    if (dMut) dMut.textContent = 'Mutator: ' + mod.title + ' (' + mod.desc + ') · ' + (BLESSINGS[blessIdx] || BLESSINGS[0]);
    var dBounties = document.getElementById('drawer-bounties');
    if (dBounties) {
      dBounties.innerHTML = bounties.map(function (b) {
        return '<li><strong>' + esc(b.label) + '</strong> (+750 PTS)</li>';
      }).join('');
    }

    // Boss Siege Bar
    var kills = (data && Number(data.bossKillsToday)) || 0;
    var target = (data && Number(data.siegeTarget)) || 25;
    var pct = Math.min(100, Math.round((kills / Math.max(1, target)) * 100));
    var siegeCountEl = document.getElementById('siege-count');
    if (siegeCountEl) siegeCountEl.textContent = kills + ' / ' + target + ' Bosses Slain';
    var siegeFillEl = document.getElementById('siege-fill');
    if (siegeFillEl) siegeFillEl.style.width = pct + '%';

    // Live Faction War Percentage Bar
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
        : [33, 33, 34]);
    var seg0 = document.getElementById('f-seg-0');
    var seg1 = document.getElementById('f-seg-1');
    var seg2 = document.getElementById('f-seg-2');
    if (seg0) seg0.style.width = Math.max(6, fp[0]) + '%';
    if (seg1) seg1.style.width = Math.max(6, fp[1]) + '%';
    if (seg2) seg2.style.width = Math.max(6, fp[2]) + '%';

    var factionEl = document.getElementById('faction-line');
    if (factionEl) {
      factionEl.innerHTML =
        '<span>☀️ Solar <strong>' + fp[0] + '%</strong></span>' +
        '<span>🔮 Arcane <strong>' + fp[1] + '%</strong></span>' +
        '<span>🗡️ Shadow <strong>' + fp[2] + '%</strong></span>';
    }

    // Siege Activity Feed
    var feedEl = document.getElementById('siege-feed');
    if (feedEl) {
      var feed = (data && Array.isArray(data.siegeFeed)) ? data.siegeFeed : [];
      if (feed.length > 0) {
        var items = feed.slice(0, 3).map(function (entry) {
          var icon = entry.slain ? '💀' : '⚔️';
          var clsEmoji = CLASS_EMOJI[entry.heroClass % 3] || '⚔️';
          return clsEmoji + ' <strong>u/' + esc(entry.username) + '</strong> ' + icon;
        });
        feedEl.innerHTML = 'Siege Feed: ' + items.join(' · ');
      } else {
        feedEl.textContent = '⚔️ Strike ' + bossName + ' to appear in the live Siege Feed!';
      }
    }

    // Yesterday's Champion
    var ydayEl = document.getElementById('yday-champ');
    if (ydayEl) {
      if (data && data.yesterdayChampion && data.yesterdayChampion.username) {
        ydayEl.textContent = '🏆 J-1: u/' + data.yesterdayChampion.username;
      } else {
        ydayEl.textContent = '90s Ranked';
      }
    }

    // Top 3 Leaderboard
    var lb = (data && Array.isArray(data.leaderboard)) ? data.leaderboard.slice(0, 3) : [];
    var lbEl = document.getElementById('lb-list');
    if (lbEl) {
      if (lb.length === 0) {
        lbEl.innerHTML = '<div class="lb-row"><span class="lb-user" style="color:#94a3b8;">No runs yet today — claim #1 &amp; set the Champion Ghost!</span></div>';
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

    // Tomorrow's Dungeon Teaser
    var tomorrowSeed = (data && data.tomorrowSeed) ? String(data.tomorrowSeed) : utcSeed(1);
    var tBiomeIdx = (data && typeof data.tomorrowBiomeIndex === 'number')
      ? ((data.tomorrowBiomeIndex % 3 + 3) % 3)
      : defaultBiomeIdx(tomorrowSeed);
    var tBiomeName = (data && data.tomorrowBiomeName) ? String(data.tomorrowBiomeName) : (BIOMES[tBiomeIdx] || BIOMES[0]).short;
    var tMutIdx = (data && typeof data.tomorrowLeadingMutator === 'number' && data.tomorrowLeadingMutator >= 0)
      ? (data.tomorrowLeadingMutator % 4)
      : defaultMutator(tomorrowSeed);
    var tVotes = (data && Number(data.tomorrowTotalVotes)) || 0;
    var tInfoEl = document.getElementById('tomorrow-info');
    if (tInfoEl) {
      var shortBiome = tBiomeName.split(' ')[0];
      var voteSuffix = tVotes > 0 ? ' (' + tVotes + 'v)' : '';
      tInfoEl.innerHTML = 'Next: <strong>' + esc(shortBiome) + '</strong> · 🗳️ <strong>' + esc(MODIFIERS[tMutIdx].title) + '</strong>' + esc(voteSuffix);
    }

    // Player Rank & Daily Streak (with clean Guest fallback)
    var meRankEl = document.getElementById('me-rank');
    var meStreakEl = document.getElementById('me-streak');
    if (data && data.username) {
      var rankTxt = data.myRank > 0
        ? 'u/' + esc(data.username) + ' · <strong>#' + data.myRank + '/' + data.totalPlayers + '</strong>'
        : 'u/' + esc(data.username) + ' · Ready to rank';
      if (meRankEl) meRankEl.innerHTML = rankTxt;
      if (meStreakEl) meStreakEl.textContent = '🔥 ' + (Number(data.streak) || 0) + '-Day Streak';
    } else {
      if (meRankEl) meRankEl.textContent = 'Guest Mode · Sign in on Reddit for Ranked & Votes';
      var localStreak = 0;
      try {
        var rawP = localStorage.getItem('ef_profile_v1');
        if (rawP) {
          var parsedP = JSON.parse(rawP);
          localStreak = Number(parsedP && parsedP.localStreak) || 0;
        }
      } catch (_) {}
      if (meStreakEl) meStreakEl.textContent = '🔥 ' + localStreak + '-Day Streak';
    }
  }

  renderData(null);
  updateCountdown();
  setInterval(updateCountdown, 1000);

  fetch('/api/init', { cache: 'no-store' })
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(function (json) { if (json) renderData(json); })
    .catch(function () {});
})();

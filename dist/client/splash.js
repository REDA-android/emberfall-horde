(function () {
  "use strict";

  function utcSeed() {
    var d = new Date();
    var y = d.getUTCFullYear();
    var m = String(d.getUTCMonth() + 1).padStart(2, "0");
    var day = String(d.getUTCDate()).padStart(2, "0");
    return y + "-" + m + "-" + day;
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
  if (helpBtn) helpBtn.addEventListener("click", function () { if (guideDrawer) guideDrawer.classList.add("open"); });
  if (closeGuideBtn) closeGuideBtn.addEventListener("click", function () { if (guideDrawer) guideDrawer.classList.remove("open"); });

  function updateCountdown() {
    var now = new Date();
    var next = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() + 1);
    var sec = Math.max(0, Math.floor((next - now.getTime()) / 1000));
    var h = String(Math.floor(sec / 3600)).padStart(2, "0");
    var m = String(Math.floor((sec % 3600) / 60)).padStart(2, "0");
    var s = String(sec % 60).padStart(2, "0");
    var el = document.getElementById("utc-countdown");
    if (el) el.textContent = "RESET " + h + ":" + m + ":" + s;
  }
  updateCountdown();
  setInterval(updateCountdown, 1000);

  var seedEl = document.getElementById("seed-pill");
  if (seedEl) seedEl.textContent = "DAILY #" + utcSeed();

  fetch("/api/init", { cache: "no-store" })
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(function (d) {
      if (!d) return;
      if (seedEl && d.seed) seedEl.textContent = "DAILY #" + d.seed;
      var sc = document.getElementById("siege-count");
      if (sc) sc.textContent = (Number(d.bossKillsToday) || 0) + " BOSS KOs TODAY";
      var lbEl = document.getElementById("lb-list");
      if (lbEl && Array.isArray(d.leaderboard) && d.leaderboard.length > 0) {
        lbEl.innerHTML = d.leaderboard.slice(0, 3).map(function (row, idx) {
          return '<div class="splash-lb-row"><span>#' + (idx + 1) + ' u/' + esc(row.username || "Player") + '</span><strong>' + Number(row.score || 0).toLocaleString("en-US") + ' PTS</strong></div>';
        }).join("");
      }
      var mr = document.getElementById("me-rank");
      if (mr && d.myRank > 0) {
        mr.textContent = "Your Rank #" + d.myRank + " · Personal Best " + Number(d.personalBest || 0).toLocaleString("en-US") + " PTS";
      }
    })
    .catch(function () {});

  // Genuine 16-bit SNES Pixel-Art Harbor Panorama & Rolling Surf on Splash Background!
  var bgCanvas = document.getElementById("splash-harbor-bg");
  var bctx = bgCanvas ? bgCanvas.getContext("2d") : null;
  var bgImg = new Image();
  bgImg.src = "./arcade/bg_street_a.webp";
  var t = 0;

  function drawSplashHarbor() {
    if (!bctx) return;
    if (bgCanvas.width !== 960 || bgCanvas.height !== 540) {
      bgCanvas.width = 960;
      bgCanvas.height = 540;
    }
    bctx.imageSmoothingEnabled = false;
    t += 0.016;

    if (bgImg.complete && bgImg.naturalWidth > 0) {
      var pan = Math.round((Math.sin(t * 0.35) * 0.5 + 0.5) * 180);
      bctx.drawImage(bgImg, -pan, 0, 960, 540);
      bctx.drawImage(bgImg, 960 - pan, 0, 960, 540);
      // Animated pixel surf strip
      var surfShift = Math.round((t * 36) % 960);
      bctx.drawImage(bgImg, 0, 472, 960, 68, -surfShift, 472, 960, 68);
      bctx.drawImage(bgImg, 0, 472, 960, 68, 960 - surfShift, 472, 960, 68);
    } else {
      bctx.fillStyle = "#0e2442";
      bctx.fillRect(0, 0, 960, 540);
    }
    requestAnimationFrame(drawSplashHarbor);
  }
  requestAnimationFrame(drawSplashHarbor);
})();

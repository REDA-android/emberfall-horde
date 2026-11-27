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
      if (sc) sc.textContent = (Number(d.bossKillsToday) || 0) + " KINGPIN KOs TODAY";
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

  // Animated 16-bit Harbor Street & Rolling Waves Background Canvas
  var bgCanvas = document.getElementById("splash-harbor-bg");
  var bctx = bgCanvas ? bgCanvas.getContext("2d") : null;
  var t = 0;

  function drawSplashHarbor() {
    if (!bctx) return;
    var w = window.innerWidth || 480;
    var h = window.innerHeight || 640;
    if (bgCanvas.width !== w || bgCanvas.height !== h) {
      bgCanvas.width = w;
      bgCanvas.height = h;
    }
    t += 0.025;

    var sky = bctx.createLinearGradient(0, 0, 0, h * 0.65);
    sky.addColorStop(0, "#104db5");
    sky.addColorStop(1, "#6ebbf7");
    bctx.fillStyle = sky;
    bctx.fillRect(0, 0, w, h);

    // Colorful 16-bit Harbor Facades Silhouette
    var houses = [
      { col: "#92b59c", h: 0.34 },
      { col: "#e7b944", h: 0.44 },
      { col: "#255aa8", h: 0.46 },
      { col: "#e04248", h: 0.38 },
      { col: "#dce4ee", h: 0.32 }
    ];
    var bw = Math.ceil(w / houses.length);
    var groundY = Math.round(h * 0.78);
    for (var i = 0; i < houses.length; i++) {
      var bh = Math.round(h * houses[i].h);
      var bx = i * bw;
      bctx.fillStyle = houses[i].col;
      bctx.fillRect(bx, groundY - bh, bw, bh);
      bctx.fillStyle = "#3f5066";
      bctx.fillRect(bx, groundY - bh, bw, 32);
      bctx.strokeStyle = "#141821";
      bctx.lineWidth = 2;
      bctx.strokeRect(bx, groundY - bh, bw, bh);
    }

    // Quay sidewalk
    bctx.fillStyle = "#9aa4b2";
    bctx.fillRect(0, groundY, w, Math.round(h * 0.09));

    // Rolling Royal Blue Harbor Water
    var waterY = Math.round(h * 0.87);
    bctx.fillStyle = "#124ca6";
    bctx.fillRect(0, waterY, w, h - waterY);
    for (var layer = 0; layer < 3; layer++) {
      bctx.strokeStyle = layer % 2 === 0 ? "#e0f2fe" : "#38bdf8";
      bctx.lineWidth = 3;
      bctx.beginPath();
      for (var x = 0; x <= w; x += 10) {
        var wy = waterY + 12 + layer * 16 + Math.sin(x * 0.04 + t * (layer + 1.5)) * 4;
        if (x === 0) bctx.moveTo(x, wy);
        else bctx.lineTo(x, wy);
      }
      bctx.stroke();
    }
    requestAnimationFrame(drawSplashHarbor);
  }
  requestAnimationFrame(drawSplashHarbor);
})();

import type { IncomingMessage, ServerResponse } from 'node:http';
import { context, createServer, getServerPort, reddit, redis } from '@devvit/web/server';

interface StoredRunMeta {
  score: number;
  kills: number;
  bossSlain: boolean;
  level: number;
  rank: string;
}

function getUtcDateString(offsetDays = 0): string {
  const d = new Date(Date.now() + offsetDays * 86400000);
  const y = d.getUTCFullYear();
  const m = String(d.getUTCMonth() + 1).padStart(2, '0');
  const day = String(d.getUTCDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

function computeRank(score: number, bossSlain: boolean): string {
  if (bossSlain && score >= 14000) return 'SSS';
  if (bossSlain && score >= 9500) return 'SS';
  if (score >= 7000) return 'S';
  if (score >= 4500) return 'A';
  if (score >= 2500) return 'B';
  if (score >= 1200) return 'C';
  if (score >= 500) return 'D';
  return 'E';
}

function sendJson(res: ServerResponse, status: number, payload: unknown): void {
  res.writeHead(status, {
    'Content-Type': 'application/json; charset=utf-8',
    'Cache-Control': 'no-store',
  });
  res.end(JSON.stringify(payload));
}

function readJsonBody<T>(req: IncomingMessage, maxBytes = 64 * 1024): Promise<T | null> {
  return new Promise((resolve) => {
    const chunks: Buffer[] = [];
    let total = 0;
    req.on('data', (chunk: Buffer | string) => {
      const buf = typeof chunk === 'string' ? Buffer.from(chunk) : chunk;
      total += buf.length;
      if (total <= maxBytes) chunks.push(buf);
    });
    req.on('end', () => {
      if (total === 0 || total > maxBytes) {
        resolve(null);
        return;
      }
      try {
        resolve(JSON.parse(Buffer.concat(chunks).toString('utf8')) as T);
      } catch {
        resolve(null);
      }
    });
    req.on('error', () => resolve(null));
  });
}

const lbKey = (sub: string, seed: string) => `ef:lb:${sub}:${seed}`;
const metaKey = (sub: string, seed: string, user: string) => `ef:meta:${sub}:${seed}:${user}`;
const streakKey = (sub: string, user: string) => `ef:streak:${sub}:${user}`;
const bossKillsKey = (sub: string, seed: string) => `ef:bosses:${sub}:${seed}`;
const shareGuardKey = (postId: string, seed: string, user: string) => `ef:shared:${postId}:${seed}:${user}`;

async function createDailyPost(): Promise<{ id: string; url?: string }> {
  const subredditName = context.subredditName;
  if (!subredditName) throw new Error('Missing subredditName');
  const today = getUtcDateString();
  const post = await reddit.submitCustomPost({
    subredditName,
    title: `EMBERFALL: DAILY HORDE — Dungeon #${today}`,
    entry: 'default',
  });
  return { id: post.id, url: post.url };
}

const server = createServer(async (req: IncomingMessage, res: ServerResponse) => {
  const method = req.method ?? 'GET';
  const parsedUrl = new URL(req.url ?? '/', 'http://localhost');
  const pathname = parsedUrl.pathname;

  try {
    if (pathname === '/internal/on-app-install' && method === 'POST') {
      const post = await createDailyPost();
      sendJson(res, 200, { status: 'success', postId: post.id });
      return;
    }

    if (pathname === '/internal/menu/post-create' && method === 'POST') {
      const post = await createDailyPost();
      sendJson(res, 200, {
        navigateTo: post.url ?? `https://reddit.com/r/${context.subredditName}/comments/${post.id}`,
        showToast: { text: 'EMBERFALL Daily Dungeon created!', appearance: 'success' },
      });
      return;
    }

    if (pathname === '/api/init' && method === 'GET') {
      const sub = context.subredditName ?? 'emberfall';
      const seed = getUtcDateString();
      let username: string | undefined;
      try {
        username = await reddit.getCurrentUsername();
      } catch {
        username = undefined;
      }

      const top = await redis.zRange(lbKey(sub, seed), 0, 9, { by: 'rank', reverse: true });
      const bossRaw = await redis.get(bossKillsKey(sub, seed));
      const bossKillsToday = bossRaw ? Math.max(0, Number(bossRaw) || 0) : 0;

      sendJson(res, 200, {
        ok: true,
        username: username ?? 'Guest',
        subredditName: sub,
        seed,
        bossKillsToday,
        leaderboard: top.map((e, i) => ({ username: e.member, score: Math.floor(e.score), rank: i + 1 })),
      });
      return;
    }

    if (pathname === '/api/submit-run' && method === 'POST') {
      const body = await readJsonBody<{
        score?: number;
        kills?: number;
        bossSlain?: boolean;
        level?: number;
      }>(req);
      if (!body) {
        sendJson(res, 400, { ok: false, error: 'Invalid payload' });
        return;
      }

      const sub = context.subredditName ?? 'emberfall';
      const seed = getUtcDateString();
      const score = Math.max(0, Math.min(500000, Math.floor(Number(body.score) || 0)));
      const kills = Math.max(0, Math.min(10000, Math.floor(Number(body.kills) || 0)));
      const bossSlain = Boolean(body.bossSlain);
      const level = Math.max(1, Math.min(99, Math.floor(Number(body.level) || 1)));
      const rank = computeRank(score, bossSlain);

      let username: string | undefined;
      try {
        username = await reddit.getCurrentUsername();
      } catch {
        username = undefined;
      }

      if (bossSlain) {
        await redis.incrBy(bossKillsKey(sub, seed), 1);
      }

      if (username) {
        const key = lbKey(sub, seed);
        const prev = await redis.zScore(key, username);
        if (prev === undefined || score > prev) {
          await redis.zAdd(key, { member: username, score });
          const meta: StoredRunMeta = { score, kills, bossSlain, level, rank };
          await redis.set(metaKey(sub, seed, username), JSON.stringify(meta));
        }

        // Update Daily Streak
        const today = getUtcDateString(0);
        const yesterday = getUtcDateString(-1);
        let streak = 1;
        try {
          const raw = await redis.get(streakKey(sub, username));
          if (raw) {
            const p = JSON.parse(raw) as { count?: number; lastDate?: string };
            if (p.lastDate === today) streak = Math.max(1, Number(p.count) || 1);
            else if (p.lastDate === yesterday) streak = (Number(p.count) || 0) + 1;
          }
        } catch {
          streak = 1;
        }
        await redis.set(streakKey(sub, username), JSON.stringify({ count: streak, lastDate: today }));
      }

      sendJson(res, 200, { ok: true, rank });
      return;
    }

    if (pathname === '/api/share-comment' && method === 'POST') {
      const body = await readJsonBody<{
        score?: number;
        kills?: number;
        bossSlain?: boolean;
      }>(req);
      if (!body || !context.postId) {
        sendJson(res, 400, { ok: false, error: 'Missing post context' });
        return;
      }

      let username: string | undefined;
      try {
        username = await reddit.getCurrentUsername();
      } catch {
        username = undefined;
      }
      if (!username) {
        sendJson(res, 401, { ok: false, error: 'Sign in to Reddit to post your score' });
        return;
      }

      const seed = getUtcDateString();
      const score = Math.max(0, Math.floor(Number(body.score) || 0));
      const kills = Math.max(0, Math.floor(Number(body.kills) || 0));
      const bossSlain = Boolean(body.bossSlain);
      const rank = computeRank(score, bossSlain);

      const guardKey = shareGuardKey(context.postId, seed, username);
      const prevRaw = await redis.get(guardKey);
      if (prevRaw && Number(prevRaw) >= score) {
        sendJson(res, 200, { ok: true, alreadyShared: true });
        return;
      }

      const bossBadge = bossSlain ? '💀 Daily Boss SLAIN' : '⚔️ Survived the Horde';
      const text = [
        `**🔥 EMBERFALL: DAILY HORDE #${seed}** — **RANK ${rank}**`,
        `Score: **${score.toLocaleString('en-US')} pts** · Kills: **${kills}** · ${bossBadge}`,
        `Think you can reach **RANK SSS**? Hit Play above!`,
      ].join('\n\n');

      await reddit.submitComment({
        id: context.postId,
        text,
        runAs: 'USER',
      });
      await redis.set(guardKey, String(score));
      sendJson(res, 200, { ok: true, alreadyShared: false });
      return;
    }

    sendJson(res, 404, { ok: false, error: 'Not found' });
  } catch (err) {
    sendJson(res, 500, { ok: false, error: err instanceof Error ? err.message : 'Server error' });
  }
});

server.listen(getServerPort());

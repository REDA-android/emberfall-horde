import { randomBytes } from 'node:crypto';
import type { IncomingMessage, ServerResponse } from 'node:http';
import { context, createServer, getServerPort, reddit, redis } from '@devvit/web/server';

interface StoredRunMeta {
  score: number;
  kills: number;
  bossSlain: boolean;
  level: number;
  rank: string;
  buildSummary?: string;
}

interface StoredGhost {
  username: string;
  score: number;
  points: [number, number][];
}

interface StoredRunSession {
  startedAt: number;
  username: string | null;
  sub: string;
  seed: string;
}

interface StoredProfile {
  medalsMask: number;
  bestEndlessWave: number;
  lifetimeKills: number;
  lifetimeBosses: number;
}

interface SiegeFeedEntry {
  username: string;
  heroClass: number;
  slain: boolean;
  score: number;
  ts: number;
}

interface DailyBountyInfo {
  kind: string;
  label: string;
  target: number;
  reward: number;
}

const THIRTY_DAYS_SEC = 30 * 86400;
const SIEGE_TARGET = 25;
const DAILY_MODIFIERS = [
  'MUSTACHE STORM',
  'TURBO RELOAD',
  'HEAVY SYNDICATE',
  'CAR SMASH BONUS',
] as const;

const DAILY_BIOMES = [
  'HARBOR DOCKS',
  'SUNSET BOULEVARD',
  'SYNDICATE NIGHT',
] as const;

const BIOME_BOSSES = [
  'DON MUSTACHIO, HARBOR KINGPIN',
  'DON MUSTACHIO, HARBOR KINGPIN',
  'DON MUSTACHIO, HARBOR KINGPIN',
] as const;

const HERO_CLASSES = [
  { title: 'GOLDEN TITAN (ARM RELOAD)', faction: 'HARBOR VANGUARD', emoji: '🥊' },
  { title: 'ROYAL RANGER (BLITZ DASH)', faction: 'QUAY PATROL', emoji: '⚡' },
  { title: 'CRIMSON FIST (DRAGON UPPERCUT)', faction: 'STREET BRAWLERS', emoji: '🔥' },
] as const;

const BOUNTY_POOL: readonly DailyBountyInfo[] = [
  { kind: 'WitchTimeDodges', label: '2 Witch-Time Dodges', target: 2, reward: 750 },
  { kind: 'ReachLevel', label: 'Reach Level 6', target: 6, reward: 750 },
  { kind: 'SlayDailyBoss', label: "Slay Today's Boss", target: 1, reward: 750 },
  { kind: 'DetonateBarrels', label: 'Detonate 3 Barrels', target: 3, reward: 750 },
  { kind: 'ClaimShrines', label: 'Claim 1 Blood Shrine', target: 1, reward: 750 },
  { kind: 'ReachCombo', label: 'Reach 30x Combo', target: 30, reward: 750 },
  { kind: 'OpenChests', label: 'Open 2 Runic Chests', target: 2, reward: 750 },
  { kind: 'SlayHorde', label: 'Slay 75 Enemies', target: 75, reward: 750 },
] as const;

function getUtcDateString(offsetDays = 0): string {
  const d = new Date(Date.now() + offsetDays * 86400000);
  const y = d.getUTCFullYear();
  const m = String(d.getUTCMonth() + 1).padStart(2, '0');
  const day = String(d.getUTCDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

function createSeedRng(seed: string): () => bigint {
  const mask = 0xffffffffffffffffn;
  let state = 0xcbf29ce484222325n;
  for (let i = 0; i < seed.length; i++) {
    state ^= BigInt(seed.charCodeAt(i) & 0xff);
    state = (state * 0x100000001b3n) & mask;
  }
  if (state === 0n) state = 0x9e3779b97f4a7c15n;
  return () => {
    let x = state;
    x ^= x >> 12n;
    x ^= (x << 25n) & mask;
    x ^= x >> 27n;
    state = x;
    return (x * 0x2545f4914f6cdd1dn) & mask;
  };
}

function hashSeedU64(seed: string): bigint {
  return createSeedRng(seed)();
}

function defaultMutatorFromSeed(seed: string): number {
  return Number(hashSeedU64(seed) % 4n);
}

function defaultBiomeFromSeed(seed: string): { index: number; name: string } {
  const idx = Number(hashSeedU64(`${seed}-biome`) % 3n);
  return { index: idx, name: DAILY_BIOMES[idx] };
}

function getDailyBountiesFromSeed(seed: string): DailyBountyInfo[] {
  const next = createSeedRng(`bounties:${seed}`);
  const pool = [...BOUNTY_POOL];
  for (let i = pool.length - 1; i >= 1; i--) {
    const j = Number(next() % BigInt(i + 1));
    const tmp = pool[i];
    pool[i] = pool[j];
    pool[j] = tmp;
  }
  return [pool[0], pool[1], pool[2]];
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
  const body = JSON.stringify(payload);
  res.writeHead(status, {
    'Content-Type': 'application/json; charset=utf-8',
    'Content-Length': String(Buffer.byteLength(body, 'utf8')),
    'Cache-Control': 'no-store',
  });
  res.end(body);
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
const factionsKey = (sub: string, seed: string) => `ef:factions:${sub}:${seed}`;
const shareGuardKey = (postId: string, seed: string, user: string) =>
  `ef:shared:${postId}:${seed}:${user}`;
const postSharesKey = (postId: string) => `ef:post-shares:${postId}`;
const postKey = (sub: string, seed: string) => `ef:post:${sub}:${seed}`;
const postIndexKey = (postId: string) => `ef:postidx:${postId}`;
const lockKey = (sub: string, seed: string) => `ef:lock:${sub}:${seed}`;
const runKey = (runId: string) => `ef:run:${runId}`;
const rateKey = (sub: string, user: string) => `ef:rate:${sub}:${user}`;
const ghostKey = (sub: string, seed: string) => `ef:ghost:${sub}:${seed}`;
const mutatorVoteKey = (sub: string, seed: string) => `ef:mutator-vote:${sub}:${seed}`;
const profileKey = (sub: string, user: string) => `ef:profile:${sub}:${user}`;
const siegeFeedKey = (sub: string, seed: string) => `ef:siege-feed:${sub}:${seed}`;

function computeFactionPercents(scores: [number, number, number]): [number, number, number] {
  const total = scores[0] + scores[1] + scores[2];
  if (total <= 0) return [33, 33, 34];
  const p0 = Math.round((scores[0] / total) * 100);
  const p1 = Math.round((scores[1] / total) * 100);
  const p2 = Math.max(0, 100 - p0 - p1);
  return [p0, p1, p2];
}

async function getSiegeFeed(sub: string, seed: string): Promise<SiegeFeedEntry[]> {
  try {
    const raw = await redis.get(siegeFeedKey(sub, seed));
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter((e): e is SiegeFeedEntry => Boolean(e && typeof e.username === 'string'))
      .slice(0, 6);
  } catch {
    return [];
  }
}

async function pushSiegeFeed(
  sub: string,
  seed: string,
  entry: SiegeFeedEntry
): Promise<SiegeFeedEntry[]> {
  const existing = await getSiegeFeed(sub, seed);
  const filtered = existing.filter((e) => e.username !== entry.username || e.slain !== entry.slain);
  const next = [entry, ...filtered].slice(0, 6);
  try {
    await redis.set(siegeFeedKey(sub, seed), JSON.stringify(next));
    await redis.expire(siegeFeedKey(sub, seed), THIRTY_DAYS_SEC);
  } catch {
    // non-fatal
  }
  return next;
}

async function getUserProfile(sub: string, username: string | undefined): Promise<StoredProfile> {
  const empty: StoredProfile = {
    medalsMask: 0,
    bestEndlessWave: 0,
    lifetimeKills: 0,
    lifetimeBosses: 0,
  };
  if (!username) return empty;
  try {
    const raw = await redis.get(profileKey(sub, username));
    if (!raw) return empty;
    const p = JSON.parse(raw) as Partial<StoredProfile>;
    return {
      medalsMask: Math.max(0, Math.floor(Number(p.medalsMask) || 0)) & 63,
      bestEndlessWave: Math.max(0, Math.floor(Number(p.bestEndlessWave) || 0)),
      lifetimeKills: Math.max(0, Math.floor(Number(p.lifetimeKills) || 0)),
      lifetimeBosses: Math.max(0, Math.floor(Number(p.lifetimeBosses) || 0)),
    };
  } catch {
    return empty;
  }
}

async function getFactionScores(sub: string, seed: string): Promise<[number, number, number]> {
  try {
    const raw = await redis.hGetAll(factionsKey(sub, seed));
    return [
      Math.max(0, Math.floor(Number(raw['0']) || 0)),
      Math.max(0, Math.floor(Number(raw['1']) || 0)),
      Math.max(0, Math.floor(Number(raw['2']) || 0)),
    ];
  } catch {
    return [0, 0, 0];
  }
}

async function getYesterdayChampion(
  sub: string
): Promise<{ username: string; score: number } | null> {
  try {
    const yesterday = getUtcDateString(-1);
    const top = await redis.zRange(lbKey(sub, yesterday), 0, 0, { by: 'rank', reverse: true });
    if (top.length > 0 && top[0].member) {
      return { username: top[0].member, score: Math.floor(top[0].score) };
    }
    return null;
  } catch {
    return null;
  }
}

async function getMutatorVoteSummary(
  sub: string,
  seed: string,
  username?: string
): Promise<{
  counts: [number, number, number, number];
  totalVotes: number;
  leadingIndex: number;
  hasVotes: boolean;
  myVote: number;
}> {
  const counts: [number, number, number, number] = [0, 0, 0, 0];
  let totalVotes = 0;
  let myVote = -1;
  try {
    const votes = await redis.hGetAll(mutatorVoteKey(sub, seed));
    for (const [voter, val] of Object.entries(votes)) {
      const idx = Number(val);
      if (Number.isInteger(idx) && idx >= 0 && idx <= 3) {
        counts[idx]++;
        totalVotes++;
        if (username && voter === username) myVote = idx;
      }
    }
  } catch {
    // fallback to seed default
  }
  if (totalVotes === 0) {
    return {
      counts,
      totalVotes: 0,
      leadingIndex: defaultMutatorFromSeed(seed),
      hasVotes: false,
      myVote,
    };
  }
  let bestIdx = 0;
  for (let i = 1; i < 4; i++) {
    if (counts[i] > counts[bestIdx]) bestIdx = i;
  }
  return { counts, totalVotes, leadingIndex: bestIdx, hasVotes: true, myVote };
}

async function resolveWinningMutator(sub: string, seed: string): Promise<number> {
  const summary = await getMutatorVoteSummary(sub, seed);
  return summary.hasVotes ? summary.leadingIndex : -1;
}

async function getEffectiveMutator(
  sub: string,
  seed: string
): Promise<{ override: number; effective: number; name: string }> {
  const override = await resolveWinningMutator(sub, seed);
  const effective = override >= 0 && override <= 3 ? override : defaultMutatorFromSeed(seed);
  return { override, effective, name: DAILY_MODIFIERS[effective] };
}

async function createDailyPost(options: { respectLock: boolean }): Promise<{
  id: string;
  url?: string;
  skipped?: boolean;
}> {
  const subredditName = context.subredditName;
  if (!subredditName) throw new Error('Missing subredditName');
  const today = getUtcDateString();

  if (options.respectLock) {
    const existingPostId = await redis.get(postKey(subredditName, today));
    if (existingPostId) {
      return { id: existingPostId, skipped: true };
    }
    const lockAcquired = await redis.set(lockKey(subredditName, today), '1', {
      nx: true,
      expiration: new Date(Date.now() + 600_000),
    });
    if (lockAcquired !== 'OK') {
      const postIdAfterLock = await redis.get(postKey(subredditName, today));
      return { id: postIdAfterLock ?? '', skipped: true };
    }
    const recheck = await redis.get(postKey(subredditName, today));
    if (recheck) {
      return { id: recheck, skipped: true };
    }
  }

  const mutator = await getEffectiveMutator(subredditName, today);
  const biome = defaultBiomeFromSeed(today);
  const post = await reddit.submitCustomPost({
    subredditName,
    title: `EMBERFALL: DAILY HORDE — #${today} [${biome.name} · ${mutator.name}]`,
    entry: 'default',
  });

  await redis.set(postKey(subredditName, today), post.id);
  await redis.expire(postKey(subredditName, today), THIRTY_DAYS_SEC);
  await redis.set(postIndexKey(post.id), JSON.stringify({ sub: subredditName, seed: today }));
  await redis.expire(postIndexKey(post.id), THIRTY_DAYS_SEC);

  // Automatically crown yesterday's #1 Champion & winning Reddit Faction on the new daily post!
  try {
    const yesterday = getUtcDateString(-1);
    const champ = await getYesterdayChampion(subredditName);
    const yFactions = await getFactionScores(subredditName, yesterday);
    if (champ) {
      let winFactionIdx = 0;
      if (yFactions[1] > yFactions[winFactionIdx]) winFactionIdx = 1;
      if (yFactions[2] > yFactions[winFactionIdx]) winFactionIdx = 2;
      const winFaction = HERO_CLASSES[winFactionIdx];
      await reddit.submitComment({
        id: post.id,
        text: [
          `🏆 **YESTERDAY'S CROWNED CHAMPION (#${yesterday})**: **u/${champ.username}** with **${champ.score.toLocaleString('en-US')} pts**!`,
          `${winFaction.emoji} Winning Order: **${winFaction.faction}** — *Choose your Hero Class & Faction above to claim today's crown!*`,
        ].join('\n\n'),
      });
    }
  } catch {
    // Non-fatal if comment submission fails in local test environment
  }

  return { id: post.id, url: post.url, skipped: false };
}

async function getMyRankAndTotal(
  sub: string,
  seed: string,
  username: string | undefined
): Promise<{ myRank: number; totalPlayers: number }> {
  const key = lbKey(sub, seed);
  const totalPlayers = Number(await redis.zCard(key)) || 0;
  if (!username || totalPlayers === 0) return { myRank: 0, totalPlayers };
  const ascRank = await redis.zRank(key, username);
  if (ascRank === undefined) return { myRank: 0, totalPlayers };
  return { myRank: Math.max(1, totalPlayers - Number(ascRank)), totalPlayers };
}

async function getUserStreak(sub: string, username: string | undefined): Promise<number> {
  if (!username) return 0;
  const today = getUtcDateString(0);
  const yesterday = getUtcDateString(-1);
  try {
    const raw = await redis.get(streakKey(sub, username));
    if (!raw) return 0;
    const p = JSON.parse(raw) as { count?: number; lastDate?: string };
    if (p.lastDate === today || p.lastDate === yesterday) {
      return Math.max(0, Number(p.count) || 0);
    }
    return 0;
  } catch {
    return 0;
  }
}

const server = createServer(async (req: IncomingMessage, res: ServerResponse) => {
  const method = req.method ?? 'GET';
  const parsedUrl = new URL(req.url ?? '/', 'http://localhost');
  const pathname = parsedUrl.pathname;

  try {
    if (pathname === '/internal/on-app-install' && method === 'POST') {
      const post = await createDailyPost({ respectLock: false });
      sendJson(res, 200, { status: 'success', postId: post.id, skipped: Boolean(post.skipped) });
      return;
    }

    if (pathname === '/internal/scheduler/daily-post' && method === 'POST') {
      const post = await createDailyPost({ respectLock: true });
      sendJson(res, 200, { status: 'success', postId: post.id, skipped: Boolean(post.skipped) });
      return;
    }

    if (pathname === '/internal/menu/post-create' && method === 'POST') {
      const post = await createDailyPost({ respectLock: false });
      sendJson(res, 200, {
        navigateTo: post.url ?? `https://reddit.com/r/${context.subredditName}/comments/${post.id}`,
        showToast: { text: 'EMBERFALL Daily Dungeon created!', appearance: 'success' },
      });
      return;
    }

    if (pathname === '/internal/on-post-delete' && method === 'POST') {
      const body = await readJsonBody<{ postId?: string }>(req);
      const deletedPostId = body?.postId ?? context.postId;
      if (deletedPostId) {
        const idxRaw = await redis.get(postIndexKey(deletedPostId));
        let seedForPost = getUtcDateString();
        if (idxRaw) {
          try {
            const idx = JSON.parse(idxRaw) as { sub?: string; seed?: string };
            if (idx.sub && idx.seed) {
              seedForPost = idx.seed;
              const currentPost = await redis.get(postKey(idx.sub, idx.seed));
              if (currentPost === deletedPostId) {
                await redis.del(postKey(idx.sub, idx.seed), lockKey(idx.sub, idx.seed));
              }
            }
          } catch {
            // ignore malformed index JSON
          }
          await redis.del(postIndexKey(deletedPostId));
        }

        try {
          const sharedUsers = await redis.hGetAll(postSharesKey(deletedPostId));
          const keysToDelete = Object.keys(sharedUsers).map((u) =>
            shareGuardKey(deletedPostId, seedForPost, u)
          );
          if (keysToDelete.length > 0) {
            await redis.del(...keysToDelete);
          }
          await redis.del(postSharesKey(deletedPostId));
        } catch {
          // ignore if no shared keys
        }
      }
      sendJson(res, 200, { ok: true });
      return;
    }

    if ((pathname === '/api/init' || pathname === '/api/state') && method === 'GET') {
      const sub = context.subredditName ?? 'emberfall';
      const seed = getUtcDateString();
      let username: string | null = null;
      try {
        username = (await reddit.getCurrentUsername()) ?? null;
      } catch {
        username = null;
      }

      const mutator = await getEffectiveMutator(sub, seed);
      const biome = defaultBiomeFromSeed(seed);
      const top = await redis.zRange(lbKey(sub, seed), 0, 9, { by: 'rank', reverse: true });
      const bossRaw = await redis.get(bossKillsKey(sub, seed));
      const bossKillsToday = bossRaw ? Math.max(0, Number(bossRaw) || 0) : 0;
      const factionScores = await getFactionScores(sub, seed);
      const yesterdayChampion = await getYesterdayChampion(sub);

      const { myRank, totalPlayers } = await getMyRankAndTotal(sub, seed, username ?? undefined);
      const streak = await getUserStreak(sub, username ?? undefined);
      const profile = await getUserProfile(sub, username ?? undefined);

      let personalBest = 0;
      if (username) {
        const pbScore = await redis.zScore(lbKey(sub, seed), username);
        if (pbScore !== undefined) personalBest = Math.floor(pbScore);
      }

      const leaderboard = await Promise.all(
        top.map(async (entry) => {
          const score = Math.floor(entry.score);
          let tier = computeRank(score, false);
          try {
            const metaRaw = await redis.get(metaKey(sub, seed, entry.member));
            if (metaRaw) {
              const meta = JSON.parse(metaRaw) as Partial<StoredRunMeta>;
              if (meta.rank) tier = meta.rank;
            }
          } catch {
            // fallback to score-derived rank
          }
          return { username: entry.member, score, rank: tier };
        })
      );

      let topGhost: StoredGhost | null = null;
      try {
        const ghostRaw = await redis.get(ghostKey(sub, seed));
        if (ghostRaw) {
          const parsed = JSON.parse(ghostRaw) as StoredGhost;
          if (parsed && typeof parsed.username === 'string' && Array.isArray(parsed.points)) {
            topGhost = parsed;
          }
        }
      } catch {
        topGhost = null;
      }

      const yesterday = getUtcDateString(-1);
      const yFactions = await getFactionScores(sub, yesterday);
      const yTotal = yFactions[0] + yFactions[1] + yFactions[2];
      let yesterdayWinningFaction = Number(hashSeedU64(`${seed}-blessing`) % 3n);
      if (yTotal > 0) {
        yesterdayWinningFaction = 0;
        if (yFactions[1] > yFactions[yesterdayWinningFaction]) yesterdayWinningFaction = 1;
        if (yFactions[2] > yFactions[yesterdayWinningFaction]) yesterdayWinningFaction = 2;
      }

      const tomorrow = getUtcDateString(1);
      const tomorrowBiome = defaultBiomeFromSeed(tomorrow);
      const tomorrowVotes = await getMutatorVoteSummary(sub, tomorrow, username ?? undefined);
      const siegeFeed = await getSiegeFeed(sub, seed);
      const dailyBounties = getDailyBountiesFromSeed(seed);

      sendJson(res, 200, {
        seed,
        biomeIndex: biome.index,
        biomeName: biome.name,
        bossName: BIOME_BOSSES[biome.index],
        mutatorOverride: mutator.override,
        username,
        streak,
        personalBest,
        myRank,
        totalPlayers,
        bossKillsToday,
        siegeTarget: SIEGE_TARGET,
        siegeFeed,
        factionScores,
        factionPercents: computeFactionPercents(factionScores),
        yesterdayChampion,
        yesterdayWinningFaction,
        leaderboard,
        topGhost,
        canVoteMutator: Boolean(username),
        tomorrowSeed: tomorrow,
        tomorrowBiomeIndex: tomorrowBiome.index,
        tomorrowBiomeName: tomorrowBiome.name,
        tomorrowBossName: BIOME_BOSSES[tomorrowBiome.index],
        tomorrowMutatorVotes: tomorrowVotes.counts,
        tomorrowTotalVotes: tomorrowVotes.totalVotes,
        tomorrowLeadingMutator: tomorrowVotes.leadingIndex,
        tomorrowHasVotes: tomorrowVotes.hasVotes,
        myTomorrowVote: tomorrowVotes.myVote,
        dailyBounties,
        serverNowMs: Date.now(),
        medalsMask: profile.medalsMask,
        bestEndlessWave: profile.bestEndlessWave,
        lifetimeKills: profile.lifetimeKills,
        lifetimeBosses: profile.lifetimeBosses,
      });
      return;
    }

    if (pathname === '/api/start-run' && method === 'POST') {
      const sub = context.subredditName ?? 'emberfall';
      const seed = getUtcDateString();
      let username: string | null = null;
      try {
        username = (await reddit.getCurrentUsername()) ?? null;
      } catch {
        username = null;
      }

      const runId = randomBytes(16).toString('hex');
      const startedAt = Date.now();
      const session: StoredRunSession = { startedAt, username, sub, seed };
      await redis.set(runKey(runId), JSON.stringify(session), {
        expiration: new Date(startedAt + 300_000),
      });

      sendJson(res, 200, { ok: true, runId, startedAt });
      return;
    }

    if (pathname === '/api/submit-run' && method === 'POST') {
      const body = await readJsonBody<{
        runId?: string;
        score?: number;
        kills?: number;
        bossSlain?: boolean;
        bossReached?: boolean;
        level?: number;
        heroClass?: number;
        buildSummary?: string;
        ghostPoints?: unknown;
        isEndless?: boolean;
        endlessWave?: number;
        medalsMask?: number;
      }>(req);
      if (!body) {
        sendJson(res, 400, { ok: false, error: 'Invalid payload' });
        return;
      }

      const sub = context.subredditName ?? 'emberfall';
      const seed = getUtcDateString();

      let username: string | undefined;
      try {
        username = await reddit.getCurrentUsername();
      } catch {
        username = undefined;
      }

      const isEndless = Boolean(body.isEndless);
      const maxRunMs = isEndless ? 1800_000 : 180_000;

      if (typeof body.runId === 'string' && body.runId.length > 0) {
        const sessionRaw = await redis.get(runKey(body.runId));
        if (!sessionRaw) {
          sendJson(res, 400, { ok: false, error: 'Invalid or expired run session' });
          return;
        }
        try {
          const session = JSON.parse(sessionRaw) as StoredRunSession;
          const elapsedMs = Date.now() - Number(session.startedAt);
          if (!Number.isFinite(elapsedMs) || elapsedMs < 10_000 || elapsedMs > maxRunMs) {
            sendJson(res, 400, { ok: false, error: 'Run duration out of bounds' });
            return;
          }
        } catch {
          sendJson(res, 400, { ok: false, error: 'Corrupt run session' });
          return;
        }
        await redis.del(runKey(body.runId));
      }

      const rateUser = username ?? req.socket.remoteAddress ?? 'anon';
      const rateOk = await redis.set(rateKey(sub, rateUser), '1', {
        nx: true,
        expiration: new Date(Date.now() + 10_000),
      });
      if (rateOk !== 'OK') {
        sendJson(res, 429, { ok: false, error: 'Too many submissions, please wait 10s' });
        return;
      }

      const score = Math.max(0, Math.min(500000, Math.floor(Number(body.score) || 0)));
      const kills = Math.max(0, Math.min(10000, Math.floor(Number(body.kills) || 0)));
      const bossSlain = Boolean(body.bossSlain);
      const level = Math.max(1, Math.min(99, Math.floor(Number(body.level) || 1)));
      const heroClassIdx = Math.max(0, Math.min(2, Math.floor(Number(body.heroClass) || 0)));
      const buildSummary =
        typeof body.buildSummary === 'string' ? body.buildSummary.slice(0, 128) : undefined;
      const rank = computeRank(score, bossSlain);

      // Contribute score to the player's chosen Reddit Faction Order!
      if (score > 0) {
        await redis.hIncrBy(factionsKey(sub, seed), String(heroClassIdx), score);
        await redis.expire(factionsKey(sub, seed), THIRTY_DAYS_SEC);
      }
      const factionScores = await getFactionScores(sub, seed);

      let bossKillsToday = 0;
      if (bossSlain) {
        bossKillsToday = Number(await redis.incrBy(bossKillsKey(sub, seed), 1)) || 1;
        await redis.expire(bossKillsKey(sub, seed), THIRTY_DAYS_SEC);
      } else {
        const bossRaw = await redis.get(bossKillsKey(sub, seed));
        bossKillsToday = bossRaw ? Math.max(0, Number(bossRaw) || 0) : 0;
      }

      let siegeFeed: SiegeFeedEntry[];
      if (username && (bossSlain || Boolean(body.bossReached) || score >= 1200)) {
        siegeFeed = await pushSiegeFeed(sub, seed, {
          username,
          heroClass: heroClassIdx,
          slain: bossSlain,
          score,
          ts: Date.now(),
        });
      } else {
        siegeFeed = await getSiegeFeed(sub, seed);
      }

      let isNewBest = false;
      let streak = 0;

      if (username) {
        const key = lbKey(sub, seed);
        const prev = await redis.zScore(key, username);
        if (prev === undefined || score > prev) {
          isNewBest = true;
          await redis.zAdd(key, { member: username, score });
          await redis.expire(key, THIRTY_DAYS_SEC);

          const meta: StoredRunMeta = { score, kills, bossSlain, level, rank, buildSummary };
          await redis.set(metaKey(sub, seed, username), JSON.stringify(meta));
          await redis.expire(metaKey(sub, seed, username), THIRTY_DAYS_SEC);
        }

        // Check if this user now holds #1 and save ghost if valid
        const topEntry = await redis.zRange(key, 0, 0, { by: 'rank', reverse: true });
        const topScore = topEntry.length > 0 ? topEntry[0].score : 0;
        if (score >= topScore && Array.isArray(body.ghostPoints) && body.ghostPoints.length > 0) {
          const pts: [number, number][] = [];
          const limit = Math.min(200, body.ghostPoints.length);
          for (let i = 0; i < limit; i++) {
            const p = body.ghostPoints[i];
            if (Array.isArray(p) && p.length >= 2) {
              const x = Math.max(-32768, Math.min(32767, Math.round(Number(p[0]) || 0)));
              const y = Math.max(-32768, Math.min(32767, Math.round(Number(p[1]) || 0)));
              pts.push([x, y]);
            }
          }
          if (pts.length > 0) {
            const ghost: StoredGhost = { username, score, points: pts };
            await redis.set(ghostKey(sub, seed), JSON.stringify(ghost));
            await redis.expire(ghostKey(sub, seed), THIRTY_DAYS_SEC);
          }
        }

        // Update Daily Streak
        const today = getUtcDateString(0);
        const yesterday = getUtcDateString(-1);
        streak = 1;
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
        await redis.expire(streakKey(sub, username), THIRTY_DAYS_SEC);
      }

      const incomingMedals = Math.max(0, Math.floor(Number(body.medalsMask) || 0)) & 63;
      const endlessWave = isEndless ? Math.max(1, Math.floor(Number(body.endlessWave) || 1)) : 0;
      let medalsMask = incomingMedals;
      let newlyUnlockedMask = 0;
      let bestEndlessWave = endlessWave;

      if (username) {
        const prevProfile = await getUserProfile(sub, username);
        newlyUnlockedMask = incomingMedals & ~prevProfile.medalsMask;
        medalsMask = (prevProfile.medalsMask | incomingMedals) & 63;
        bestEndlessWave = Math.max(prevProfile.bestEndlessWave, endlessWave);
        const updatedProfile: StoredProfile = {
          medalsMask,
          bestEndlessWave,
          lifetimeKills: prevProfile.lifetimeKills + kills,
          lifetimeBosses: prevProfile.lifetimeBosses + (bossSlain ? 1 : 0),
        };
        await redis.set(profileKey(sub, username), JSON.stringify(updatedProfile));
        await redis.expire(profileKey(sub, username), THIRTY_DAYS_SEC * 3);
      }

      const tomorrow = getUtcDateString(1);
      const tomorrowVotes = await getMutatorVoteSummary(sub, tomorrow, username);
      const { myRank, totalPlayers } = await getMyRankAndTotal(sub, seed, username);
      sendJson(res, 200, {
        ok: true,
        rank,
        myRank,
        totalPlayers,
        streak,
        bossKillsToday,
        siegeFeed,
        factionScores,
        factionPercents: computeFactionPercents(factionScores),
        isNewBest,
        canVoteMutator: Boolean(username),
        tomorrowMutatorVotes: tomorrowVotes.counts,
        tomorrowTotalVotes: tomorrowVotes.totalVotes,
        tomorrowLeadingMutator: tomorrowVotes.leadingIndex,
        tomorrowHasVotes: tomorrowVotes.hasVotes,
        myTomorrowVote: tomorrowVotes.myVote,
        medalsMask,
        newlyUnlockedMask,
        bestEndlessWave,
      });
      return;
    }

    if (pathname === '/api/vote-mutator' && method === 'POST') {
      const body = await readJsonBody<{ mutatorIndex?: number }>(req);
      if (!body || !Number.isInteger(body.mutatorIndex) || body.mutatorIndex! < 0 || body.mutatorIndex! > 3) {
        sendJson(res, 400, { ok: false, error: 'mutatorIndex must be 0..3' });
        return;
      }

      let username: string | undefined;
      try {
        username = await reddit.getCurrentUsername();
      } catch {
        username = undefined;
      }
      if (!username) {
        sendJson(res, 401, { ok: false, error: 'Sign in to Reddit to vote' });
        return;
      }

      const sub = context.subredditName ?? 'emberfall';
      const tomorrow = getUtcDateString(1);
      const voteKey = mutatorVoteKey(sub, tomorrow);
      await redis.hSet(voteKey, { [username]: String(body.mutatorIndex) });
      await redis.expire(voteKey, THIRTY_DAYS_SEC);

      const summary = await getMutatorVoteSummary(sub, tomorrow, username);
      sendJson(res, 200, {
        ok: true,
        mutatorIndex: body.mutatorIndex,
        tomorrowMutatorVotes: summary.counts,
        tomorrowTotalVotes: summary.totalVotes,
        tomorrowLeadingMutator: summary.leadingIndex,
        tomorrowHasVotes: summary.hasVotes,
        myTomorrowVote: body.mutatorIndex,
      });
      return;
    }

    if (pathname === '/api/share-comment' && method === 'POST') {
      const body = await readJsonBody<{
        score?: number;
        kills?: number;
        bossSlain?: boolean;
        level?: number;
        heroClass?: number;
        streak?: number;
        mutatorName?: string;
        buildEmojis?: string;
        rank?: string;
        myRank?: number;
        chronicle?: string;
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

      const sub = context.subredditName ?? 'emberfall';
      const seed = getUtcDateString();
      const score = Math.max(0, Math.floor(Number(body.score) || 0));
      const kills = Math.max(0, Math.floor(Number(body.kills) || 0));
      const bossSlain = Boolean(body.bossSlain);
      const level = Math.max(1, Math.min(99, Math.floor(Number(body.level) || 1)));
      const heroClassIdx = Math.max(0, Math.min(2, Math.floor(Number(body.heroClass) || 0)));
      const heroInfo = HERO_CLASSES[heroClassIdx];
      const biome = defaultBiomeFromSeed(seed);
      const rank =
        typeof body.rank === 'string' && body.rank.length > 0
          ? body.rank.slice(0, 6)
          : computeRank(score, bossSlain);
      const storedStreak = await getUserStreak(sub, username);
      const streakVal = Math.max(1, Math.floor(Number(body.streak) || 0), storedStreak);
      const myRankVal = Math.max(0, Math.floor(Number(body.myRank) || 0));
      const mutator = await getEffectiveMutator(sub, seed);
      const mutatorName =
        typeof body.mutatorName === 'string' && body.mutatorName.trim().length > 0
          ? body.mutatorName.trim().slice(0, 32)
          : mutator.name;
      const buildEmojis =
        typeof body.buildEmojis === 'string' && body.buildEmojis.trim().length > 0
          ? body.buildEmojis.trim().slice(0, 64)
          : '⚔️🔥';
      const chronicle =
        typeof body.chronicle === 'string' && body.chronicle.trim().length > 0
          ? body.chronicle.trim().slice(0, 240)
          : '';

      const guardKey = shareGuardKey(context.postId, seed, username);
      const prevRaw = await redis.get(guardKey);
      if (prevRaw && Number(prevRaw) >= score) {
        sendJson(res, 429, { ok: false, error: 'Already shared this score today' });
        return;
      }

      const bossName = BIOME_BOSSES[biome.index] ?? BIOME_BOSSES[0];
      const bossRaw = await redis.get(bossKillsKey(sub, seed));
      const bossKillsToday = bossRaw ? Math.max(0, Number(bossRaw) || 0) : 0;
      const filledBlocks = Math.max(0, Math.min(10, Math.round((bossKillsToday / SIEGE_TARGET) * 10)));
      const siegeBar = '█'.repeat(filledBlocks) + '░'.repeat(10 - filledBlocks);
      const factionScores = await getFactionScores(sub, seed);
      const [fp0, fp1, fp2] = computeFactionPercents(factionScores);
      const tomorrow = getUtcDateString(1);
      const tomorrowBiome = defaultBiomeFromSeed(tomorrow);
      const tomorrowVotes = await getMutatorVoteSummary(sub, tomorrow);
      const tomorrowMutName = DAILY_MODIFIERS[tomorrowVotes.leadingIndex];

      const bossBadge = bossSlain ? `💀 ${bossName} SLAIN` : '🛡️ Survived';
      const rankBadge = myRankVal > 0 ? ` · #${myRankVal} Today` : '';
      const streakBadge = ` · 🔥 ${streakVal}-Day Streak`;
      const lines = [
        `**╔══ 🔥 EMBERFALL: DAILY HORDE #${seed} ══╗**`,
        `🏆 **RANK ${rank}**${rankBadge}${streakBadge}`,
        `${heroInfo.emoji} **${heroInfo.title}** (*${heroInfo.faction}*) · Lv.**${level}** · **${score.toLocaleString('en-US')} PTS** · **${kills}** Kills`,
      ];
      if (chronicle) {
        lines.push(`📜 *Blood Chronicle:* "${chronicle}"`);
      }
      lines.push(
        `⚔️ Build & Bounties: ${buildEmojis}`,
        `🌍 Arena: **${biome.name}** (*${mutatorName}*) · ${bossBadge}`,
        `🏰 Subreddit Siege: \`[${siegeBar}]\` **${bossKillsToday}/${SIEGE_TARGET}** · ⚔️ Factions: ☀️ ${fp0}% | 🔮 ${fp1}% | 🗡️ ${fp2}%`,
        `🗳️ Tomorrow (*${tomorrowBiome.name}*): **${tomorrowMutName}** leading (${tomorrowVotes.totalVotes} votes)`,
        `**╚══** *Can your Faction beat my build? Enter the Abyss above!* **══╝**`,
      );
      const text = lines.join('\n\n');

      await reddit.submitComment({
        id: context.postId,
        text,
        runAs: 'USER',
      });
      await redis.set(guardKey, String(score));
      await redis.expire(guardKey, THIRTY_DAYS_SEC);
      await redis.hSet(postSharesKey(context.postId), { [username]: '1' });
      await redis.expire(postSharesKey(context.postId), THIRTY_DAYS_SEC);

      sendJson(res, 200, { ok: true });
      return;
    }

    sendJson(res, 404, { ok: false, error: 'Not found' });
  } catch (err) {
    sendJson(res, 500, { ok: false, error: err instanceof Error ? err.message : 'Server error' });
  }
});

server.listen(getServerPort());


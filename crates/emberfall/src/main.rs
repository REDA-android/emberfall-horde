mod assets;
mod rng;
mod shaders;
mod weapons;

use assets::GameAssets;
use macroquad::prelude::*;
use rng::{
    compute_rank, fever_multiplier, generate_barrel_positions, generate_daily_bounties,
    generate_shrine_positions, BountyKind, DailyBiome, DailyBounty, DailyModifier, Rng,
};
use shaders::GameShaders;
use weapons::{HeroBuild, HeroClass, RelicId, UpgradeId};

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn emberfall_get_day_number() -> u32;
    fn emberfall_play_sfx_pitched(kind: i32, pitch_mul: f32);
    fn emberfall_set_muted(muted: i32);
    fn emberfall_toggle_drone(active: i32);
    fn emberfall_set_boss_music(active: i32);
    fn emberfall_on_run_start();
    fn emberfall_on_run_complete(
        score: u32,
        kills: u32,
        boss_slain: i32,
        level: u32,
        hero_class: u32,
        build_ptr: *const u8,
        build_len: u32,
        ghost_ptr: *const i16,
        ghost_count: u32,
        is_endless: i32,
        endless_wave: u32,
        medals_mask: u32,
    );
    fn emberfall_share_comment(
        score: u32,
        kills: u32,
        boss_slain: i32,
        level: u32,
        hero_class: u32,
        build_ptr: *const u8,
        build_len: u32,
    );
    fn emberfall_vote_mutator(mutator_idx: i32);
    fn emberfall_set_selected_aura(aura_idx: i32);
    fn emberfall_poll_init_ready() -> i32;
    fn emberfall_get_init_field(field_id: i32) -> i32;
    fn emberfall_copy_str_field(
        field_id: i32,
        index: i32,
        out_ptr: *mut u8,
        max_len: u32,
    ) -> u32;
    fn emberfall_get_lb_score(index: i32) -> u32;
    fn emberfall_get_faction_score(index: i32) -> u32;
    fn emberfall_get_ghost_x(index: i32) -> i32;
    fn emberfall_get_ghost_y(index: i32) -> i32;
    fn emberfall_get_share_state() -> i32;
    fn emberfall_get_submit_state() -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_day_number() -> u32 {
    20261121
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_play_sfx_pitched(_kind: i32, _pitch_mul: f32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_set_muted(_muted: i32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_toggle_drone(_active: i32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_set_boss_music(_active: i32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_on_run_start() {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_on_run_complete(
    _score: u32,
    _kills: u32,
    _boss_slain: i32,
    _level: u32,
    _hero_class: u32,
    _build_ptr: *const u8,
    _build_len: u32,
    _ghost_ptr: *const i16,
    _ghost_count: u32,
    _is_endless: i32,
    _endless_wave: u32,
    _medals_mask: u32,
) {
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_share_comment(
    _score: u32,
    _kills: u32,
    _boss_slain: i32,
    _level: u32,
    _hero_class: u32,
    _build_ptr: *const u8,
    _build_len: u32,
) {
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_vote_mutator(_mutator_idx: i32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_set_selected_aura(_aura_idx: i32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_poll_init_ready() -> i32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_init_field(_field_id: i32) -> i32 {
    -1
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_copy_str_field(
    _field_id: i32,
    _index: i32,
    _out_ptr: *mut u8,
    _max_len: u32,
) -> u32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_lb_score(_index: i32) -> u32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_faction_score(_index: i32) -> u32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_ghost_x(_index: i32) -> i32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_ghost_y(_index: i32) -> i32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_share_state() -> i32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_submit_state() -> i32 {
    0
}

fn play_sfx(kind: i32) {
    unsafe {
        emberfall_play_sfx_pitched(kind, 1.0);
    }
}

fn play_sfx_pitched(kind: i32, pitch_mul: f32) {
    unsafe {
        emberfall_play_sfx_pitched(kind, pitch_mul);
    }
}

fn read_ffi_str(field_id: i32, index: i32) -> String {
    let mut buf = [0u8; 96];
    let len = unsafe { emberfall_copy_str_field(field_id, index, buf.as_mut_ptr(), buf.len() as u32) };
    let n = (len as usize).min(buf.len());
    String::from_utf8_lossy(&buf[..n]).into_owned()
}

/// Renders text with our embedded Open-Source TTF font (`ChakraPetch-Bold.ttf`, SIL OFL)
/// plus a dark drop shadow for crisp readability on mobile and desktop.
fn draw_ui_text(font: Option<&Font>, text: &str, x: f32, y: f32, size: f32, color: Color) {
    let font_size = size.round().max(8.0) as u16;
    draw_text_ex(
        text,
        x + 1.4,
        y + 1.4,
        TextParams {
            font,
            font_size,
            color: Color::new(0.0, 0.0, 0.0, color.a * 0.72),
            ..Default::default()
        },
    );
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font,
            font_size,
            color,
            ..Default::default()
        },
    );
}

fn measure_ui_text(font: Option<&Font>, text: &str, size: u16) -> TextDimensions {
    measure_text(text, font, size, 1.0)
}

const ARENA_HALF: f32 = 1050.0;
const RUN_DURATION: f32 = 90.0;
const BOSS_SPAWN_TIME: f32 = 55.0;

const MEDAL_NAMES: [&str; 6] = [
    "FIRST BLOOD",
    "LICH BANE",
    "SYNERGY",
    "BOUNTY",
    "COMBO GOD",
    "ETERNAL",
];
const MEDAL_SHORT: [&str; 6] = ["100K", "BOSS", "EVO", "3/3", "100x", "WAVE3"];
const AURA_NAMES: [&str; 4] = ["EMBER CORE", "SOLAR FLARE", "VOID ECLIPSE", "CELESTIAL"];

fn is_aura_unlocked(aura: usize, medals_mask: u32) -> bool {
    match aura {
        0 => true,
        1 => (medals_mask & ((1 << 0) | (1 << 1))) != 0,
        2 => (medals_mask & ((1 << 2) | (1 << 4))) != 0,
        3 => (medals_mask & ((1 << 3) | (1 << 5))) != 0,
        _ => false,
    }
}

fn aura_color(aura: usize) -> Color {
    match aura {
        1 => Color::from_rgba(250, 204, 21, 255),
        2 => Color::from_rgba(192, 132, 252, 255),
        3 => Color::from_rgba(56, 189, 248, 255),
        _ => Color::from_rgba(251, 146, 60, 255),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Title,
    Playing,
    Paused,
    LevelUp,
    GameOver,
    Victory,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnemyKind {
    Imp,
    Golem,
    Wraith,
    Boss,
}

struct Enemy {
    kind: EnemyKind,
    pos: Vec2,
    vel: Vec2,
    hp: f32,
    max_hp: f32,
    radius: f32,
    speed: f32,
    damage: f32,
    xp_value: u32,
    score_value: u32,
    hit_flash: f32,
    attack_cd: f32,
    facing_left: bool,
    phase_timer: f32,
    is_elite: bool,
    squash: f32,
    chill_timer: f32,
}

struct BloodShrine {
    pos: Vec2,
    progress: f32,
    completed: bool,
}

struct RunicBarrel {
    pos: Vec2,
    hp: f32,
    hit_flash: f32,
    exploded: bool,
}

struct RunicChest {
    pos: Vec2,
    relic: RelicId,
    bob_phase: f32,
}

struct ScorchMark {
    pos: Vec2,
    radius: f32,
    life: f32,
}

struct SoulObelisk {
    pos: Vec2,
    hp: f32,
    max_hp: f32,
    hit_flash: f32,
}

struct LeaderboardEntry {
    username: String,
    score: u32,
}

#[derive(Default)]
struct SocialState {
    init_loaded: bool,
    username: String,
    streak: i32,
    personal_best: i32,
    my_rank: i32,
    total_players: i32,
    boss_kills_today: i32,
    siege_target: i32,
    mutator_override: i32,
    can_vote_mutator: bool,
    leaderboard: Vec<LeaderboardEntry>,
    faction_scores: [u32; 3],
    yesterday_champion: String,
    yesterday_score: i32,
    ghost_username: String,
    ghost_points: Vec<Vec2>,
    last_status_msg: String,
    last_submit_state: i32,
    voted_mutator: Option<usize>,
    medals_mask: u32,
    best_endless_wave: u32,
    selected_aura: usize,
}

struct Projectile {
    pos: Vec2,
    vel: Vec2,
    damage: f32,
    radius: f32,
    life: f32,
    pierce: i32,
    from_enemy: bool,
    color: Color,
}

struct SlashArc {
    pos: Vec2,
    angle: f32,
    radius: f32,
    life: f32,
    max_life: f32,
    color: Color,
}

struct LightningArc {
    points: Vec<Vec2>,
    life: f32,
}

struct TelegraphRing {
    pos: Vec2,
    max_radius: f32,
    elapsed: f32,
    duration: f32,
    damage: f32,
}

struct XpGem {
    pos: Vec2,
    value: u32,
    magnetized: bool,
    gold: bool,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum VfxKind {
    Fire,
    Flame,
    Spark,
    Magic,
    Star,
    Light,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    size: f32,
    rot: f32,
    vrot: f32,
    life: f32,
    max_life: f32,
    color: Color,
    kind: VfxKind,
}

struct FloatText {
    pos: Vec2,
    text: String,
    color: Color,
    life: f32,
    scale: f32,
}

struct Afterimage {
    pos: Vec2,
    facing_left: bool,
    life: f32,
}

struct Game {
    phase: Phase,
    day_seed: String,
    modifier: DailyModifier,
    biome: DailyBiome,
    rng: Rng,

    // Hero & Class
    hero_class: HeroClass,
    hero_pos: Vec2,
    hero_vel: Vec2,
    hero_hp: f32,
    hero_max_hp: f32,
    hero_facing_left: bool,
    hero_invuln: f32,
    hero_stride: f32,
    hero_squash: f32,
    dash_cd: f32,
    dash_timer: f32,
    dash_dir: Vec2,
    build: HeroBuild,

    // Progression
    level: u32,
    xp: u32,
    xp_next: u32,
    pending_choices: [UpgradeId; 3],
    elapsed: f32,
    score: u32,
    kills: u32,
    combo: u32,
    combo_timer: f32,
    fever: f32,
    fever_idle_timer: f32,
    ult_charge: f32,
    ult_fx: f32,
    ult_pos: Vec2,
    boss_spawned: bool,
    boss_enraged: bool,
    boss_slain: bool,
    boss_banner_timer: f32,
    evo_banner_text: String,
    evo_banner_color: Color,
    evo_banner_timer: f32,
    reported_run: bool,
    endless_mode: bool,
    endless_wave: u32,
    next_endless_boss_time: f32,
    run_medals_mask: u32,

    // Shrines, Barrels & Ghost
    shrines: Vec<BloodShrine>,
    barrels: Vec<RunicBarrel>,
    shrine_score_mult: f32,
    ghost_record: Vec<i16>,
    ghost_sample_timer: f32,

    // Timers
    spawn_cd: f32,
    slash_cd: f32,
    bolt_cd: f32,
    lightning_cd: f32,
    orbit_angle: f32,
    orbit_hit_cd: f32,

    // World entities
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    slashes: Vec<SlashArc>,
    lightnings: Vec<LightningArc>,
    telegraphs: Vec<TelegraphRing>,
    gems: Vec<XpGem>,
    chests: Vec<RunicChest>,
    scorches: Vec<ScorchMark>,
    obelisks: Vec<SoulObelisk>,
    relics: Vec<RelicId>,
    bounties: [(DailyBounty, u32, bool); 3],
    barrel_detonations: u32,
    shrines_claimed: u32,
    chests_opened: u32,
    combo_milestone: u32,
    dmg_by_weapon: [f32; 4],
    particles: Vec<Particle>,
    float_texts: Vec<FloatText>,
    afterimages: Vec<Afterimage>,
    weather_timer: f32,

    // Camera, Juice, Audio & Social
    cam_pos: Vec2,
    trauma: f32,
    hitstop: f32,
    aberration: f32,
    shockwave: f32,
    muted: bool,
    joy_active: bool,
    joy_origin: Vec2,
    joy_curr: Vec2,
    joy_vec: Vec2,
    title_time: f32,
    social: SocialState,
}

impl Game {
    fn new() -> Self {
        let day_num = unsafe { emberfall_get_day_number() };
        let y = day_num / 10000;
        let m = (day_num / 100) % 100;
        let d = day_num % 100;
        let day_seed = format!("{:04}-{:02}-{:02}", y, m, d);
        let modifier = DailyModifier::from_seed(&day_seed);
        let biome = DailyBiome::from_seed(&day_seed);
        let rng = Rng::from_seed_str(&day_seed);
        let hero_class = HeroClass::SunKnight;
        let b_arr = generate_daily_bounties(&day_seed);

        Self {
            phase: Phase::Title,
            day_seed,
            modifier,
            biome,
            rng,
            hero_class,
            hero_pos: Vec2::ZERO,
            hero_vel: Vec2::ZERO,
            hero_hp: hero_class.base_max_hp(),
            hero_max_hp: hero_class.base_max_hp(),
            hero_facing_left: false,
            hero_invuln: 0.0,
            hero_stride: 0.0,
            hero_squash: 0.0,
            dash_cd: 0.0,
            dash_timer: 0.0,
            dash_dir: Vec2::X,
            build: HeroBuild::for_class(hero_class),
            level: 1,
            xp: 0,
            xp_next: 24,
            pending_choices: [
                UpgradeId::ArcaneSlash,
                UpgradeId::EmberBolts,
                UpgradeId::ChainLightning,
            ],
            elapsed: 0.0,
            score: 0,
            kills: 0,
            combo: 0,
            combo_timer: 0.0,
            fever: 0.0,
            fever_idle_timer: 0.0,
            ult_charge: 0.0,
            ult_fx: 0.0,
            ult_pos: Vec2::ZERO,
            boss_spawned: false,
            boss_enraged: false,
            boss_slain: false,
            boss_banner_timer: 0.0,
            evo_banner_text: String::new(),
            evo_banner_color: GOLD,
            evo_banner_timer: 0.0,
            reported_run: false,
            endless_mode: false,
            endless_wave: 1,
            next_endless_boss_time: BOSS_SPAWN_TIME + 55.0,
            run_medals_mask: 0,
            shrines: Vec::new(),
            barrels: Vec::new(),
            shrine_score_mult: 1.0,
            ghost_record: Vec::with_capacity(400),
            ghost_sample_timer: 0.0,
            spawn_cd: 0.4,
            slash_cd: 0.2,
            bolt_cd: 0.5,
            lightning_cd: 1.0,
            orbit_angle: 0.0,
            orbit_hit_cd: 0.0,
            enemies: Vec::new(),
            projectiles: Vec::new(),
            slashes: Vec::new(),
            lightnings: Vec::new(),
            telegraphs: Vec::new(),
            gems: Vec::new(),
            chests: Vec::new(),
            scorches: Vec::new(),
            obelisks: Vec::new(),
            relics: Vec::new(),
            bounties: [
                (b_arr[0], 0, false),
                (b_arr[1], 0, false),
                (b_arr[2], 0, false),
            ],
            barrel_detonations: 0,
            shrines_claimed: 0,
            chests_opened: 0,
            combo_milestone: 0,
            dmg_by_weapon: [0.0; 4],
            particles: Vec::new(),
            float_texts: Vec::new(),
            afterimages: Vec::new(),
            weather_timer: 0.0,
            cam_pos: Vec2::ZERO,
            trauma: 0.0,
            hitstop: 0.0,
            aberration: 0.0,
            shockwave: 0.0,
            muted: false,
            joy_active: false,
            joy_origin: Vec2::ZERO,
            joy_curr: Vec2::ZERO,
            joy_vec: Vec2::ZERO,
            title_time: 0.0,
            social: SocialState::default(),
        }
    }

    fn hero_texture<'a>(&self, assets: &'a GameAssets) -> &'a Texture2D {
        match self.hero_class {
            HeroClass::SunKnight => &assets.hero,
            HeroClass::ArcanePyromancer => &assets.hero_mage,
            HeroClass::ShadowAssassin => &assets.hero_rogue,
        }
    }

    fn poll_social_bridge(&mut self) {
        let ready = unsafe { emberfall_poll_init_ready() };
        let submit_state = unsafe { emberfall_get_submit_state() };
        let just_submitted = submit_state == 2 && self.social.last_submit_state != 2;
        self.social.last_submit_state = submit_state;

        if ready == 1 && (!self.social.init_loaded || just_submitted) {
            self.social.init_loaded = true;
            self.social.streak = unsafe { emberfall_get_init_field(0) }.max(0);
            self.social.personal_best = unsafe { emberfall_get_init_field(1) }.max(0);
            self.social.my_rank = unsafe { emberfall_get_init_field(2) }.max(0);
            self.social.total_players = unsafe { emberfall_get_init_field(3) }.max(0);
            self.social.boss_kills_today = unsafe { emberfall_get_init_field(4) }.max(0);
            self.social.siege_target = unsafe { emberfall_get_init_field(5) }.max(1);
            self.social.mutator_override = unsafe { emberfall_get_init_field(6) };
            self.social.can_vote_mutator = unsafe { emberfall_get_init_field(7) } == 1;
            self.social.yesterday_score = unsafe { emberfall_get_init_field(10) }.max(0);

            if self.social.mutator_override >= 0 && self.phase != Phase::Playing {
                self.modifier = DailyModifier::from_index(self.social.mutator_override as usize);
            }

            self.social.username = read_ffi_str(0, 0);
            self.social.ghost_username = read_ffi_str(2, 0);
            self.social.yesterday_champion = read_ffi_str(4, 0);

            for fi in 0..3 {
                self.social.faction_scores[fi] = unsafe { emberfall_get_faction_score(fi as i32) };
            }

            let lb_count = (unsafe { emberfall_get_init_field(8) }).clamp(0, 10) as usize;
            self.social.leaderboard.clear();
            for i in 0..lb_count {
                let uname = read_ffi_str(1, i as i32);
                let sc = unsafe { emberfall_get_lb_score(i as i32) };
                if !uname.is_empty() {
                    self.social.leaderboard.push(LeaderboardEntry {
                        username: uname,
                        score: sc,
                    });
                }
            }

            let ghost_count = (unsafe { emberfall_get_init_field(9) }).clamp(0, 200) as usize;
            self.social.ghost_points.clear();
            for i in 0..ghost_count {
                let gx = unsafe { emberfall_get_ghost_x(i as i32) } as f32;
                let gy = unsafe { emberfall_get_ghost_y(i as i32) } as f32;
                self.social.ghost_points.push(vec2(gx, gy));
            }

            let mm = unsafe { emberfall_get_init_field(11) };
            if mm >= 0 {
                self.social.medals_mask |= (mm as u32) & 63;
            }
            let bw = unsafe { emberfall_get_init_field(12) };
            if bw > 0 {
                self.social.best_endless_wave = self.social.best_endless_wave.max(bw as u32);
            }
            let sa = unsafe { emberfall_get_init_field(13) };
            if (0..=3).contains(&sa) && is_aura_unlocked(sa as usize, self.social.medals_mask) {
                self.social.selected_aura = sa as usize;
            }
        }
        self.social.last_status_msg = read_ffi_str(3, 0);
    }

    fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    fn start_run(&mut self, endless: bool) {
        self.rng = Rng::from_seed_str(&self.day_seed);
        self.biome = DailyBiome::from_seed(&self.day_seed);
        self.phase = Phase::Playing;
        self.endless_mode = endless;
        self.endless_wave = 1;
        self.next_endless_boss_time = BOSS_SPAWN_TIME + 55.0;
        self.run_medals_mask = 0;
        self.hero_pos = Vec2::ZERO;
        self.hero_vel = Vec2::ZERO;
        let max_hp = self.hero_class.base_max_hp();
        self.hero_max_hp = max_hp;
        self.hero_hp = max_hp;
        self.hero_invuln = 0.6;
        self.hero_facing_left = false;
        self.hero_squash = 0.0;
        self.dash_cd = 0.0;
        self.dash_timer = 0.0;
        self.build = HeroBuild::for_class(self.hero_class);
        self.level = 1;
        self.xp = 0;
        self.xp_next = 22;
        self.elapsed = 0.0;
        self.score = 0;
        self.kills = 0;
        self.combo = 0;
        self.combo_timer = 0.0;
        self.fever = 0.0;
        self.fever_idle_timer = 0.0;
        self.ult_charge = 0.0;
        self.ult_fx = 0.0;
        self.boss_spawned = false;
        self.boss_enraged = false;
        self.boss_slain = false;
        self.boss_banner_timer = 0.0;
        self.evo_banner_text.clear();
        self.evo_banner_timer = 0.0;
        self.reported_run = false;
        self.shrines = generate_shrine_positions(&self.day_seed)
            .into_iter()
            .map(|(x, y)| BloodShrine {
                pos: vec2(x, y),
                progress: 0.0,
                completed: false,
            })
            .collect();
        self.barrels = generate_barrel_positions(&self.day_seed)
            .into_iter()
            .map(|(x, y)| RunicBarrel {
                pos: vec2(x, y),
                hp: 55.0,
                hit_flash: 0.0,
                exploded: false,
            })
            .collect();
        self.shrine_score_mult = 1.0;
        self.ghost_record.clear();
        self.ghost_sample_timer = 0.0;
        self.spawn_cd = 0.25;
        self.slash_cd = 0.25;
        self.bolt_cd = 0.45;
        self.lightning_cd = 0.8;
        self.orbit_angle = 0.0;
        self.enemies.clear();
        self.projectiles.clear();
        self.slashes.clear();
        self.lightnings.clear();
        self.telegraphs.clear();
        self.gems.clear();
        self.chests.clear();
        self.scorches.clear();
        self.obelisks.clear();
        self.relics.clear();
        let b_arr = generate_daily_bounties(&self.day_seed);
        self.bounties = [
            (b_arr[0], 0, false),
            (b_arr[1], 0, false),
            (b_arr[2], 0, false),
        ];
        self.barrel_detonations = 0;
        self.shrines_claimed = 0;
        self.chests_opened = 0;
        self.combo_milestone = 0;
        self.dmg_by_weapon = [0.0; 4];
        self.particles.clear();
        self.float_texts.clear();
        self.afterimages.clear();
        self.weather_timer = 0.0;
        self.cam_pos = Vec2::ZERO;
        self.trauma = 0.0;
        self.hitstop = 0.0;
        self.aberration = 0.0;
        self.shockwave = 0.0;
        self.joy_active = false;
        self.joy_vec = Vec2::ZERO;

        unsafe {
            emberfall_on_run_start();
            emberfall_set_boss_music(0);
            emberfall_toggle_drone(if self.muted { 0 } else { 1 });
        }

        // Opening burst of enemies so action starts in second 1!
        for _ in 0..6 {
            self.spawn_enemy(EnemyKind::Imp, false);
        }
        play_sfx(0);
    }

    fn spawn_particle(
        &mut self,
        pos: Vec2,
        vel: Vec2,
        size: f32,
        life: f32,
        color: Color,
        kind: VfxKind,
    ) {
        if self.particles.len() > 260 {
            return;
        }
        let rot = self.rng.range_f32(0.0, std::f32::consts::TAU);
        let vrot = self.rng.range_f32(-4.0, 4.0);
        self.particles.push(Particle {
            pos,
            vel,
            size,
            rot,
            vrot,
            life,
            max_life: life,
            color,
            kind,
        });
    }

    fn spawn_burst(&mut self, pos: Vec2, count: usize, color: Color, kind: VfxKind, speed: f32) {
        for _ in 0..count {
            let ang = self.rng.range_f32(0.0, std::f32::consts::TAU);
            let spd = self.rng.range_f32(speed * 0.3, speed);
            let sz = self.rng.range_f32(18.0, 42.0);
            let life = self.rng.range_f32(0.22, 0.52);
            self.spawn_particle(pos, Vec2::from_angle(ang) * spd, sz, life, color, kind);
        }
    }

    fn spawn_enemy(&mut self, kind: EnemyKind, is_elite: bool) {
        let ang = self.rng.range_f32(0.0, std::f32::consts::TAU);
        let dist = match kind {
            EnemyKind::Boss => 460.0,
            _ if is_elite => 220.0,
            _ => self.rng.range_f32(430.0, 620.0),
        };
        let pos = (self.hero_pos + Vec2::from_angle(ang) * dist).clamp(
            Vec2::splat(-ARENA_HALF + 60.0),
            Vec2::splat(ARENA_HALF - 60.0),
        );

        let time_scale = 1.0 + (self.elapsed / 90.0) * 0.85;
        let speed_mult = if self.modifier == DailyModifier::BloodMoon {
            1.16
        } else {
            1.0
        };
        let elite_hp = if is_elite { 2.3 } else { 1.0 };
        let elite_spd = if is_elite { 1.22 } else { 1.0 };

        let (hp, radius, speed, damage, xp_value, score_value) = match kind {
            EnemyKind::Imp => (
                32.0 * time_scale * elite_hp,
                if is_elite { 28.0 } else { 22.0 },
                148.0 * speed_mult * elite_spd,
                12.0,
                if is_elite { 24 } else { 6 },
                if is_elite { 220 } else { 45 },
            ),
            EnemyKind::Golem => (
                125.0 * time_scale * elite_hp,
                if is_elite { 42.0 } else { 34.0 },
                88.0 * speed_mult * elite_spd,
                22.0,
                if is_elite { 48 } else { 18 },
                if is_elite { 450 } else { 140 },
            ),
            EnemyKind::Wraith => (
                60.0 * time_scale * elite_hp,
                if is_elite { 31.0 } else { 25.0 },
                118.0 * speed_mult * elite_spd,
                15.0,
                if is_elite { 36 } else { 12 },
                if is_elite { 340 } else { 95 },
            ),
            EnemyKind::Boss => {
                let boss_hp = if self.modifier == DailyModifier::TitansWrath {
                    2100.0
                } else {
                    1650.0
                };
                (boss_hp, 64.0, 112.0 * speed_mult, 28.0, 120, 4000)
            }
        };

        self.enemies.push(Enemy {
            kind,
            pos,
            vel: Vec2::ZERO,
            hp,
            max_hp: hp,
            radius,
            speed,
            damage,
            xp_value,
            score_value,
            hit_flash: 0.0,
            attack_cd: 1.4,
            facing_left: false,
            phase_timer: 0.0,
            is_elite,
            squash: 0.45,
            chill_timer: 0.0,
        });
    }

    fn trigger_dash(&mut self) {
        if self.dash_cd > 0.0 || self.phase != Phase::Playing {
            return;
        }
        let dir = if self.hero_vel.length_squared() > 10.0 {
            self.hero_vel.normalize()
        } else if self.hero_facing_left {
            -Vec2::X
        } else {
            Vec2::X
        };
        self.dash_dir = dir;
        self.dash_timer = 0.24;
        self.hero_invuln = self.hero_invuln.max(0.32);
        self.hero_squash = 0.36;
        self.dash_cd = (2.2 - (self.build.dash_lvl as f32 - 1.0) * 0.26).max(0.95);
        self.add_trauma(0.26);
        play_sfx(4);

        // Inferno Nova explosion on dash!
        let nova_r = 95.0 + (self.build.dash_lvl as f32) * 26.0;
        let nova_dmg = 34.0 + (self.build.dash_lvl as f32) * 18.0;
        self.spawn_burst(
            self.hero_pos,
            16,
            Color::from_rgba(251, 146, 60, 255),
            VfxKind::Flame,
            240.0,
        );
        self.damage_enemies_in_radius(self.hero_pos, nova_r, nova_dmg, 240.0);
    }

    fn trigger_ultimate(&mut self) {
        if self.ult_charge < 1.0 || self.phase != Phase::Playing {
            return;
        }
        self.ult_charge = 0.0;
        self.ult_fx = 1.0;
        self.ult_pos = self.hero_pos;
        self.hero_invuln = self.hero_invuln.max(1.0);
        self.add_trauma(1.0);
        self.hitstop = self.hitstop.max(0.12);
        self.shockwave = 1.0;
        self.aberration = 0.02;
        play_sfx(5);
        for (n, col) in [
            Color::from_rgba(253, 224, 71, 255),
            Color::from_rgba(251, 146, 60, 255),
            Color::from_rgba(244, 63, 94, 255),
        ]
        .iter()
        .enumerate()
        {
            self.spawn_burst(self.hero_pos, 40, *col, VfxKind::Flame, 380.0 + n as f32 * 160.0);
        }
        let dmg = 160.0 + (self.level as f32) * 14.0;
        self.damage_enemies_in_radius(self.hero_pos, 460.0, dmg, 520.0);
    }

    fn check_bounties(&mut self) {
        let mut newly_done: Vec<String> = Vec::new();
        for i in 0..3 {
            let (bounty, ref mut prog, ref mut done) = self.bounties[i];
            let new_p = match bounty.kind {
                BountyKind::DetonateBarrels => self.barrel_detonations,
                BountyKind::ClaimShrines => self.shrines_claimed,
                BountyKind::ReachCombo => (*prog).max(self.combo),
                BountyKind::OpenChests => self.chests_opened,
                BountyKind::SlayHorde => self.kills,
            };
            *prog = new_p;
            if !*done && *prog >= bounty.target {
                *done = true;
                newly_done.push(bounty.label(*prog));
            }
        }
        for lbl in newly_done {
            self.score += 750;
            self.shockwave = self.shockwave.max(0.85);
            play_sfx(5);
            self.float_texts.push(FloatText {
                pos: self.hero_pos + vec2(0.0, -44.0),
                text: format!("BOUNTY COMPLETE: {} (+750)!", lbl),
                color: Color::from_rgba(250, 204, 21, 255),
                life: 1.6,
                scale: 22.0,
            });
        }
        self.check_medals();
    }

    fn check_medals(&mut self) {
        let mut mask = self.run_medals_mask;
        if self.kills >= 100 {
            mask |= 1 << 0;
        }
        if self.boss_slain {
            mask |= 1 << 1;
        }
        if !self.build.evolutions().is_empty() {
            mask |= 1 << 2;
        }
        if self.bounties.iter().all(|(_, _, d)| *d) {
            mask |= 1 << 3;
        }
        if self.combo_milestone >= 100 {
            mask |= 1 << 4;
        }
        if (self.endless_mode && self.endless_wave >= 3) || self.score >= 20000 {
            mask |= 1 << 5;
        }

        let newly = mask & !self.run_medals_mask;
        if newly != 0 {
            self.run_medals_mask = mask;
            let brand_new = newly & !self.social.medals_mask;
            self.social.medals_mask |= mask;
            if brand_new != 0 {
                for bit in 0..6 {
                    if (brand_new & (1 << bit)) != 0 {
                        self.evo_banner_text = format!("MEDAL UNLOCKED: {}!", MEDAL_NAMES[bit]);
                        self.evo_banner_color = Color::from_rgba(250, 204, 21, 255);
                        self.evo_banner_timer = 2.8;
                        play_sfx(5);
                        break;
                    }
                }
            }
        }
    }

    fn ascend_to_endless(&mut self) {
        self.phase = Phase::Playing;
        self.endless_mode = true;
        self.endless_wave = 2;
        self.next_endless_boss_time = self.elapsed + 55.0;
        self.reported_run = false;
        self.hero_hp = (self.hero_hp + self.hero_max_hp * 0.35).min(self.hero_max_hp);
        self.evo_banner_text = "ASCENDED TO ENDLESS ABYSS · WAVE 2!".to_string();
        self.evo_banner_color = Color::from_rgba(192, 132, 252, 255);
        self.evo_banner_timer = 3.0;
        self.shockwave = 1.0;
        unsafe {
            emberfall_toggle_drone(if self.muted { 0 } else { 1 });
        }
        play_sfx(5);
    }

    fn damage_barrels_in_radius(&mut self, center: Vec2, radius: f32, dmg: f32) {
        let mut to_detonate: Vec<usize> = Vec::new();
        for (bi, b) in self.barrels.iter_mut().enumerate() {
            if b.exploded {
                continue;
            }
            if b.pos.distance(center) <= radius + 30.0 {
                b.hp -= dmg;
                b.hit_flash = 0.14;
                if b.hp <= 0.0 {
                    to_detonate.push(bi);
                }
            }
        }
        for bi in to_detonate {
            self.detonate_barrel(bi);
        }

        // Also damage active Boss Soul Obelisks
        let mut ob_i = 0;
        while ob_i < self.obelisks.len() {
            if self.obelisks[ob_i].pos.distance(center) <= radius + 34.0 {
                self.obelisks[ob_i].hp -= dmg;
                self.obelisks[ob_i].hit_flash = 0.14;
                if self.obelisks[ob_i].hp <= 0.0 {
                    let ob = self.obelisks.swap_remove(ob_i);
                    self.score += 600;
                    self.shockwave = self.shockwave.max(0.9);
                    self.add_trauma(0.55);
                    play_sfx(4);
                    self.spawn_burst(ob.pos, 26, Color::from_rgba(192, 132, 252, 255), VfxKind::Star, 260.0);
                    self.float_texts.push(FloatText {
                        pos: ob.pos + vec2(0.0, -32.0),
                        text: "SOUL OBELISK SHATTERED (+600)!".to_string(),
                        color: Color::from_rgba(192, 132, 252, 255),
                        life: 1.4,
                        scale: 22.0,
                    });
                    continue;
                }
            }
            ob_i += 1;
        }
    }

    fn detonate_barrel(&mut self, idx: usize) {
        if idx >= self.barrels.len() || self.barrels[idx].exploded {
            return;
        }
        self.barrels[idx].exploded = true;
        self.barrel_detonations += 1;
        let bpos = self.barrels[idx].pos;
        self.shockwave = self.shockwave.max(0.85);
        self.add_trauma(0.55);
        self.hitstop = self.hitstop.max(0.045);
        play_sfx(4);

        let (rad, dmg, col, label) = match self.biome {
            DailyBiome::ObsidianSanctuary => (
                175.0,
                165.0,
                Color::from_rgba(192, 132, 252, 255),
                "ARCANE BLAST!",
            ),
            DailyBiome::VolcanicRift => (
                228.0,
                145.0,
                Color::from_rgba(251, 146, 60, 255),
                "MAGMA ERUPTION!",
            ),
            DailyBiome::FrostAbyss => (
                185.0,
                125.0,
                Color::from_rgba(56, 189, 248, 255),
                "CRYO SHATTER!",
            ),
        };

        self.spawn_burst(bpos, 28, col, VfxKind::Flame, 310.0);
        self.spawn_burst(bpos, 16, Color::from_rgba(253, 224, 71, 255), VfxKind::Spark, 250.0);
        self.scorches.push(ScorchMark {
            pos: bpos,
            radius: rad * 0.72,
            life: 14.0,
        });
        self.float_texts.push(FloatText {
            pos: bpos + vec2(0.0, -32.0),
            text: label.to_string(),
            color: col,
            life: 1.1,
            scale: 22.0,
        });

        let is_frost = self.biome == DailyBiome::FrostAbyss;
        let mut hits: Vec<(usize, Vec2)> = Vec::new();
        for (i, e) in self.enemies.iter_mut().enumerate() {
            let delta = e.pos - bpos;
            let dist = delta.length();
            if dist <= rad + e.radius {
                if is_frost {
                    e.chill_timer = 3.5;
                }
                let dir = if dist > 1.0 { delta / dist } else { Vec2::X };
                hits.push((i, dir));
            }
        }
        for (ei, dir) in hits {
            self.apply_enemy_damage(ei, dmg, true, dir * 320.0);
        }

        // Chain-detonate nearby barrels!
        let mut chain: Vec<usize> = Vec::new();
        for (bi, b) in self.barrels.iter_mut().enumerate() {
            if !b.exploded && b.pos.distance(bpos) <= rad * 0.95 {
                b.hp = 0.0;
                chain.push(bi);
            }
        }
        for bi in chain {
            self.detonate_barrel(bi);
        }
        self.check_bounties();
    }

    fn damage_enemies_in_radius(&mut self, center: Vec2, radius: f32, base_dmg: f32, knockback: f32) -> usize {
        self.damage_barrels_in_radius(center, radius, base_dmg);
        let mut crit_chance = if self.modifier == DailyModifier::BloodMoon {
            0.35
        } else {
            0.14
        };
        if self.relics.contains(&RelicId::CritPrism) {
            crit_chance += 0.15;
        }
        let mut hits: Vec<(usize, f32, bool, Vec2)> = Vec::new();
        for (i, e) in self.enemies.iter().enumerate() {
            let delta = e.pos - center;
            let dist = delta.length();
            if dist <= radius + e.radius {
                let is_crit = self.rng.next_f32() < crit_chance;
                let dmg = if is_crit { base_dmg * 1.8 } else { base_dmg };
                let dir = if dist > 1.0 { delta / dist } else { Vec2::X };
                hits.push((i, dmg, is_crit, dir));
            }
        }
        let hit_count = hits.len();
        for (idx, dmg, is_crit, dir) in hits {
            self.apply_enemy_damage(idx, dmg, is_crit, dir * knockback);
        }
        hit_count
    }

    fn apply_enemy_damage(&mut self, idx: usize, mut dmg: f32, is_crit: bool, knock: Vec2) {
        if idx >= self.enemies.len() {
            return;
        }
        let has_cryo_core = self.relics.contains(&RelicId::CryoCore);
        let has_obelisk_shield = !self.obelisks.is_empty();
        let pos;
        let is_boss;
        let mut trigger_enrage = false;
        {
            let e = &mut self.enemies[idx];
            if has_cryo_core && e.chill_timer > 0.0 {
                dmg *= 1.25;
            }
            is_boss = e.kind == EnemyKind::Boss;
            if is_boss && has_obelisk_shield {
                dmg *= 0.55;
            }
            e.hp -= dmg;
            e.hit_flash = 0.12;
            e.squash = -0.22;
            if is_boss && !self.boss_enraged && e.hp > 0.0 && e.hp <= e.max_hp * 0.5 {
                trigger_enrage = true;
            }
            if !is_boss {
                e.vel += knock;
            } else {
                e.vel += knock * 0.12;
            }
            pos = e.pos;
        }

        if trigger_enrage {
            self.boss_enraged = true;
            self.evo_banner_text = "ARCH-LICH PHASE II — SHATTER SOUL OBELISKS!".to_string();
            self.evo_banner_color = Color::from_rgba(244, 63, 94, 255);
            self.evo_banner_timer = 3.4;
            self.shockwave = 1.0;
            self.add_trauma(0.85);
            play_sfx(6);
            self.spawn_burst(pos, 32, Color::from_rgba(244, 63, 94, 255), VfxKind::Flame, 320.0);
            for side in [-1.0f32, 1.0f32] {
                let ob_pos = vec2(
                    (pos.x + side * 210.0).clamp(-ARENA_HALF + 110.0, ARENA_HALF - 110.0),
                    (pos.y - 120.0).clamp(-ARENA_HALF + 110.0, ARENA_HALF - 110.0),
                );
                self.obelisks.push(SoulObelisk {
                    pos: ob_pos,
                    hp: 240.0,
                    max_hp: 240.0,
                    hit_flash: 0.0,
                });
                self.spawn_burst(ob_pos, 18, Color::from_rgba(192, 132, 252, 255), VfxKind::Magic, 210.0);
            }
        }

        self.float_texts.push(FloatText {
            pos: pos + vec2(self.rng.range_f32(-14.0, 14.0), -22.0),
            text: if is_crit {
                format!("{}!", dmg as u32)
            } else {
                format!("{}", dmg as u32)
            },
            color: if is_crit {
                Color::from_rgba(250, 204, 21, 255)
            } else {
                WHITE
            },
            life: 0.62,
            scale: if is_crit { 26.0 } else { 19.0 },
        });
        if is_crit {
            self.add_trauma(0.22);
            self.hitstop = self.hitstop.max(0.035);
        }
    }

    fn collect_dead_enemies(&mut self) {
        let score_mult = if self.modifier == DailyModifier::SwarmRush {
            1.3
        } else {
            1.0
        };
        let fever_mult = fever_multiplier(self.fever);
        let shrine_mult = self.shrine_score_mult;
        let has_soul_harvester = self.build.has_soul_harvester();
        let mut killed_any = false;
        let mut i = 0;
        while i < self.enemies.len() {
            if self.enemies[i].hp <= 0.0 {
                let dead = self.enemies.swap_remove(i);
                killed_any = true;
                self.kills += 1;
                self.combo += 1;
                self.combo_timer = 2.6;
                self.fever_idle_timer = 1.2;
                let fever_gain = if dead.kind == EnemyKind::Boss || dead.is_elite {
                    0.25
                } else {
                    0.08
                };
                self.fever = (self.fever + fever_gain).min(1.0);
                self.ult_charge = (self.ult_charge + fever_gain * 0.45).min(1.0);

                // Kill-Streak Announcer Milestones
                let milestone = if self.combo >= 100 && self.combo_milestone < 100 {
                    Some((100, "100x GODLIKE! (+1000 PTS)", 1000))
                } else if self.combo >= 50 && self.combo_milestone < 50 {
                    Some((50, "50x UNSTOPPABLE! (+500 PTS)", 500))
                } else if self.combo >= 25 && self.combo_milestone < 25 {
                    Some((25, "25x RAMPAGE! (+250 PTS)", 250))
                } else {
                    None
                };
                if let Some((m_val, m_txt, m_pts)) = milestone {
                    self.combo_milestone = m_val;
                    self.score += m_pts;
                    self.evo_banner_text = m_txt.to_string();
                    self.evo_banner_color = Color::from_rgba(250, 204, 21, 255);
                    self.evo_banner_timer = 2.4;
                    play_sfx(5);
                }

                // 5th Evolution: Soul Harvester (fires homing crimson Soul Bolt & heals +2 HP every 3rd kill)
                if has_soul_harvester && self.kills % 3 == 0 {
                    self.hero_hp = (self.hero_hp + 2.0).min(self.hero_max_hp);
                    for k in 0..2 {
                        let ang = (k as f32) * std::f32::consts::PI + self.elapsed * 3.0;
                        let dir = Vec2::from_angle(ang);
                        self.projectiles.push(Projectile {
                            pos: dead.pos,
                            vel: dir * 540.0,
                            damage: 48.0 + (self.level as f32) * 4.0,
                            radius: 13.0,
                            life: 1.25,
                            pierce: 3,
                            from_enemy: false,
                            color: Color::from_rgba(192, 132, 252, 255),
                        });
                    }
                }

                let combo_bonus = 1.0 + (self.combo.min(50) as f32) * 0.02;
                let mut pts = (dead.score_value as f32
                    * score_mult
                    * combo_bonus
                    * fever_mult
                    * shrine_mult) as u32;
                if dead.kind == EnemyKind::Boss {
                    if self.modifier == DailyModifier::TitansWrath {
                        pts = (pts as f32 * 1.5) as u32;
                    }
                    self.boss_slain = true;
                    self.obelisks.clear();
                    unsafe {
                        emberfall_set_boss_music(0);
                    }
                    self.add_trauma(0.95);
                    self.hitstop = self.hitstop.max(0.08);
                    self.shockwave = 1.0;
                    self.aberration = 0.018;
                    play_sfx(5);
                    self.spawn_burst(
                        dead.pos,
                        40,
                        Color::from_rgba(251, 191, 36, 255),
                        VfxKind::Fire,
                        340.0,
                    );
                    self.spawn_burst(
                        dead.pos,
                        28,
                        Color::from_rgba(56, 189, 248, 255),
                        VfxKind::Star,
                        290.0,
                    );
                    // Fountain of golden XP gems!
                    for _ in 0..14 {
                        let ang = self.rng.range_f32(0.0, std::f32::consts::TAU);
                        let dist = self.rng.range_f32(16.0, 95.0);
                        self.gems.push(XpGem {
                            pos: dead.pos + Vec2::from_angle(ang) * dist,
                            value: 14,
                            magnetized: true,
                            gold: true,
                        });
                    }
                } else {
                    play_sfx(2);
                    let col = if dead.is_elite {
                        Color::from_rgba(250, 204, 21, 255)
                    } else {
                        match dead.kind {
                            EnemyKind::Imp => Color::from_rgba(251, 146, 60, 255),
                            EnemyKind::Golem => Color::from_rgba(250, 204, 21, 255),
                            EnemyKind::Wraith => Color::from_rgba(232, 121, 249, 255),
                            EnemyKind::Boss => GOLD,
                        }
                    };
                    self.spawn_burst(dead.pos, if dead.is_elite { 18 } else { 9 }, col, VfxKind::Fire, 165.0);
                    let xp_val = if self.modifier == DailyModifier::ArcaneSurge {
                        dead.xp_value * 2
                    } else {
                        dead.xp_value
                    };
                    self.gems.push(XpGem {
                        pos: dead.pos,
                        value: xp_val,
                        magnetized: dead.is_elite,
                        gold: dead.is_elite || dead.kind == EnemyKind::Golem,
                    });
                }
                if self.relics.contains(&RelicId::VampiricFang)
                    && (dead.is_elite || dead.kind == EnemyKind::Boss || self.kills % 10 == 0)
                {
                    self.hero_hp = (self.hero_hp + 4.0).min(self.hero_max_hp);
                }
                if dead.is_elite || dead.kind == EnemyKind::Boss {
                    let relic = RelicId::from_index(self.rng.range_usize(0, 4));
                    self.chests.push(RunicChest {
                        pos: dead.pos,
                        relic,
                        bob_phase: self.elapsed,
                    });
                }
                self.score += pts;
            } else {
                i += 1;
            }
        }
        if killed_any {
            self.check_bounties();
        }
    }

    fn nearest_enemy_index(&self, from: Vec2, max_dist: f32) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (i, e) in self.enemies.iter().enumerate() {
            let d = e.pos.distance(from);
            if d <= max_dist && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        best.map(|(i, _)| i)
    }

    fn update_input(&mut self) -> Vec2 {
        let mut mv = Vec2::ZERO;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            mv.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            mv.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            mv.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            mv.x += 1.0;
        }
        if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Right) {
            self.trigger_dash();
        }
        if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::E) {
            self.trigger_ultimate();
        }

        // Dash button hitbox (bottom-right) & HUD button hitboxes (top-right)
        let sw = screen_width();
        let sh = screen_height();
        let dash_center = vec2(sw - 68.0, sh - 76.0);
        let dash_r = 44.0;
        let ult_center = vec2(sw - 150.0, sh - 56.0);
        let pause_rect = Rect::new(sw - 46.0, 16.0, 32.0, 28.0);
        let mute_rect = Rect::new(sw - 84.0, 16.0, 32.0, 28.0);

        // Touch / Mouse 1-thumb virtual joystick
        let mut active_pointer: Option<(Vec2, bool)> = None;
        let ts = touches();
        if !ts.is_empty() {
            for t in &ts {
                if !self.joy_active && (mute_rect.contains(t.position) || pause_rect.contains(t.position)) {
                    continue;
                }
                if t.phase == TouchPhase::Started && t.position.distance(ult_center) <= 38.0 {
                    self.trigger_ultimate();
                    continue;
                }
                if t.phase == TouchPhase::Started && t.position.distance(dash_center) <= dash_r + 14.0 {
                    self.trigger_dash();
                    continue;
                }
                if t.phase == TouchPhase::Started || t.phase == TouchPhase::Moved || t.phase == TouchPhase::Stationary {
                    active_pointer = Some((t.position, t.phase == TouchPhase::Started));
                    break;
                }
            }
        } else if is_mouse_button_down(MouseButton::Left) {
            let mp = vec2(mouse_position().0, mouse_position().1);
            let started = is_mouse_button_pressed(MouseButton::Left);
            if !self.joy_active && (mute_rect.contains(mp) || pause_rect.contains(mp)) {
                // Ignore HUD Mute/Pause button clicks so joystick does not activate
            } else if started && mp.distance(ult_center) <= 34.0 {
                self.trigger_ultimate();
            } else if started && mp.distance(dash_center) <= dash_r + 10.0 {
                self.trigger_dash();
            } else {
                active_pointer = Some((mp, started));
            }
        }

        if let Some((pos, started)) = active_pointer {
            if started || !self.joy_active {
                self.joy_active = true;
                self.joy_origin = pos;
                self.joy_curr = pos;
                self.joy_vec = Vec2::ZERO;
            } else {
                let delta = pos - self.joy_origin;
                let max_r = 56.0;
                let len = delta.length();
                if len > 4.0 {
                    let clamped = len.min(max_r);
                    let dir = delta / len;
                    self.joy_curr = self.joy_origin + dir * clamped;
                    self.joy_vec = dir * (clamped / max_r);
                } else {
                    self.joy_curr = pos;
                    self.joy_vec = Vec2::ZERO;
                }
            }
        } else {
            self.joy_active = false;
            self.joy_vec = Vec2::ZERO;
        }

        if self.joy_vec.length_squared() > 0.01 {
            mv = self.joy_vec;
        } else if mv.length_squared() > 1.0 {
            mv = mv.normalize();
        }
        mv
    }

    fn update_playing(&mut self, dt: f32) {
        self.elapsed += dt;
        self.hero_invuln = (self.hero_invuln - dt).max(0.0);
        self.dash_cd = (self.dash_cd - dt).max(0.0);
        self.boss_banner_timer = (self.boss_banner_timer - dt).max(0.0);
        self.evo_banner_timer = (self.evo_banner_timer - dt).max(0.0);
        self.trauma = (self.trauma - dt * 1.8).max(0.0);
        self.aberration = (self.aberration - dt * 0.08).max(0.0);
        self.shockwave = (self.shockwave - dt * 1.9).max(0.0);
        self.ult_fx = (self.ult_fx - dt * 1.4).max(0.0);
        self.hero_squash *= (1.0 - dt * 12.0).max(0.0);
        for b in &mut self.barrels {
            b.hit_flash = (b.hit_flash - dt).max(0.0);
        }
        for ob in &mut self.obelisks {
            ob.hit_flash = (ob.hit_flash - dt).max(0.0);
        }

        if self.combo_timer > 0.0 {
            self.combo_timer -= dt;
            if self.combo_timer <= 0.0 {
                self.combo = 0;
            }
        }
        if self.fever_idle_timer > 0.0 {
            self.fever_idle_timer -= dt;
        } else {
            self.fever = (self.fever - dt * 0.14).max(0.0);
        }

        // Ambient biome weather particles
        self.weather_timer -= dt;
        if self.weather_timer <= 0.0 {
            self.weather_timer = 0.09;
            let wx = self.cam_pos.x + self.rng.range_f32(-640.0, 640.0);
            let wy = self.cam_pos.y + self.rng.range_f32(-420.0, 420.0);
            let (wcol, wvel, wkind, wsize) = match self.biome {
                DailyBiome::VolcanicRift => (
                    Color::from_rgba(251, 146, 60, 145),
                    vec2(self.rng.range_f32(-18.0, 18.0), -58.0),
                    VfxKind::Flame,
                    self.rng.range_f32(12.0, 22.0),
                ),
                DailyBiome::FrostAbyss => (
                    Color::from_rgba(186, 230, 253, 155),
                    vec2(self.rng.range_f32(-22.0, 22.0), 46.0),
                    VfxKind::Star,
                    self.rng.range_f32(10.0, 18.0),
                ),
                DailyBiome::ObsidianSanctuary => (
                    Color::from_rgba(192, 132, 252, 140),
                    vec2(self.rng.range_f32(-16.0, 16.0), -34.0),
                    VfxKind::Light,
                    self.rng.range_f32(14.0, 24.0),
                ),
            };
            self.spawn_particle(vec2(wx, wy), wvel, wsize, 1.45, wcol, wkind);
        }

        // Record #1 Champion Ghost path every 0.5s (max 200 samples = 400 i16s)
        self.ghost_sample_timer += dt;
        if self.ghost_sample_timer >= 0.5 && self.ghost_record.len() < 400 {
            self.ghost_sample_timer -= 0.5;
            self.ghost_record.push(self.hero_pos.x as i16);
            self.ghost_record.push(self.hero_pos.y as i16);
        }

        // Check 90-second Daily Run completion!
        if self.elapsed >= RUN_DURATION {
            self.phase = Phase::Victory;
            self.score += 1500;
            unsafe {
                emberfall_set_boss_music(0);
                emberfall_toggle_drone(0);
            }
            self.report_run_once();
            play_sfx(5);
            return;
        }

        // Movement & Dash
        let input_dir = self.update_input();
        if self.dash_timer > 0.0 {
            self.dash_timer -= dt;
            self.hero_vel = self.dash_dir * 760.0;
            self.afterimages.push(Afterimage {
                pos: self.hero_pos,
                facing_left: self.hero_facing_left,
                life: 0.22,
            });
            self.spawn_particle(
                self.hero_pos,
                -self.dash_dir * 70.0,
                30.0,
                0.28,
                Color::from_rgba(56, 189, 248, 200),
                VfxKind::Flame,
            );
        } else {
            let target_vel = input_dir * self.build.move_speed();
            self.hero_vel = self.hero_vel.lerp(target_vel, (dt * 14.0).min(1.0));
        }

        if self.hero_vel.x.abs() > 12.0 {
            self.hero_facing_left = self.hero_vel.x < 0.0;
        }
        if self.hero_vel.length() > 25.0 {
            self.hero_stride += dt * 13.0;
        }

        self.hero_pos += self.hero_vel * dt;
        self.hero_pos = self.hero_pos.clamp(
            Vec2::splat(-ARENA_HALF + 38.0),
            Vec2::splat(ARENA_HALF - 38.0),
        );
        self.cam_pos = self.cam_pos.lerp(self.hero_pos, (dt * 8.5).min(1.0));

        // Blood Shrines channeling (2.0s -> Elite + 0.5x score mult + bonus upgrade)
        let mut shrine_reward_idx: Option<usize> = None;
        for (idx, shrine) in self.shrines.iter_mut().enumerate() {
            if shrine.completed {
                continue;
            }
            if self.hero_pos.distance(shrine.pos) <= 68.0 {
                shrine.progress += dt;
                if shrine.progress >= 2.0 {
                    shrine.completed = true;
                    shrine_reward_idx = Some(idx);
                    break;
                }
            } else {
                shrine.progress = (shrine.progress - dt * 0.5).max(0.0);
            }
        }
        if let Some(idx) = shrine_reward_idx {
            let spos = self.shrines[idx].pos;
            self.shrines_claimed += 1;
            self.shrine_score_mult += 0.5;
            self.shockwave = 1.0;
            self.add_trauma(0.65);
            self.spawn_burst(spos, 26, Color::from_rgba(244, 63, 94, 255), VfxKind::Fire, 260.0);
            self.float_texts.push(FloatText {
                pos: spos + vec2(0.0, -36.0),
                text: "SHRINE BLESSING +0.5x!".to_string(),
                color: Color::from_rgba(250, 204, 21, 255),
                life: 1.4,
                scale: 24.0,
            });
            self.check_bounties();
            let elite_kind = if self.rng.next_f32() < 0.5 {
                EnemyKind::Golem
            } else {
                EnemyKind::Wraith
            };
            self.spawn_enemy(elite_kind, true);
            self.pending_choices = self.build.roll_three(&mut self.rng);
            self.phase = Phase::LevelUp;
            self.joy_active = false;
            self.joy_vec = Vec2::ZERO;
            play_sfx(5);
            return;
        }

        // Enemy Spawning (deterministic daily waves)
        if !self.boss_spawned && self.elapsed >= BOSS_SPAWN_TIME {
            self.boss_spawned = true;
            self.boss_banner_timer = 3.2;
            self.add_trauma(0.75);
            self.hitstop = self.hitstop.max(0.075);
            self.shockwave = 1.0;
            self.spawn_enemy(EnemyKind::Boss, false);
            unsafe {
                emberfall_set_boss_music(1);
            }
            play_sfx(6);
        }

        self.spawn_cd -= dt;
        if self.spawn_cd <= 0.0 && self.enemies.len() < 75 {
            let density = if self.modifier == DailyModifier::SwarmRush {
                0.76
            } else {
                1.0
            };
            let base_cd = (0.88 - (self.elapsed / RUN_DURATION) * 0.46).max(0.32) * density;
            self.spawn_cd = base_cd;

            let pack = if self.elapsed > 40.0 { 3 } else if self.elapsed > 15.0 { 2 } else { 1 };
            for _ in 0..pack {
                let roll = self.rng.next_f32();
                let kind = if self.elapsed > 28.0 && roll < 0.16 {
                    EnemyKind::Golem
                } else if self.elapsed > 16.0 && roll < 0.36 {
                    EnemyKind::Wraith
                } else {
                    EnemyKind::Imp
                };
                self.spawn_enemy(kind, false);
            }
        }

        // WEAPONS (Auto-aim & Auto-fire scaled by Haste + Evolutions)
        let mut haste = self.build.haste_mult();
        if self.modifier == DailyModifier::ArcaneSurge {
            haste *= 1.2;
        }

        // 1. Arcane Slash (or Evolved: Crimson Reaper)
        self.slash_cd -= dt * haste;
        if self.slash_cd <= 0.0 && self.build.slash_lvl > 0 {
            let evolved = self.build.has_crimson_reaper();
            let search_r = if evolved { 380.0 } else { 320.0 };
            if let Some(target_idx) = self.nearest_enemy_index(self.hero_pos, search_r) {
                let target_pos = self.enemies[target_idx].pos;
                let dir = (target_pos - self.hero_pos).normalize_or_zero();
                let angle = dir.y.atan2(dir.x);
                let base_r = 92.0 + (self.build.slash_lvl as f32) * 18.0;
                let radius = if evolved { base_r * 1.42 } else { base_r };
                let base_dmg = 34.0 + (self.build.slash_lvl as f32) * 14.0;
                let dmg = if evolved { base_dmg * 1.48 } else { base_dmg };
                let slash_pos = self.hero_pos + dir * (radius * 0.48);
                let col = if evolved {
                    Color::from_rgba(244, 63, 94, 255)
                } else {
                    Color::from_rgba(64, 217, 255, 255)
                };

                self.slashes.push(SlashArc {
                    pos: slash_pos,
                    angle,
                    radius,
                    life: 0.22,
                    max_life: 0.22,
                    color: col,
                });
                let hits = self.damage_enemies_in_radius(slash_pos, radius * 0.85, dmg, 190.0);
                self.dmg_by_weapon[0] += (hits as f32) * dmg;
                if evolved && hits > 0 {
                    self.hero_hp = (self.hero_hp + 2.5).min(self.hero_max_hp);
                }
                self.spawn_burst(slash_pos, if evolved { 10 } else { 6 }, col, VfxKind::Spark, 180.0);
                play_sfx(1);
                self.slash_cd = (0.98 - (self.build.slash_lvl as f32 - 1.0) * 0.08).max(0.55);
            } else {
                self.slash_cd = 0.18;
            }
        }

        // 2. Ember Bolts (or Evolved: Meteor Barrage)
        self.bolt_cd -= dt * haste;
        if self.bolt_cd <= 0.0 && self.build.bolt_lvl > 0 {
            if let Some(target_idx) = self.nearest_enemy_index(self.hero_pos, 580.0) {
                let evolved = self.build.has_meteor_barrage();
                let base_dir = (self.enemies[target_idx].pos - self.hero_pos).normalize_or_zero();
                let count = self.build.bolt_lvl as usize + if evolved { 2 } else { 0 };
                let spread = if evolved { 0.14 } else { 0.16 };
                let base_dmg = 24.0 + (self.build.bolt_lvl as f32) * 9.0;
                for i in 0..count {
                    let offset = (i as f32 - (count as f32 - 1.0) * 0.5) * spread;
                    let dir = Vec2::from_angle(base_dir.y.atan2(base_dir.x) + offset);
                    self.projectiles.push(Projectile {
                        pos: self.hero_pos + dir * 24.0,
                        vel: dir * if evolved { 660.0 } else { 560.0 },
                        damage: if evolved { base_dmg * 1.55 } else { base_dmg },
                        radius: if evolved { 16.0 } else { 11.0 },
                        life: 1.35,
                        pierce: if evolved {
                            4
                        } else if self.build.bolt_lvl >= 3 {
                            2
                        } else {
                            1
                        },
                        from_enemy: false,
                        color: if evolved {
                            Color::from_rgba(244, 63, 94, 255)
                        } else {
                            Color::from_rgba(251, 146, 60, 255)
                        },
                    });
                }
                play_sfx(1);
                self.bolt_cd = (0.92 - (self.build.bolt_lvl as f32 - 1.0) * 0.07).max(0.5);
            } else {
                self.bolt_cd = 0.2;
            }
        }

        // 3. Chain Lightning (or Evolved: Celestial Tempest)
        self.lightning_cd -= dt * haste;
        if self.lightning_cd <= 0.0 && self.build.lightning_lvl > 0 {
            if let Some(first_idx) = self.nearest_enemy_index(self.hero_pos, 460.0) {
                let tempest = self.build.has_celestial_tempest();
                let jumps = 2 + (self.build.lightning_lvl as usize) * 2 + if tempest { 4 } else { 0 };
                let dmg = (28.0 + (self.build.lightning_lvl as f32) * 14.0) * if tempest { 1.35 } else { 1.0 };
                let mut visited = vec![first_idx];
                let mut pts = vec![self.hero_pos, self.enemies[first_idx].pos];
                let mut curr_pos = self.enemies[first_idx].pos;
                for _ in 1..jumps {
                    let mut next_best: Option<(usize, f32)> = None;
                    for (i, e) in self.enemies.iter().enumerate() {
                        if visited.contains(&i) {
                            continue;
                        }
                        let d = e.pos.distance(curr_pos);
                        if d < 250.0 && next_best.map_or(true, |(_, bd)| d < bd) {
                            next_best = Some((i, d));
                        }
                    }
                    if let Some((ni, _)) = next_best {
                        visited.push(ni);
                        curr_pos = self.enemies[ni].pos;
                        pts.push(curr_pos);
                    } else {
                        break;
                    }
                }
                self.dmg_by_weapon[2] += (visited.len() as f32) * dmg;
                for idx in visited {
                    let p = self.enemies[idx].pos;
                    self.apply_enemy_damage(idx, dmg, true, Vec2::ZERO);
                    self.spawn_burst(p, 5, Color::from_rgba(125, 211, 252, 255), VfxKind::Spark, 150.0);
                }
                self.lightnings.push(LightningArc { points: pts, life: 0.22 });
                play_sfx(3);
                self.lightning_cd = (2.0 - (self.build.lightning_lvl as f32 - 1.0) * 0.22).max(0.9);
            } else {
                self.lightning_cd = 0.25;
            }
        }

        // 4. Runic Orbit Blades (or Evolved: Inferno Vortex)
        if self.build.orbit_lvl > 0 {
            let evolved = self.build.has_inferno_vortex();
            let spin_mult = if evolved { 1.65 } else { 1.0 };
            self.orbit_angle += dt * (3.2 + (self.build.orbit_lvl as f32) * 0.45) * spin_mult;
            self.orbit_hit_cd -= dt;
            if self.orbit_hit_cd <= 0.0 {
                self.orbit_hit_cd = if evolved { 0.14 } else { 0.2 };
                let orb_count = self.build.orbit_lvl as usize + 1 + if evolved { 2 } else { 0 };
                let orb_r = 88.0 + (self.build.orbit_lvl as f32) * 8.0 + if evolved { 30.0 } else { 0.0 };
                let orb_dmg = (18.0 + (self.build.orbit_lvl as f32) * 8.0) * if evolved { 1.5 } else { 1.0 };
                for k in 0..orb_count {
                    let ang = self.orbit_angle + (k as f32) * (std::f32::consts::TAU / orb_count as f32);
                    let orb_pos = self.hero_pos + Vec2::from_angle(ang) * orb_r;
                    let hits = self.damage_enemies_in_radius(orb_pos, if evolved { 34.0 } else { 26.0 }, orb_dmg, 120.0);
                    self.dmg_by_weapon[3] += (hits as f32) * orb_dmg;
                }
            }
        }

        // Update Enemies & Boss AI
        let hero_p = self.hero_pos;
        let boss_enraged = self.boss_enraged;
        let mut enemy_shots: Vec<Projectile> = Vec::new();
        let mut new_telegraphs: Vec<TelegraphRing> = Vec::new();
        let mut hero_hit_dmg = 0.0f32;

        for e in &mut self.enemies {
            e.hit_flash = (e.hit_flash - dt).max(0.0);
            e.chill_timer = (e.chill_timer - dt).max(0.0);
            e.squash *= (1.0 - dt * 12.0).max(0.0);
            let chill_mult = if e.chill_timer > 0.0 { 0.58 } else { 1.0 };
            let eff_speed = e.speed * chill_mult;
            let to_hero = hero_p - e.pos;
            let dist = to_hero.length().max(0.001);
            let dir = to_hero / dist;
            e.facing_left = dir.x < 0.0;

            match e.kind {
                EnemyKind::Wraith => {
                    let desired = if dist > 260.0 { dir * eff_speed } else if dist < 170.0 { -dir * eff_speed * 0.6 } else { Vec2::ZERO };
                    e.vel = e.vel.lerp(desired, (dt * 6.0).min(1.0));
                    e.attack_cd -= dt;
                    if e.attack_cd <= 0.0 && dist < 440.0 {
                        e.attack_cd = 2.4;
                        enemy_shots.push(Projectile {
                            pos: e.pos,
                            vel: dir * 245.0,
                            damage: 14.0,
                            radius: 9.0,
                            life: 2.5,
                            pierce: 1,
                            from_enemy: true,
                            color: Color::from_rgba(232, 121, 249, 255),
                        });
                    }
                }
                EnemyKind::Boss => {
                    let p2_mult = if boss_enraged { 1.24 } else { 1.0 };
                    e.vel = e.vel.lerp(dir * eff_speed * p2_mult, (dt * 4.0).min(1.0));
                    e.attack_cd -= dt;
                    e.phase_timer += dt;
                    if e.attack_cd <= 0.0 {
                        e.attack_cd = if boss_enraged { 1.85 } else { 2.5 };
                        // Telegraph fiery slam at hero's position (+ second offset slam in Phase 2!)
                        new_telegraphs.push(TelegraphRing {
                            pos: hero_p,
                            max_radius: if boss_enraged { 138.0 } else { 125.0 },
                            elapsed: 0.0,
                            duration: if boss_enraged { 0.92 } else { 1.05 },
                            damage: if boss_enraged { 30.0 } else { 26.0 },
                        });
                        if boss_enraged {
                            new_telegraphs.push(TelegraphRing {
                                pos: hero_p + dir * 135.0,
                                max_radius: 115.0,
                                elapsed: 0.0,
                                duration: 0.95,
                                damage: 28.0,
                            });
                        }
                        // Spiral ember nova (12-way in Phase 2, 8-way in Phase 1)
                        let spokes = if boss_enraged { 12 } else { 8 };
                        let base_a = e.phase_timer * 1.4;
                        for k in 0..spokes {
                            let a = base_a + (k as f32) * (std::f32::consts::TAU / (spokes as f32));
                            enemy_shots.push(Projectile {
                                pos: e.pos,
                                vel: Vec2::from_angle(a) * if boss_enraged { 265.0 } else { 235.0 },
                                damage: 16.0,
                                radius: 10.0,
                                life: 2.8,
                                pierce: 1,
                                from_enemy: true,
                                color: if boss_enraged {
                                    Color::from_rgba(244, 63, 94, 255)
                                } else {
                                    Color::from_rgba(251, 146, 60, 255)
                                },
                            });
                        }
                    }
                }
                _ => {
                    e.vel = e.vel.lerp(dir * eff_speed, (dt * 7.0).min(1.0));
                }
            }

            e.pos += e.vel * dt;
            e.pos = e.pos.clamp(
                Vec2::splat(-ARENA_HALF + 30.0),
                Vec2::splat(ARENA_HALF - 30.0),
            );

            if dist < e.radius + 20.0 && self.hero_invuln <= 0.0 {
                hero_hit_dmg = hero_hit_dmg.max(e.damage);
            }
        }

        // Soft separation between enemies so hordes look crisp and readable
        let n = self.enemies.len();
        for i in 0..n {
            for j in (i + 1)..n.min(i + 10) {
                let d = self.enemies[j].pos - self.enemies[i].pos;
                let min_d = (self.enemies[i].radius + self.enemies[j].radius) * 0.72;
                let dist_sq = d.length_squared();
                if dist_sq > 0.01 && dist_sq < min_d * min_d {
                    let dist = dist_sq.sqrt();
                    let push = (d / dist) * (min_d - dist) * 0.45;
                    self.enemies[i].pos -= push;
                    self.enemies[j].pos += push;
                }
            }
        }

        self.projectiles.extend(enemy_shots);
        self.telegraphs.extend(new_telegraphs);

        // Update Telegraphed Boss Slams
        let mut t_i = 0;
        while t_i < self.telegraphs.len() {
            self.telegraphs[t_i].elapsed += dt;
            if self.telegraphs[t_i].elapsed >= self.telegraphs[t_i].duration {
                let ring = self.telegraphs.swap_remove(t_i);
                self.add_trauma(0.55);
                self.spawn_burst(
                    ring.pos,
                    22,
                    Color::from_rgba(244, 63, 94, 255),
                    VfxKind::Flame,
                    240.0,
                );
                self.scorches.push(ScorchMark {
                    pos: ring.pos,
                    radius: ring.max_radius * 0.72,
                    life: 14.0,
                });
                play_sfx(4);
                if self.hero_pos.distance(ring.pos) <= ring.max_radius && self.hero_invuln <= 0.0 {
                    hero_hit_dmg = hero_hit_dmg.max(ring.damage);
                }
            } else {
                t_i += 1;
            }
        }

        // Update Projectiles
        let mut p_i = 0;
        while p_i < self.projectiles.len() {
            let p_vel = self.projectiles[p_i].vel;
            self.projectiles[p_i].pos += p_vel * dt;
            self.projectiles[p_i].life -= dt;
            let p_pos = self.projectiles[p_i].pos;
            let p_rad = self.projectiles[p_i].radius;
            let from_enemy = self.projectiles[p_i].from_enemy;
            let p_dmg = self.projectiles[p_i].damage;

            if !from_enemy && self.rng.next_f32() < 0.45 {
                self.spawn_particle(
                    p_pos,
                    -p_vel * 0.12,
                    16.0,
                    0.18,
                    Color::from_rgba(251, 146, 60, 190),
                    VfxKind::Fire,
                );
            }

            let mut remove = self.projectiles[p_i].life <= 0.0;
            if !remove {
                if from_enemy {
                    if p_pos.distance(self.hero_pos) < p_rad + 18.0 && self.hero_invuln <= 0.0 {
                        hero_hit_dmg = hero_hit_dmg.max(p_dmg);
                        remove = true;
                    }
                } else {
                    // Check projectile hit against Runic Barrels
                    let mut hit_barrel: Option<usize> = None;
                    for (bi, b) in self.barrels.iter_mut().enumerate() {
                        if !b.exploded && b.pos.distance(p_pos) < 30.0 + p_rad {
                            b.hp -= p_dmg;
                            b.hit_flash = 0.14;
                            remove = true;
                            if b.hp <= 0.0 {
                                hit_barrel = Some(bi);
                            }
                            break;
                        }
                    }
                    if let Some(bi) = hit_barrel {
                        self.detonate_barrel(bi);
                    }
                    // Check projectile hit against Boss Soul Obelisks
                    if !remove {
                        for oi in 0..self.obelisks.len() {
                            if self.obelisks[oi].pos.distance(p_pos) < 34.0 + p_rad {
                                self.obelisks[oi].hp -= p_dmg;
                                self.obelisks[oi].hit_flash = 0.14;
                                remove = true;
                                if self.obelisks[oi].hp <= 0.0 {
                                    let ob = self.obelisks.swap_remove(oi);
                                    self.score += 600;
                                    self.shockwave = self.shockwave.max(0.9);
                                    self.add_trauma(0.55);
                                    play_sfx(4);
                                    self.spawn_burst(ob.pos, 26, Color::from_rgba(192, 132, 252, 255), VfxKind::Star, 260.0);
                                    self.float_texts.push(FloatText {
                                        pos: ob.pos + vec2(0.0, -32.0),
                                        text: "SOUL OBELISK SHATTERED (+600)!".to_string(),
                                        color: Color::from_rgba(192, 132, 252, 255),
                                        life: 1.4,
                                        scale: 22.0,
                                    });
                                }
                                break;
                            }
                        }
                    }
                    if !remove {
                        for ei in 0..self.enemies.len() {
                            if self.enemies[ei].pos.distance(p_pos) < self.enemies[ei].radius + p_rad {
                                let is_crit = self.rng.next_f32() < 0.16;
                                let dmg = if is_crit { p_dmg * 1.75 } else { p_dmg };
                                self.dmg_by_weapon[1] += dmg;
                                self.apply_enemy_damage(ei, dmg, is_crit, p_vel.normalize_or_zero() * 95.0);
                                self.spawn_burst(p_pos, 4, Color::from_rgba(251, 146, 60, 255), VfxKind::Spark, 120.0);
                                self.projectiles[p_i].pierce -= 1;
                                if self.projectiles[p_i].pierce <= 0 {
                                    remove = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if remove {
                self.projectiles.swap_remove(p_i);
            } else {
                p_i += 1;
            }
        }

        // Apply Hero Damage
        if hero_hit_dmg > 0.0 && self.hero_invuln <= 0.0 {
            self.hero_hp -= hero_hit_dmg;
            self.hero_invuln = 0.55;
            self.hero_squash = -0.28;
            self.add_trauma(0.6);
            self.hitstop = self.hitstop.max(0.055);
            self.aberration = 0.016;
            self.spawn_burst(
                self.hero_pos,
                14,
                Color::from_rgba(244, 63, 94, 255),
                VfxKind::Spark,
                200.0,
            );
            play_sfx(7);
            if self.hero_hp <= 0.0 {
                self.hero_hp = 0.0;
                self.phase = Phase::GameOver;
                unsafe {
                    emberfall_set_boss_music(0);
                    emberfall_toggle_drone(0);
                }
                self.report_run_once();
                return;
            }
        }

        // Collect dead enemies
        self.collect_dead_enemies();

        // Collect Runic Chests
        let mut c_i = 0;
        while c_i < self.chests.len() {
            if self.hero_pos.distance(self.chests[c_i].pos) < 42.0 {
                let chest = self.chests.swap_remove(c_i);
                self.chests_opened += 1;
                let relic = chest.relic;
                if !self.relics.contains(&relic) {
                    self.relics.push(relic);
                }
                self.score += 350;
                self.shockwave = self.shockwave.max(0.85);
                self.add_trauma(0.45);
                play_sfx(5);
                self.spawn_burst(chest.pos, 24, relic.accent(), VfxKind::Star, 260.0);
                self.float_texts.push(FloatText {
                    pos: chest.pos + vec2(0.0, -34.0),
                    text: relic.title().to_string(),
                    color: relic.accent(),
                    life: 1.5,
                    scale: 24.0,
                });
                // Global XP gem magnet pull on chest open (and especially MagnetOrb!)
                for g in &mut self.gems {
                    g.magnetized = true;
                }
                if relic == RelicId::MagnetOrb {
                    self.hero_hp = (self.hero_hp + 15.0).min(self.hero_max_hp);
                }
                self.check_bounties();
            } else {
                c_i += 1;
            }
        }

        // Update XP Gems (magnetic pull + combo pitch escalation)
        let pickup_r = self.build.pickup_radius()
            + if self.relics.contains(&RelicId::MagnetOrb) {
                70.0
            } else {
                0.0
            };
        let xp_mult = (1.0 + (self.build.magnet_lvl as f32) * 0.2) * fever_multiplier(self.fever);
        let mut g_i = 0;
        while g_i < self.gems.len() {
            let d = self.hero_pos - self.gems[g_i].pos;
            let dist = d.length();
            if dist < pickup_r {
                self.gems[g_i].magnetized = true;
            }
            if self.gems[g_i].magnetized && dist > 1.0 {
                self.gems[g_i].pos += (d / dist) * 520.0 * dt;
            }
            if dist < 24.0 {
                let gem = self.gems.swap_remove(g_i);
                self.xp += (gem.value as f32 * xp_mult) as u32;
                self.score += 15;
                let pitch = (1.0 + (self.combo.min(20) as f32) * 0.04).min(1.8);
                play_sfx_pitched(8, pitch);
                if self.xp >= self.xp_next {
                    self.xp -= self.xp_next;
                    self.level += 1;
                    self.xp_next = ((self.xp_next as f32) * 1.34) as u32 + 12;
                    self.pending_choices = self.build.roll_three(&mut self.rng);
                    self.phase = Phase::LevelUp;
                    self.shockwave = 1.0;
                    self.joy_active = false;
                    self.joy_vec = Vec2::ZERO;
                    play_sfx(5);
                    return;
                }
            } else {
                g_i += 1;
            }
        }

        // Update visual effects
        self.scorches.retain_mut(|s| {
            s.life -= dt;
            s.life > 0.0
        });
        self.slashes.retain_mut(|s| {
            s.life -= dt;
            s.life > 0.0
        });
        self.lightnings.retain_mut(|l| {
            l.life -= dt;
            l.life > 0.0
        });
        self.afterimages.retain_mut(|a| {
            a.life -= dt;
            a.life > 0.0
        });
        self.float_texts.retain_mut(|f| {
            f.pos.y -= 42.0 * dt;
            f.life -= dt;
            f.life > 0.0
        });
        self.particles.retain_mut(|p| {
            p.pos += p.vel * dt;
            p.vel *= 0.93;
            p.rot += p.vrot * dt;
            p.life -= dt;
            p.life > 0.0
        });
    }

    fn apply_upgrade_choice(&mut self, idx: usize) {
        let up = self.pending_choices[idx.min(2)];
        let prev = self.build.clone();
        self.build.apply(up);
        if up == UpgradeId::VampiricWard {
            self.hero_max_hp += 25.0;
            self.hero_hp = (self.hero_hp + 35.0).min(self.hero_max_hp);
        }
        if let Some(evo) = self.build.newly_unlocked_evolution(&prev) {
            self.evo_banner_text = format!("EVOLVED: {} {}!", evo.emoji(), evo.title());
            self.evo_banner_color = evo.accent();
            self.evo_banner_timer = 3.2;
            self.shockwave = 1.0;
            self.add_trauma(0.6);
            play_sfx(5);
        } else {
            play_sfx(0);
        }
        self.hero_invuln = self.hero_invuln.max(0.6);
        self.phase = Phase::Playing;
        self.spawn_burst(self.hero_pos, 18, up.accent(), VfxKind::Star, 220.0);
    }

    fn report_run_once(&mut self) {
        if self.reported_run {
            return;
        }
        self.reported_run = true;
        self.check_medals();
        if self.endless_mode {
            self.social.best_endless_wave = self.social.best_endless_wave.max(self.endless_wave);
        }
        let summary = self.build.build_summary();
        let bytes = summary.as_bytes();
        let ghost_count = (self.ghost_record.len() / 2) as u32;
        unsafe {
            emberfall_on_run_complete(
                self.score,
                self.kills,
                if self.boss_slain { 1 } else { 0 },
                self.level,
                self.hero_class.to_index() as u32,
                bytes.as_ptr(),
                bytes.len() as u32,
                self.ghost_record.as_ptr(),
                ghost_count,
                if self.endless_mode { 1 } else { 0 },
                self.endless_wave,
                self.run_medals_mask,
            );
        }
    }

    fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        unsafe {
            emberfall_set_muted(if self.muted { 1 } else { 0 });
            if self.muted {
                emberfall_toggle_drone(0);
            } else if self.phase == Phase::Playing {
                emberfall_toggle_drone(1);
            }
        }
    }

    fn draw_world(&self, assets: &GameAssets, shaders: &GameShaders, rt: Option<&RenderTarget>) {
        let sw = screen_width();
        let sh = screen_height();
        let font = assets.ui_font.as_ref();
        let hero_tex = self.hero_texture(assets);
        let shake_mag = 22.0 * self.trauma * self.trauma;
        let shake_off = if shake_mag > 0.1 {
            vec2(
                ((self.elapsed * 97.0).sin()) * shake_mag,
                ((self.elapsed * 131.0).cos()) * shake_mag,
            )
        } else {
            Vec2::ZERO
        };

        let cam = Camera2D {
            target: self.cam_pos + shake_off,
            zoom: vec2(2.0 / sw, -2.0 / sh),
            render_target: rt.cloned(),
            ..Default::default()
        };
        set_camera(&cam);
        if rt.is_some() {
            clear_background(Color::from_rgba(6, 9, 20, 255));
        }

        // 1. Seamless Basalt Dungeon Floor Tiling tinted by Daily Biome!
        let tile = 256.0;
        let min_x = ((self.cam_pos.x - sw * 0.65) / tile).floor() as i32;
        let max_x = ((self.cam_pos.x + sw * 0.65) / tile).ceil() as i32;
        let min_y = ((self.cam_pos.y - sh * 0.65) / tile).floor() as i32;
        let max_y = ((self.cam_pos.y + sh * 0.65) / tile).ceil() as i32;

        let floor_tint = match self.biome {
            DailyBiome::ObsidianSanctuary => Color::from_rgba(175, 175, 220, 255),
            DailyBiome::VolcanicRift => Color::from_rgba(225, 170, 145, 255),
            DailyBiome::FrostAbyss => Color::from_rgba(145, 205, 235, 255),
        };
        let border_col = match self.biome {
            DailyBiome::ObsidianSanctuary => Color::from_rgba(192, 132, 252, 200),
            DailyBiome::VolcanicRift => Color::from_rgba(249, 115, 22, 205),
            DailyBiome::FrostAbyss => Color::from_rgba(56, 189, 248, 205),
        };

        for tx in min_x..=max_x {
            for ty in min_y..=max_y {
                draw_texture_ex(
                    &assets.floor,
                    tx as f32 * tile,
                    ty as f32 * tile,
                    floor_tint,
                    DrawTextureParams {
                        dest_size: Some(vec2(tile, tile)),
                        ..Default::default()
                    },
                );
            }
        }

        // Arena Glowing Runic Border
        draw_rectangle_lines(
            -ARENA_HALF,
            -ARENA_HALF,
            ARENA_HALF * 2.0,
            ARENA_HALF * 2.0,
            8.0,
            border_col,
        );

        // 1a. Persistent Scorch Marks
        for sc in &self.scorches {
            let alpha = ((sc.life / 14.0).clamp(0.0, 1.0) * 115.0) as u8;
            draw_circle(
                sc.pos.x,
                sc.pos.y,
                sc.radius,
                Color::from_rgba(12, 10, 18, alpha),
            );
            draw_circle(
                sc.pos.x,
                sc.pos.y,
                sc.radius * 0.52,
                Color::from_rgba(30, 20, 15, (alpha as f32 * 0.75) as u8),
            );
        }

        // 1b. Blood Shrines
        for shrine in &self.shrines {
            let p = (shrine.progress / 2.0).clamp(0.0, 1.0);
            let ring_col = if shrine.completed {
                Color::from_rgba(100, 116, 139, 90)
            } else {
                Color::from_rgba(244, 63, 94, 210)
            };
            draw_circle(
                shrine.pos.x,
                shrine.pos.y,
                64.0,
                if shrine.completed {
                    Color::from_rgba(30, 41, 59, 60)
                } else {
                    Color::from_rgba(244, 63, 94, 35)
                },
            );
            if !shrine.completed && p > 0.01 {
                draw_circle(
                    shrine.pos.x,
                    shrine.pos.y,
                    64.0 * p,
                    Color::from_rgba(244, 63, 94, 95),
                );
            }
            draw_circle_lines(shrine.pos.x, shrine.pos.y, 64.0, 3.0, ring_col);
            let lbl = if shrine.completed {
                "SHRINE CLAIMED"
            } else if p > 0.05 {
                "CHANNELING..."
            } else {
                "BLOOD SHRINE"
            };
            let ld = measure_ui_text(font, lbl, 14);
            draw_ui_text(
                font,
                lbl,
                shrine.pos.x - ld.width * 0.5,
                shrine.pos.y - 74.0,
                14.0,
                ring_col,
            );
        }

        // 1c. Runic Explosive Barrels (Real illustrated RGBA sprite!)
        for barrel in &self.barrels {
            if barrel.exploded {
                draw_circle(
                    barrel.pos.x,
                    barrel.pos.y,
                    26.0,
                    Color::from_rgba(15, 23, 42, 125),
                );
                continue;
            }
            let pulse = 1.0 + (self.elapsed * 5.0 + barrel.pos.x * 0.02).sin() * 0.04;
            let bsz = 64.0 * pulse;
            draw_ellipse(
                barrel.pos.x,
                barrel.pos.y + 20.0,
                20.0,
                8.0,
                0.0,
                Color::from_rgba(0, 0, 0, 125),
            );
            if barrel.hit_flash > 0.0 {
                if let Some(mat) = &shaders.hit_flash {
                    mat.set_uniform("u_flash", (barrel.hit_flash / 0.14).clamp(0.0, 1.0));
                    gl_use_material(mat);
                }
            }
            draw_texture_ex(
                &assets.barrel,
                barrel.pos.x - bsz * 0.5,
                barrel.pos.y - bsz * 0.5,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(bsz, bsz)),
                    ..Default::default()
                },
            );
            if barrel.hit_flash > 0.0 {
                gl_use_default_material();
            }
        }

        // 1d. Runic Treasure Chests (Elite/Boss Drops)
        for chest in &self.chests {
            let bob = ((self.elapsed - chest.bob_phase) * 4.5).sin() * 4.0;
            let glow_sz = 92.0 + ((self.elapsed * 6.0).sin() * 8.0);
            draw_texture_ex(
                &assets.vfx_light,
                chest.pos.x - glow_sz * 0.5,
                chest.pos.y - glow_sz * 0.5 + bob,
                Color::from_rgba(250, 204, 21, 145),
                DrawTextureParams {
                    dest_size: Some(vec2(glow_sz, glow_sz)),
                    ..Default::default()
                },
            );
            draw_ellipse(
                chest.pos.x,
                chest.pos.y + 20.0,
                22.0,
                8.0,
                0.0,
                Color::from_rgba(0, 0, 0, 130),
            );
            draw_texture_ex(
                &assets.chest,
                chest.pos.x - 27.0,
                chest.pos.y - 27.0 + bob,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(54.0, 54.0)),
                    ..Default::default()
                },
            );
        }

        // 1e. Arch-Lich Phase II Soul Obelisks + Shield Tether Beams
        let boss_pos_opt = self
            .enemies
            .iter()
            .find(|e| e.kind == EnemyKind::Boss)
            .map(|e| e.pos);
        for ob in &self.obelisks {
            if let Some(bp) = boss_pos_opt {
                draw_line(
                    ob.pos.x,
                    ob.pos.y,
                    bp.x,
                    bp.y,
                    4.0,
                    Color::from_rgba(192, 132, 252, 190),
                );
                draw_line(
                    ob.pos.x,
                    ob.pos.y,
                    bp.x,
                    bp.y,
                    1.8,
                    Color::from_rgba(250, 204, 21, 220),
                );
            }
            let pulse = 32.0 + (self.elapsed * 6.0).sin() * 4.0;
            draw_circle(ob.pos.x, ob.pos.y, pulse, Color::from_rgba(192, 132, 252, 70));
            draw_poly(
                ob.pos.x,
                ob.pos.y,
                6,
                26.0,
                self.elapsed * 35.0,
                if ob.hit_flash > 0.0 {
                    WHITE
                } else {
                    Color::from_rgba(147, 51, 234, 240)
                },
            );
            draw_poly_lines(
                ob.pos.x,
                ob.pos.y,
                6,
                28.0,
                self.elapsed * 35.0,
                2.5,
                Color::from_rgba(250, 204, 21, 240),
            );
            let pct = (ob.hp / ob.max_hp.max(1.0)).clamp(0.0, 1.0);
            draw_rectangle(ob.pos.x - 24.0, ob.pos.y - 38.0, 48.0, 5.0, Color::from_rgba(15, 23, 42, 220));
            draw_rectangle(
                ob.pos.x - 24.0,
                ob.pos.y - 38.0,
                48.0 * pct,
                5.0,
                Color::from_rgba(192, 132, 252, 255),
            );
        }

        // 2. Telegraphed Boss Rings (outer warning + inner filling progress circle)
        for ring in &self.telegraphs {
            let p = (ring.elapsed / ring.duration).clamp(0.0, 1.0);
            draw_circle(
                ring.pos.x,
                ring.pos.y,
                ring.max_radius,
                Color::from_rgba(244, 63, 94, 24),
            );
            draw_circle(
                ring.pos.x,
                ring.pos.y,
                ring.max_radius * p,
                Color::from_rgba(244, 63, 94, (65.0 + 95.0 * p) as u8),
            );
            draw_circle_lines(
                ring.pos.x,
                ring.pos.y,
                ring.max_radius * p,
                2.0,
                Color::from_rgba(253, 164, 175, 210),
            );
            draw_circle_lines(
                ring.pos.x,
                ring.pos.y,
                ring.max_radius,
                3.5,
                Color::from_rgba(244, 63, 94, 235),
            );
        }

        // 3. XP Gems (glowing Kenney star + magic)
        for gem in &self.gems {
            let col = if gem.gold {
                Color::from_rgba(250, 204, 21, 255)
            } else {
                Color::from_rgba(56, 189, 248, 255)
            };
            let sz = if gem.gold { 28.0 } else { 22.0 };
            draw_texture_ex(
                &assets.vfx_star,
                gem.pos.x - sz * 0.5,
                gem.pos.y - sz * 0.5,
                col,
                DrawTextureParams {
                    dest_size: Some(vec2(sz, sz)),
                    rotation: self.elapsed * 3.0,
                    ..Default::default()
                },
            );
        }

        // 3b. #1 Champion Ghost Replay (translucent golden spectral phantom)
        if !self.social.ghost_points.is_empty()
            && matches!(self.phase, Phase::Playing | Phase::Paused | Phase::LevelUp)
        {
            let pts = &self.social.ghost_points;
            let idx_f = (self.elapsed / 0.5).max(0.0);
            let i0 = (idx_f.floor() as usize).min(pts.len() - 1);
            let i1 = (i0 + 1).min(pts.len() - 1);
            let frac = idx_f - (i0 as f32);
            let gpos = pts[i0].lerp(pts[i1], frac);
            let gflip = pts[i1].x < pts[i0].x;
            draw_texture_ex(
                &assets.vfx_light,
                gpos.x - 52.0,
                gpos.y - 52.0,
                Color::new(1.0, 0.85, 0.25, 0.25),
                DrawTextureParams {
                    dest_size: Some(vec2(104.0, 104.0)),
                    ..Default::default()
                },
            );
            draw_texture_ex(
                &assets.hero,
                gpos.x - 42.0,
                gpos.y - 42.0,
                Color::new(1.0, 0.86, 0.28, 0.42),
                DrawTextureParams {
                    dest_size: Some(vec2(84.0, 84.0)),
                    flip_x: gflip,
                    ..Default::default()
                },
            );
            let glbl = if self.social.ghost_username.is_empty() {
                "u/Champion".to_string()
            } else {
                format!("u/{}", self.social.ghost_username)
            };
            let gd = measure_ui_text(font, &glbl, 14);
            draw_ui_text(
                font,
                &glbl,
                gpos.x - gd.width * 0.5,
                gpos.y - 48.0,
                14.0,
                Color::new(1.0, 0.88, 0.35, 0.85),
            );
        }

        // 4. Hero Aura & Dash Afterimages
        let aura_col = self.hero_class.accent();
        let aura_sz = 110.0 + (self.elapsed * 4.0).sin() * 6.0;
        draw_texture_ex(
            &assets.vfx_light,
            self.hero_pos.x - aura_sz * 0.5,
            self.hero_pos.y - aura_sz * 0.5,
            Color::new(aura_col.r, aura_col.g, aura_col.b, 0.26),
            DrawTextureParams {
                dest_size: Some(vec2(aura_sz, aura_sz)),
                ..Default::default()
            },
        );

        if self.ult_fx > 0.0 {
            let t = 1.0 - self.ult_fx;
            for k in 0..3 {
                let tk = (t - k as f32 * 0.08).max(0.0);
                let r = tk * 520.0;
                let a = ((1.0 - tk) * 255.0) as u8;
                if r > 1.0 {
                    draw_circle_lines(
                        self.ult_pos.x,
                        self.ult_pos.y,
                        r,
                        10.0 - k as f32 * 3.0,
                        Color::from_rgba(253, 224 - k as u8 * 60, 71 + k as u8 * 40, a),
                    );
                }
            }
            draw_circle(
                self.ult_pos.x,
                self.ult_pos.y,
                120.0 * self.ult_fx,
                Color::from_rgba(255, 244, 200, (self.ult_fx * 170.0) as u8),
            );
        }
        for a in &self.afterimages {
            let alpha = (a.life / 0.22).clamp(0.0, 1.0) * 0.55;
            draw_texture_ex(
                hero_tex,
                a.pos.x - 42.0,
                a.pos.y - 42.0,
                Color::new(aura_col.r, aura_col.g, aura_col.b, alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(84.0, 84.0)),
                    flip_x: a.facing_left,
                    ..Default::default()
                },
            );
        }

        // 5. Enemies (Real Illustrated Sprites + squash/stretch + GLSL100 hit flash!)
        for e in &self.enemies {
            let (tex, base_sz) = match e.kind {
                EnemyKind::Imp => (&assets.imp, 64.0),
                EnemyKind::Golem => (&assets.golem, 94.0),
                EnemyKind::Wraith => (&assets.wraith, 76.0),
                EnemyKind::Boss => (&assets.boss, 186.0),
            };
            let draw_sz = if e.is_elite { base_sz * 1.24 } else { base_sz };
            let sx = (1.0 + e.squash).clamp(0.65, 1.45);
            let sy = (1.0 - e.squash).clamp(0.65, 1.45);
            let dw = draw_sz * sx;
            let dh = draw_sz * sy;
            let bob = ((self.elapsed * 8.0 + e.pos.x * 0.05).sin()) * 3.5;

            // Ground shadow
            draw_ellipse(
                e.pos.x,
                e.pos.y + draw_sz * 0.34,
                dw * 0.32,
                draw_sz * 0.12,
                0.0,
                Color::from_rgba(0, 0, 0, 115),
            );

            if e.kind == EnemyKind::Boss && self.boss_enraged {
                draw_circle_lines(
                    e.pos.x,
                    e.pos.y + 8.0,
                    draw_sz * 0.52,
                    3.5,
                    Color::from_rgba(244, 63, 94, 225),
                );
            } else if e.is_elite {
                draw_circle_lines(
                    e.pos.x,
                    e.pos.y + draw_sz * 0.15,
                    draw_sz * 0.44,
                    2.5,
                    Color::from_rgba(250, 204, 21, 190),
                );
            }

            let tint = if e.chill_timer > 0.0 {
                Color::from_rgba(125, 211, 252, 255)
            } else if e.kind == EnemyKind::Boss && self.boss_enraged {
                Color::from_rgba(255, 185, 190, 255)
            } else if e.is_elite {
                Color::from_rgba(255, 224, 150, 255)
            } else {
                Color::from_rgba(242, 242, 248, 255)
            };

            if e.hit_flash > 0.0 {
                if let Some(mat) = &shaders.hit_flash {
                    mat.set_uniform("u_flash", (e.hit_flash / 0.12).clamp(0.0, 1.0));
                    gl_use_material(mat);
                }
            }
            draw_texture_ex(
                tex,
                e.pos.x - dw * 0.5,
                e.pos.y - dh * 0.5 + bob,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(dw, dh)),
                    flip_x: e.facing_left,
                    ..Default::default()
                },
            );
            if e.hit_flash > 0.0 {
                gl_use_default_material();
            }

            // Mini HP bar for damaged elites/golems/wraiths
            if (e.is_elite || (e.kind != EnemyKind::Imp && e.kind != EnemyKind::Boss))
                && e.hp < e.max_hp
            {
                let bw = draw_sz * 0.65;
                let pct = (e.hp / e.max_hp).clamp(0.0, 1.0);
                draw_rectangle(
                    e.pos.x - bw * 0.5,
                    e.pos.y - draw_sz * 0.56,
                    bw,
                    5.0,
                    Color::from_rgba(0, 0, 0, 180),
                );
                draw_rectangle(
                    e.pos.x - bw * 0.5,
                    e.pos.y - draw_sz * 0.56,
                    bw * pct,
                    5.0,
                    Color::from_rgba(244, 63, 94, 240),
                );
            }
        }

        // 6. Hero Sprite (Selected Class Sprite + squash & stretch!)
        let hero_bob = self.hero_stride.sin().abs() * -5.0;
        let hsx = (1.0 + self.hero_squash).clamp(0.7, 1.4);
        let hsy = (1.0 - self.hero_squash).clamp(0.7, 1.4);
        let hw = 92.0 * hsx;
        let hh = 92.0 * hsy;
        draw_ellipse(
            self.hero_pos.x,
            self.hero_pos.y + 30.0,
            28.0 * hsx,
            11.0,
            0.0,
            Color::from_rgba(0, 0, 0, 130),
        );
        let hero_tint = if self.hero_invuln > 0.0 && ((self.elapsed * 24.0) as i32) % 2 == 0 {
            Color::from_rgba(255, 160, 175, 240)
        } else {
            WHITE
        };
        draw_texture_ex(
            hero_tex,
            self.hero_pos.x - hw * 0.5,
            self.hero_pos.y - hh * 0.5 + hero_bob,
            hero_tint,
            DrawTextureParams {
                dest_size: Some(vec2(hw, hh)),
                flip_x: self.hero_facing_left,
                ..Default::default()
            },
        );

        // 7. Runic Orbit Blades (or Evolved Inferno Vortex)
        if self.build.orbit_lvl > 0 {
            let evolved = self.build.has_inferno_vortex();
            let orb_count = self.build.orbit_lvl as usize + 1 + if evolved { 2 } else { 0 };
            let orb_r = 88.0 + (self.build.orbit_lvl as f32) * 8.0 + if evolved { 30.0 } else { 0.0 };
            let orb_sz = if evolved { 62.0 } else { 52.0 };
            let orb_col = if evolved {
                Color::from_rgba(251, 146, 60, 250)
            } else {
                Color::from_rgba(232, 121, 249, 240)
            };
            for k in 0..orb_count {
                let ang = self.orbit_angle + (k as f32) * (std::f32::consts::TAU / orb_count as f32);
                let p = self.hero_pos + Vec2::from_angle(ang) * orb_r;
                draw_texture_ex(
                    &assets.vfx_magic,
                    p.x - orb_sz * 0.5,
                    p.y - orb_sz * 0.5,
                    orb_col,
                    DrawTextureParams {
                        dest_size: Some(vec2(orb_sz, orb_sz)),
                        rotation: -self.orbit_angle * 2.5,
                        ..Default::default()
                    },
                );
            }
        }

        // 8. Arcane Slash Arcs (Kenney slash_01)
        for s in &self.slashes {
            let a = (s.life / s.max_life).clamp(0.0, 1.0);
            let sz = s.radius * 2.15;
            draw_texture_ex(
                &assets.vfx_slash,
                s.pos.x - sz * 0.5,
                s.pos.y - sz * 0.5,
                Color::new(s.color.r, s.color.g, s.color.b, a),
                DrawTextureParams {
                    dest_size: Some(vec2(sz, sz)),
                    rotation: s.angle + std::f32::consts::FRAC_PI_4,
                    ..Default::default()
                },
            );
        }

        // 9. Projectiles (Kenney fire_01 / magic_05)
        for p in &self.projectiles {
            let tex = if p.from_enemy { &assets.vfx_magic } else { &assets.vfx_fire };
            let sz = p.radius * 3.8;
            draw_texture_ex(
                tex,
                p.pos.x - sz * 0.5,
                p.pos.y - sz * 0.5,
                p.color,
                DrawTextureParams {
                    dest_size: Some(vec2(sz, sz)),
                    rotation: p.vel.y.atan2(p.vel.x),
                    ..Default::default()
                },
            );
        }

        // 10. Chain Lightning Bolts
        for l in &self.lightnings {
            let a = (l.life / 0.22).clamp(0.0, 1.0);
            for w in l.points.windows(2) {
                draw_line(
                    w[0].x,
                    w[0].y,
                    w[1].x,
                    w[1].y,
                    6.0,
                    Color::new(0.22, 0.74, 0.97, a * 0.55),
                );
                draw_line(
                    w[0].x,
                    w[0].y,
                    w[1].x,
                    w[1].y,
                    2.4,
                    Color::new(0.92, 0.98, 1.0, a),
                );
            }
        }

        // 11. Kenney CC0 Particles
        for p in &self.particles {
            let a = (p.life / p.max_life).clamp(0.0, 1.0);
            let tex = match p.kind {
                VfxKind::Fire => &assets.vfx_fire,
                VfxKind::Flame => &assets.vfx_flame,
                VfxKind::Spark => &assets.vfx_spark,
                VfxKind::Magic => &assets.vfx_magic,
                VfxKind::Star => &assets.vfx_star,
                VfxKind::Light => &assets.vfx_light,
            };
            let sz = p.size * (0.5 + a * 0.7);
            draw_texture_ex(
                tex,
                p.pos.x - sz * 0.5,
                p.pos.y - sz * 0.5,
                Color::new(p.color.r, p.color.g, p.color.b, a),
                DrawTextureParams {
                    dest_size: Some(vec2(sz, sz)),
                    rotation: p.rot,
                    ..Default::default()
                },
            );
        }

        // 12. Floating Damage Numbers (crisp TTF font!)
        for ft in &self.float_texts {
            let a = (ft.life / 0.62).clamp(0.0, 1.0);
            draw_ui_text(
                font,
                &ft.text,
                ft.pos.x - 10.0,
                ft.pos.y,
                ft.scale,
                Color::new(ft.color.r, ft.color.g, ft.color.b, a),
            );
        }

        set_default_camera();
    }

    fn draw_button(&self, font: Option<&Font>, rect: Rect, label: &str, fill: Color, border: Color) -> bool {
        let mp = vec2(mouse_position().0, mouse_position().1);
        let hovered = rect.contains(mp);
        let bg = if hovered {
            Color::new(
                (fill.r + 0.08).min(1.0),
                (fill.g + 0.08).min(1.0),
                (fill.b + 0.08).min(1.0),
                fill.a,
            )
        } else {
            fill
        };
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.2, border);
        let dims = measure_ui_text(font, label, 19);
        draw_ui_text(
            font,
            label,
            rect.x + (rect.w - dims.width) * 0.5,
            rect.y + (rect.h + dims.height) * 0.5 - 2.0,
            19.0,
            WHITE,
        );

        let mut clicked = is_mouse_button_pressed(MouseButton::Left) && hovered;
        for t in touches() {
            if t.phase == TouchPhase::Started && rect.contains(t.position) {
                clicked = true;
            }
        }
        clicked
    }

    fn draw_social_panel(&self, font: Option<&Font>, x: f32, y: f32, w: f32, max_rows: usize) {
        // 1. Subreddit Faction War bar (3 Reddit Orders)
        let f0 = self.social.faction_scores[0].max(1) as f32;
        let f1 = self.social.faction_scores[1].max(1) as f32;
        let f2 = self.social.faction_scores[2].max(1) as f32;
        let f_total = (f0 + f1 + f2).max(1.0);
        let w0 = w * (f0 / f_total);
        let w1 = w * (f1 / f_total);
        let w2 = (w - w0 - w1).max(0.0);

        draw_rectangle(x, y, w0, 18.0, Color::from_rgba(234, 179, 8, 225));
        draw_rectangle(x + w0, y, w1, 18.0, Color::from_rgba(14, 165, 233, 225));
        draw_rectangle(x + w0 + w1, y, w2, 18.0, Color::from_rgba(168, 85, 247, 225));
        draw_rectangle_lines(x, y, w, 18.0, 1.4, Color::from_rgba(226, 232, 240, 160));
        let fw_lbl = format!(
            "FACTION WAR: SOLAR {} · ARCANE {} · SHADOW {}",
            self.social.faction_scores[0],
            self.social.faction_scores[1],
            self.social.faction_scores[2]
        );
        let fwd = measure_ui_text(font, &fw_lbl, 11);
        draw_ui_text(font, &fw_lbl, x + (w - fwd.width) * 0.5, y + 13.0, 11.0, WHITE);

        // 2. Subreddit Boss Siege bar
        let sy = y + 22.0;
        let siege_t = self.social.siege_target.max(1);
        let siege_pct = (self.social.boss_kills_today as f32 / siege_t as f32).clamp(0.0, 1.0);
        draw_rectangle(x, sy, w, 20.0, Color::from_rgba(15, 23, 42, 210));
        draw_rectangle(x, sy, w * siege_pct, 20.0, Color::from_rgba(234, 88, 12, 220));
        draw_rectangle_lines(x, sy, w, 20.0, 1.4, Color::from_rgba(251, 191, 36, 170));
        let siege_lbl = format!(
            "SUBREDDIT BOSS SIEGE: {}/{}",
            self.social.boss_kills_today, siege_t
        );
        let sd = measure_ui_text(font, &siege_lbl, 12);
        draw_ui_text(font, &siege_lbl, x + (w - sd.width) * 0.5, sy + 14.0, 12.0, WHITE);

        // 3. Daily Leaderboard + Yesterday's Crowned Champion
        let mut ly = sy + 25.0;
        if !self.social.yesterday_champion.is_empty() {
            let y_lbl = format!(
                "YESTERDAY'S #1 CHAMPION: u/{} ({} PTS)",
                self.social.yesterday_champion, self.social.yesterday_score
            );
            let yd = measure_ui_text(font, &y_lbl, 12);
            draw_ui_text(
                font,
                &y_lbl,
                x + (w - yd.width) * 0.5,
                ly + 11.0,
                12.0,
                Color::from_rgba(250, 204, 21, 255),
            );
            ly += 17.0;
        }

        if !self.social.leaderboard.is_empty() {
            let rows = self.social.leaderboard.len().min(max_rows);
            let box_h = 24.0 + (rows as f32) * 18.0;
            draw_rectangle(x, ly, w, box_h, Color::from_rgba(8, 13, 28, 205));
            draw_rectangle_lines(x, ly, w, box_h, 1.2, Color::from_rgba(148, 163, 184, 90));
            draw_ui_text(
                font,
                "DAILY LEADERBOARD",
                x + 10.0,
                ly + 15.0,
                12.0,
                Color::from_rgba(56, 189, 248, 255),
            );
            for (idx, entry) in self.social.leaderboard.iter().take(rows).enumerate() {
                let ry = ly + 32.0 + (idx as f32) * 18.0;
                let col = if idx == 0 {
                    Color::from_rgba(250, 204, 21, 255)
                } else {
                    Color::from_rgba(226, 232, 240, 255)
                };
                draw_ui_text(
                    font,
                    &format!("#{} u/{}", idx + 1, entry.username),
                    x + 10.0,
                    ry,
                    13.0,
                    col,
                );
                let sc_str = format!("{}", entry.score);
                let sc_d = measure_ui_text(font, &sc_str, 13);
                draw_ui_text(font, &sc_str, x + w - sc_d.width - 10.0, ry, 13.0, col);
            }
        }
    }

    fn draw_minimap_radar(&self, sw: f32) {
        let map_sz = 78.0;
        let mx = sw - map_sz - 14.0;
        let my = 88.0;
        let cx = mx + map_sz * 0.5;
        let cy = my + map_sz * 0.5;
        let scale = (map_sz * 0.46) / ARENA_HALF;

        draw_rectangle(mx, my, map_sz, map_sz, Color::from_rgba(8, 13, 28, 195));
        draw_rectangle_lines(
            mx,
            my,
            map_sz,
            map_sz,
            1.5,
            Color::from_rgba(148, 163, 184, 140),
        );

        // Barrels (orange dots)
        for b in &self.barrels {
            if !b.exploded {
                let bx = cx + b.pos.x * scale;
                let by = cy + b.pos.y * scale;
                draw_circle(bx, by, 2.2, Color::from_rgba(251, 146, 60, 220));
            }
        }
        // Shrines (crimson dots)
        for s in &self.shrines {
            if !s.completed {
                let sx = cx + s.pos.x * scale;
                let sy = cy + s.pos.y * scale;
                draw_circle(sx, sy, 3.2, Color::from_rgba(244, 63, 94, 255));
            }
        }
        // #1 Ghost (gold dot)
        if !self.social.ghost_points.is_empty() {
            let pts = &self.social.ghost_points;
            let idx = ((self.elapsed / 0.5).floor() as usize).min(pts.len() - 1);
            let gp = pts[idx];
            draw_circle(
                cx + gp.x * scale,
                cy + gp.y * scale,
                2.8,
                Color::from_rgba(250, 204, 21, 240),
            );
        }
        // Boss (pulsing magenta/crimson beacon)
        for e in &self.enemies {
            if e.kind == EnemyKind::Boss {
                draw_circle(
                    cx + e.pos.x * scale,
                    cy + e.pos.y * scale,
                    4.4,
                    Color::from_rgba(232, 121, 249, 255),
                );
            }
        }
        // Hero (cyan center beacon)
        draw_circle(
            cx + self.hero_pos.x * scale,
            cy + self.hero_pos.y * scale,
            3.2,
            Color::from_rgba(56, 189, 248, 255),
        );
    }

    fn draw_ui(&mut self, assets: &GameAssets) {
        let sw = screen_width();
        let sh = screen_height();
        let font = assets.ui_font.as_ref();

        if self.phase == Phase::Title {
            self.title_time += get_frame_time();
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(6, 9, 20, 205));

            // Top-left user, streak, PB & Best Endless Wave badge
            let user_lbl = if self.social.username.is_empty() {
                "u/Guest".to_string()
            } else {
                format!("u/{}", self.social.username)
            };
            let info_lbl = if self.social.best_endless_wave > 0 {
                format!(
                    "{} · {}d · PB {} · W{}",
                    user_lbl,
                    self.social.streak,
                    self.social.personal_best.max(0),
                    self.social.best_endless_wave
                )
            } else {
                format!(
                    "{} · STREAK {}d · PB {}",
                    user_lbl,
                    self.social.streak,
                    self.social.personal_best.max(0)
                )
            };
            draw_ui_text(
                font,
                &info_lbl,
                14.0,
                26.0,
                13.0,
                Color::from_rgba(203, 213, 225, 255),
            );

            // Top-right Aura selector + Mute button
            let unlocked_auras = (0..4)
                .filter(|&a| is_aura_unlocked(a, self.social.medals_mask))
                .count();
            let aura_lbl = format!(
                "AURA: {} ({}/4)",
                AURA_NAMES[self.social.selected_aura.min(3)],
                unlocked_auras
            );
            let aura_rect = Rect::new(sw - 278.0, 10.0, 164.0, 28.0);
            if self.draw_button(
                font,
                aura_rect,
                &aura_lbl,
                Color::from_rgba(15, 23, 42, 220),
                aura_color(self.social.selected_aura),
            ) {
                for step in 1..=4 {
                    let cand = (self.social.selected_aura + step) % 4;
                    if is_aura_unlocked(cand, self.social.medals_mask) {
                        self.social.selected_aura = cand;
                        unsafe {
                            emberfall_set_selected_aura(cand as i32);
                        }
                        play_sfx(0);
                        break;
                    }
                }
            }

            let mute_rect = Rect::new(sw - 108.0, 10.0, 94.0, 28.0);
            if self.draw_button(
                font,
                mute_rect,
                if self.muted { "SFX: OFF" } else { "SFX: ON" },
                Color::from_rgba(15, 23, 42, 220),
                Color::from_rgba(148, 163, 184, 180),
            ) {
                self.toggle_mute();
            }

            let cx = sw * 0.5;
            let top_y = (sh * 0.07).max(38.0);

            let title = "EMBERFALL";
            let td = measure_ui_text(font, title, 40);
            draw_ui_text(
                font,
                title,
                cx - td.width * 0.5,
                top_y + 24.0,
                40.0,
                Color::from_rgba(251, 191, 36, 255),
            );

            let sub = format!(
                "#{} · {} · {}",
                self.day_seed,
                self.biome.title(),
                self.modifier.title()
            );
            let sd = measure_ui_text(font, &sub, 14);
            draw_ui_text(
                font,
                &sub,
                cx - sd.width * 0.5,
                top_y + 46.0,
                14.0,
                Color::from_rgba(56, 189, 248, 255),
            );

            let bdesc = format!("{}  |  {}", self.biome.subtitle(), self.modifier.subtitle());
            let bdd = measure_ui_text(font, &bdesc, 12);
            draw_ui_text(
                font,
                &bdesc,
                cx - bdd.width * 0.5,
                top_y + 64.0,
                12.0,
                Color::from_rgba(203, 213, 225, 255),
            );

            // 3 Playable Hero Class & Reddit Faction Cards (with real illustrated sprites!)
            let cards_y = top_y + 76.0;
            let total_w = (sw - 24.0).min(430.0);
            let gap = 8.0;
            let card_w = (total_w - gap * 2.0) / 3.0;
            let card_h = 128.0;
            let start_x = cx - total_w * 0.5;
            let bob = (self.title_time * 3.2).sin() * 4.0;

            let classes = [
                (HeroClass::SunKnight, &assets.hero),
                (HeroClass::ArcanePyromancer, &assets.hero_mage),
                (HeroClass::ShadowAssassin, &assets.hero_rogue),
            ];

            if is_key_pressed(KeyCode::Key1) {
                self.hero_class = HeroClass::SunKnight;
                play_sfx(0);
            } else if is_key_pressed(KeyCode::Key2) {
                self.hero_class = HeroClass::ArcanePyromancer;
                play_sfx(0);
            } else if is_key_pressed(KeyCode::Key3) {
                self.hero_class = HeroClass::ShadowAssassin;
                play_sfx(0);
            }

            for (ci, (hclass, htex)) in classes.iter().enumerate() {
                let rx = start_x + (ci as f32) * (card_w + gap);
                let rect = Rect::new(rx, cards_y, card_w, card_h);
                let selected = self.hero_class == *hclass;
                let mp = vec2(mouse_position().0, mouse_position().1);
                let hovered = rect.contains(mp);

                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    if selected {
                        Color::from_rgba(30, 41, 75, 245)
                    } else if hovered {
                        Color::from_rgba(20, 29, 55, 230)
                    } else {
                        Color::from_rgba(11, 17, 34, 215)
                    },
                );
                draw_rectangle_lines(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    if selected { 2.8 } else { 1.4 },
                    if selected {
                        hclass.accent()
                    } else {
                        Color::from_rgba(100, 116, 139, 130)
                    },
                );

                let spr_sz = 66.0;
                draw_texture_ex(
                    htex,
                    rect.x + (rect.w - spr_sz) * 0.5,
                    rect.y + 6.0 + if selected { bob } else { 0.0 },
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(spr_sz, spr_sz)),
                        ..Default::default()
                    },
                );

                let cname = hclass.title();
                let cnd = measure_ui_text(font, cname, 13);
                draw_ui_text(
                    font,
                    cname,
                    rect.x + (rect.w - cnd.width) * 0.5,
                    rect.y + 88.0,
                    13.0,
                    hclass.accent(),
                );

                let fname = hclass.faction_name();
                let fnd = measure_ui_text(font, fname, 11);
                draw_ui_text(
                    font,
                    fname,
                    rect.x + (rect.w - fnd.width) * 0.5,
                    rect.y + 104.0,
                    11.0,
                    Color::from_rgba(203, 213, 225, 240),
                );

                let fpts = format!("{} PTS", self.social.faction_scores[ci]);
                let fpd = measure_ui_text(font, &fpts, 11);
                draw_ui_text(
                    font,
                    &fpts,
                    rect.x + (rect.w - fpd.width) * 0.5,
                    rect.y + 120.0,
                    11.0,
                    Color::from_rgba(250, 204, 21, 235),
                );

                let mut clicked = is_mouse_button_pressed(MouseButton::Left) && hovered;
                for t in touches() {
                    if t.phase == TouchPhase::Started && rect.contains(t.position) {
                        clicked = true;
                    }
                }
                if clicked {
                    self.hero_class = *hclass;
                    play_sfx(0);
                }
            }

            let ldesc = self.hero_class.loadout_desc();
            let ldd = measure_ui_text(font, ldesc, 13);
            draw_ui_text(
                font,
                ldesc,
                cx - ldd.width * 0.5,
                cards_y + card_h + 18.0,
                13.0,
                self.hero_class.accent(),
            );

            // Compact 6-Medal Strip
            let medal_w = 52.0;
            let medal_gap = 5.0;
            let medals_total_w = medal_w * 6.0 + medal_gap * 5.0;
            let mx0 = cx - medals_total_w * 0.5;
            let my0 = cards_y + card_h + 24.0;
            for mi in 0..6 {
                let unlocked = (self.social.medals_mask & (1 << mi)) != 0;
                let rx = mx0 + (mi as f32) * (medal_w + medal_gap);
                draw_rectangle(
                    rx,
                    my0,
                    medal_w,
                    18.0,
                    if unlocked {
                        Color::from_rgba(120, 53, 15, 225)
                    } else {
                        Color::from_rgba(15, 23, 42, 185)
                    },
                );
                draw_rectangle_lines(
                    rx,
                    my0,
                    medal_w,
                    18.0,
                    1.0,
                    if unlocked {
                        Color::from_rgba(250, 204, 21, 240)
                    } else {
                        Color::from_rgba(71, 85, 105, 140)
                    },
                );
                let mlbl = MEDAL_SHORT[mi];
                let mdim = measure_ui_text(font, mlbl, 10);
                draw_ui_text(
                    font,
                    mlbl,
                    rx + (medal_w - mdim.width) * 0.5,
                    my0 + 13.0,
                    10.0,
                    if unlocked {
                        Color::from_rgba(253, 224, 71, 255)
                    } else {
                        Color::from_rgba(100, 116, 139, 190)
                    },
                );
            }

            let btn_y = my0 + 24.0;
            let btn_w = ((sw - 36.0).min(380.0) - 10.0) * 0.5;
            let daily_btn = Rect::new(cx - btn_w - 5.0, btn_y, btn_w, 40.0);
            if self.draw_button(
                font,
                daily_btn,
                "PLAY DAILY (90s)",
                Color::from_rgba(234, 88, 12, 235),
                Color::from_rgba(253, 224, 71, 255),
            ) || is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
            {
                self.start_run(false);
            }

            let endless_btn = Rect::new(cx + 5.0, btn_y, btn_w, 40.0);
            if self.draw_button(
                font,
                endless_btn,
                "ENDLESS ABYSS",
                Color::from_rgba(109, 40, 217, 235),
                Color::from_rgba(192, 132, 252, 255),
            ) {
                self.start_run(true);
            }

            let panel_w = (sw - 28.0).min(380.0);
            self.draw_social_panel(
                font,
                cx - panel_w * 0.5,
                btn_y + 48.0,
                panel_w,
                4,
            );

            let tip = "1-Thumb Drag · Shoot Barrels · SPACE Dash · Q/E Nova · 1/2/3 Class";
            let tip_d = measure_ui_text(font, tip, 12);
            draw_ui_text(
                font,
                tip,
                cx - tip_d.width * 0.5,
                sh - 10.0,
                12.0,
                Color::from_rgba(148, 163, 184, 255),
            );
            return;
        }

        // LIVE IN-GAME HUD
        // Top XP Bar
        let xp_pct = (self.xp as f32 / self.xp_next.max(1) as f32).clamp(0.0, 1.0);
        draw_rectangle(0.0, 0.0, sw, 8.0, Color::from_rgba(15, 23, 42, 220));
        draw_rectangle(0.0, 0.0, sw * xp_pct, 8.0, Color::from_rgba(56, 189, 248, 255));

        // Top-Left: HP Bar, Class/Level & Fever Meter
        draw_rectangle(14.0, 16.0, 184.0, 62.0, Color::from_rgba(8, 13, 28, 205));
        draw_rectangle_lines(14.0, 16.0, 184.0, 62.0, 1.5, self.hero_class.accent());
        draw_ui_text(
            font,
            &format!("LV.{} {}", self.level, self.hero_class.title()),
            22.0,
            33.0,
            14.0,
            self.hero_class.accent(),
        );
        let hp_pct = (self.hero_hp / self.hero_max_hp).clamp(0.0, 1.0);
        draw_rectangle(22.0, 40.0, 166.0, 9.0, Color::from_rgba(30, 41, 59, 240));
        draw_rectangle(
            22.0,
            40.0,
            166.0 * hp_pct,
            9.0,
            Color::from_rgba(244, 63, 94, 255),
        );

        let f_mult = fever_multiplier(self.fever);
        draw_rectangle(22.0, 54.0, 166.0, 7.0, Color::from_rgba(30, 41, 59, 240));
        draw_rectangle(
            22.0,
            54.0,
            166.0 * self.fever.clamp(0.0, 1.0),
            7.0,
            Color::from_rgba(251, 146, 60, 255),
        );
        draw_ui_text(
            font,
            &format!("FEVER x{:.1} · {}", f_mult, self.biome.title()),
            22.0,
            73.0,
            11.0,
            if self.fever > 0.5 {
                Color::from_rgba(251, 146, 60, 255)
            } else {
                Color::from_rgba(148, 163, 184, 255)
            },
        );
        if !self.relics.is_empty() {
            let rel_txt = format!("RELICS: {}", self.relics.len());
            draw_ui_text(
                font,
                &rel_txt,
                16.0,
                93.0,
                12.0,
                Color::from_rgba(250, 204, 21, 240),
            );
        }
        // Compact 3 Daily Bounties Checklist (Top-Left under HUD)
        let mut by_off = if self.relics.is_empty() { 92.0 } else { 108.0 };
        for (bounty, prog, done) in &self.bounties {
            let b_str = if *done {
                format!("✓ {}", bounty.label(*prog))
            } else {
                format!("• {}", bounty.label(*prog))
            };
            draw_ui_text(
                font,
                &b_str,
                16.0,
                by_off,
                11.0,
                if *done {
                    Color::from_rgba(74, 222, 128, 245)
                } else {
                    Color::from_rgba(203, 213, 225, 215)
                },
            );
            by_off += 14.0;
        }

        // Top-Center: Countdown Timer (90s -> 0s) or Endless Wave Timer
        if self.endless_mode {
            let secs = self.elapsed as u32;
            let timer_str = format!("W{} · {:02}:{:02}", self.endless_wave, secs / 60, secs % 60);
            let td = measure_ui_text(font, &timer_str, 26);
            draw_ui_text(
                font,
                &timer_str,
                sw * 0.5 - td.width * 0.5,
                40.0,
                26.0,
                Color::from_rgba(192, 132, 252, 255),
            );
        } else {
            let rem = (RUN_DURATION - self.elapsed).max(0.0);
            let timer_str = format!("{:02}:{:02}", (rem as u32) / 60, (rem as u32) % 60);
            let td = measure_ui_text(font, &timer_str, 30);
            draw_ui_text(font, &timer_str, sw * 0.5 - td.width * 0.5, 42.0, 30.0, WHITE);
        }

        // Top-Right: Mute + Pause buttons & Score/Kills + Minimap Radar
        let pause_rect = Rect::new(sw - 46.0, 16.0, 32.0, 28.0);
        let mute_rect = Rect::new(sw - 84.0, 16.0, 32.0, 28.0);
        if self.phase == Phase::Playing {
            if self.draw_button(
                font,
                mute_rect,
                if self.muted { "X" } else { "S" },
                Color::from_rgba(15, 23, 42, 210),
                Color::from_rgba(148, 163, 184, 150),
            ) {
                self.toggle_mute();
            }
            if self.draw_button(
                font,
                pause_rect,
                "||",
                Color::from_rgba(15, 23, 42, 210),
                Color::from_rgba(251, 191, 36, 180),
            ) {
                self.phase = Phase::Paused;
                self.joy_active = false;
                self.joy_vec = Vec2::ZERO;
                unsafe {
                    emberfall_toggle_drone(0);
                }
            }
        }

        let score_str = format!("{} PTS", self.score);
        let sd = measure_ui_text(font, &score_str, 18);
        draw_ui_text(
            font,
            &score_str,
            sw - sd.width - 16.0,
            60.0,
            18.0,
            Color::from_rgba(251, 191, 36, 255),
        );
        let kill_str = if self.combo > 3 {
            format!("{} KILLS · x{}", self.kills, self.combo)
        } else {
            format!("{} KILLS", self.kills)
        };
        let kd = measure_ui_text(font, &kill_str, 13);
        draw_ui_text(
            font,
            &kill_str,
            sw - kd.width - 16.0,
            78.0,
            13.0,
            Color::from_rgba(148, 163, 184, 255),
        );

        // Tactical Minimap Radar
        if matches!(self.phase, Phase::Playing | Phase::Paused | Phase::LevelUp) {
            self.draw_minimap_radar(sw);
        }

        // Off-screen directional edge arrows (Shrines, Chests, Boss)
        if self.phase == Phase::Playing {
            let draw_edge_arrow = |target: Vec2, col: Color| {
                let delta = target - self.cam_pos;
                if delta.x.abs() > sw * 0.46 || delta.y.abs() > sh * 0.46 {
                    let ang = delta.y.atan2(delta.x);
                    let margin = 28.0;
                    let sx = (sw * 0.5 + delta.x).clamp(margin, sw - margin);
                    let sy = (sh * 0.5 + delta.y).clamp(margin + 50.0, sh - margin);
                    let dir = Vec2::from_angle(ang);
                    let perp = vec2(-dir.y, dir.x);
                    let tip = vec2(sx, sy) + dir * 10.0;
                    let left = vec2(sx, sy) - dir * 6.0 + perp * 7.0;
                    let right = vec2(sx, sy) - dir * 6.0 - perp * 7.0;
                    draw_triangle(tip, left, right, col);
                }
            };
            for s in &self.shrines {
                if !s.completed {
                    draw_edge_arrow(s.pos, Color::from_rgba(244, 63, 94, 235));
                }
            }
            for c in &self.chests {
                draw_edge_arrow(c.pos, Color::from_rgba(250, 204, 21, 245));
            }
            for e in &self.enemies {
                if e.kind == EnemyKind::Boss {
                    draw_edge_arrow(e.pos, Color::from_rgba(232, 121, 249, 245));
                }
            }
        }

        // Boss HP Bar
        if let Some(boss) = self.enemies.iter().find(|e| e.kind == EnemyKind::Boss) {
            let bw = (sw - 190.0).min(380.0).max(160.0);
            let bx = (sw - bw) * 0.5;
            let by = 54.0;
            let pct = (boss.hp / boss.max_hp).clamp(0.0, 1.0);
            draw_rectangle(bx, by, bw, 16.0, Color::from_rgba(15, 23, 42, 220));
            draw_rectangle(
                bx,
                by,
                bw * pct,
                16.0,
                if self.boss_enraged {
                    Color::from_rgba(225, 29, 72, 255)
                } else {
                    Color::from_rgba(239, 68, 68, 255)
                },
            );
            draw_rectangle_lines(bx, by, bw, 16.0, 2.0, Color::from_rgba(251, 191, 36, 230));
            let bl = if self.boss_enraged {
                "ARCH-LICH OVERLORD (PHASE II)"
            } else {
                "ARCH-LICH OVERLORD"
            };
            let bld = measure_ui_text(font, bl, 12);
            draw_ui_text(font, bl, sw * 0.5 - bld.width * 0.5, by + 12.0, 12.0, WHITE);
        }

        if self.boss_banner_timer > 0.0 {
            let msg = "DAILY BOSS AWAKENS!";
            let md = measure_ui_text(font, msg, 32);
            draw_ui_text(
                font,
                msg,
                sw * 0.5 - md.width * 0.5,
                sh * 0.34,
                32.0,
                Color::from_rgba(251, 146, 60, 255),
            );
        }

        if self.evo_banner_timer > 0.0 {
            let md = measure_ui_text(font, &self.evo_banner_text, 26);
            draw_ui_text(
                font,
                &self.evo_banner_text,
                sw * 0.5 - md.width * 0.5,
                sh * 0.26,
                26.0,
                self.evo_banner_color,
            );
        }

        // Virtual Joystick Indicator
        if self.joy_active && self.phase == Phase::Playing {
            draw_circle_lines(
                self.joy_origin.x,
                self.joy_origin.y,
                54.0,
                2.0,
                Color::from_rgba(255, 255, 255, 70),
            );
            draw_circle(
                self.joy_curr.x,
                self.joy_curr.y,
                20.0,
                Color::from_rgba(56, 189, 248, 140),
            );
        }

        // Bottom-Right Tactile DASH Button
        if self.phase == Phase::Playing {
            let dc = vec2(sw - 68.0, sh - 76.0);
            let ready = self.dash_cd <= 0.0;
            draw_circle(
                dc.x,
                dc.y,
                42.0,
                if ready {
                    Color::from_rgba(234, 88, 12, 210)
                } else {
                    Color::from_rgba(30, 41, 59, 180)
                },
            );
            draw_circle_lines(
                dc.x,
                dc.y,
                42.0,
                2.5,
                if ready {
                    Color::from_rgba(253, 224, 71, 255)
                } else {
                    Color::from_rgba(148, 163, 184, 120)
                },
            );
            let lbl = if ready {
                "DASH".to_string()
            } else {
                format!("{:.1}", self.dash_cd)
            };
            let ld = measure_ui_text(font, &lbl, 18);
            draw_ui_text(font, &lbl, dc.x - ld.width * 0.5, dc.y + 6.0, 18.0, WHITE);
        }

        // Ultimate SUPERNOVA button
        if self.phase == Phase::Playing {
            let uc = vec2(sw - 150.0, sh - 56.0);
            let ready = self.ult_charge >= 1.0;
            let pulse = if ready {
                1.0 + (get_time() as f32 * 7.0).sin() * 0.08
            } else {
                1.0
            };
            draw_circle(
                uc.x,
                uc.y,
                32.0 * pulse,
                if ready {
                    Color::from_rgba(190, 24, 93, 235)
                } else {
                    Color::from_rgba(15, 23, 42, 200)
                },
            );
            let sweep = self.ult_charge.clamp(0.0, 1.0);
            let steps = (sweep * 40.0) as i32;
            for i in 0..steps {
                let ang = -std::f32::consts::FRAC_PI_2 + (i as f32 / 40.0) * std::f32::consts::TAU;
                draw_circle(
                    uc.x + ang.cos() * 32.0,
                    uc.y + ang.sin() * 32.0,
                    3.2,
                    Color::from_rgba(253, 224, 71, 255),
                );
            }
            let lbl = if ready { "NOVA" } else { "ULT" };
            let ld = measure_ui_text(font, lbl, 16);
            draw_ui_text(font, lbl, uc.x - ld.width * 0.5, uc.y + 6.0, 16.0, WHITE);
        }

        // PAUSED OVERLAY
        if self.phase == Phase::Paused {
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(5, 8, 18, 205));
            let cx = sw * 0.5;
            let cy = sh * 0.5;
            let hdr = "PAUSED";
            let hd = measure_ui_text(font, hdr, 36);
            draw_ui_text(
                font,
                hdr,
                cx - hd.width * 0.5,
                cy - 110.0,
                36.0,
                Color::from_rgba(250, 204, 21, 255),
            );

            if self.draw_button(
                font,
                Rect::new(cx - 120.0, cy - 60.0, 240.0, 48.0),
                "RESUME",
                Color::from_rgba(234, 88, 12, 235),
                Color::from_rgba(253, 224, 71, 255),
            ) {
                self.phase = Phase::Playing;
                unsafe {
                    emberfall_toggle_drone(if self.muted { 0 } else { 1 });
                }
            }
            if self.draw_button(
                font,
                Rect::new(cx - 120.0, cy + 2.0, 240.0, 48.0),
                if self.muted { "UNMUTE AUDIO" } else { "MUTE AUDIO" },
                Color::from_rgba(30, 41, 59, 235),
                Color::from_rgba(148, 163, 184, 220),
            ) {
                self.toggle_mute();
            }
            if self.draw_button(
                font,
                Rect::new(cx - 120.0, cy + 64.0, 240.0, 48.0),
                "QUIT TO TITLE",
                Color::from_rgba(127, 29, 29, 220),
                Color::from_rgba(244, 63, 94, 220),
            ) {
                self.phase = Phase::Title;
                unsafe {
                    emberfall_set_boss_music(0);
                    emberfall_toggle_drone(0);
                }
            }

            // Evolution Synergy Codex (below Pause buttons)
            let codex_y = cy + 128.0;
            let chdr = "EVOLUTION SYNERGY CODEX (LV.3 + LV.3)";
            let cd = measure_ui_text(font, chdr, 13);
            draw_ui_text(
                font,
                chdr,
                cx - cd.width * 0.5,
                codex_y,
                13.0,
                Color::from_rgba(250, 204, 21, 240),
            );
            let recipes = [
                ("METEOR BARRAGE", "Ember Bolts + Arcane Haste"),
                ("CRIMSON REAPER", "Arcane Slash + Vampiric Ward"),
                ("INFERNO VORTEX", "Runic Orbit + Inferno Dash"),
                ("CELESTIAL TEMPEST", "Chain Lightning + Soul Magnet"),
                ("SOUL HARVESTER", "Vampiric Ward + Soul Magnet"),
            ];
            for (ri, (rname, rreq)) in recipes.iter().enumerate() {
                let txt = format!("{} = {}", rname, rreq);
                let td = measure_ui_text(font, &txt, 11);
                draw_ui_text(
                    font,
                    &txt,
                    cx - td.width * 0.5,
                    codex_y + 18.0 + (ri as f32) * 14.0,
                    11.0,
                    Color::from_rgba(203, 213, 225, 220),
                );
            }
            return;
        }

        // LEVEL-UP 3-CARD PICKER
        if self.phase == Phase::LevelUp {
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(5, 8, 18, 205));
            let hdr = format!("LEVEL {} — CHOOSE POWER", self.level);
            let hd = measure_ui_text(font, &hdr, 28);
            draw_ui_text(
                font,
                &hdr,
                sw * 0.5 - hd.width * 0.5,
                sh * 0.16,
                28.0,
                Color::from_rgba(250, 204, 21, 255),
            );

            let card_w = (sw - 40.0).min(340.0);
            let card_h = 92.0;
            let gap = 14.0;
            let start_y = sh * 0.24;
            let choices = self.pending_choices;

            for (i, up) in choices.iter().enumerate() {
                let rect = Rect::new(
                    (sw - card_w) * 0.5,
                    start_y + i as f32 * (card_h + gap),
                    card_w,
                    card_h,
                );
                let mp = vec2(mouse_position().0, mouse_position().1);
                let hovered = rect.contains(mp);
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    if hovered {
                        Color::from_rgba(30, 41, 72, 245)
                    } else {
                        Color::from_rgba(15, 23, 46, 235)
                    },
                );
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.5, up.accent());

                let next_lvl = self.build.level_of(*up) + 1;
                draw_ui_text(
                    font,
                    &format!("[{}] {}  LV.{}", i + 1, up.name(), next_lvl),
                    rect.x + 16.0,
                    rect.y + 34.0,
                    20.0,
                    up.accent(),
                );
                draw_ui_text(
                    font,
                    up.desc(),
                    rect.x + 16.0,
                    rect.y + 66.0,
                    15.0,
                    Color::from_rgba(226, 232, 240, 255),
                );

                let mut picked = is_mouse_button_pressed(MouseButton::Left) && hovered;
                for t in touches() {
                    if t.phase == TouchPhase::Started && rect.contains(t.position) {
                        picked = true;
                    }
                }
                if (i == 0 && is_key_pressed(KeyCode::Key1))
                    || (i == 1 && is_key_pressed(KeyCode::Key2))
                    || (i == 2 && is_key_pressed(KeyCode::Key3))
                    || picked
                {
                    self.apply_upgrade_choice(i);
                    break;
                }
            }
            return;
        }

        // VICTORY / GAME OVER SCREEN
        if self.phase == Phase::Victory || self.phase == Phase::GameOver {
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(5, 8, 18, 220));
            let cx = sw * 0.5;
            let top_y = (sh * 0.09).max(34.0);
            let rank = compute_rank(self.score, self.boss_slain);

            let heading = if self.phase == Phase::Victory {
                "DAILY DUNGEON CLEARED!"
            } else {
                "FALLEN IN BATTLE"
            };
            let hd = measure_ui_text(font, heading, 28);
            draw_ui_text(
                font,
                heading,
                cx - hd.width * 0.5,
                top_y,
                28.0,
                if self.phase == Phase::Victory {
                    Color::from_rgba(250, 204, 21, 255)
                } else {
                    Color::from_rgba(244, 63, 94, 255)
                },
            );

            let rank_str = format!("RANK {} · {}", rank, self.hero_class.title());
            let rd = measure_ui_text(font, &rank_str, 34);
            draw_ui_text(
                font,
                &rank_str,
                cx - rd.width * 0.5,
                top_y + 38.0,
                34.0,
                Color::from_rgba(56, 189, 248, 255),
            );

            let stats = format!(
                "SCORE: {}  ·  KILLS: {}  ·  BOSS: {}",
                self.score,
                self.kills,
                if self.boss_slain { "SLAIN" } else { "SURVIVED" }
            );
            let std = measure_ui_text(font, &stats, 16);
            draw_ui_text(font, &stats, cx - std.width * 0.5, top_y + 66.0, 16.0, WHITE);

            let bounties_done = self.bounties.iter().filter(|(_, _, d)| *d).count();
            let social_line = format!(
                "{} · STREAK: {}d · BOUNTIES: {}/3 · #{} / {}",
                self.hero_class.faction_name(),
                self.social.streak,
                bounties_done,
                self.social.my_rank.max(0),
                self.social.total_players.max(0)
            );
            let sld = measure_ui_text(font, &social_line, 12);
            draw_ui_text(
                font,
                &social_line,
                cx - sld.width * 0.5,
                top_y + 84.0,
                12.0,
                Color::from_rgba(203, 213, 225, 255),
            );

            // Compact Weapon DPS Breakdown Strip
            let dps_line = format!(
                "DMG  SLASH:{}k  BOLTS:{}k  LIGHTNING:{}k  ORBIT:{}k",
                (self.dmg_by_weapon[0] / 1000.0).round() as u32,
                (self.dmg_by_weapon[1] / 1000.0).round() as u32,
                (self.dmg_by_weapon[2] / 1000.0).round() as u32,
                (self.dmg_by_weapon[3] / 1000.0).round() as u32,
            );
            let dld = measure_ui_text(font, &dps_line, 11);
            draw_ui_text(
                font,
                &dps_line,
                cx - dld.width * 0.5,
                top_y + 98.0,
                11.0,
                Color::from_rgba(250, 204, 21, 230),
            );

            let share_state = unsafe { emberfall_get_share_state() };
            let share_lbl = match share_state {
                1 => "SHARING...",
                2 => "SHARED TO REDDIT ✓",
                3 => "LOGIN ON REDDIT TO SHARE",
                _ => "POST SCORE TO REDDIT",
            };
            let share_rect = Rect::new(cx - 140.0, top_y + 106.0, 280.0, 40.0);
            if self.draw_button(
                font,
                share_rect,
                share_lbl,
                Color::from_rgba(37, 99, 235, 235),
                Color::from_rgba(125, 211, 252, 255),
            ) && share_state == 0
            {
                let summary = format!(
                    "{} 🎯{}/3",
                    self.build.build_summary(),
                    bounties_done
                );
                let bytes = summary.as_bytes();
                unsafe {
                    emberfall_share_comment(
                        self.score,
                        self.kills,
                        if self.boss_slain { 1 } else { 0 },
                        self.level,
                        self.hero_class.to_index() as u32,
                        bytes.as_ptr(),
                        bytes.len() as u32,
                    );
                }
            }

            if self.phase == Phase::Victory {
                let bw = 136.0;
                let retry_rect = Rect::new(cx - bw - 4.0, top_y + 152.0, bw, 40.0);
                if self.draw_button(
                    font,
                    retry_rect,
                    "PLAY AGAIN",
                    Color::from_rgba(234, 88, 12, 235),
                    Color::from_rgba(253, 224, 71, 255),
                ) || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::R)
                {
                    self.start_run(false);
                }

                let ascend_rect = Rect::new(cx + 4.0, top_y + 152.0, bw, 40.0);
                if self.draw_button(
                    font,
                    ascend_rect,
                    "ASCEND ENDLESS",
                    Color::from_rgba(109, 40, 217, 235),
                    Color::from_rgba(192, 132, 252, 255),
                ) {
                    self.ascend_to_endless();
                }
            } else {
                let retry_rect = Rect::new(cx - 140.0, top_y + 152.0, 280.0, 40.0);
                if self.draw_button(
                    font,
                    retry_rect,
                    if self.endless_mode { "RETRY ENDLESS" } else { "PLAY AGAIN" },
                    Color::from_rgba(234, 88, 12, 235),
                    Color::from_rgba(253, 224, 71, 255),
                ) || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::R)
                {
                    self.start_run(self.endless_mode);
                }
            }

            let mut panel_y = top_y + 200.0;
            if self.social.can_vote_mutator {
                let vhdr = "TOP-3 BONUS: VOTE TOMORROW'S MUTATOR";
                let vd = measure_ui_text(font, vhdr, 13);
                draw_ui_text(
                    font,
                    vhdr,
                    cx - vd.width * 0.5,
                    panel_y + 10.0,
                    13.0,
                    Color::from_rgba(250, 204, 21, 255),
                );
                let labels = ["BLOOD", "ARCANE", "SWARM", "TITAN"];
                let bw = 74.0;
                let total_w = bw * 4.0 + 18.0;
                let sx = cx - total_w * 0.5;
                for (mi, lbl) in labels.iter().enumerate() {
                    let selected = self.social.voted_mutator == Some(mi);
                    let r = Rect::new(sx + (mi as f32) * (bw + 6.0), panel_y + 16.0, bw, 28.0);
                    if self.draw_button(
                        font,
                        r,
                        lbl,
                        if selected {
                            Color::from_rgba(234, 88, 12, 240)
                        } else {
                            Color::from_rgba(15, 23, 42, 230)
                        },
                        Color::from_rgba(250, 204, 21, 200),
                    ) {
                        self.social.voted_mutator = Some(mi);
                        unsafe {
                            emberfall_vote_mutator(mi as i32);
                        }
                    }
                }
                panel_y += 52.0;
            }

            let panel_w = (sw - 32.0).min(360.0);
            self.draw_social_panel(font, cx - panel_w * 0.5, panel_y, panel_w, 4);
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "EMBERFALL: DAILY HORDE".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = GameAssets::load();
    let shaders = GameShaders::load();
    let mut game = Game::new();
    let mut rt = render_target(screen_width().max(1.0) as u32, screen_height().max(1.0) as u32);

    loop {
        let dt = get_frame_time().min(0.05);
        game.poll_social_bridge();

        if is_key_pressed(KeyCode::M) {
            game.toggle_mute();
        }
        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            if game.phase == Phase::Playing {
                game.phase = Phase::Paused;
                game.joy_active = false;
                game.joy_vec = Vec2::ZERO;
                unsafe {
                    emberfall_toggle_drone(0);
                }
            } else if game.phase == Phase::Paused {
                game.phase = Phase::Playing;
                unsafe {
                    emberfall_toggle_drone(if game.muted { 0 } else { 1 });
                }
            }
        }

        if game.phase == Phase::Playing {
            if game.hitstop > 0.0 {
                game.hitstop = (game.hitstop - dt).max(0.0);
                game.trauma = (game.trauma - dt * 0.9).max(0.0);
            } else {
                game.update_playing(dt);
            }
        }

        let sw = screen_width().max(1.0);
        let sh = screen_height().max(1.0);
        if (sw as u32) != (rt.texture.width() as u32) || (sh as u32) != (rt.texture.height() as u32) {
            rt = render_target(sw as u32, sh as u32);
        }

        game.draw_world(&assets, &shaders, Some(&rt));

        set_default_camera();
        clear_background(Color::from_rgba(6, 9, 20, 255));
        if let Some(mat) = &shaders.post_process {
            let low_hp = if game.hero_hp / game.hero_max_hp.max(1.0) < 0.35
                && matches!(game.phase, Phase::Playing | Phase::Paused | Phase::LevelUp)
            {
                (1.0 - (game.hero_hp / (game.hero_max_hp * 0.35))).clamp(0.0, 1.0)
            } else {
                0.0
            };
            mat.set_uniform("u_time", get_time() as f32);
            mat.set_uniform("u_low_hp", low_hp);
            mat.set_uniform("u_aberration", game.aberration);
            mat.set_uniform("u_shockwave", game.shockwave);
            mat.set_uniform(
                "u_blood_moon",
                if game.modifier == DailyModifier::BloodMoon {
                    1.0f32
                } else {
                    0.0f32
                },
            );
            mat.set_uniform("u_biome", game.biome.to_index() as f32);
            mat.set_uniform("u_fever", game.fever);
            gl_use_material(mat);
        }
        draw_texture_ex(
            &rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(sw, sh)),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();

        game.draw_ui(&assets);

        next_frame().await;
    }
}


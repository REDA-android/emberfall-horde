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
use weapons::{HeroBuild, HeroClass, RelicId, UpgradeId, ALL_UPGRADES};

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn emberfall_get_day_number() -> u32;
    fn emberfall_play_sfx_pitched(kind: i32, pitch_mul: f32);
    fn emberfall_play_voice_blip(speaker_kind: i32);
    fn emberfall_set_biome_music(biome_idx: i32);
    fn emberfall_set_crt(enabled: i32);
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
    fn emberfall_get_gamepad_axis(axis_idx: i32) -> i32;
    fn emberfall_get_gamepad_btn(btn_idx: i32) -> i32;
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
    fn emberfall_get_tomorrow_vote(index: i32) -> u32;
    fn emberfall_get_ghost_x(index: i32) -> i32;
    fn emberfall_get_ghost_y(index: i32) -> i32;
    fn emberfall_get_share_state() -> i32;
    fn emberfall_get_submit_state() -> i32;
    fn emberfall_sync_hud(
        phase: i32,
        biome: i32,
        hero_class: i32,
        hp: f32,
        max_hp: f32,
        ember: f32,
        overdrive_sec: f32,
        xp: u32,
        xp_next: u32,
        lv: u32,
        score: u32,
        kills: u32,
        combo: u32,
        time_rem: f32,
        boss_hp: f32,
        boss_max_hp: f32,
        boss_trail: f32,
        ghost_delta: i32,
        muted: i32,
        crt: i32,
        rank_ptr: *const u8,
        rank_len: u32,
        chron_ptr: *const u8,
        chron_len: u32,
    );
    fn emberfall_sync_choices(
        c0: i32,
        c1: i32,
        c2: i32,
        r0: i32,
        r1: i32,
        r2: i32,
        b0: u32,
        b1: u32,
        b2: u32,
    );
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_day_number() -> u32 {
    20261121
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_play_sfx_pitched(_kind: i32, _pitch_mul: f32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_play_voice_blip(_speaker_kind: i32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_set_biome_music(_biome_idx: i32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_set_crt(_enabled: i32) {}
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
unsafe fn emberfall_get_gamepad_axis(_axis_idx: i32) -> i32 {
    0
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_gamepad_btn(_btn_idx: i32) -> i32 {
    0
}
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
unsafe fn emberfall_get_tomorrow_vote(_index: i32) -> u32 {
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
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_sync_hud(
    _phase: i32,
    _biome: i32,
    _hero_class: i32,
    _hp: f32,
    _max_hp: f32,
    _ember: f32,
    _overdrive_sec: f32,
    _xp: u32,
    _xp_next: u32,
    _lv: u32,
    _score: u32,
    _kills: u32,
    _combo: u32,
    _time_rem: f32,
    _boss_hp: f32,
    _boss_max_hp: f32,
    _boss_trail: f32,
    _ghost_delta: i32,
    _muted: i32,
    _crt: i32,
    _rank_ptr: *const u8,
    _rank_len: u32,
    _chron_ptr: *const u8,
    _chron_len: u32,
) {
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_sync_choices(
    _c0: i32,
    _c1: i32,
    _c2: i32,
    _r0: i32,
    _r1: i32,
    _r2: i32,
    _b0: u32,
    _b1: u32,
    _b2: u32,
) {
}

#[derive(Clone, Copy)]
enum JsCmd {
    StartRun(i32, i32),
    SkipIntro,
    TriggerOverdrive,
    PickUpgrade(i32),
    TogglePause,
    ToggleMute,
    ToggleCrt,
    Replay,
    Share,
    Vote(i32),
}

static mut JS_CMD_QUEUE: Vec<JsCmd> = Vec::new();

fn push_js_cmd(cmd: JsCmd) {
    unsafe {
        let ptr = &raw mut JS_CMD_QUEUE;
        (*ptr).push(cmd);
    }
}

#[no_mangle]
pub extern "C" fn ef_cmd_start_run(hero_class: i32, endless: i32) {
    push_js_cmd(JsCmd::StartRun(hero_class, endless));
}
#[no_mangle]
pub extern "C" fn ef_cmd_skip_intro() {
    push_js_cmd(JsCmd::SkipIntro);
}
#[no_mangle]
pub extern "C" fn ef_cmd_trigger_overdrive() {
    push_js_cmd(JsCmd::TriggerOverdrive);
}
#[no_mangle]
pub extern "C" fn ef_cmd_pick_upgrade(idx: i32) {
    push_js_cmd(JsCmd::PickUpgrade(idx));
}
#[no_mangle]
pub extern "C" fn ef_cmd_toggle_pause() {
    push_js_cmd(JsCmd::TogglePause);
}
#[no_mangle]
pub extern "C" fn ef_cmd_toggle_mute() {
    push_js_cmd(JsCmd::ToggleMute);
}
#[no_mangle]
pub extern "C" fn ef_cmd_toggle_crt() {
    push_js_cmd(JsCmd::ToggleCrt);
}
#[no_mangle]
pub extern "C" fn ef_cmd_replay() {
    push_js_cmd(JsCmd::Replay);
}
#[no_mangle]
pub extern "C" fn ef_cmd_share() {
    push_js_cmd(JsCmd::Share);
}
#[no_mangle]
pub extern "C" fn ef_cmd_vote(idx: i32) {
    push_js_cmd(JsCmd::Vote(idx));
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

fn play_voice_blip(speaker_kind: i32) {
    unsafe {
        emberfall_play_voice_blip(speaker_kind);
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

fn blessing_name(idx: usize) -> &'static str {
    match idx {
        0 => "SOLAR AEGIS",
        1 => "ARCANE OVERCHARGE",
        _ => "SHADOW VELOCITY",
    }
}

fn blessing_effect(idx: usize) -> &'static str {
    match idx {
        0 => "+25 MAX HP & +15% AREA",
        1 => "+18% ATK SPD & +15% XP",
        _ => "+15% MOVE SPD & +15% CRIT",
    }
}

fn biome_boss_name(b: DailyBiome) -> &'static str {
    match b {
        DailyBiome::ObsidianSanctuary => "MALAKOR, VOID LICH",
        DailyBiome::VolcanicRift => "IGNIS REX, MAGMA TITAN",
        DailyBiome::FrostAbyss => "SKADI, GLACIAL EMPRESS",
    }
}

fn biome_boss_intro(b: DailyBiome) -> &'static str {
    match b {
        DailyBiome::ObsidianSanctuary => "THE VOID AWAKENS!",
        DailyBiome::VolcanicRift => "THE RIFT ERUPTS!",
        DailyBiome::FrostAbyss => "ETERNAL WINTER FALLS!",
    }
}

fn biome_boss_color(b: DailyBiome) -> Color {
    match b {
        DailyBiome::ObsidianSanctuary => Color::from_rgba(192, 132, 252, 255),
        DailyBiome::VolcanicRift => Color::from_rgba(251, 146, 60, 255),
        DailyBiome::FrostAbyss => Color::from_rgba(56, 189, 248, 255),
    }
}

/// Renders a character or monster with a 100% RIGID upper body (head, torso, arms, cape, and sword
/// never shear or peck like a chicken!) while the Left and Right lower boots step smoothly below the knees.
#[allow(clippy::too_many_arguments)]
fn draw_articulated_sprite(
    tex: &Texture2D,
    center: Vec2,
    size: Vec2,
    flip_x: bool,
    tint: Color,
    stride_phase: f32,
    move_ratio: f32,
    lean_x: f32,
    attack_lunge: Vec2,
    breath: f32,
    split_u: f32,
) {
    let hw = size.x * 0.5;
    let hh = size.y * 0.5;
    let s = stride_phase.sin() * move_ratio;
    let c = stride_phase.cos() * move_ratio;

    // Gentle human vertical stride bounce (no head-pecking!)
    let body_bob = if move_ratio > 0.05 {
        (stride_phase * 2.0).cos() * -2.2 * move_ratio
    } else {
        breath * 1.4
    };
    let body_center = center + attack_lunge * 0.65 + vec2(lean_x * 2.0, body_bob);

    // Knee split line is at 72% down the sprite (v = 0.72):
    // Above v = 0.72 (Head, Torso, Cape, Sword) is rendered as ONE 100% rigid quad so it never distorts!
    let knee_v = 0.72;
    let knee_y = -hh + size.y * knee_v;

    // Subtle human boot step: mostly vertical knee/foot lift + tiny 2.5px horizontal step
    let step_dx = size.x * 0.028;
    let step_lift = size.y * 0.065;
    let l_dx = s * step_dx;
    let l_dy = -c.max(0.0) * step_lift;
    let r_dx = -s * step_dx;
    let r_dy = -(-c).max(0.0) * step_lift;

    let split_screen_u = if flip_x { 1.0 - split_u } else { split_u };
    let split_x = -hw + size.x * split_screen_u;
    let u_left_edge = if flip_x { 1.0 } else { 0.0 };
    let u_right_edge = if flip_x { 0.0 } else { 1.0 };
    let u_mid = split_u;

    let c_bytes: [u8; 4] = tint.into();
    let vtx = |pos: Vec2, u: f32, v: f32| Vertex {
        position: vec3(pos.x, pos.y, 0.0),
        uv: vec2(u, v),
        color: c_bytes,
        normal: vec4(0.0, 0.0, 1.0, 0.0),
    };

    // 1) Upper Body Quad (v = 0.0 .. 0.72): 100% rigid!
    let u_tl = body_center + vec2(-hw, -hh);
    let u_tr = body_center + vec2(hw, -hh);
    let u_bl = body_center + vec2(-hw, knee_y);
    let u_br = body_center + vec2(hw, knee_y);

    // 2) Left Boot Quad (x = -hw .. split_x, v = 0.72 .. 1.0): steps smoothly
    let lb_tl = body_center + vec2(-hw, knee_y);
    let lb_tr = body_center + vec2(split_x, knee_y);
    let lb_bl = body_center + vec2(-hw + l_dx, hh + l_dy);
    let lb_br = body_center + vec2(split_x + l_dx, hh + l_dy);

    // 3) Right Boot Quad (x = split_x .. +hw, v = 0.72 .. 1.0): steps in opposite phase
    let rb_tl = body_center + vec2(split_x, knee_y);
    let rb_tr = body_center + vec2(hw, knee_y);
    let rb_bl = body_center + vec2(split_x + r_dx, hh + r_dy);
    let rb_br = body_center + vec2(hw + r_dx, hh + r_dy);

    let vertices = vec![
        // Upper rigid quad (0..3)
        vtx(u_tl, u_left_edge, 0.0),
        vtx(u_tr, u_right_edge, 0.0),
        vtx(u_bl, u_left_edge, knee_v),
        vtx(u_br, u_right_edge, knee_v),
        // Left boot quad (4..7)
        vtx(lb_tl, u_left_edge, knee_v),
        vtx(lb_tr, u_mid, knee_v),
        vtx(lb_bl, u_left_edge, 1.0),
        vtx(lb_br, u_mid, 1.0),
        // Right boot quad (8..11)
        vtx(rb_tl, u_mid, knee_v),
        vtx(rb_tr, u_right_edge, knee_v),
        vtx(rb_bl, u_mid, 1.0),
        vtx(rb_br, u_right_edge, 1.0),
    ];

    let indices: Vec<u16> = vec![
        0, 1, 2, 1, 3, 2, // Upper body
        4, 5, 6, 5, 7, 6, // Left boot
        8, 9, 10, 9, 11, 10, // Right boot
    ];

    let mesh = Mesh {
        vertices,
        indices,
        texture: Some(tex.clone()),
    };
    draw_mesh(&mesh);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Intro,
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
    anim_phase: f32,
    lunge_timer: f32,
}

struct DyingEnemy {
    kind: EnemyKind,
    pos: Vec2,
    vel: Vec2,
    rot: f32,
    vrot: f32,
    life: f32,
    max_life: f32,
    is_elite: bool,
    facing_left: bool,
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
    blessing_faction: i32,
    tomorrow_biome_idx: usize,
    tomorrow_leading_mutator: usize,
    tomorrow_votes: [u32; 4],
    tomorrow_total_votes: u32,
    siege_feed: Vec<String>,
    utc_seconds_to_midnight: u32,
    countdown_sync_timer: f32,
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
    revenge_claimed: bool,

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
    witch_time_dodges: u32,
    combo_milestone: u32,
    dmg_by_weapon: [f32; 4],
    particles: Vec<Particle>,
    float_texts: Vec<FloatText>,
    afterimages: Vec<Afterimage>,
    dying_enemies: Vec<DyingEnemy>,
    weather_timer: f32,
    storm_timer: f32,
    lightning_flash: f32,
    hero_aim_angle: f32,
    hero_attack_swing: f32,
    slow_mo_timer: f32,
    dodge_triggered_this_dash: bool,

    // Camera, Juice, Audio, Companion, Banter & Social
    cam_pos: Vec2,
    trauma: f32,
    hitstop: f32,
    aberration: f32,
    shockwave: f32,
    muted: bool,
    crt_enabled: bool,
    hurt_flash: f32,
    pet_pos: Vec2,
    pet_attack_cd: f32,
    pet_facing_left: bool,
    banter_is_boss: bool,
    banter_title: String,
    banter_line: String,
    banter_timer: f32,
    banter_accent: Color,
    joy_active: bool,
    joy_origin: Vec2,
    joy_curr: Vec2,
    joy_vec: Vec2,
    title_time: f32,
    intro_time: f32,
    intro_stage_sfx: u8,
    intro_last_blip_char: usize,
    intro_pending_endless: bool,
    intro_seen: bool,
    title_modal: u8,
    social: SocialState,

    // v0.1.0 Overdrive, Destructible Runic Pillars, CameraRig & Blood Chronicle Telemetry
    pillars: Vec<RunicPillar>,
    pillars_shattered: u32,
    overdrive_timer: f32,
    overdrive_cutin_timer: f32,
    overdrive_meteor_cd: f32,
    overdrive_count: u32,
    peak_combo: u32,
    closest_dodge_hp: f32,
    boss_hp_trail: f32,
    cam_zoom: f32,
}

struct RunicPillar {
    pos: Vec2,
    hp: f32,
    max_hp: f32,
    hit_flash: f32,
    shattered: bool,
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
            revenge_claimed: false,
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
            witch_time_dodges: 0,
            combo_milestone: 0,
            dmg_by_weapon: [0.0; 4],
            particles: Vec::new(),
            float_texts: Vec::new(),
            afterimages: Vec::new(),
            dying_enemies: Vec::new(),
            weather_timer: 0.0,
            storm_timer: 9.0,
            lightning_flash: 0.0,
            hero_aim_angle: 0.0,
            hero_attack_swing: 0.0,
            slow_mo_timer: 0.0,
            dodge_triggered_this_dash: false,
            cam_pos: Vec2::ZERO,
            trauma: 0.0,
            hitstop: 0.0,
            aberration: 0.0,
            shockwave: 0.0,
            muted: false,
            crt_enabled: false,
            hurt_flash: 0.0,
            pet_pos: vec2(-42.0, -24.0),
            pet_attack_cd: 1.2,
            pet_facing_left: false,
            banter_is_boss: false,
            banter_title: String::new(),
            banter_line: String::new(),
            banter_timer: 0.0,
            banter_accent: GOLD,
            joy_active: false,
            joy_origin: Vec2::ZERO,
            joy_curr: Vec2::ZERO,
            joy_vec: Vec2::ZERO,
            title_time: 0.0,
            intro_time: 0.0,
            intro_stage_sfx: 0,
            intro_last_blip_char: 0,
            intro_pending_endless: false,
            intro_seen: false,
            title_modal: 0,
            social: SocialState::default(),
            pillars: Vec::new(),
            pillars_shattered: 0,
            overdrive_timer: 0.0,
            overdrive_cutin_timer: 0.0,
            overdrive_meteor_cd: 0.0,
            overdrive_count: 0,
            peak_combo: 0,
            closest_dodge_hp: hero_class.base_max_hp(),
            boss_hp_trail: 0.0,
            cam_zoom: 1.0,
        }
    }

    fn launch_cutscene(&mut self, endless: bool, _force_full: bool) {
        self.intro_pending_endless = endless;
        self.intro_stage_sfx = 0;
        self.intro_last_blip_char = 0;
        self.title_modal = 0;
        self.intro_time = 0.0;
        self.intro_seen = true;
        self.cam_zoom = 0.72;
        self.phase = Phase::Intro;
        play_sfx(14);
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
            let bf = unsafe { emberfall_get_init_field(14) };
            if (0..=2).contains(&bf) {
                self.social.blessing_faction = bf;
            } else {
                self.social.blessing_faction = self.biome.to_index() as i32;
            }
            let tb = unsafe { emberfall_get_init_field(15) };
            if (0..=2).contains(&tb) {
                self.social.tomorrow_biome_idx = tb as usize;
            } else {
                self.social.tomorrow_biome_idx = (self.biome.to_index() + 1) % 3;
            }
            let tm = unsafe { emberfall_get_init_field(16) };
            if (0..=3).contains(&tm) {
                self.social.tomorrow_leading_mutator = tm as usize;
            }
            let my_v = unsafe { emberfall_get_init_field(17) };
            if (0..=3).contains(&my_v) {
                self.social.voted_mutator = Some(my_v as usize);
            }
            self.social.tomorrow_total_votes =
                unsafe { emberfall_get_init_field(18) }.max(0) as u32;
            for vi in 0..4 {
                self.social.tomorrow_votes[vi] =
                    unsafe { emberfall_get_tomorrow_vote(vi as i32) };
            }
            let sf_count = (unsafe { emberfall_get_init_field(19) }).clamp(0, 6) as usize;
            self.social.siege_feed.clear();
            for si in 0..sf_count {
                let s = read_ffi_str(5, si as i32);
                if !s.is_empty() {
                    self.social.siege_feed.push(s);
                }
            }
            let rem_sec = unsafe { emberfall_get_init_field(20) };
            if rem_sec >= 0 {
                self.social.utc_seconds_to_midnight = rem_sec as u32;
            }
            if !just_submitted {
                let persisted_mute = unsafe { emberfall_get_init_field(21) };
                if persisted_mute == 1 {
                    self.muted = true;
                }
                let persisted_crt = unsafe { emberfall_get_init_field(22) };
                if persisted_crt == 1 {
                    self.crt_enabled = true;
                }
            }
        }
        // Refresh UTC countdown & live tomorrow vote tallies periodically
        self.social.countdown_sync_timer += get_frame_time();
        if self.social.countdown_sync_timer >= 0.75 {
            self.social.countdown_sync_timer = 0.0;
            let rem_sec = unsafe { emberfall_get_init_field(20) };
            if rem_sec >= 0 {
                self.social.utc_seconds_to_midnight = rem_sec as u32;
            }
            let tv = unsafe { emberfall_get_init_field(18) };
            if tv >= 0 {
                self.social.tomorrow_total_votes = tv as u32;
                let tm = unsafe { emberfall_get_init_field(16) };
                if (0..=3).contains(&tm) {
                    self.social.tomorrow_leading_mutator = tm as usize;
                }
                for vi in 0..4 {
                    self.social.tomorrow_votes[vi] =
                        unsafe { emberfall_get_tomorrow_vote(vi as i32) };
                }
            }
        }
        self.social.last_status_msg = read_ffi_str(3, 0);
    }

    fn toggle_crt(&mut self) {
        self.crt_enabled = !self.crt_enabled;
        play_sfx(0);
        unsafe {
            emberfall_set_crt(if self.crt_enabled { 1 } else { 0 });
        }
    }

    fn trigger_banter(&mut self, is_boss: bool, line: &str) {
        self.banter_is_boss = is_boss;
        self.banter_line = line.to_string();
        self.banter_timer = 2.8;
        if is_boss {
            self.banter_title = biome_boss_name(self.biome).to_string();
            self.banter_accent = biome_boss_color(self.biome);
            play_voice_blip(3 + self.biome.to_index() as i32);
        } else {
            self.banter_title = self.hero_class.title().to_string();
            self.banter_accent = self.hero_class.accent();
            play_voice_blip(self.hero_class.to_index() as i32);
        }
    }

    fn active_blessing(&self) -> usize {
        if (0..=2).contains(&self.social.blessing_faction) {
            self.social.blessing_faction as usize
        } else {
            self.biome.to_index()
        }
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
        self.revenge_claimed = false;
        self.hero_pos = Vec2::ZERO;
        self.hero_vel = Vec2::ZERO;
        let hp_bonus = if self.active_blessing() == 0 { 25.0 } else { 0.0 };
        let max_hp = self.hero_class.base_max_hp() + hp_bonus;
        self.hero_max_hp = max_hp;
        self.hero_hp = max_hp;
        self.hero_invuln = 0.6;
        self.hero_facing_left = false;
        self.hero_squash = 0.0;
        self.hurt_flash = 0.0;
        self.pet_pos = vec2(-42.0, -24.0);
        self.pet_attack_cd = 1.0;
        self.pet_facing_left = false;
        self.banter_timer = 0.0;
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
        self.witch_time_dodges = 0;
        self.combo_milestone = 0;
        self.dmg_by_weapon = [0.0; 4];
        self.pillars = [
            vec2(-280.0, -210.0),
            vec2(280.0, -210.0),
            vec2(-280.0, 210.0),
            vec2(280.0, 210.0),
        ]
        .into_iter()
        .map(|pos| RunicPillar {
            pos,
            hp: 95.0,
            max_hp: 95.0,
            hit_flash: 0.0,
            shattered: false,
        })
        .collect();
        self.pillars_shattered = 0;
        self.overdrive_timer = 0.0;
        self.overdrive_cutin_timer = 0.0;
        self.overdrive_meteor_cd = 0.0;
        self.overdrive_count = 0;
        self.peak_combo = 0;
        self.closest_dodge_hp = max_hp;
        self.boss_hp_trail = 0.0;
        self.cam_zoom = 1.0;

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
            emberfall_set_biome_music(self.biome.to_index() as i32);
            emberfall_set_boss_music(0);
            emberfall_toggle_drone(if self.muted { 0 } else { 1 });
        }

        // Opening burst of enemies so action starts in second 1!
        for _ in 0..6 {
            self.spawn_enemy(EnemyKind::Imp, false);
        }
        let opening_line = match self.hero_class {
            HeroClass::SunKnight => "For the Solar Order! Hold the line!",
            HeroClass::ArcanePyromancer => "Let the Arcane flames consume them!",
            HeroClass::ShadowAssassin => "From the shadows... strike fast!",
        };
        self.trigger_banter(false, opening_line);
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

        let init_phase = self.rng.range_f32(0.0, std::f32::consts::TAU);
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
            anim_phase: init_phase,
            lunge_timer: 0.0,
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
        self.dodge_triggered_this_dash = false;
        self.hero_invuln = self.hero_invuln.max(0.32);
        self.hero_squash = 0.36;
        self.dash_cd = (2.2 - (self.build.dash_lvl as f32 - 1.0) * 0.26).max(0.95);
        self.add_trauma(0.26);
        play_sfx(10);

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
        if self.ult_charge < 1.0 || self.phase != Phase::Playing || self.overdrive_timer > 0.0 {
            return;
        }
        self.ult_charge = 0.0;
        self.ult_fx = 1.0;
        self.ult_pos = self.hero_pos;
        self.overdrive_cutin_timer = 1.0;
        self.overdrive_timer = 6.0;
        self.overdrive_meteor_cd = 0.25;
        self.overdrive_count += 1;
        self.hero_invuln = self.hero_invuln.max(1.5);
        self.add_trauma(1.0);
        self.hitstop = self.hitstop.max(0.09);
        self.shockwave = 1.0;
        self.aberration = 0.024;
        play_sfx(4);
        play_sfx(3);
        unsafe {
            emberfall_set_boss_music(2);
        }
        self.trigger_banter(false, "EMBER OVERDRIVE AWAKENED!");
        for (n, col) in [
            Color::from_rgba(253, 224, 71, 255),
            Color::from_rgba(255, 77, 45, 255),
            Color::from_rgba(157, 78, 221, 255),
        ]
        .iter()
        .enumerate()
        {
            self.spawn_burst(self.hero_pos, 44, *col, VfxKind::Flame, 400.0 + n as f32 * 160.0);
        }
        let dmg = 175.0 + (self.level as f32) * 16.0;
        self.damage_enemies_in_radius(self.hero_pos, 480.0, dmg, 540.0);
    }

    fn check_bounties(&mut self) {
        let mut newly_done: Vec<String> = Vec::new();
        for i in 0..3 {
            let (bounty, ref mut prog, ref mut done) = self.bounties[i];
            let new_p = match bounty.kind {
                BountyKind::WitchTimeDodges => self.witch_time_dodges,
                BountyKind::ReachLevel => self.level,
                BountyKind::SlayDailyBoss => {
                    if self.boss_slain {
                        1
                    } else {
                        0
                    }
                }
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

    fn damage_pillars_in_radius(&mut self, center: Vec2, radius: f32, dmg: f32) {
        let mut to_shatter: Vec<usize> = Vec::new();
        for (pi, p) in self.pillars.iter_mut().enumerate() {
            if p.shattered {
                continue;
            }
            if p.pos.distance(center) <= radius + 30.0 {
                p.hp -= dmg;
                p.hit_flash = 0.14;
                if p.hp <= 0.0 {
                    to_shatter.push(pi);
                }
            }
        }
        for pi in to_shatter {
            self.pillars[pi].shattered = true;
            self.pillars_shattered += 1;
            let ppos = self.pillars[pi].pos;
            self.score += 250;
            self.ult_charge = (self.ult_charge + 0.10).min(1.0);
            self.add_trauma(0.55);
            self.shockwave = self.shockwave.max(0.8);
            play_sfx(4);
            self.spawn_burst(
                ppos,
                32,
                Color::from_rgba(168, 85, 247, 255),
                VfxKind::Magic,
                340.0,
            );
            self.float_texts.push(FloatText {
                pos: ppos + vec2(0.0, -34.0),
                text: "RUNIC PILLAR SHATTER! +250".to_string(),
                color: Color::from_rgba(192, 132, 252, 255),
                life: 1.25,
                scale: 22.0,
            });
            let blast_dmg = 125.0 + (self.level as f32) * 10.0;
            for e in &mut self.enemies {
                let d = e.pos.distance(ppos);
                if d <= 290.0 + e.radius {
                    e.hp -= blast_dmg;
                    e.hit_flash = 0.16;
                    e.chill_timer = e.chill_timer.max(2.0);
                    let dir = (e.pos - ppos).normalize_or_zero();
                    e.vel += dir * if e.kind == EnemyKind::Boss { 80.0 } else { 420.0 };
                }
            }
        }
    }

    fn damage_enemies_in_radius(&mut self, center: Vec2, radius: f32, base_dmg: f32, knockback: f32) -> usize {
        self.damage_barrels_in_radius(center, radius, base_dmg);
        self.damage_pillars_in_radius(center, radius, base_dmg);
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
        if is_crit {
            self.hitstop = self.hitstop.max(0.025);
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
            self.evo_banner_text = format!("{} PHASE II — SHATTER OBELISKS!", biome_boss_name(self.biome));
            self.evo_banner_color = Color::from_rgba(244, 63, 94, 255);
            self.evo_banner_timer = 3.4;
            self.shockwave = 1.0;
            self.add_trauma(0.85);
            play_sfx(6);
            unsafe {
                emberfall_set_boss_music(2);
            }
            let p2_line = match self.biome {
                DailyBiome::ObsidianSanctuary => "ENOUGH! The Abyss devours your soul!",
                DailyBiome::VolcanicRift => "FURY UNLEASHED! Burn in molten wrath!",
                DailyBiome::FrostAbyss => "SHATTER! Eternal winter claims you now!",
            };
            self.trigger_banter(true, p2_line);
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
                let away = (dead.pos - self.hero_pos).normalize_or_zero();
                let spin_dir = if dead.facing_left { 1.0 } else { -1.0 };
                let d_life = if dead.kind == EnemyKind::Boss { 0.65 } else { 0.36 };
                self.dying_enemies.push(DyingEnemy {
                    kind: dead.kind,
                    pos: dead.pos,
                    vel: away * if dead.kind == EnemyKind::Boss { 80.0 } else { 220.0 } + vec2(0.0, -70.0),
                    rot: 0.0,
                    vrot: spin_dir * if dead.kind == EnemyKind::Boss { 3.5 } else { 9.5 },
                    life: d_life,
                    max_life: d_life,
                    is_elite: dead.is_elite,
                    facing_left: dead.facing_left,
                });
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

        // Web Gamepad API support (inspired by fornwall/gamepads): Left Stick + A/B/RT
        let gp_x = (unsafe { emberfall_get_gamepad_axis(0) } as f32) / 1000.0;
        let gp_y = (unsafe { emberfall_get_gamepad_axis(1) } as f32) / 1000.0;
        if gp_x.abs() > 0.15 || gp_y.abs() > 0.15 {
            mv.x += gp_x;
            mv.y += gp_y;
        }
        if unsafe { emberfall_get_gamepad_btn(0) } == 1 || unsafe { emberfall_get_gamepad_btn(7) } == 1 {
            self.trigger_dash();
        }
        if unsafe { emberfall_get_gamepad_btn(1) } == 1 || unsafe { emberfall_get_gamepad_btn(5) } == 1 {
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
        let info_rect = Rect::new(sw - 122.0, 16.0, 32.0, 28.0);
        let crt_rect = Rect::new(sw - 160.0, 16.0, 34.0, 28.0);

        // Touch / Mouse 1-thumb virtual joystick
        let mut active_pointer: Option<(Vec2, bool)> = None;
        let ts = touches();
        if !ts.is_empty() {
            for t in &ts {
                if !self.joy_active
                    && (mute_rect.contains(t.position)
                        || pause_rect.contains(t.position)
                        || info_rect.contains(t.position)
                        || crt_rect.contains(t.position))
                {
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
            if !self.joy_active
                && (mute_rect.contains(mp)
                    || pause_rect.contains(mp)
                    || info_rect.contains(mp)
                    || crt_rect.contains(mp))
            {
                // Ignore HUD CRT/Info/Mute/Pause button clicks so joystick does not activate
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

    fn update_playing(&mut self, raw_dt: f32) {
        self.slow_mo_timer = (self.slow_mo_timer - raw_dt).max(0.0);
        self.lightning_flash = (self.lightning_flash - raw_dt * 2.2).max(0.0);
        self.hero_attack_swing = (self.hero_attack_swing - raw_dt * 4.5).max(0.0);
        self.banter_timer = (self.banter_timer - raw_dt).max(0.0);
        self.hurt_flash = (self.hurt_flash - raw_dt * 2.5).max(0.0);
        let dt = if self.slow_mo_timer > 0.0 {
            raw_dt * 0.38
        } else {
            raw_dt
        };
        let hero_dt = raw_dt;

        self.elapsed += dt;
        self.hero_invuln = (self.hero_invuln - hero_dt).max(0.0);
        self.dash_cd = (self.dash_cd - hero_dt).max(0.0);
        self.boss_banner_timer = (self.boss_banner_timer - raw_dt).max(0.0);
        self.evo_banner_timer = (self.evo_banner_timer - raw_dt).max(0.0);
        self.trauma = (self.trauma - raw_dt * 1.8).max(0.0);
        self.aberration = (self.aberration - raw_dt * 0.08).max(0.0);
        self.shockwave = (self.shockwave - raw_dt * 1.9).max(0.0);
        self.ult_fx = (self.ult_fx - raw_dt * 1.4).max(0.0);
        self.hero_squash *= (1.0 - raw_dt * 12.0).max(0.0);
        self.overdrive_cutin_timer = (self.overdrive_cutin_timer - raw_dt).max(0.0);
        if self.overdrive_timer > 0.0 {
            self.overdrive_timer = (self.overdrive_timer - raw_dt).max(0.0);
            self.overdrive_meteor_cd -= raw_dt;
            if self.overdrive_meteor_cd <= 0.0 {
                self.overdrive_meteor_cd = 0.28;
                let angle = self.rng.next_f32() * std::f32::consts::TAU;
                let dist = 70.0 + self.rng.next_f32() * 240.0;
                let mpos = self.hero_pos + vec2(angle.cos() * dist, angle.sin() * dist);
                self.scorches.push(ScorchMark {
                    pos: mpos,
                    radius: 44.0,
                    life: 8.0,
                });
                self.spawn_burst(
                    mpos,
                    14,
                    Color::from_rgba(255, 110, 45, 255),
                    VfxKind::Flame,
                    220.0,
                );
                let mdmg = 48.0 + (self.level as f32) * 5.0;
                self.damage_enemies_in_radius(mpos, 86.0, mdmg, 240.0);
            }
            if self.overdrive_timer <= 0.0 {
                unsafe {
                    emberfall_set_boss_music(if self.boss_spawned && !self.boss_slain {
                        if self.boss_enraged { 2 } else { 1 }
                    } else {
                        0
                    });
                }
            }
        }
        let target_zoom = if self.overdrive_timer > 0.0 { 1.07 } else { 1.0 };
        self.cam_zoom += (target_zoom - self.cam_zoom) * (raw_dt * 5.0).min(1.0);
        self.peak_combo = self.peak_combo.max(self.combo);

        if let Some(boss) = self.enemies.iter().find(|e| e.kind == EnemyKind::Boss) {
            let ratio = (boss.hp / boss.max_hp.max(1.0)).clamp(0.0, 1.0);
            if self.boss_hp_trail < ratio {
                self.boss_hp_trail = ratio;
            } else {
                self.boss_hp_trail = (self.boss_hp_trail - raw_dt * 0.22).max(ratio);
            }
        } else {
            self.boss_hp_trail = 0.0;
        }

        for b in &mut self.barrels {
            b.hit_flash = (b.hit_flash - dt).max(0.0);
        }
        for p in &mut self.pillars {
            p.hit_flash = (p.hit_flash - dt).max(0.0);
        }
        for ob in &mut self.obelisks {
            ob.hit_flash = (ob.hit_flash - dt).max(0.0);
        }
        let mut d_i = 0;
        while d_i < self.dying_enemies.len() {
            self.dying_enemies[d_i].life -= raw_dt;
            if self.dying_enemies[d_i].life <= 0.0 {
                self.dying_enemies.swap_remove(d_i);
            } else {
                let de = &mut self.dying_enemies[d_i];
                de.pos += de.vel * raw_dt;
                de.vel *= (1.0 - raw_dt * 5.0).max(0.0);
                de.rot += de.vrot * raw_dt;
                d_i += 1;
            }
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

        // Dense Ambient Biome Weather Particles (2.5x density!)
        self.weather_timer -= raw_dt;
        if self.weather_timer <= 0.0 {
            self.weather_timer = 0.04;
            for _ in 0..2 {
                let wx = self.cam_pos.x + self.rng.range_f32(-640.0, 640.0);
                let wy = self.cam_pos.y + self.rng.range_f32(-420.0, 420.0);
                let (wcol, wvel, wkind, wsize) = match self.biome {
                    DailyBiome::VolcanicRift => (
                        Color::from_rgba(251, 146, 60, 165),
                        vec2(self.rng.range_f32(-28.0, 28.0), -74.0),
                        VfxKind::Flame,
                        self.rng.range_f32(13.0, 24.0),
                    ),
                    DailyBiome::FrostAbyss => (
                        Color::from_rgba(186, 230, 253, 175),
                        vec2(self.rng.range_f32(-48.0, -12.0), 64.0),
                        VfxKind::Star,
                        self.rng.range_f32(11.0, 20.0),
                    ),
                    DailyBiome::ObsidianSanctuary => (
                        Color::from_rgba(192, 132, 252, 160),
                        vec2(self.rng.range_f32(-22.0, 22.0), -42.0),
                        VfxKind::Light,
                        self.rng.range_f32(14.0, 26.0),
                    ),
                };
                self.spawn_particle(vec2(wx, wy), wvel, wsize, 1.45, wcol, wkind);
            }
        }

        // Active Biome Weather Storm Events every 11s
        self.storm_timer -= dt;
        if self.storm_timer <= 0.0 {
            self.storm_timer = 11.0;
            match self.biome {
                DailyBiome::ObsidianSanctuary => {
                    // Void Blood Lightning strike on an enemy cluster
                    let strike_pos = if !self.enemies.is_empty() {
                        let idx = self.rng.range_usize(0, self.enemies.len());
                        self.enemies[idx].pos
                    } else {
                        self.hero_pos + vec2(self.rng.range_f32(-220.0, 220.0), self.rng.range_f32(-180.0, 180.0))
                    };
                    self.lightning_flash = 0.45;
                    self.add_trauma(0.42);
                    self.lightnings.push(LightningArc {
                        points: vec![
                            strike_pos + vec2(-35.0, -380.0),
                            strike_pos + vec2(22.0, -210.0),
                            strike_pos + vec2(-18.0, -95.0),
                            strike_pos,
                        ],
                        life: 0.34,
                    });
                    self.spawn_burst(strike_pos, 22, Color::from_rgba(192, 132, 252, 255), VfxKind::Spark, 260.0);
                    self.damage_enemies_in_radius(strike_pos, 135.0, 52.0, 180.0);
                    self.float_texts.push(FloatText {
                        pos: strike_pos + vec2(0.0, -36.0),
                        text: "VOID STORM!".to_string(),
                        color: Color::from_rgba(192, 132, 252, 255),
                        life: 1.1,
                        scale: 20.0,
                    });
                    play_sfx(3);
                }
                DailyBiome::VolcanicRift => {
                    // Magma Geyser Eruption
                    let gpos = (self.hero_pos + vec2(self.rng.range_f32(-240.0, 240.0), self.rng.range_f32(-180.0, 180.0)))
                        .clamp(Vec2::splat(-ARENA_HALF + 80.0), Vec2::splat(ARENA_HALF - 80.0));
                    self.add_trauma(0.45);
                    self.scorches.push(ScorchMark {
                        pos: gpos,
                        radius: 95.0,
                        life: 16.0,
                    });
                    self.spawn_burst(gpos, 26, Color::from_rgba(251, 146, 60, 255), VfxKind::Fire, 290.0);
                    self.damage_enemies_in_radius(gpos, 140.0, 58.0, 210.0);
                    self.float_texts.push(FloatText {
                        pos: gpos + vec2(0.0, -36.0),
                        text: "MAGMA ERUPTION!".to_string(),
                        color: Color::from_rgba(251, 146, 60, 255),
                        life: 1.1,
                        scale: 20.0,
                    });
                    play_sfx(4);
                }
                DailyBiome::FrostAbyss => {
                    // Blizzard Squall: chills nearby enemies & spawns ice crystals
                    self.lightning_flash = 0.28;
                    let mut chilled = 0;
                    for e in &mut self.enemies {
                        if e.pos.distance(self.hero_pos) < 380.0 && chilled < 8 {
                            e.chill_timer = e.chill_timer.max(2.8);
                            chilled += 1;
                        }
                    }
                    self.spawn_burst(
                        self.hero_pos,
                        20,
                        Color::from_rgba(125, 211, 252, 240),
                        VfxKind::Star,
                        250.0,
                    );
                    self.float_texts.push(FloatText {
                        pos: self.hero_pos + vec2(0.0, -48.0),
                        text: "BLIZZARD SQUALL!".to_string(),
                        color: Color::from_rgba(125, 211, 252, 255),
                        life: 1.1,
                        scale: 20.0,
                    });
                    play_sfx(3);
                }
            }
        }

        // Record #1 Champion Ghost path every 0.5s (max 200 samples = 400 i16s)
        self.ghost_sample_timer += dt;
        if self.ghost_sample_timer >= 0.5 && self.ghost_record.len() < 400 {
            self.ghost_sample_timer -= 0.5;
            self.ghost_record.push(self.hero_pos.x as i16);
            self.ghost_record.push(self.hero_pos.y as i16);
        }

        // Champion's Revenge: Surpass #1 Champion Ghost score for +1500 PTS!
        if !self.revenge_claimed {
            let champ_target = self
                .social
                .leaderboard
                .first()
                .map(|e| e.score)
                .unwrap_or(0)
                .max(self.social.yesterday_score.max(0) as u32);
            if champ_target > 0 && self.score > champ_target {
                self.revenge_claimed = true;
                self.score += 1500;
                self.shockwave = 1.0;
                self.add_trauma(0.55);
                let cname = if !self.social.ghost_username.is_empty() {
                    self.social.ghost_username.clone()
                } else if !self.social.yesterday_champion.is_empty() {
                    self.social.yesterday_champion.clone()
                } else {
                    "CHAMPION".to_string()
                };
                self.evo_banner_text = format!("CHAMPION'S REVENGE vs u/{}! (+1500)", cname);
                self.evo_banner_color = Color::from_rgba(250, 204, 21, 255);
                self.evo_banner_timer = 3.0;
                play_sfx(5);
            }
        }

        // Check 90-second Daily Run completion (skipped in Endless Abyss)
        if !self.endless_mode && self.elapsed >= RUN_DURATION {
            self.check_medals();
            self.phase = Phase::Victory;
            self.score += 1500;
            unsafe {
                emberfall_set_boss_music(0);
                emberfall_toggle_drone(0);
            }
            self.report_run_once();
            play_sfx(12);
            return;
        }

        // Endless Abyss: spawn scaling Ascended Bosses every 55s
        if self.endless_mode && self.elapsed >= self.next_endless_boss_time {
            self.endless_wave += 1;
            self.next_endless_boss_time += 55.0;
            self.social.best_endless_wave = self.social.best_endless_wave.max(self.endless_wave);
            self.boss_enraged = false;
            self.boss_banner_timer = 3.2;
            self.evo_banner_text = format!("WAVE {} · ASCENDED LICH!", self.endless_wave);
            self.evo_banner_color = Color::from_rgba(192, 132, 252, 255);
            self.evo_banner_timer = 3.0;
            self.add_trauma(0.75);
            self.hitstop = self.hitstop.max(0.075);
            self.shockwave = 1.0;
            self.spawn_enemy(EnemyKind::Boss, false);
            self.check_medals();
            let bquote = match self.biome {
                DailyBiome::ObsidianSanctuary => "THE ABYSS RECLAIMS YOUR SOUL!",
                DailyBiome::VolcanicRift => "BURN IN ETERNAL MAGMA, MORTAL!",
                DailyBiome::FrostAbyss => "KNEEL BEFORE THE ENDLESS WINTER!",
            };
            self.trigger_banter(true, bquote);
            unsafe {
                emberfall_set_boss_music(1);
            }
            play_sfx(6);
        }

        // Movement, Articulated Stride Dust & Witch-Time Perfect Dodge
        let input_dir = self.update_input();
        if self.dash_timer > 0.0 {
            self.dash_timer -= hero_dt;
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

            // Witch-Time Perfect Dodge: Dashing close to an enemy shot or charging Boss ring!
            if !self.dodge_triggered_this_dash {
                let near_shot = self
                    .projectiles
                    .iter()
                    .any(|p| p.from_enemy && p.pos.distance(self.hero_pos) <= 58.0);
                let near_ring = self.telegraphs.iter().any(|r| {
                    r.elapsed >= r.duration * 0.5
                        && r.pos.distance(self.hero_pos) <= r.max_radius + 16.0
                });
                if near_shot || near_ring {
                    self.dodge_triggered_this_dash = true;
                    self.witch_time_dodges += 1;
                    self.closest_dodge_hp = self.closest_dodge_hp.min(self.hero_hp);
                    self.ult_charge = (self.ult_charge + 0.12).min(1.0);
                    self.slow_mo_timer = 0.42;
                    self.score += 150;
                    self.dash_cd = (self.dash_cd * 0.65).max(0.2);
                    self.shockwave = 0.85;
                    self.float_texts.push(FloatText {
                        pos: self.hero_pos + vec2(0.0, -42.0),
                        text: "WITCH-TIME! +12% EMBER".to_string(),
                        color: Color::from_rgba(56, 232, 255, 255),
                        life: 1.2,
                        scale: 23.0,
                    });
                    play_sfx(11);
                    self.check_bounties();
                }
            }
        } else {
            let spd_bless = if self.active_blessing() == 2 { 1.15 } else { 1.0 };
            let target_vel = input_dir * self.build.move_speed() * spd_bless;
            self.hero_vel = self.hero_vel.lerp(target_vel, (hero_dt * 14.0).min(1.0));
        }

        if self.hero_vel.x.abs() > 12.0 {
            self.hero_facing_left = self.hero_vel.x < 0.0;
        }
        if self.hero_vel.length() > 25.0 {
            let prev_step = (self.hero_stride / std::f32::consts::PI).floor() as i32;
            self.hero_stride += hero_dt * 14.5;
            let new_step = (self.hero_stride / std::f32::consts::PI).floor() as i32;
            if new_step != prev_step {
                let foot_off = if new_step % 2 == 0 { -10.0 } else { 10.0 };
                self.spawn_particle(
                    self.hero_pos + vec2(foot_off, 28.0),
                    -self.hero_vel * 0.18 + vec2(0.0, -14.0),
                    15.0,
                    0.26,
                    Color::from_rgba(203, 213, 225, 130),
                    VfxKind::Light,
                );
            }
        }

        self.hero_pos += self.hero_vel * hero_dt;
        self.hero_pos = self.hero_pos.clamp(
            Vec2::splat(-ARENA_HALF + 38.0),
            Vec2::splat(ARENA_HALF - 38.0),
        );
        self.cam_pos = self.cam_pos.lerp(self.hero_pos, (raw_dt * 8.5).min(1.0));

        // Faction Companion Pet: spring-follow hero & fire themed homing bolts!
        let pet_target = self.hero_pos
            + vec2(
                if self.hero_facing_left { 46.0 } else { -46.0 },
                -24.0 + (self.elapsed * 4.8).sin() * 7.0,
            );
        self.pet_pos = self.pet_pos.lerp(pet_target, (hero_dt * 6.5).min(1.0));
        self.pet_facing_left = self.hero_facing_left;
        self.pet_attack_cd -= dt;
        if self.pet_attack_cd <= 0.0 {
            if let Some(t_idx) = self.nearest_enemy_index(self.pet_pos, 420.0) {
                self.pet_attack_cd = 1.65;
                let dir = (self.enemies[t_idx].pos - self.pet_pos).normalize_or_zero();
                let pet_col = self.hero_class.accent();
                self.projectiles.push(Projectile {
                    pos: self.pet_pos,
                    vel: dir * 480.0,
                    damage: 18.0 + (self.level as f32) * 3.0,
                    radius: 10.0,
                    life: 1.2,
                    pierce: 1,
                    from_enemy: false,
                    color: pet_col,
                });
                self.spawn_burst(self.pet_pos, 5, pet_col, VfxKind::Star, 120.0);
                play_sfx_pitched(1, 1.38);
            } else {
                self.pet_attack_cd = 0.25;
            }
        }

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
            let bquote = match self.biome {
                DailyBiome::ObsidianSanctuary => "MALAKOR AWAKENS! YOUR LIGHT DIES HERE!",
                DailyBiome::VolcanicRift => "IGNIS REX RISES! TASTE MOLTEN FURY!",
                DailyBiome::FrostAbyss => "SKADI DESCENDS! FREEZE IN ETERNITY!",
            };
            self.trigger_banter(true, bquote);
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

        // WEAPONS (Auto-aim & Auto-fire scaled by Haste + Evolutions + Overdrive)
        let mut haste = self.build.haste_mult();
        if self.modifier == DailyModifier::ArcaneSurge {
            haste *= 1.2;
        }
        if self.overdrive_timer > 0.0 {
            haste *= 2.0;
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
                self.hero_aim_angle = angle;
                self.hero_attack_swing = 1.0;
                self.hero_squash = 0.24;
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
                self.hero_aim_angle = base_dir.y.atan2(base_dir.x);
                self.hero_attack_swing = self.hero_attack_swing.max(0.85);
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
            e.lunge_timer = (e.lunge_timer - dt * 3.2).max(0.0);
            e.squash *= (1.0 - dt * 12.0).max(0.0);
            let chill_mult = if e.chill_timer > 0.0 { 0.58 } else { 1.0 };
            let eff_speed = e.speed * chill_mult;
            e.anim_phase += dt * (eff_speed * 0.095).max(3.5);
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
                        e.lunge_timer = 1.0;
                        e.squash = 0.28;
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
                        e.lunge_timer = 1.0;
                        e.squash = 0.34;
                        match self.biome {
                            DailyBiome::VolcanicRift => {
                                // IGNIS REX: Meteor Barrage (3 staggered magma rings!)
                                for m in 0..3 {
                                    let off = if m == 0 {
                                        Vec2::ZERO
                                    } else {
                                        Vec2::from_angle(e.phase_timer * 2.1 + (m as f32) * 2.2) * 125.0
                                    };
                                    new_telegraphs.push(TelegraphRing {
                                        pos: hero_p + off,
                                        max_radius: if boss_enraged { 132.0 } else { 116.0 },
                                        elapsed: 0.0,
                                        duration: 0.88 + (m as f32) * 0.14,
                                        damage: if boss_enraged { 30.0 } else { 25.0 },
                                    });
                                }
                            }
                            DailyBiome::FrostAbyss => {
                                // SKADI: Glacial Shatter Ring
                                new_telegraphs.push(TelegraphRing {
                                    pos: hero_p,
                                    max_radius: if boss_enraged { 155.0 } else { 135.0 },
                                    elapsed: 0.0,
                                    duration: 0.95,
                                    damage: if boss_enraged { 28.0 } else { 24.0 },
                                });
                            }
                            DailyBiome::ObsidianSanctuary => {
                                // MALAKOR: Void Slam + Phase 2 Echo Slam
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
                            }
                        }
                        let spokes = match self.biome {
                            DailyBiome::FrostAbyss => if boss_enraged { 14 } else { 10 },
                            _ => if boss_enraged { 12 } else { 8 },
                        };
                        let shot_spd = match self.biome {
                            DailyBiome::FrostAbyss => if boss_enraged { 295.0 } else { 260.0 },
                            _ => if boss_enraged { 265.0 } else { 235.0 },
                        };
                        let shot_col = if boss_enraged {
                            Color::from_rgba(244, 63, 94, 255)
                        } else {
                            biome_boss_color(self.biome)
                        };
                        let base_a = e.phase_timer * 1.4;
                        for k in 0..spokes {
                            let a = base_a + (k as f32) * (std::f32::consts::TAU / (spokes as f32));
                            enemy_shots.push(Projectile {
                                pos: e.pos,
                                vel: Vec2::from_angle(a) * shot_spd,
                                damage: 16.0,
                                radius: 10.0,
                                life: 2.8,
                                pierce: 1,
                                from_enemy: true,
                                color: shot_col,
                            });
                        }
                    }
                }
                _ => {
                    let lunge_boost = if dist < 110.0 {
                        e.lunge_timer = e.lunge_timer.max(0.45);
                        1.18
                    } else {
                        1.0
                    };
                    e.vel = e.vel.lerp(dir * eff_speed * lunge_boost, (dt * 7.0).min(1.0));
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
            self.hurt_flash = 0.55;
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
                play_sfx(13);
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
                    self.check_bounties();
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

        let z = self.cam_zoom.clamp(0.55, 1.35);
        let cam = Camera2D {
            target: self.cam_pos + shake_off,
            zoom: vec2(2.0 * z / sw, -2.0 * z / sh),
            render_target: rt.cloned(),
            ..Default::default()
        };
        set_camera(&cam);
        if rt.is_some() {
            clear_background(Color::from_rgba(6, 9, 20, 255));
        }

        // 1. Seamless Basalt Dungeon Floor Tiling tinted by Daily Biome!
        let tile = 256.0;
        let cull_x = (sw * 0.72) / z;
        let cull_y = (sh * 0.72) / z;
        let min_x = ((self.cam_pos.x - cull_x) / tile).floor() as i32;
        let max_x = ((self.cam_pos.x + cull_x) / tile).ceil() as i32;
        let min_y = ((self.cam_pos.y - cull_y) / tile).floor() as i32;
        let max_y = ((self.cam_pos.y + cull_y) / tile).ceil() as i32;

        let floor_tex = assets.floor_for_biome(self.biome);
        let floor_tint = match self.biome {
            DailyBiome::ObsidianSanctuary => Color::from_rgba(195, 195, 232, 255),
            DailyBiome::VolcanicRift => Color::from_rgba(242, 234, 228, 255),
            DailyBiome::FrostAbyss => Color::from_rgba(232, 244, 252, 255),
        };
        let border_col = match self.biome {
            DailyBiome::ObsidianSanctuary => Color::from_rgba(192, 132, 252, 200),
            DailyBiome::VolcanicRift => Color::from_rgba(249, 115, 22, 205),
            DailyBiome::FrostAbyss => Color::from_rgba(56, 189, 248, 205),
        };

        for tx in min_x..=max_x {
            for ty in min_y..=max_y {
                draw_texture_ex(
                    floor_tex,
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

        // Arena Glowing Runic Border & Central Sanctuary Sigil
        draw_rectangle_lines(
            -ARENA_HALF,
            -ARENA_HALF,
            ARENA_HALF * 2.0,
            ARENA_HALF * 2.0,
            8.0,
            border_col,
        );
        let sigil_pulse = 0.22 + (get_time() as f32 * 2.1).sin().abs() * 0.16;
        draw_circle_lines(
            0.0,
            0.0,
            180.0,
            2.5,
            Color::new(border_col.r, border_col.g, border_col.b, sigil_pulse),
        );
        draw_circle_lines(
            0.0,
            0.0,
            115.0,
            1.8,
            Color::new(0.90, 0.72, 0.36, sigil_pulse),
        );

        // Live SplashOrbit / IntroCinematic Boss Rift Invocation in World Space
        if self.phase == Phase::Title || self.phase == Phase::Intro {
            let boss_tex = assets.boss_for_biome(self.biome);
            let show_boss = self.phase == Phase::Title || self.intro_time >= 1.15;
            if show_boss {
                let appear = if self.phase == Phase::Title {
                    0.82
                } else {
                    ((self.intro_time - 1.15) * 2.6).clamp(0.0, 1.0)
                };
                let b_sz = 148.0 * appear;
                draw_circle(
                    0.0,
                    -20.0,
                    92.0 * appear,
                    Color::new(border_col.r, border_col.g, border_col.b, 0.22 * appear),
                );
                draw_texture_ex(
                    boss_tex,
                    -b_sz * 0.5,
                    -36.0 - b_sz * 0.5 + (get_time() as f32 * 2.4).sin() * 4.0,
                    Color::new(1.0, 1.0, 1.0, appear),
                    DrawTextureParams {
                        dest_size: Some(vec2(b_sz, b_sz)),
                        ..Default::default()
                    },
                );
            }
        }

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

        // 1a-bis. Destructible Runic Pillars
        for pillar in &self.pillars {
            if pillar.shattered {
                draw_circle(pillar.pos.x, pillar.pos.y, 24.0, Color::from_rgba(24, 18, 38, 130));
                continue;
            }
            let ratio = (pillar.hp / pillar.max_hp.max(1.0)).clamp(0.0, 1.0);
            let flash = pillar.hit_flash > 0.0;
            draw_ellipse(pillar.pos.x, pillar.pos.y + 22.0, 26.0, 11.0, 0.0, Color::from_rgba(0, 0, 0, 135));
            draw_rectangle(
                pillar.pos.x - 16.0,
                pillar.pos.y - 42.0,
                32.0,
                58.0,
                if flash { WHITE } else { Color::from_rgba(38, 41, 58, 255) },
            );
            draw_rectangle_lines(
                pillar.pos.x - 16.0,
                pillar.pos.y - 42.0,
                32.0,
                58.0,
                2.0,
                border_col,
            );
            let rune_h = 38.0 * ratio;
            draw_rectangle(
                pillar.pos.x - 4.0,
                pillar.pos.y + 6.0 - rune_h,
                8.0,
                rune_h,
                border_col,
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

        // 3b. #1 Champion Ghost Replay (Skeletal Deformable Phantom + Crown + Live Duel Delta)
        if !self.social.ghost_points.is_empty()
            && matches!(self.phase, Phase::Playing | Phase::Paused | Phase::LevelUp)
        {
            let pts = &self.social.ghost_points;
            let idx_f = (self.elapsed / 0.5).max(0.0);
            let i0 = (idx_f.floor() as usize).min(pts.len() - 1);
            let i1 = (i0 + 1).min(pts.len() - 1);
            let frac = idx_f - (i0 as f32);
            let gpos = pts[i0].lerp(pts[i1], frac);
            let gvel = (pts[i1] - pts[i0]) * 2.0;
            let gflip = gvel.x < -2.0;
            let glean = (gvel.x / 220.0).clamp(-1.0, 1.0);
            let gmove = (gvel.length() / 180.0).clamp(0.15, 1.0);
            let gstride = self.elapsed * 13.0;

            draw_texture_ex(
                &assets.vfx_light,
                gpos.x - 54.0,
                gpos.y - 54.0,
                Color::new(1.0, 0.85, 0.25, 0.28),
                DrawTextureParams {
                    dest_size: Some(vec2(108.0, 108.0)),
                    ..Default::default()
                },
            );
            draw_articulated_sprite(
                &assets.hero,
                gpos,
                vec2(84.0, 84.0),
                gflip,
                Color::new(1.0, 0.86, 0.28, 0.48),
                gstride,
                gmove,
                glean,
                Vec2::ZERO,
                (self.elapsed * 3.5).sin(),
                0.68,
            );

            // Floating Golden Crown above #1 Ghost
            let cy = gpos.y - 48.0;
            draw_triangle(
                vec2(gpos.x - 10.0, cy),
                vec2(gpos.x - 6.0, cy - 9.0),
                vec2(gpos.x - 2.0, cy),
                Color::from_rgba(250, 204, 21, 235),
            );
            draw_triangle(
                vec2(gpos.x - 4.0, cy),
                vec2(gpos.x, cy - 12.0),
                vec2(gpos.x + 4.0, cy),
                Color::from_rgba(253, 224, 71, 250),
            );
            draw_triangle(
                vec2(gpos.x + 2.0, cy),
                vec2(gpos.x + 6.0, cy - 9.0),
                vec2(gpos.x + 10.0, cy),
                Color::from_rgba(250, 204, 21, 235),
            );

            let champ_sc = self
                .social
                .leaderboard
                .first()
                .map(|e| e.score)
                .unwrap_or(self.social.yesterday_score.max(0) as u32);
            let pace_sc = ((champ_sc as f32) * (self.elapsed / RUN_DURATION).clamp(0.05, 1.0)) as i32;
            let delta = (self.score as i32) - pace_sc;
            let glbl = if self.social.ghost_username.is_empty() {
                format!("#1 GHOST ({:+})", delta)
            } else {
                format!("u/{} ({:+})", self.social.ghost_username, delta)
            };
            let gd = measure_ui_text(font, &glbl, 13);
            draw_ui_text(
                font,
                &glbl,
                gpos.x - gd.width * 0.5,
                cy - 15.0,
                13.0,
                if delta >= 0 {
                    Color::from_rgba(74, 222, 128, 240)
                } else {
                    Color::from_rgba(253, 224, 71, 235)
                },
            );
        }

        // 4. Hero Aura & Dash Afterimages
        let aura_col = self.hero_class.accent();
        let aura_sz = 110.0 + (self.elapsed * 4.0).sin() * 6.0 + self.fever * 24.0;
        draw_texture_ex(
            &assets.vfx_light,
            self.hero_pos.x - aura_sz * 0.5,
            self.hero_pos.y - aura_sz * 0.5,
            Color::new(aura_col.r, aura_col.g, aura_col.b, 0.26 + self.fever * 0.14),
            DrawTextureParams {
                dest_size: Some(vec2(aura_sz, aura_sz)),
                ..Default::default()
            },
        );
        if self.fever > 0.65 {
            let f_pulse = 44.0 + (self.elapsed * 9.0).sin() * 5.0;
            draw_circle_lines(
                self.hero_pos.x,
                self.hero_pos.y + 18.0,
                f_pulse,
                2.4,
                Color::from_rgba(250, 204, 21, 185),
            );
        }

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

        // 4b. Dying Enemies (Ragdoll knockback, spin & dissolve!)
        for de in &self.dying_enemies {
            let (tex, base_sz) = match de.kind {
                EnemyKind::Imp => (&assets.imp, 64.0),
                EnemyKind::Golem => (&assets.golem, 94.0),
                EnemyKind::Wraith => (&assets.wraith, 76.0),
                EnemyKind::Boss => (assets.boss_for_biome(self.biome), 186.0),
            };
            let t = (de.life / de.max_life).clamp(0.0, 1.0);
            let sz = (if de.is_elite { base_sz * 1.24 } else { base_sz }) * (0.45 + 0.55 * t);
            draw_texture_ex(
                tex,
                de.pos.x - sz * 0.5,
                de.pos.y - sz * 0.5,
                Color::new(1.0, 0.45 * t, 0.35 * t, t * 0.88),
                DrawTextureParams {
                    dest_size: Some(vec2(sz, sz)),
                    flip_x: de.facing_left,
                    rotation: de.rot,
                    ..Default::default()
                },
            );
        }

        // 5. Enemies (Real 2D Skeletal Mesh Deformation: actual legs walk, torso leans & lunges!)
        for e in &self.enemies {
            let (tex, base_sz) = match e.kind {
                EnemyKind::Imp => (&assets.imp, 64.0),
                EnemyKind::Golem => (&assets.golem, 94.0),
                EnemyKind::Wraith => (&assets.wraith, 76.0),
                EnemyKind::Boss => (assets.boss_for_biome(self.biome), 186.0),
            };
            let draw_sz = if e.is_elite { base_sz * 1.24 } else { base_sz };
            let sx = (1.0 + e.squash).clamp(0.68, 1.42);
            let sy = (1.0 - e.squash).clamp(0.68, 1.42);
            let dw = draw_sz * sx;
            let dh = draw_sz * sy;
            let to_h = (self.hero_pos - e.pos).normalize_or_zero();
            let lunge_vec = to_h * (e.lunge_timer * 18.0);

            // Ground shadow
            draw_ellipse(
                e.pos.x,
                e.pos.y + draw_sz * 0.34,
                dw * 0.33,
                draw_sz * 0.12,
                0.0,
                Color::from_rgba(0, 0, 0, 125),
            );

            // Boss Ritual Circle & Phase II Crimson Eclipse Sigil
            if e.kind == EnemyKind::Boss {
                let bcol = if self.boss_enraged {
                    Color::from_rgba(244, 63, 94, 230)
                } else {
                    biome_boss_color(self.biome)
                };
                draw_poly_lines(
                    e.pos.x,
                    e.pos.y + 18.0,
                    8,
                    draw_sz * 0.54,
                    self.elapsed * 28.0,
                    3.0,
                    bcol,
                );
                if self.boss_enraged {
                    draw_poly_lines(
                        e.pos.x,
                        e.pos.y + 18.0,
                        6,
                        draw_sz * 0.66,
                        -self.elapsed * 42.0,
                        2.5,
                        Color::from_rgba(250, 204, 21, 210),
                    );
                }
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

            let e_lean = (e.vel.x / 180.0).clamp(-1.0, 1.0);
            let e_move = (e.vel.length() / 110.0).clamp(0.25, 1.15);

            if e.hit_flash > 0.0 {
                if let Some(mat) = &shaders.hit_flash {
                    mat.set_uniform("u_flash", (e.hit_flash / 0.12).clamp(0.0, 1.0));
                    gl_use_material(mat);
                }
            }
            let e_split_u = match e.kind {
                EnemyKind::Imp => 0.48,
                EnemyKind::Golem => 0.52,
                EnemyKind::Wraith => 0.50,
                EnemyKind::Boss => 0.50,
            };
            draw_articulated_sprite(
                tex,
                e.pos,
                vec2(dw, dh),
                e.facing_left,
                tint,
                e.anim_phase,
                e_move,
                e_lean,
                lunge_vec,
                (e.anim_phase * 0.5).sin(),
                e_split_u,
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

        // 6. Hero Sprite (Real 2D Skeletal Mesh Deformation: BOTH legs walk symmetrically, torso leans & weapon arm lunges!)
        let moving_ratio = (self.hero_vel.length() / 230.0).clamp(0.0, 1.0);
        let hsx = (1.0 + self.hero_squash).clamp(0.72, 1.38);
        let hsy = (1.0 - self.hero_squash).clamp(0.72, 1.38);
        let hw = 92.0 * hsx;
        let hh = 92.0 * hsy;
        let hero_lean = (self.hero_vel.x / 240.0).clamp(-1.0, 1.0);
        let attack_lunge = Vec2::from_angle(self.hero_aim_angle) * (self.hero_attack_swing * 15.0);

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
        let hero_split_u = match self.hero_class {
            HeroClass::SunKnight => 0.68,
            HeroClass::ArcanePyromancer => 0.42,
            HeroClass::ShadowAssassin => 0.52,
        };
        draw_articulated_sprite(
            hero_tex,
            self.hero_pos,
            vec2(hw, hh),
            self.hero_facing_left,
            hero_tint,
            self.hero_stride,
            moving_ratio,
            hero_lean,
            attack_lunge,
            (self.elapsed * 3.8).sin(),
            hero_split_u,
        );

        // 6b. Faction Companion Pet (Solar Phoenix / Astral Owl / Shadow Wolf Pup)
        {
            let pet_tex = assets.pet_for_class(self.hero_class);
            let pet_bob = (self.elapsed * 6.0).sin() * 3.5;
            let pet_sz = 50.0;
            let p_col = self.hero_class.accent();
            draw_ellipse(
                self.pet_pos.x,
                self.pet_pos.y + 24.0,
                14.0,
                5.5,
                0.0,
                Color::from_rgba(0, 0, 0, 115),
            );
            draw_texture_ex(
                &assets.vfx_light,
                self.pet_pos.x - 28.0,
                self.pet_pos.y + pet_bob - 28.0,
                Color::new(p_col.r, p_col.g, p_col.b, 0.30),
                DrawTextureParams {
                    dest_size: Some(vec2(56.0, 56.0)),
                    ..Default::default()
                },
            );
            draw_texture_ex(
                pet_tex,
                self.pet_pos.x - pet_sz * 0.5,
                self.pet_pos.y + pet_bob - pet_sz * 0.5,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(pet_sz, pet_sz)),
                    flip_x: self.pet_facing_left,
                    rotation: (self.elapsed * 4.5).cos() * 0.07,
                    ..Default::default()
                },
            );
        }

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

    fn process_js_commands(&mut self) {
        let cmds: Vec<JsCmd> = unsafe {
            let ptr = &raw mut JS_CMD_QUEUE;
            (*ptr).drain(..).collect()
        };
        for cmd in cmds {
            match cmd {
                JsCmd::StartRun(hc, endless) => {
                    self.hero_class = match hc {
                        1 => HeroClass::ArcanePyromancer,
                        2 => HeroClass::ShadowAssassin,
                        _ => HeroClass::SunKnight,
                    };
                    self.launch_cutscene(endless != 0, false);
                }
                JsCmd::SkipIntro => {
                    if self.phase == Phase::Intro {
                        self.start_run(self.intro_pending_endless);
                    }
                }
                JsCmd::TriggerOverdrive => {
                    self.trigger_ultimate();
                }
                JsCmd::PickUpgrade(idx) => {
                    if self.phase == Phase::LevelUp {
                        self.apply_upgrade_choice((idx.max(0) as usize).min(2));
                    }
                }
                JsCmd::TogglePause => {
                    if self.phase == Phase::Playing {
                        self.phase = Phase::Paused;
                        self.joy_active = false;
                        self.joy_vec = Vec2::ZERO;
                        unsafe {
                            emberfall_toggle_drone(0);
                        }
                    } else if self.phase == Phase::Paused {
                        self.phase = Phase::Playing;
                        unsafe {
                            emberfall_toggle_drone(if self.muted { 0 } else { 1 });
                        }
                    }
                }
                JsCmd::ToggleMute => self.toggle_mute(),
                JsCmd::ToggleCrt => self.toggle_crt(),
                JsCmd::Replay => {
                    if matches!(self.phase, Phase::GameOver | Phase::Victory) {
                        self.start_run(false);
                    }
                }
                JsCmd::Share => {
                    if matches!(self.phase, Phase::GameOver | Phase::Victory) {
                        let summary = format!(
                            "{} · Lv.{} · {}x Peak",
                            self.build.build_emojis(),
                            self.level,
                            self.peak_combo
                        );
                        unsafe {
                            emberfall_share_comment(
                                self.score,
                                self.kills,
                                if self.boss_slain { 1 } else { 0 },
                                self.level,
                                self.hero_class.to_index() as u32,
                                summary.as_ptr(),
                                summary.len() as u32,
                            );
                        }
                    }
                }
                JsCmd::Vote(idx) => {
                    let mi = (idx.max(0) as usize).min(3);
                    self.social.voted_mutator = Some(mi);
                    play_sfx(0);
                    unsafe {
                        emberfall_vote_mutator(mi as i32);
                    }
                }
            }
        }
    }

    fn build_chronicle(&self) -> String {
        let bname = biome_boss_name(self.biome);
        let act1 = if self.witch_time_dodges > 0 {
            format!(
                "Outmaneuvered {} in Witch-Time at {} HP",
                bname,
                self.closest_dodge_hp.max(1.0).round() as u32
            )
        } else {
            format!("Stood firm against {}'s relentless horde", bname)
        };
        let act2 = if self.pillars_shattered > 0 || self.overdrive_count > 0 {
            format!(
                "shattered {} Runic Pillars at {}x Carnage and ignited {} Ember Overdrive(s)",
                self.pillars_shattered, self.peak_combo, self.overdrive_count
            )
        } else {
            format!(
                "carved through {} foes reaching {}x Carnage",
                self.kills, self.peak_combo
            )
        };
        let act3 = if self.boss_slain {
            format!("before executing {} in the {}", bname, self.biome.title())
        } else {
            format!("falling at Lv.{} in the {}", self.level, self.biome.title())
        };
        format!("{}, {}, {}.", act1, act2, act3)
    }

    fn sync_hud_to_js(&self) {
        let phase_id = match self.phase {
            Phase::Title => 0,
            Phase::Intro => 1,
            Phase::Playing => 2,
            Phase::Paused => 3,
            Phase::LevelUp => 4,
            Phase::GameOver | Phase::Victory => 5,
        };
        let time_rem = if self.endless_mode {
            self.elapsed
        } else {
            (RUN_DURATION - self.elapsed).max(0.0)
        };
        let (boss_hp, boss_max_hp) = if let Some(b) = self.enemies.iter().find(|e| e.kind == EnemyKind::Boss) {
            (b.hp.max(0.0), b.max_hp.max(1.0))
        } else {
            (0.0, 0.0)
        };
        let ghost_delta = if !self.social.ghost_points.is_empty() && !self.social.leaderboard.is_empty() {
            let top_score = self.social.leaderboard[0].score as f32;
            let expected = top_score * (self.elapsed / RUN_DURATION).clamp(0.0, 1.0);
            (self.score as f32 - expected).round() as i32
        } else {
            0
        };
        let rank = compute_rank(self.score, self.boss_slain);
        let chron = if matches!(self.phase, Phase::GameOver | Phase::Victory) {
            self.build_chronicle()
        } else {
            String::new()
        };
        unsafe {
            emberfall_sync_hud(
                phase_id,
                self.biome.to_index() as i32,
                self.hero_class.to_index() as i32,
                self.hero_hp.max(0.0),
                self.hero_max_hp.max(1.0),
                (self.ult_charge * 100.0).clamp(0.0, 100.0),
                self.overdrive_timer,
                self.xp,
                self.xp_next.max(1),
                self.level,
                self.score,
                self.kills,
                self.combo,
                time_rem,
                boss_hp,
                boss_max_hp,
                self.boss_hp_trail,
                ghost_delta,
                if self.muted { 1 } else { 0 },
                if self.crt_enabled { 1 } else { 0 },
                rank.as_ptr(),
                rank.len() as u32,
                chron.as_ptr(),
                chron.len() as u32,
            );
            let up_idx = |u: UpgradeId| -> i32 {
                ALL_UPGRADES.iter().position(|x| *x == u).unwrap_or(0) as i32
            };
            let up_rarity = |u: UpgradeId| -> i32 {
                let lvl = self.build.level_of(u);
                if lvl >= 3 { 2 } else if lvl >= 1 { 1 } else { 0 }
            };
            emberfall_sync_choices(
                up_idx(self.pending_choices[0]),
                up_idx(self.pending_choices[1]),
                up_idx(self.pending_choices[2]),
                up_rarity(self.pending_choices[0]),
                up_rarity(self.pending_choices[1]),
                up_rarity(self.pending_choices[2]),
                if self.bounties[0].2 { 1 } else { 0 },
                if self.bounties[1].2 { 1 } else { 0 },
                if self.bounties[2].2 { 1 } else { 0 },
            );
        }
    }

    fn draw_ui(&mut self, assets: &GameAssets) {
        let sw = screen_width();
        let sh = screen_height();
        let font = assets.ui_font.as_ref();
        let cx = sw * 0.5;
        let bcol = biome_boss_color(self.biome);
        let bname = biome_boss_name(self.biome);

        // 0. SplashOrbit: living 60fps sanctuary orbit behind Glassmorphism Title Overlay
        if self.phase == Phase::Title {
            let dt = get_frame_time().min(0.05);
            self.title_time += dt;
            self.cam_pos = vec2(
                (self.title_time * 0.34).cos() * 125.0,
                (self.title_time * 0.25).sin() * 85.0,
            );
            self.cam_zoom = 0.84;
            return;
        }

        // 1. 3.5-Second Real-Time Procedural Boss Invocation Cinematic (God of War / Hades Seamless)
        if self.phase == Phase::Intro {
            let dt = get_frame_time().min(0.05);
            self.intro_time += dt;
            let t = self.intro_time;

            if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                self.start_run(self.intro_pending_endless);
                return;
            }

            if t < 1.2 {
                let p = (t / 1.2).clamp(0.0, 1.0);
                let ease = 1.0 - (1.0 - p).powi(3);
                self.cam_zoom = 0.70 + 0.36 * ease;
                self.cam_pos = vec2(0.0, -140.0 * (1.0 - ease) - 24.0 * ease);
            } else if t < 2.5 {
                if self.intro_stage_sfx < 1 {
                    self.intro_stage_sfx = 1;
                    self.add_trauma(0.85);
                    self.shockwave = 1.0;
                    self.aberration = 0.025;
                    play_sfx(6);
                }
                self.cam_zoom = 1.06;
                self.cam_pos = vec2(0.0, -24.0);
            } else if t < 3.5 {
                let p = ((t - 2.5) / 1.0).clamp(0.0, 1.0);
                let ease = if p < 0.5 {
                    16.0 * p.powi(5)
                } else {
                    1.0 - (-2.0 * p + 2.0).powi(5) * 0.5
                };
                self.cam_zoom = 1.06 - 0.06 * ease;
                self.cam_pos = vec2(0.0, -24.0 * (1.0 - ease));
            } else {
                self.start_run(self.intro_pending_endless);
                return;
            }

            let bar_frac = if t > 2.5 {
                (1.0 - ((t - 2.5) / 1.0)).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let bar_h = sh * 0.115 * bar_frac;
            draw_rectangle(0.0, 0.0, sw, bar_h, Color::from_rgba(5, 6, 9, 250));
            draw_rectangle(0.0, sh - bar_h, sw, bar_h, Color::from_rgba(5, 6, 9, 250));

            let headline = if t < 1.25 {
                format!("DAY #{} — {}", self.day_seed, self.biome.title())
            } else {
                format!("OVERLORD AWAKENS: {}", bname)
            };
            let h_sz = if sw < 520.0 { 19.0 } else { 26.0 };
            let hd = measure_ui_text(font, &headline, h_sz as u16);
            draw_ui_text(
                font,
                &headline,
                cx - hd.width * 0.5,
                sh * 0.18,
                h_sz,
                if t < 1.25 {
                    Color::from_rgba(252, 225, 148, 245)
                } else {
                    bcol
                },
            );
            return;
        }

        // 2. Playing: Dramatic Ember Overdrive Cut-In Lightning Flash & World Banners
        if self.phase == Phase::Playing {
            if self.overdrive_cutin_timer > 0.0 {
                let p = (self.overdrive_cutin_timer / 1.0).clamp(0.0, 1.0);
                draw_rectangle_lines(
                    4.0,
                    4.0,
                    sw - 8.0,
                    sh - 8.0,
                    8.0,
                    Color::new(1.0, 0.45, 0.18, p * 0.9),
                );
                let txt = "EMBER OVERDRIVE!";
                let td = measure_ui_text(font, txt, 30);
                draw_ui_text(
                    font,
                    txt,
                    cx - td.width * 0.5,
                    sh * 0.28,
                    30.0,
                    Color::new(1.0, 0.88, 0.45, p),
                );
            }
            if self.boss_banner_timer > 0.0 {
                let a = (self.boss_banner_timer / 3.0).clamp(0.0, 1.0);
                let txt = format!("OVERLORD: {}", bname);
                let td = measure_ui_text(font, &txt, 22);
                draw_ui_text(
                    font,
                    &txt,
                    cx - td.width * 0.5,
                    sh * 0.22,
                    22.0,
                    Color::new(bcol.r, bcol.g, bcol.b, a),
                );
            }
            if self.evo_banner_timer > 0.0 && !self.evo_banner_text.is_empty() {
                let a = (self.evo_banner_timer / 3.2).clamp(0.0, 1.0);
                let td = measure_ui_text(font, &self.evo_banner_text, 20);
                draw_ui_text(
                    font,
                    &self.evo_banner_text,
                    cx - td.width * 0.5,
                    sh * 0.34,
                    20.0,
                    Color::new(
                        self.evo_banner_color.r,
                        self.evo_banner_color.g,
                        self.evo_banner_color.b,
                        a,
                    ),
                );
            }
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
        game.process_js_commands();

        if is_key_pressed(KeyCode::M) {
            game.toggle_mute();
        }
        if is_key_pressed(KeyCode::C) {
            game.toggle_crt();
        }
        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::H) {
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
            mat.set_uniform("u_crt", if game.crt_enabled { 1.0f32 } else { 0.0f32 });
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
        game.sync_hud_to_js();

        next_frame().await;
    }
}


mod assets;
mod rng;
mod weapons;

use assets::GameAssets;
use macroquad::prelude::*;
use rng::{compute_rank, DailyModifier, Rng};
use weapons::{HeroBuild, UpgradeId};

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn emberfall_get_day_number() -> u32;
    fn emberfall_on_run_complete(score: u32, kills: u32, boss_slain: u32, level: u32);
    fn emberfall_share_comment(score: u32, kills: u32, boss_slain: u32);
    fn emberfall_play_sfx(kind: u32);
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_get_day_number() -> u32 {
    20261121
}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_on_run_complete(_score: u32, _kills: u32, _boss_slain: u32, _level: u32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_share_comment(_score: u32, _kills: u32, _boss_slain: u32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn emberfall_play_sfx(_kind: u32) {}

fn play_sfx(kind: u32) {
    unsafe {
        emberfall_play_sfx(kind);
    }
}

const ARENA_HALF: f32 = 1050.0;
const RUN_DURATION: f32 = 90.0;
const BOSS_SPAWN_TIME: f32 = 55.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Title,
    Playing,
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
    rng: Rng,

    // Hero
    hero_pos: Vec2,
    hero_vel: Vec2,
    hero_hp: f32,
    hero_max_hp: f32,
    hero_facing_left: bool,
    hero_invuln: f32,
    hero_stride: f32,
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
    boss_spawned: bool,
    boss_slain: bool,
    boss_banner_timer: f32,
    shared_comment: bool,
    reported_run: bool,

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
    particles: Vec<Particle>,
    float_texts: Vec<FloatText>,
    afterimages: Vec<Afterimage>,

    // Camera & input
    cam_pos: Vec2,
    shake: f32,
    joy_active: bool,
    joy_origin: Vec2,
    joy_curr: Vec2,
    joy_vec: Vec2,
    title_time: f32,
}

impl Game {
    fn new() -> Self {
        let day_num = unsafe { emberfall_get_day_number() };
        let y = day_num / 10000;
        let m = (day_num / 100) % 100;
        let d = day_num % 100;
        let day_seed = format!("{:04}-{:02}-{:02}", y, m, d);
        let modifier = DailyModifier::from_seed(&day_seed);
        let rng = Rng::from_seed_str(&day_seed);

        Self {
            phase: Phase::Title,
            day_seed,
            modifier,
            rng,
            hero_pos: Vec2::ZERO,
            hero_vel: Vec2::ZERO,
            hero_hp: 120.0,
            hero_max_hp: 120.0,
            hero_facing_left: false,
            hero_invuln: 0.0,
            hero_stride: 0.0,
            dash_cd: 0.0,
            dash_timer: 0.0,
            dash_dir: Vec2::X,
            build: HeroBuild::default(),
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
            boss_spawned: false,
            boss_slain: false,
            boss_banner_timer: 0.0,
            shared_comment: false,
            reported_run: false,
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
            particles: Vec::new(),
            float_texts: Vec::new(),
            afterimages: Vec::new(),
            cam_pos: Vec2::ZERO,
            shake: 0.0,
            joy_active: false,
            joy_origin: Vec2::ZERO,
            joy_curr: Vec2::ZERO,
            joy_vec: Vec2::ZERO,
            title_time: 0.0,
        }
    }

    fn start_run(&mut self) {
        self.rng = Rng::from_seed_str(&self.day_seed);
        self.phase = Phase::Playing;
        self.hero_pos = Vec2::ZERO;
        self.hero_vel = Vec2::ZERO;
        self.hero_max_hp = 120.0;
        self.hero_hp = 120.0;
        self.hero_invuln = 0.6;
        self.hero_facing_left = false;
        self.dash_cd = 0.0;
        self.dash_timer = 0.0;
        self.build = HeroBuild::default();
        self.level = 1;
        self.xp = 0;
        self.xp_next = 22;
        self.elapsed = 0.0;
        self.score = 0;
        self.kills = 0;
        self.combo = 0;
        self.combo_timer = 0.0;
        self.boss_spawned = false;
        self.boss_slain = false;
        self.boss_banner_timer = 0.0;
        self.shared_comment = false;
        self.reported_run = false;
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
        self.particles.clear();
        self.float_texts.clear();
        self.afterimages.clear();
        self.cam_pos = Vec2::ZERO;
        self.shake = 0.0;
        self.joy_active = false;
        self.joy_vec = Vec2::ZERO;

        // Opening burst of enemies so action starts in second 1!
        for _ in 0..6 {
            self.spawn_enemy(EnemyKind::Imp);
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

    fn spawn_enemy(&mut self, kind: EnemyKind) {
        let ang = self.rng.range_f32(0.0, std::f32::consts::TAU);
        let dist = match kind {
            EnemyKind::Boss => 460.0,
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

        let (hp, radius, speed, damage, xp_value, score_value) = match kind {
            EnemyKind::Imp => (
                32.0 * time_scale,
                22.0,
                148.0 * speed_mult,
                12.0,
                6,
                45,
            ),
            EnemyKind::Golem => (
                125.0 * time_scale,
                34.0,
                88.0 * speed_mult,
                22.0,
                18,
                140,
            ),
            EnemyKind::Wraith => (
                60.0 * time_scale,
                25.0,
                118.0 * speed_mult,
                15.0,
                12,
                95,
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
        self.dash_cd = (2.2 - (self.build.dash_lvl as f32 - 1.0) * 0.26).max(0.95);
        self.shake = self.shake.max(6.0);
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

    fn damage_enemies_in_radius(&mut self, center: Vec2, radius: f32, base_dmg: f32, knockback: f32) {
        let crit_chance = if self.modifier == DailyModifier::BloodMoon {
            0.35
        } else {
            0.14
        };
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
        for (idx, dmg, is_crit, dir) in hits {
            self.apply_enemy_damage(idx, dmg, is_crit, dir * knockback);
        }
    }

    fn apply_enemy_damage(&mut self, idx: usize, dmg: f32, is_crit: bool, knock: Vec2) {
        if idx >= self.enemies.len() {
            return;
        }
        let pos;
        let is_boss;
        {
            let e = &mut self.enemies[idx];
            e.hp -= dmg;
            e.hit_flash = 0.12;
            is_boss = e.kind == EnemyKind::Boss;
            if !is_boss {
                e.vel += knock;
            } else {
                e.vel += knock * 0.12;
            }
            pos = e.pos;
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
            self.shake = self.shake.max(4.5);
        }
    }

    fn collect_dead_enemies(&mut self) {
        let score_mult = if self.modifier == DailyModifier::SwarmRush {
            1.3
        } else {
            1.0
        };
        let mut i = 0;
        while i < self.enemies.len() {
            if self.enemies[i].hp <= 0.0 {
                let dead = self.enemies.swap_remove(i);
                self.kills += 1;
                self.combo += 1;
                self.combo_timer = 2.6;

                let combo_bonus = 1.0 + (self.combo.min(50) as f32) * 0.02;
                let mut pts = (dead.score_value as f32 * score_mult * combo_bonus) as u32;
                if dead.kind == EnemyKind::Boss {
                    if self.modifier == DailyModifier::TitansWrath {
                        pts = (pts as f32 * 1.5) as u32;
                    }
                    self.boss_slain = true;
                    self.shake = 18.0;
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
                    let col = match dead.kind {
                        EnemyKind::Imp => Color::from_rgba(251, 146, 60, 255),
                        EnemyKind::Golem => Color::from_rgba(250, 204, 21, 255),
                        EnemyKind::Wraith => Color::from_rgba(232, 121, 249, 255),
                        EnemyKind::Boss => GOLD,
                    };
                    self.spawn_burst(dead.pos, 9, col, VfxKind::Fire, 165.0);
                    let xp_val = if self.modifier == DailyModifier::ArcaneSurge {
                        dead.xp_value * 2
                    } else {
                        dead.xp_value
                    };
                    self.gems.push(XpGem {
                        pos: dead.pos,
                        value: xp_val,
                        magnetized: false,
                        gold: dead.kind == EnemyKind::Golem,
                    });
                }
                self.score += pts;
            } else {
                i += 1;
            }
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

        // Dash button hitbox (bottom-right)
        let sw = screen_width();
        let sh = screen_height();
        let dash_center = vec2(sw - 68.0, sh - 76.0);
        let dash_r = 44.0;

        // Touch / Mouse 1-thumb virtual joystick
        let mut active_pointer: Option<(Vec2, bool)> = None;
        let ts = touches();
        if !ts.is_empty() {
            for t in &ts {
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
            if started && mp.distance(dash_center) <= dash_r + 10.0 {
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
        self.shake = (self.shake - dt * 28.0).max(0.0);

        if self.combo_timer > 0.0 {
            self.combo_timer -= dt;
            if self.combo_timer <= 0.0 {
                self.combo = 0;
            }
        }

        // Check 90-second Daily Run completion!
        if self.elapsed >= RUN_DURATION {
            self.phase = Phase::Victory;
            self.score += 1500;
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

        // Enemy Spawning (deterministic daily waves)
        if !self.boss_spawned && self.elapsed >= BOSS_SPAWN_TIME {
            self.boss_spawned = true;
            self.boss_banner_timer = 3.2;
            self.shake = 14.0;
            self.spawn_enemy(EnemyKind::Boss);
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
                self.spawn_enemy(kind);
            }
        }

        // WEAPONS (Auto-aim & Auto-fire scaled by Haste)
        let mut haste = self.build.haste_mult();
        if self.modifier == DailyModifier::ArcaneSurge {
            haste *= 1.2;
        }

        // 1. Arcane Slash
        self.slash_cd -= dt * haste;
        if self.slash_cd <= 0.0 {
            if let Some(target_idx) = self.nearest_enemy_index(self.hero_pos, 320.0) {
                let target_pos = self.enemies[target_idx].pos;
                let dir = (target_pos - self.hero_pos).normalize_or_zero();
                let angle = dir.y.atan2(dir.x);
                let radius = 92.0 + (self.build.slash_lvl as f32) * 18.0;
                let dmg = 34.0 + (self.build.slash_lvl as f32) * 14.0;
                let slash_pos = self.hero_pos + dir * (radius * 0.48);

                self.slashes.push(SlashArc {
                    pos: slash_pos,
                    angle,
                    radius,
                    life: 0.22,
                    max_life: 0.22,
                });
                self.damage_enemies_in_radius(slash_pos, radius * 0.85, dmg, 190.0);
                self.spawn_burst(
                    slash_pos,
                    6,
                    Color::from_rgba(56, 189, 248, 255),
                    VfxKind::Spark,
                    180.0,
                );
                play_sfx(1);
                self.slash_cd = (0.98 - (self.build.slash_lvl as f32 - 1.0) * 0.08).max(0.55);
            } else {
                self.slash_cd = 0.18;
            }
        }

        // 2. Ember Bolts
        self.bolt_cd -= dt * haste;
        if self.bolt_cd <= 0.0 && self.build.bolt_lvl > 0 {
            if let Some(target_idx) = self.nearest_enemy_index(self.hero_pos, 560.0) {
                let base_dir = (self.enemies[target_idx].pos - self.hero_pos).normalize_or_zero();
                let count = self.build.bolt_lvl as usize;
                let spread = 0.16;
                for i in 0..count {
                    let offset = (i as f32 - (count as f32 - 1.0) * 0.5) * spread;
                    let dir = Vec2::from_angle(base_dir.y.atan2(base_dir.x) + offset);
                    self.projectiles.push(Projectile {
                        pos: self.hero_pos + dir * 24.0,
                        vel: dir * 560.0,
                        damage: 24.0 + (self.build.bolt_lvl as f32) * 9.0,
                        radius: 11.0,
                        life: 1.3,
                        pierce: if self.build.bolt_lvl >= 3 { 2 } else { 1 },
                        from_enemy: false,
                        color: Color::from_rgba(251, 146, 60, 255),
                    });
                }
                play_sfx(1);
                self.bolt_cd = (0.92 - (self.build.bolt_lvl as f32 - 1.0) * 0.07).max(0.5);
            } else {
                self.bolt_cd = 0.2;
            }
        }

        // 3. Chain Lightning
        self.lightning_cd -= dt * haste;
        if self.lightning_cd <= 0.0 && self.build.lightning_lvl > 0 {
            if let Some(first_idx) = self.nearest_enemy_index(self.hero_pos, 460.0) {
                let jumps = 2 + (self.build.lightning_lvl as usize) * 2;
                let dmg = 28.0 + (self.build.lightning_lvl as f32) * 14.0;
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

        // 4. Runic Orbit Blades
        if self.build.orbit_lvl > 0 {
            self.orbit_angle += dt * (3.2 + (self.build.orbit_lvl as f32) * 0.45);
            self.orbit_hit_cd -= dt;
            if self.orbit_hit_cd <= 0.0 {
                self.orbit_hit_cd = 0.2;
                let orb_count = self.build.orbit_lvl as usize + 1;
                let orb_r = 88.0 + (self.build.orbit_lvl as f32) * 8.0;
                let orb_dmg = 18.0 + (self.build.orbit_lvl as f32) * 8.0;
                for k in 0..orb_count {
                    let ang = self.orbit_angle + (k as f32) * (std::f32::consts::TAU / orb_count as f32);
                    let orb_pos = self.hero_pos + Vec2::from_angle(ang) * orb_r;
                    self.damage_enemies_in_radius(orb_pos, 26.0, orb_dmg, 120.0);
                }
            }
        }

        // Update Enemies & Boss AI
        let hero_p = self.hero_pos;
        let mut enemy_shots: Vec<Projectile> = Vec::new();
        let mut new_telegraphs: Vec<TelegraphRing> = Vec::new();
        let mut hero_hit_dmg = 0.0f32;

        for e in &mut self.enemies {
            e.hit_flash = (e.hit_flash - dt).max(0.0);
            let to_hero = hero_p - e.pos;
            let dist = to_hero.length().max(0.001);
            let dir = to_hero / dist;
            e.facing_left = dir.x < 0.0;

            match e.kind {
                EnemyKind::Wraith => {
                    let desired = if dist > 260.0 { dir * e.speed } else if dist < 170.0 { -dir * e.speed * 0.6 } else { Vec2::ZERO };
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
                    e.vel = e.vel.lerp(dir * e.speed, (dt * 4.0).min(1.0));
                    e.attack_cd -= dt;
                    e.phase_timer += dt;
                    if e.attack_cd <= 0.0 {
                        e.attack_cd = 2.5;
                        // Telegraph fiery slam at hero's position!
                        new_telegraphs.push(TelegraphRing {
                            pos: hero_p,
                            max_radius: 125.0,
                            elapsed: 0.0,
                            duration: 1.05,
                            damage: 26.0,
                        });
                        // Plus 8-way spiral ember nova!
                        let base_a = e.phase_timer * 1.4;
                        for k in 0..8 {
                            let a = base_a + (k as f32) * (std::f32::consts::TAU / 8.0);
                            enemy_shots.push(Projectile {
                                pos: e.pos,
                                vel: Vec2::from_angle(a) * 235.0,
                                damage: 16.0,
                                radius: 10.0,
                                life: 2.8,
                                pierce: 1,
                                from_enemy: true,
                                color: Color::from_rgba(251, 146, 60, 255),
                            });
                        }
                    }
                }
                _ => {
                    e.vel = e.vel.lerp(dir * e.speed, (dt * 7.0).min(1.0));
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
                self.shake = self.shake.max(10.0);
                self.spawn_burst(
                    ring.pos,
                    22,
                    Color::from_rgba(244, 63, 94, 255),
                    VfxKind::Flame,
                    240.0,
                );
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
                    for ei in 0..self.enemies.len() {
                        if self.enemies[ei].pos.distance(p_pos) < self.enemies[ei].radius + p_rad {
                            let is_crit = self.rng.next_f32() < 0.16;
                            let dmg = if is_crit { p_dmg * 1.75 } else { p_dmg };
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
            self.shake = self.shake.max(11.0);
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
                self.report_run_once();
                return;
            }
        }

        // Collect dead enemies
        self.collect_dead_enemies();

        // Update XP Gems (magnetic pull)
        let pickup_r = self.build.pickup_radius();
        let xp_mult = 1.0 + (self.build.magnet_lvl as f32) * 0.2;
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
                play_sfx(8);
                if self.xp >= self.xp_next {
                    self.xp -= self.xp_next;
                    self.level += 1;
                    self.xp_next = ((self.xp_next as f32) * 1.34) as u32 + 12;
                    self.pending_choices = self.build.roll_three(&mut self.rng);
                    self.phase = Phase::LevelUp;
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
        self.build.apply(up);
        if up == UpgradeId::VampiricWard {
            self.hero_max_hp += 25.0;
            self.hero_hp = (self.hero_hp + 35.0).min(self.hero_max_hp);
        }
        self.hero_invuln = self.hero_invuln.max(0.6);
        self.phase = Phase::Playing;
        self.spawn_burst(self.hero_pos, 18, up.accent(), VfxKind::Star, 220.0);
        play_sfx(0);
    }

    fn report_run_once(&mut self) {
        if self.reported_run {
            return;
        }
        self.reported_run = true;
        unsafe {
            emberfall_on_run_complete(
                self.score,
                self.kills,
                if self.boss_slain { 1 } else { 0 },
                self.level,
            );
        }
    }

    fn draw_world(&self, assets: &GameAssets) {
        let sw = screen_width();
        let sh = screen_height();
        let shake_off = if self.shake > 0.1 {
            vec2(
                ((self.elapsed * 97.0).sin()) * self.shake,
                ((self.elapsed * 131.0).cos()) * self.shake,
            )
        } else {
            Vec2::ZERO
        };

        let cam = Camera2D {
            target: self.cam_pos + shake_off,
            zoom: vec2(2.0 / sw, -2.0 / sh),
            ..Default::default()
        };
        set_camera(&cam);

        // 1. Seamless Basalt Dungeon Floor Tiling (real texture!)
        let tile = 256.0;
        let min_x = ((self.cam_pos.x - sw * 0.65) / tile).floor() as i32;
        let max_x = ((self.cam_pos.x + sw * 0.65) / tile).ceil() as i32;
        let min_y = ((self.cam_pos.y - sh * 0.65) / tile).floor() as i32;
        let max_y = ((self.cam_pos.y + sh * 0.65) / tile).ceil() as i32;

        for tx in min_x..=max_x {
            for ty in min_y..=max_y {
                draw_texture_ex(
                    &assets.floor,
                    tx as f32 * tile,
                    ty as f32 * tile,
                    Color::from_rgba(175, 185, 210, 255),
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
            Color::from_rgba(249, 115, 22, 190),
        );

        // 2. Telegraphed Boss Rings
        for ring in &self.telegraphs {
            let p = (ring.elapsed / ring.duration).clamp(0.0, 1.0);
            draw_circle(
                ring.pos.x,
                ring.pos.y,
                ring.max_radius * p,
                Color::from_rgba(244, 63, 94, 55),
            );
            draw_circle_lines(
                ring.pos.x,
                ring.pos.y,
                ring.max_radius,
                3.0,
                Color::from_rgba(244, 63, 94, 220),
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

        // 4. Hero Aura & Dash Afterimages
        let aura_sz = 110.0 + (self.elapsed * 4.0).sin() * 6.0;
        draw_texture_ex(
            &assets.vfx_light,
            self.hero_pos.x - aura_sz * 0.5,
            self.hero_pos.y - aura_sz * 0.5,
            Color::from_rgba(56, 189, 248, 65),
            DrawTextureParams {
                dest_size: Some(vec2(aura_sz, aura_sz)),
                ..Default::default()
            },
        );

        for a in &self.afterimages {
            let alpha = (a.life / 0.22).clamp(0.0, 1.0) * 0.55;
            draw_texture_ex(
                &assets.hero,
                a.pos.x - 42.0,
                a.pos.y - 42.0,
                Color::new(0.22, 0.74, 0.97, alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(84.0, 84.0)),
                    flip_x: a.facing_left,
                    ..Default::default()
                },
            );
        }

        // 5. Enemies (Real Illustrated Sprites + ground shadow + hit flash!)
        for e in &self.enemies {
            let (tex, draw_sz) = match e.kind {
                EnemyKind::Imp => (&assets.imp, 64.0),
                EnemyKind::Golem => (&assets.golem, 94.0),
                EnemyKind::Wraith => (&assets.wraith, 76.0),
                EnemyKind::Boss => (&assets.boss, 186.0),
            };
            let bob = ((self.elapsed * 8.0 + e.pos.x * 0.05).sin()) * 3.5;

            // Ground shadow
            draw_ellipse(
                e.pos.x,
                e.pos.y + draw_sz * 0.34,
                draw_sz * 0.32,
                draw_sz * 0.12,
                0.0,
                Color::from_rgba(0, 0, 0, 115),
            );

            let tint = if e.hit_flash > 0.0 {
                Color::from_rgba(255, 255, 255, 255)
            } else {
                Color::from_rgba(242, 242, 248, 255)
            };
            draw_texture_ex(
                tex,
                e.pos.x - draw_sz * 0.5,
                e.pos.y - draw_sz * 0.5 + bob,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(draw_sz, draw_sz)),
                    flip_x: e.facing_left,
                    ..Default::default()
                },
            );
            if e.hit_flash > 0.0 {
                draw_texture_ex(
                    &assets.vfx_light,
                    e.pos.x - draw_sz * 0.55,
                    e.pos.y - draw_sz * 0.55,
                    Color::from_rgba(255, 255, 255, 165),
                    DrawTextureParams {
                        dest_size: Some(vec2(draw_sz * 1.1, draw_sz * 1.1)),
                        ..Default::default()
                    },
                );
            }

            // Mini HP bar for damaged elites (not regular imps)
            if e.kind != EnemyKind::Imp && e.kind != EnemyKind::Boss && e.hp < e.max_hp {
                let bw = draw_sz * 0.65;
                let pct = (e.hp / e.max_hp).clamp(0.0, 1.0);
                draw_rectangle(e.pos.x - bw * 0.5, e.pos.y - draw_sz * 0.56, bw, 5.0, Color::from_rgba(0, 0, 0, 180));
                draw_rectangle(e.pos.x - bw * 0.5, e.pos.y - draw_sz * 0.56, bw * pct, 5.0, Color::from_rgba(244, 63, 94, 240));
            }
        }

        // 6. Hero Sprite (Arcane Knight!)
        let hero_bob = self.hero_stride.sin().abs() * -5.0;
        draw_ellipse(
            self.hero_pos.x,
            self.hero_pos.y + 30.0,
            28.0,
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
            &assets.hero,
            self.hero_pos.x - 46.0,
            self.hero_pos.y - 46.0 + hero_bob,
            hero_tint,
            DrawTextureParams {
                dest_size: Some(vec2(92.0, 92.0)),
                flip_x: self.hero_facing_left,
                ..Default::default()
            },
        );

        // 7. Runic Orbit Blades (Kenney magic_05)
        if self.build.orbit_lvl > 0 {
            let orb_count = self.build.orbit_lvl as usize + 1;
            let orb_r = 88.0 + (self.build.orbit_lvl as f32) * 8.0;
            for k in 0..orb_count {
                let ang = self.orbit_angle + (k as f32) * (std::f32::consts::TAU / orb_count as f32);
                let p = self.hero_pos + Vec2::from_angle(ang) * orb_r;
                draw_texture_ex(
                    &assets.vfx_magic,
                    p.x - 26.0,
                    p.y - 26.0,
                    Color::from_rgba(232, 121, 249, 240),
                    DrawTextureParams {
                        dest_size: Some(vec2(52.0, 52.0)),
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
                Color::new(0.25, 0.85, 1.0, a),
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
                draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 6.0, Color::new(0.22, 0.74, 0.97, a * 0.55));
                draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 2.4, Color::new(0.92, 0.98, 1.0, a));
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

        // 12. Floating Damage Numbers
        for ft in &self.float_texts {
            let a = (ft.life / 0.62).clamp(0.0, 1.0);
            draw_text(
                &ft.text,
                ft.pos.x - 10.0,
                ft.pos.y,
                ft.scale,
                Color::new(ft.color.r, ft.color.g, ft.color.b, a),
            );
        }

        set_default_camera();
    }

    fn draw_button(&self, rect: Rect, label: &str, fill: Color, border: Color) -> bool {
        let mp = vec2(mouse_position().0, mouse_position().1);
        let hovered = rect.contains(mp);
        let bg = if hovered {
            Color::new((fill.r + 0.08).min(1.0), (fill.g + 0.08).min(1.0), (fill.b + 0.08).min(1.0), fill.a)
        } else {
            fill
        };
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.2, border);
        let dims = measure_text(label, None, 22, 1.0);
        draw_text(
            label,
            rect.x + (rect.w - dims.width) * 0.5,
            rect.y + (rect.h + dims.height) * 0.5 - 2.0,
            22.0,
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

    fn draw_ui(&mut self, assets: &GameAssets) {
        let sw = screen_width();
        let sh = screen_height();

        if self.phase == Phase::Title {
            self.title_time += get_frame_time();
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(6, 9, 20, 195));

            // Showcase real hero & boss sprites on title!
            let bob = (self.title_time * 3.0).sin() * 8.0;
            let cx = sw * 0.5;
            let cy = sh * 0.5;
            draw_texture_ex(
                &assets.vfx_light,
                cx - 170.0,
                cy - 210.0,
                Color::from_rgba(249, 115, 22, 90),
                DrawTextureParams { dest_size: Some(vec2(340.0, 340.0)), ..Default::default() },
            );
            draw_texture_ex(
                &assets.hero,
                cx - 135.0,
                cy - 150.0 + bob,
                WHITE,
                DrawTextureParams { dest_size: Some(vec2(128.0, 128.0)), ..Default::default() },
            );
            draw_texture_ex(
                &assets.boss,
                cx + 5.0,
                cy - 175.0 - bob * 0.7,
                WHITE,
                DrawTextureParams { dest_size: Some(vec2(158.0, 158.0)), flip_x: true, ..Default::default() },
            );

            let title = "EMBERFALL";
            let td = measure_text(title, None, 52, 1.0);
            draw_text(title, cx - td.width * 0.5, cy + 12.0, 52.0, Color::from_rgba(251, 191, 36, 255));

            let sub = format!("DAILY HORDE #{}  ·  {}", self.day_seed, self.modifier.title());
            let sd = measure_text(&sub, None, 19, 1.0);
            draw_text(&sub, cx - sd.width * 0.5, cy + 40.0, 19.0, Color::from_rgba(56, 189, 248, 255));

            let desc = self.modifier.subtitle();
            let dd = measure_text(desc, None, 17, 1.0);
            draw_text(desc, cx - dd.width * 0.5, cy + 66.0, 17.0, Color::from_rgba(203, 213, 225, 255));

            let btn = Rect::new(cx - 125.0, cy + 92.0, 250.0, 54.0);
            if self.draw_button(
                btn,
                "PLAY DAILY (90s)",
                Color::from_rgba(234, 88, 12, 235),
                Color::from_rgba(253, 224, 71, 255),
            ) || is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
            {
                self.start_run();
            }

            let tip = "1-Thumb Drag to Move  ·  Auto-Attack  ·  TAP DASH or SPACE";
            let tip_d = measure_text(tip, None, 15, 1.0);
            draw_text(tip, cx - tip_d.width * 0.5, cy + 174.0, 15.0, Color::from_rgba(148, 163, 184, 255));
            return;
        }

        // LIVE IN-GAME HUD
        // Top XP Bar
        let xp_pct = (self.xp as f32 / self.xp_next.max(1) as f32).clamp(0.0, 1.0);
        draw_rectangle(0.0, 0.0, sw, 8.0, Color::from_rgba(15, 23, 42, 220));
        draw_rectangle(0.0, 0.0, sw * xp_pct, 8.0, Color::from_rgba(56, 189, 248, 255));

        // Top-Left: HP Bar & Level
        draw_rectangle(14.0, 18.0, 176.0, 44.0, Color::from_rgba(8, 13, 28, 200));
        draw_rectangle_lines(14.0, 18.0, 176.0, 44.0, 1.5, Color::from_rgba(148, 163, 184, 90));
        draw_text(&format!("LVL {}", self.level), 24.0, 38.0, 19.0, Color::from_rgba(250, 204, 21, 255));
        let hp_pct = (self.hero_hp / self.hero_max_hp).clamp(0.0, 1.0);
        draw_rectangle(24.0, 44.0, 154.0, 10.0, Color::from_rgba(30, 41, 59, 240));
        draw_rectangle(24.0, 44.0, 154.0 * hp_pct, 10.0, Color::from_rgba(244, 63, 94, 255));

        // Top-Center: Countdown Timer (90s -> 0s)
        let rem = (RUN_DURATION - self.elapsed).max(0.0);
        let timer_str = format!("{:02}:{:02}", (rem as u32) / 60, (rem as u32) % 60);
        let td = measure_text(&timer_str, None, 32, 1.0);
        draw_text(&timer_str, sw * 0.5 - td.width * 0.5, 44.0, 32.0, WHITE);

        // Top-Right: Score & Kills
        let score_str = format!("{} PTS", self.score);
        let sd = measure_text(&score_str, None, 22, 1.0);
        draw_text(&score_str, sw - sd.width - 16.0, 38.0, 22.0, Color::from_rgba(251, 191, 36, 255));
        let kill_str = if self.combo > 3 {
            format!("{} KILLS · x{}", self.kills, self.combo)
        } else {
            format!("{} KILLS", self.kills)
        };
        let kd = measure_text(&kill_str, None, 16, 1.0);
        draw_text(&kill_str, sw - kd.width - 16.0, 58.0, 16.0, Color::from_rgba(148, 163, 184, 255));

        // Boss HP Bar
        if let Some(boss) = self.enemies.iter().find(|e| e.kind == EnemyKind::Boss) {
            let bw = (sw - 48.0).min(420.0);
            let bx = (sw - bw) * 0.5;
            let by = 72.0;
            let pct = (boss.hp / boss.max_hp).clamp(0.0, 1.0);
            draw_rectangle(bx, by, bw, 16.0, Color::from_rgba(15, 23, 42, 220));
            draw_rectangle(bx, by, bw * pct, 16.0, Color::from_rgba(239, 68, 68, 255));
            draw_rectangle_lines(bx, by, bw, 16.0, 2.0, Color::from_rgba(251, 191, 36, 230));
            let bl = "EMBER LICH OVERLORD";
            let bld = measure_text(bl, None, 14, 1.0);
            draw_text(bl, sw * 0.5 - bld.width * 0.5, by + 13.0, 14.0, WHITE);
        }

        if self.boss_banner_timer > 0.0 {
            let msg = "DAILY BOSS AWAKENS!";
            let md = measure_text(msg, None, 36, 1.0);
            draw_text(msg, sw * 0.5 - md.width * 0.5, sh * 0.34, 36.0, Color::from_rgba(251, 146, 60, 255));
        }

        // Virtual Joystick Indicator
        if self.joy_active && self.phase == Phase::Playing {
            draw_circle_lines(self.joy_origin.x, self.joy_origin.y, 54.0, 2.0, Color::from_rgba(255, 255, 255, 70));
            draw_circle(self.joy_curr.x, self.joy_curr.y, 20.0, Color::from_rgba(56, 189, 248, 140));
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
            let ld = measure_text(&lbl, None, 20, 1.0);
            draw_text(&lbl, dc.x - ld.width * 0.5, dc.y + 7.0, 20.0, WHITE);
        }

        // LEVEL-UP 3-CARD PICKER
        if self.phase == Phase::LevelUp {
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(5, 8, 18, 205));
            let hdr = format!("LEVEL {} — CHOOSE POWER", self.level);
            let hd = measure_text(&hdr, None, 30, 1.0);
            draw_text(&hdr, sw * 0.5 - hd.width * 0.5, sh * 0.16, 30.0, Color::from_rgba(250, 204, 21, 255));

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
                draw_text(
                    &format!("[{}] {}  LV.{}", i + 1, up.name(), next_lvl),
                    rect.x + 16.0,
                    rect.y + 34.0,
                    21.0,
                    up.accent(),
                );
                draw_text(up.desc(), rect.x + 16.0, rect.y + 66.0, 16.0, Color::from_rgba(226, 232, 240, 255));

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
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(5, 8, 18, 215));
            let cx = sw * 0.5;
            let cy = sh * 0.5;
            let rank = compute_rank(self.score, self.boss_slain);

            let heading = if self.phase == Phase::Victory {
                "DAILY DUNGEON CLEARED!"
            } else {
                "FALLEN IN BATTLE"
            };
            let hd = measure_text(heading, None, 32, 1.0);
            draw_text(
                heading,
                cx - hd.width * 0.5,
                cy - 140.0,
                32.0,
                if self.phase == Phase::Victory {
                    Color::from_rgba(250, 204, 21, 255)
                } else {
                    Color::from_rgba(244, 63, 94, 255)
                },
            );

            let rank_str = format!("RANK {}", rank);
            let rd = measure_text(&rank_str, None, 54, 1.0);
            draw_text(&rank_str, cx - rd.width * 0.5, cy - 72.0, 54.0, Color::from_rgba(56, 189, 248, 255));

            let stats = format!(
                "SCORE: {}   ·   KILLS: {}   ·   BOSS: {}",
                self.score,
                self.kills,
                if self.boss_slain { "SLAIN" } else { "SURVIVED" }
            );
            let std = measure_text(&stats, None, 19, 1.0);
            draw_text(&stats, cx - std.width * 0.5, cy - 26.0, 19.0, WHITE);

            let share_rect = Rect::new(cx - 140.0, cy + 12.0, 280.0, 48.0);
            let share_lbl = if self.shared_comment {
                "SHARED TO REDDIT ✓"
            } else {
                "POST SCORE TO REDDIT"
            };
            if self.draw_button(
                share_rect,
                share_lbl,
                Color::from_rgba(37, 99, 235, 235),
                Color::from_rgba(125, 211, 252, 255),
            ) && !self.shared_comment
            {
                self.shared_comment = true;
                unsafe {
                    emberfall_share_comment(
                        self.score,
                        self.kills,
                        if self.boss_slain { 1 } else { 0 },
                    );
                }
            }

            let retry_rect = Rect::new(cx - 140.0, cy + 74.0, 280.0, 48.0);
            if self.draw_button(
                retry_rect,
                "PLAY AGAIN",
                Color::from_rgba(234, 88, 12, 235),
                Color::from_rgba(253, 224, 71, 255),
            ) || is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::R)
            {
                self.start_run();
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
    let mut game = Game::new();

    loop {
        let dt = get_frame_time().min(0.05);
        clear_background(Color::from_rgba(6, 9, 20, 255));

        if game.phase == Phase::Playing {
            game.update_playing(dt);
        }

        game.draw_world(&assets);
        game.draw_ui(&assets);

        next_frame().await;
    }
}

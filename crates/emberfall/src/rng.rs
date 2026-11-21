/// Deterministic FNV-1a + xorshift64* PRNG for shared UTC Daily Runs.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn from_seed_str(seed: &str) -> Self {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in seed.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        if h == 0 {
            h = 0x9e3779b97f4a7c15;
        }
        Self { state: h }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545f4914f6cdd1d)
    }

    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1u32 << 24) as f32)
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    pub fn range_usize(&mut self, min: usize, max_exclusive: usize) -> usize {
        if max_exclusive <= min {
            return min;
        }
        min + (self.next_u64() as usize) % (max_exclusive - min)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DailyModifier {
    BloodMoon,
    ArcaneSurge,
    SwarmRush,
    TitansWrath,
}

impl DailyModifier {
    pub fn from_index(idx: usize) -> Self {
        match idx % 4 {
            0 => Self::BloodMoon,
            1 => Self::ArcaneSurge,
            2 => Self::SwarmRush,
            _ => Self::TitansWrath,
        }
    }

    #[allow(dead_code)]
    pub fn to_index(self) -> usize {
        match self {
            Self::BloodMoon => 0,
            Self::ArcaneSurge => 1,
            Self::SwarmRush => 2,
            Self::TitansWrath => 3,
        }
    }

    pub fn from_seed(seed: &str) -> Self {
        let mut r = Rng::from_seed_str(seed);
        Self::from_index(r.range_usize(0, 4))
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::BloodMoon => "BLOOD MOON",
            Self::ArcaneSurge => "ARCANE SURGE",
            Self::SwarmRush => "SWARM RUSH",
            Self::TitansWrath => "TITAN'S WRATH",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Self::BloodMoon => "+25% Crit Chance · Faster Hordes",
            Self::ArcaneSurge => "+20% Attack Speed · Double XP Gems",
            Self::SwarmRush => "Massive Swarms · +30% Score Multiplier",
            Self::TitansWrath => "Enraged Daily Boss · +50% Boss Bounty",
        }
    }
}

/// Maps Kill-Streak Fever meter [0.0..=1.0] to [1.0x..=2.5x] score/XP multiplier.
pub fn fever_multiplier(fever: f32) -> f32 {
    1.0 + fever.clamp(0.0, 1.0) * 1.5
}

/// Generates 1-2 deterministic Blood Shrine coordinates for a daily seed.
pub fn generate_shrine_positions(day_seed: &str) -> Vec<(f32, f32)> {
    let mut r = Rng::from_seed_str(&format!("{}-shrines", day_seed));
    let count = 1 + r.range_usize(0, 2); // 1 or 2 shrines
    let base_angle = r.range_f32(0.0, std::f32::consts::TAU);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let angle = base_angle + (i as f32) * std::f32::consts::PI;
        let dist = r.range_f32(280.0, 620.0);
        let x = angle.cos() * dist;
        let y = angle.sin() * dist;
        out.push((x, y));
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DailyBiome {
    ObsidianSanctuary,
    VolcanicRift,
    FrostAbyss,
}

impl DailyBiome {
    pub fn from_index(idx: usize) -> Self {
        match idx % 3 {
            0 => Self::ObsidianSanctuary,
            1 => Self::VolcanicRift,
            _ => Self::FrostAbyss,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Self::ObsidianSanctuary => 0,
            Self::VolcanicRift => 1,
            Self::FrostAbyss => 2,
        }
    }

    pub fn from_seed(seed: &str) -> Self {
        let mut r = Rng::from_seed_str(&format!("{}-biome", seed));
        Self::from_index(r.range_usize(0, 3))
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::ObsidianSanctuary => "OBSIDIAN SANCTUARY",
            Self::VolcanicRift => "VOLCANIC RIFT",
            Self::FrostAbyss => "FROST ABYSS",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Self::ObsidianSanctuary => "Arcane Barrels (+25% Shockwave Damage)",
            Self::VolcanicRift => "Magma Barrels (+30% Explosion Radius)",
            Self::FrostAbyss => "Cryo Barrels (Chills & Slows Hordes 40%)",
        }
    }
}

/// Generates 8 deterministic Runic Barrel coordinates in a ring around the arena for a daily seed.
pub fn generate_barrel_positions(day_seed: &str) -> Vec<(f32, f32)> {
    let mut r = Rng::from_seed_str(&format!("{}-barrels", day_seed));
    let count = 8usize;
    let base_angle = r.range_f32(0.0, std::f32::consts::TAU);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let sector = (i as f32) * (std::f32::consts::TAU / (count as f32));
        let jitter = r.range_f32(-0.22, 0.22);
        let angle = base_angle + sector + jitter;
        let dist = r.range_f32(210.0, 740.0);
        out.push((angle.cos() * dist, angle.sin() * dist));
    }
    out
}

pub fn compute_rank(score: u32, boss_slain: bool) -> &'static str {
    if boss_slain && score >= 14_000 {
        "SSS"
    } else if boss_slain && score >= 9_500 {
        "SS"
    } else if score >= 7_000 {
        "S"
    } else if score >= 4_500 {
        "A"
    } else if score >= 2_500 {
        "B"
    } else if score >= 1_200 {
        "C"
    } else if score >= 500 {
        "D"
    } else {
        "E"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_365_days_and_ranks() {
        let mut seen_mods = [false; 4];
        let mut seen_biomes = [false; 3];
        for day in 1..=365 {
            let seed = format!("2026-D{:03}", day);
            let mut r1 = Rng::from_seed_str(&seed);
            let mut r2 = Rng::from_seed_str(&seed);
            assert_eq!(r1.next_u64(), r2.next_u64());
            let v = r1.range_f32(0.0, 1.0);
            assert!((0.0..=1.0).contains(&v));

            let m = DailyModifier::from_seed(&seed);
            seen_mods[m.to_index()] = true;

            let b = DailyBiome::from_seed(&seed);
            seen_biomes[b.to_index()] = true;
        }
        assert!(seen_mods.iter().all(|&x| x), "All 4 daily modifiers must appear");
        assert!(seen_biomes.iter().all(|&x| x), "All 3 daily biomes must appear");
        assert_eq!(compute_rank(15_000, true), "SSS");
        assert_eq!(compute_rank(10_000, true), "SS");
        assert_eq!(compute_rank(15_000, false), "S");
        assert_eq!(compute_rank(7_500, false), "S");
        assert_eq!(compute_rank(4_500, false), "A");
        assert_eq!(compute_rank(2_500, false), "B");
        assert_eq!(compute_rank(1_200, false), "C");
        assert_eq!(compute_rank(500, false), "D");
        assert_eq!(compute_rank(200, false), "E");
    }

    #[test]
    fn mutator_and_biome_index_roundtrip() {
        for idx in 0..4 {
            let m = DailyModifier::from_index(idx);
            assert_eq!(m.to_index(), idx);
        }
        assert_eq!(DailyModifier::from_index(4), DailyModifier::BloodMoon);
        for idx in 0..3 {
            let b = DailyBiome::from_index(idx);
            assert_eq!(b.to_index(), idx);
        }
    }

    #[test]
    fn fever_multiplier_bounds() {
        assert!((fever_multiplier(-0.5) - 1.0).abs() < 1e-5);
        assert!((fever_multiplier(0.0) - 1.0).abs() < 1e-5);
        assert!((fever_multiplier(0.5) - 1.75).abs() < 1e-5);
        assert!((fever_multiplier(1.0) - 2.5).abs() < 1e-5);
        assert!((fever_multiplier(2.0) - 2.5).abs() < 1e-5);
    }

    #[test]
    fn deterministic_shrines_and_barrels_per_seed() {
        let s1 = generate_shrine_positions("2026-11-21");
        let s2 = generate_shrine_positions("2026-11-21");
        assert_eq!(s1, s2);
        assert!((1..=2).contains(&s1.len()));
        for (x, y) in &s1 {
            let dist = (x * x + y * y).sqrt();
            assert!((279.0..=621.0).contains(&dist));
        }

        let b1 = generate_barrel_positions("2026-11-21");
        let b2 = generate_barrel_positions("2026-11-21");
        assert_eq!(b1, b2);
        assert_eq!(b1.len(), 8);
        for (x, y) in &b1 {
            let dist = (x * x + y * y).sqrt();
            assert!((209.0..=741.0).contains(&dist));
        }

        let mut saw_one = false;
        let mut saw_two = false;
        for d in 1..=30 {
            let pts = generate_shrine_positions(&format!("2026-11-{:02}", d));
            if pts.len() == 1 {
                saw_one = true;
            } else if pts.len() == 2 {
                saw_two = true;
                let dx = pts[0].0 - pts[1].0;
                let dy = pts[0].1 - pts[1].1;
                let sep = (dx * dx + dy * dy).sqrt();
                assert!(sep >= 500.0, "2 shrines must be >= 500.0 px apart, got {}", sep);
            }
        }
        assert!(saw_one && saw_two, "Both 1 and 2 shrine counts should occur");
    }
}


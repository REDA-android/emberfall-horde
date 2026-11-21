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
    pub fn from_seed(seed: &str) -> Self {
        let mut r = Rng::from_seed_str(seed);
        match r.range_usize(0, 4) {
            0 => Self::BloodMoon,
            1 => Self::ArcaneSurge,
            2 => Self::SwarmRush,
            _ => Self::TitansWrath,
        }
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
        for day in 1..=365 {
            let seed = format!("2026-D{:03}", day);
            let mut r1 = Rng::from_seed_str(&seed);
            let mut r2 = Rng::from_seed_str(&seed);
            assert_eq!(r1.next_u64(), r2.next_u64());
            let v = r1.range_f32(0.0, 1.0);
            assert!((0.0..=1.0).contains(&v));

            let m = DailyModifier::from_seed(&seed);
            match m {
                DailyModifier::BloodMoon => seen_mods[0] = true,
                DailyModifier::ArcaneSurge => seen_mods[1] = true,
                DailyModifier::SwarmRush => seen_mods[2] = true,
                DailyModifier::TitansWrath => seen_mods[3] = true,
            }
        }
        assert!(seen_mods.iter().all(|&x| x), "All 4 daily modifiers must appear");
        assert_eq!(compute_rank(15_000, true), "SSS");
        assert_eq!(compute_rank(10_000, true), "SS");
        assert_eq!(compute_rank(7_500, false), "S");
        assert_eq!(compute_rank(200, false), "E");
    }
}

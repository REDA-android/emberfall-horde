use crate::rng::Rng;
use macroquad::prelude::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpgradeId {
    ArcaneSlash,
    EmberBolts,
    ChainLightning,
    RunicOrbit,
    NovaDash,
    OverclockHaste,
    SoulMagnet,
    VampiricWard,
}

pub const ALL_UPGRADES: [UpgradeId; 8] = [
    UpgradeId::ArcaneSlash,
    UpgradeId::EmberBolts,
    UpgradeId::ChainLightning,
    UpgradeId::RunicOrbit,
    UpgradeId::NovaDash,
    UpgradeId::OverclockHaste,
    UpgradeId::SoulMagnet,
    UpgradeId::VampiricWard,
];

impl UpgradeId {
    pub fn name(self) -> &'static str {
        match self {
            Self::ArcaneSlash => "ARCANE SLASH",
            Self::EmberBolts => "EMBER BOLTS",
            Self::ChainLightning => "CHAIN LIGHTNING",
            Self::RunicOrbit => "RUNIC ORBIT",
            Self::NovaDash => "INFERNO DASH",
            Self::OverclockHaste => "ARCANE HASTE",
            Self::SoulMagnet => "SOUL MAGNET",
            Self::VampiricWard => "BLOOD WARD",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Self::ArcaneSlash => "Wider, deadlier runic blade sweeps",
            Self::EmberBolts => "+1 homing fireball volley & pierce",
            Self::ChainLightning => "Arcs jump to +2 enemies with crits",
            Self::RunicOrbit => "+1 spinning blade orb around hero",
            Self::NovaDash => "Faster dash + huge fire shockwave",
            Self::OverclockHaste => "+18% attack speed & +10% move speed",
            Self::SoulMagnet => "+45% XP pickup range & +20% XP gain",
            Self::VampiricWard => "Heal +30 HP & +25 Max HP shield",
        }
    }

    pub fn accent(self) -> Color {
        match self {
            Self::ArcaneSlash => Color::from_rgba(56, 189, 248, 255),
            Self::EmberBolts => Color::from_rgba(251, 146, 60, 255),
            Self::ChainLightning => Color::from_rgba(125, 211, 252, 255),
            Self::RunicOrbit => Color::from_rgba(232, 121, 249, 255),
            Self::NovaDash => Color::from_rgba(248, 113, 113, 255),
            Self::OverclockHaste => Color::from_rgba(250, 204, 21, 255),
            Self::SoulMagnet => Color::from_rgba(52, 211, 153, 255),
            Self::VampiricWard => Color::from_rgba(244, 63, 94, 255),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvolutionId {
    MeteorBarrage,
    CrimsonReaper,
    InfernoVortex,
    CelestialTempest,
    SoulHarvester,
}

impl EvolutionId {
    pub fn title(self) -> &'static str {
        match self {
            Self::MeteorBarrage => "METEOR BARRAGE",
            Self::CrimsonReaper => "CRIMSON REAPER",
            Self::InfernoVortex => "INFERNO VORTEX",
            Self::CelestialTempest => "CELESTIAL TEMPEST",
            Self::SoulHarvester => "SOUL HARVESTER",
        }
    }

    pub fn emoji(self) -> &'static str {
        match self {
            Self::MeteorBarrage => "☄️",
            Self::CrimsonReaper => "🩸",
            Self::InfernoVortex => "🌀",
            Self::CelestialTempest => "🌩️",
            Self::SoulHarvester => "💀",
        }
    }

    pub fn accent(self) -> Color {
        match self {
            Self::MeteorBarrage => Color::from_rgba(251, 146, 60, 255),
            Self::CrimsonReaper => Color::from_rgba(244, 63, 94, 255),
            Self::InfernoVortex => Color::from_rgba(232, 121, 249, 255),
            Self::CelestialTempest => Color::from_rgba(56, 189, 248, 255),
            Self::SoulHarvester => Color::from_rgba(192, 132, 252, 255),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelicId {
    VampiricFang,
    CryoCore,
    CritPrism,
    MagnetOrb,
}

impl RelicId {
    pub fn from_index(idx: usize) -> Self {
        match idx % 4 {
            0 => Self::VampiricFang,
            1 => Self::CryoCore,
            2 => Self::CritPrism,
            _ => Self::MagnetOrb,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::VampiricFang => "RELIC: VAMPIRIC FANG (+HP)",
            Self::CryoCore => "RELIC: CRYO CORE (+25% CHILL DMG)",
            Self::CritPrism => "RELIC: CRIT PRISM (+15% CRIT)",
            Self::MagnetOrb => "RELIC: MAGNET ORB (GLOBAL PULL)",
        }
    }

    pub fn accent(self) -> Color {
        match self {
            Self::VampiricFang => Color::from_rgba(244, 63, 94, 255),
            Self::CryoCore => Color::from_rgba(56, 189, 248, 255),
            Self::CritPrism => Color::from_rgba(250, 204, 21, 255),
            Self::MagnetOrb => Color::from_rgba(192, 132, 252, 255),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeroClass {
    SunKnight,
    ArcanePyromancer,
    ShadowAssassin,
}

impl HeroClass {
    #[allow(dead_code)]
    pub fn from_index(idx: usize) -> Self {
        match idx % 3 {
            0 => Self::SunKnight,
            1 => Self::ArcanePyromancer,
            _ => Self::ShadowAssassin,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Self::SunKnight => 0,
            Self::ArcanePyromancer => 1,
            Self::ShadowAssassin => 2,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::SunKnight => "SUN KNIGHT",
            Self::ArcanePyromancer => "PYROMANCER",
            Self::ShadowAssassin => "SHADOW ASSASSIN",
        }
    }

    pub fn faction_name(self) -> &'static str {
        match self {
            Self::SunKnight => "SOLAR ORDER",
            Self::ArcanePyromancer => "ARCANE CONCLAVE",
            Self::ShadowAssassin => "SHADOW GUILD",
        }
    }

    pub fn loadout_desc(self) -> &'static str {
        match self {
            Self::SunKnight => "130 HP · Slash + Fireball · Balanced",
            Self::ArcanePyromancer => "95 HP · Fireball + Chain Lightning + Magnet",
            Self::ShadowAssassin => "105 HP · Slash + Orbitals + Fast Dash",
        }
    }

    pub fn base_max_hp(self) -> f32 {
        match self {
            Self::SunKnight => 130.0,
            Self::ArcanePyromancer => 95.0,
            Self::ShadowAssassin => 105.0,
        }
    }

    pub fn accent(self) -> Color {
        match self {
            Self::SunKnight => Color::from_rgba(250, 204, 21, 255),
            Self::ArcanePyromancer => Color::from_rgba(56, 189, 248, 255),
            Self::ShadowAssassin => Color::from_rgba(167, 139, 250, 255),
        }
    }
}

#[derive(Clone, Debug)]
pub struct HeroBuild {
    pub slash_lvl: u8,
    pub bolt_lvl: u8,
    pub lightning_lvl: u8,
    pub orbit_lvl: u8,
    pub dash_lvl: u8,
    pub haste_lvl: u8,
    pub magnet_lvl: u8,
    pub ward_lvl: u8,
}

impl Default for HeroBuild {
    fn default() -> Self {
        Self::for_class(HeroClass::SunKnight)
    }
}

impl HeroBuild {
    pub fn for_class(class: HeroClass) -> Self {
        match class {
            HeroClass::SunKnight => Self {
                slash_lvl: 1,
                bolt_lvl: 1,
                lightning_lvl: 0,
                orbit_lvl: 0,
                dash_lvl: 1,
                haste_lvl: 0,
                magnet_lvl: 0,
                ward_lvl: 0,
            },
            HeroClass::ArcanePyromancer => Self {
                slash_lvl: 0,
                bolt_lvl: 1,
                lightning_lvl: 1,
                orbit_lvl: 0,
                dash_lvl: 1,
                haste_lvl: 0,
                magnet_lvl: 1,
                ward_lvl: 0,
            },
            HeroClass::ShadowAssassin => Self {
                slash_lvl: 1,
                bolt_lvl: 0,
                lightning_lvl: 0,
                orbit_lvl: 1,
                dash_lvl: 2,
                haste_lvl: 1,
                magnet_lvl: 0,
                ward_lvl: 0,
            },
        }
    }

    pub fn level_of(&self, id: UpgradeId) -> u8 {
        match id {
            UpgradeId::ArcaneSlash => self.slash_lvl,
            UpgradeId::EmberBolts => self.bolt_lvl,
            UpgradeId::ChainLightning => self.lightning_lvl,
            UpgradeId::RunicOrbit => self.orbit_lvl,
            UpgradeId::NovaDash => self.dash_lvl,
            UpgradeId::OverclockHaste => self.haste_lvl,
            UpgradeId::SoulMagnet => self.magnet_lvl,
            UpgradeId::VampiricWard => self.ward_lvl,
        }
    }

    pub fn apply(&mut self, id: UpgradeId) {
        match id {
            UpgradeId::ArcaneSlash => self.slash_lvl = (self.slash_lvl + 1).min(5),
            UpgradeId::EmberBolts => self.bolt_lvl = (self.bolt_lvl + 1).min(5),
            UpgradeId::ChainLightning => self.lightning_lvl = (self.lightning_lvl + 1).min(5),
            UpgradeId::RunicOrbit => self.orbit_lvl = (self.orbit_lvl + 1).min(5),
            UpgradeId::NovaDash => self.dash_lvl = (self.dash_lvl + 1).min(5),
            UpgradeId::OverclockHaste => self.haste_lvl = (self.haste_lvl + 1).min(5),
            UpgradeId::SoulMagnet => self.magnet_lvl = (self.magnet_lvl + 1).min(5),
            UpgradeId::VampiricWard => self.ward_lvl = (self.ward_lvl + 1).min(5),
        }
    }

    pub fn has_meteor_barrage(&self) -> bool {
        self.bolt_lvl >= 3 && self.haste_lvl >= 3
    }

    pub fn has_crimson_reaper(&self) -> bool {
        self.slash_lvl >= 3 && self.ward_lvl >= 3
    }

    pub fn has_inferno_vortex(&self) -> bool {
        self.orbit_lvl >= 3 && self.dash_lvl >= 3
    }

    pub fn has_celestial_tempest(&self) -> bool {
        self.lightning_lvl >= 3 && self.magnet_lvl >= 3
    }

    pub fn has_soul_harvester(&self) -> bool {
        self.ward_lvl >= 3 && self.magnet_lvl >= 3
    }

    pub fn evolutions(&self) -> Vec<EvolutionId> {
        let mut out = Vec::new();
        if self.has_meteor_barrage() {
            out.push(EvolutionId::MeteorBarrage);
        }
        if self.has_crimson_reaper() {
            out.push(EvolutionId::CrimsonReaper);
        }
        if self.has_inferno_vortex() {
            out.push(EvolutionId::InfernoVortex);
        }
        if self.has_celestial_tempest() {
            out.push(EvolutionId::CelestialTempest);
        }
        if self.has_soul_harvester() {
            out.push(EvolutionId::SoulHarvester);
        }
        out
    }

    pub fn newly_unlocked_evolution(&self, prev: &HeroBuild) -> Option<EvolutionId> {
        if self.has_meteor_barrage() && !prev.has_meteor_barrage() {
            Some(EvolutionId::MeteorBarrage)
        } else if self.has_crimson_reaper() && !prev.has_crimson_reaper() {
            Some(EvolutionId::CrimsonReaper)
        } else if self.has_inferno_vortex() && !prev.has_inferno_vortex() {
            Some(EvolutionId::InfernoVortex)
        } else if self.has_celestial_tempest() && !prev.has_celestial_tempest() {
            Some(EvolutionId::CelestialTempest)
        } else if self.has_soul_harvester() && !prev.has_soul_harvester() {
            Some(EvolutionId::SoulHarvester)
        } else {
            None
        }
    }

    pub fn build_emojis(&self) -> String {
        let mut parts = Vec::new();
        if self.slash_lvl > 0 {
            parts.push(format!("⚔️{}", self.slash_lvl));
        }
        if self.bolt_lvl > 0 {
            parts.push(format!("🔥{}", self.bolt_lvl));
        }
        if self.lightning_lvl > 0 {
            parts.push(format!("⚡{}", self.lightning_lvl));
        }
        if self.orbit_lvl > 0 {
            parts.push(format!("🛡️{}", self.orbit_lvl));
        }
        if self.dash_lvl > 1 {
            parts.push(format!("💨{}", self.dash_lvl));
        }
        for evo in self.evolutions() {
            parts.push(evo.emoji().to_string());
        }
        parts.join(" ")
    }

    pub fn build_summary(&self) -> String {
        self.build_emojis()
    }

    pub fn roll_three(&self, rng: &mut Rng) -> [UpgradeId; 3] {
        let mut pool: Vec<UpgradeId> = ALL_UPGRADES
            .iter()
            .copied()
            .filter(|&u| self.level_of(u) < 5)
            .collect();
        if pool.len() < 3 {
            pool = ALL_UPGRADES.to_vec();
        }
        // Fisher-Yates deterministic shuffle
        for i in (1..pool.len()).rev() {
            let j = rng.range_usize(0, i + 1);
            pool.swap(i, j);
        }
        [pool[0], pool[1], pool[2]]
    }

    pub fn haste_mult(&self) -> f32 {
        1.0 + (self.haste_lvl as f32) * 0.18
    }

    pub fn move_speed(&self) -> f32 {
        265.0 * (1.0 + (self.haste_lvl as f32) * 0.10)
    }

    pub fn pickup_radius(&self) -> f32 {
        95.0 * (1.0 + (self.magnet_lvl as f32) * 0.42)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hero_classes_and_factions_roundtrip() {
        for idx in 0..3 {
            let c = HeroClass::from_index(idx);
            assert_eq!(c.to_index(), idx);
            assert!(!c.title().is_empty());
            assert!(!c.faction_name().is_empty());
            assert!(c.base_max_hp() >= 90.0);
            let b = HeroBuild::for_class(c);
            assert!(!b.build_summary().is_empty());
        }
    }

    #[test]
    fn weapon_evolutions_require_lv3_pair() {
        let mut b = HeroBuild::default();
        assert!(b.evolutions().is_empty());

        // EmberBolts Lv3 + OverclockHaste Lv2 -> NOT evolved yet
        b.apply(UpgradeId::EmberBolts);
        b.apply(UpgradeId::EmberBolts);
        b.apply(UpgradeId::OverclockHaste);
        b.apply(UpgradeId::OverclockHaste);
        assert_eq!(b.bolt_lvl, 3);
        assert_eq!(b.haste_lvl, 2);
        assert!(!b.has_meteor_barrage());

        let prev = b.clone();
        b.apply(UpgradeId::OverclockHaste);
        assert!(b.has_meteor_barrage());
        assert_eq!(
            b.newly_unlocked_evolution(&prev),
            Some(EvolutionId::MeteorBarrage)
        );

        // Crimson Reaper: ArcaneSlash Lv3 + VampiricWard Lv3
        b.slash_lvl = 3;
        b.ward_lvl = 2;
        assert!(!b.has_crimson_reaper());
        let prev2 = b.clone();
        b.apply(UpgradeId::VampiricWard);
        assert!(b.has_crimson_reaper());
        assert_eq!(
            b.newly_unlocked_evolution(&prev2),
            Some(EvolutionId::CrimsonReaper)
        );

        // Inferno Vortex: RunicOrbit Lv3 + NovaDash Lv3
        b.orbit_lvl = 3;
        b.dash_lvl = 2;
        assert!(!b.has_inferno_vortex());
        let prev3 = b.clone();
        b.apply(UpgradeId::NovaDash);
        assert!(b.has_inferno_vortex());
        assert_eq!(
            b.newly_unlocked_evolution(&prev3),
            Some(EvolutionId::InfernoVortex)
        );

        // Celestial Tempest: ChainLightning Lv3 + SoulMagnet Lv3 (also unlocks SoulHarvester since ward_lvl == 3!)
        b.lightning_lvl = 3;
        b.magnet_lvl = 2;
        assert!(!b.has_celestial_tempest());
        assert!(!b.has_soul_harvester());
        let prev4 = b.clone();
        b.apply(UpgradeId::SoulMagnet);
        assert!(b.has_celestial_tempest());
        assert!(b.has_soul_harvester());
        assert_eq!(
            b.newly_unlocked_evolution(&prev4),
            Some(EvolutionId::CelestialTempest)
        );
        assert_eq!(b.evolutions().len(), 5);

        for i in 0..4 {
            assert!(!RelicId::from_index(i).title().is_empty());
        }
    }

    #[test]
    fn build_summary_contains_levels_and_evolutions() {
        let mut b = HeroBuild::default();
        b.bolt_lvl = 3;
        b.haste_lvl = 3;
        b.lightning_lvl = 2;
        let s = b.build_summary();
        assert!(s.contains("⚔️1"));
        assert!(s.contains("🔥3"));
        assert!(s.contains("⚡2"));
        assert!(s.contains("☄️"));
    }

    #[test]
    fn roll_three_is_deterministic_and_distinct() {
        let b = HeroBuild::default();
        let mut r1 = Rng::from_seed_str("2026-11-21");
        let mut r2 = Rng::from_seed_str("2026-11-21");
        let c1 = b.roll_three(&mut r1);
        let c2 = b.roll_three(&mut r2);
        assert_eq!(c1, c2);
        assert_ne!(c1[0], c1[1]);
        assert_ne!(c1[1], c1[2]);
        assert_ne!(c1[0], c1[2]);
    }
}


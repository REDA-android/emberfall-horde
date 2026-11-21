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
        Self {
            slash_lvl: 1,
            bolt_lvl: 1,
            lightning_lvl: 0,
            orbit_lvl: 0,
            dash_lvl: 1,
            haste_lvl: 0,
            magnet_lvl: 0,
            ward_lvl: 0,
        }
    }
}

impl HeroBuild {
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

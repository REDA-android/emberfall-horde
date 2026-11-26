use crate::rng::DailyBiome;
use crate::weapons::{HeroClass, UpgradeId};
use macroquad::prelude::*;

/// All high-detail dark-fantasy sprites, biome floors, faction companions, 8-upgrade icon atlas,
/// CC0 Kenney VFX textures, and Open-Source TTF typography (`ChakraPetch-Bold.ttf`, SIL OFL 1.1)
/// baked directly into the WASM binary via `include_bytes!()` so startup is instant and 100% CSP-safe.
pub struct GameAssets {
    pub hero: Texture2D,
    pub hero_mage: Texture2D,
    pub hero_rogue: Texture2D,
    pub pet_solar: Texture2D,
    pub pet_arcane: Texture2D,
    pub pet_shadow: Texture2D,
    pub barrel: Texture2D,
    pub chest: Texture2D,
    pub imp: Texture2D,
    pub golem: Texture2D,
    pub wraith: Texture2D,
    pub boss_malakor: Texture2D,
    pub boss_ignis: Texture2D,
    pub boss_skadi: Texture2D,
    pub floor: Texture2D,
    pub floor_volcanic: Texture2D,
    pub floor_frost: Texture2D,
    pub icons_atlas: Texture2D,
    pub vfx_fire: Texture2D,
    pub vfx_flame: Texture2D,
    pub vfx_spark: Texture2D,
    pub vfx_slash: Texture2D,
    pub vfx_magic: Texture2D,
    pub vfx_light: Texture2D,
    pub vfx_star: Texture2D,
    pub ui_font: Option<Font>,
}

fn load_tex(bytes: &[u8]) -> Texture2D {
    let tex = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    tex.set_filter(FilterMode::Linear);
    tex
}

impl GameAssets {
    pub fn load() -> Self {
        let ui_font = load_ttf_font_from_bytes(include_bytes!(
            "../assets/fonts/chakra_petch_bold.ttf"
        ))
        .ok();
        if let Some(ref f) = ui_font {
            let ascii = Font::ascii_character_list();
            for &sz in &[10u16, 11, 12, 13, 14, 15, 16, 18, 19, 20, 22, 26, 28, 30, 32, 34, 36, 40, 48] {
                f.populate_font_cache(&ascii, sz);
            }
        }
        Self {
            hero: load_tex(include_bytes!("../assets/sprites/hero_knight.png")),
            hero_mage: load_tex(include_bytes!("../assets/sprites/hero_mage.png")),
            hero_rogue: load_tex(include_bytes!("../assets/sprites/hero_rogue.png")),
            pet_solar: load_tex(include_bytes!("../assets/sprites/pet_solar.png")),
            pet_arcane: load_tex(include_bytes!("../assets/sprites/pet_arcane.png")),
            pet_shadow: load_tex(include_bytes!("../assets/sprites/pet_shadow.png")),
            barrel: load_tex(include_bytes!("../assets/sprites/runic_barrel.png")),
            chest: load_tex(include_bytes!("../assets/sprites/runic_chest.png")),
            imp: load_tex(include_bytes!("../assets/sprites/enemy_imp.png")),
            golem: load_tex(include_bytes!("../assets/sprites/enemy_golem.png")),
            wraith: load_tex(include_bytes!("../assets/sprites/enemy_wraith.png")),
            boss_malakor: load_tex(include_bytes!("../assets/sprites/boss_malakor.png")),
            boss_ignis: load_tex(include_bytes!("../assets/sprites/boss_ignis.png")),
            boss_skadi: load_tex(include_bytes!("../assets/sprites/boss_skadi.png")),
            floor: load_tex(include_bytes!("../assets/sprites/arena_floor.png")),
            floor_volcanic: load_tex(include_bytes!("../assets/sprites/floor_volcanic.png")),
            floor_frost: load_tex(include_bytes!("../assets/sprites/floor_frost.png")),
            icons_atlas: load_tex(include_bytes!("../assets/sprites/icons_atlas.png")),
            vfx_fire: load_tex(include_bytes!("../assets/vfx/fire_01.png")),
            vfx_flame: load_tex(include_bytes!("../assets/vfx/flame_01.png")),
            vfx_spark: load_tex(include_bytes!("../assets/vfx/spark_01.png")),
            vfx_slash: load_tex(include_bytes!("../assets/vfx/slash_01.png")),
            vfx_magic: load_tex(include_bytes!("../assets/vfx/magic_05.png")),
            vfx_light: load_tex(include_bytes!("../assets/vfx/light_01.png")),
            vfx_star: load_tex(include_bytes!("../assets/vfx/star_07.png")),
            ui_font,
        }
    }

    pub fn boss_for_biome(&self, biome: DailyBiome) -> &Texture2D {
        match biome {
            DailyBiome::ObsidianSanctuary => &self.boss_malakor,
            DailyBiome::VolcanicRift => &self.boss_ignis,
            DailyBiome::FrostAbyss => &self.boss_skadi,
        }
    }

    pub fn floor_for_biome(&self, biome: DailyBiome) -> &Texture2D {
        match biome {
            DailyBiome::ObsidianSanctuary => &self.floor,
            DailyBiome::VolcanicRift => &self.floor_volcanic,
            DailyBiome::FrostAbyss => &self.floor_frost,
        }
    }

    pub fn pet_for_class(&self, class: HeroClass) -> &Texture2D {
        match class {
            HeroClass::SunKnight => &self.pet_solar,
            HeroClass::ArcanePyromancer => &self.pet_arcane,
            HeroClass::ShadowAssassin => &self.pet_shadow,
        }
    }

    pub fn upgrade_icon_src(up: UpgradeId) -> Rect {
        let idx = match up {
            UpgradeId::ArcaneSlash => 0,
            UpgradeId::EmberBolts => 1,
            UpgradeId::ChainLightning => 2,
            UpgradeId::RunicOrbit => 3,
            UpgradeId::NovaDash => 4,
            UpgradeId::OverclockHaste => 5,
            UpgradeId::SoulMagnet => 6,
            UpgradeId::VampiricWard => 7,
        };
        let col = (idx % 4) as f32;
        let row = (idx / 4) as f32;
        Rect::new(col * 64.0, row * 64.0, 64.0, 64.0)
    }
}

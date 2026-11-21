use macroquad::prelude::*;

/// All high-detail sprites and CC0 Kenney VFX textures baked directly into the WASM binary
/// via `include_bytes!()` so startup is instant and 100% CSP-safe (zero external network fetches).
pub struct GameAssets {
    pub hero: Texture2D,
    pub imp: Texture2D,
    pub golem: Texture2D,
    pub wraith: Texture2D,
    pub boss: Texture2D,
    pub floor: Texture2D,
    pub vfx_fire: Texture2D,
    pub vfx_flame: Texture2D,
    pub vfx_spark: Texture2D,
    pub vfx_slash: Texture2D,
    pub vfx_magic: Texture2D,
    pub vfx_light: Texture2D,
    pub vfx_star: Texture2D,
}

fn load_tex(bytes: &[u8]) -> Texture2D {
    let tex = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    tex.set_filter(FilterMode::Linear);
    tex
}

impl GameAssets {
    pub fn load() -> Self {
        Self {
            hero: load_tex(include_bytes!("../assets/sprites/hero_knight.png")),
            imp: load_tex(include_bytes!("../assets/sprites/enemy_imp.png")),
            golem: load_tex(include_bytes!("../assets/sprites/enemy_golem.png")),
            wraith: load_tex(include_bytes!("../assets/sprites/enemy_wraith.png")),
            boss: load_tex(include_bytes!("../assets/sprites/boss_lich.png")),
            floor: load_tex(include_bytes!("../assets/sprites/arena_floor.png")),
            vfx_fire: load_tex(include_bytes!("../assets/vfx/fire_01.png")),
            vfx_flame: load_tex(include_bytes!("../assets/vfx/flame_01.png")),
            vfx_spark: load_tex(include_bytes!("../assets/vfx/spark_01.png")),
            vfx_slash: load_tex(include_bytes!("../assets/vfx/slash_01.png")),
            vfx_magic: load_tex(include_bytes!("../assets/vfx/magic_05.png")),
            vfx_light: load_tex(include_bytes!("../assets/vfx/light_01.png")),
            vfx_star: load_tex(include_bytes!("../assets/vfx/star_07.png")),
        }
    }
}

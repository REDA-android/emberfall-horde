use macroquad::prelude::*;

const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const HIT_FLASH_FRAGMENT: &str = r#"#version 100
precision mediump float;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform sampler2D Texture;
uniform float u_flash;

void main() {
    vec4 tex = texture2D(Texture, uv) * color;
    tex.rgb = mix(tex.rgb, vec3(1.0, 0.92, 0.8), clamp(u_flash, 0.0, 1.0));
    gl_FragColor = tex;
}
"#;

const POST_PROCESS_FRAGMENT: &str = r#"#version 100
precision mediump float;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform sampler2D Texture;
uniform float u_time;
uniform float u_low_hp;
uniform float u_aberration;
uniform float u_shockwave;
uniform float u_blood_moon;

void main() {
    vec2 center = vec2(0.5, 0.5);
    vec2 delta = uv - center;
    float dist = length(delta);
    vec2 dir = dist > 0.0001 ? delta / dist : vec2(0.0, 0.0);

    vec2 sample_uv = uv;
    if (u_shockwave > 0.001) {
        float progress = 1.0 - clamp(u_shockwave, 0.0, 1.0);
        float radius = progress * 0.65;
        float band = 1.0 - smoothstep(0.0, 0.085, abs(dist - radius));
        sample_uv -= dir * band * 0.028 * u_shockwave;
    }

    vec3 col;
    if (u_aberration > 0.0002) {
        vec2 off = delta * u_aberration;
        float r = texture2D(Texture, clamp(sample_uv + off, 0.0, 1.0)).r;
        float g = texture2D(Texture, clamp(sample_uv, 0.0, 1.0)).g;
        float b = texture2D(Texture, clamp(sample_uv - off, 0.0, 1.0)).b;
        col = vec3(r, g, b);
    } else {
        col = texture2D(Texture, clamp(sample_uv, 0.0, 1.0)).rgb;
    }

    // Dark fantasy radial vignette
    float vig = smoothstep(0.34, 0.94, dist);
    col *= 1.0 - vig * 0.42;

    // Subtle Blood Moon tint on edges
    if (u_blood_moon > 0.01) {
        col += vec3(0.16, 0.01, 0.03) * vig * u_blood_moon;
    }

    // Low-HP crimson pulse
    if (u_low_hp > 0.01) {
        float pulse = 0.5 + 0.5 * sin(u_time * 9.0);
        float edge = smoothstep(0.26, 0.88, dist);
        col = mix(col, vec3(0.75, 0.04, 0.08), edge * u_low_hp * (0.28 + 0.34 * pulse));
    }

    gl_FragColor = vec4(col, 1.0) * color;
}
"#;

pub struct GameShaders {
    pub hit_flash: Option<Material>,
    pub post_process: Option<Material>,
}

impl GameShaders {
    pub fn load() -> Self {
        let hit_flash = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: HIT_FLASH_FRAGMENT,
            },
            MaterialParams {
                uniforms: vec![UniformDesc::new("u_flash", UniformType::Float1)],
                ..Default::default()
            },
        )
        .ok();

        let post_process = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: POST_PROCESS_FRAGMENT,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("u_time", UniformType::Float1),
                    UniformDesc::new("u_low_hp", UniformType::Float1),
                    UniformDesc::new("u_aberration", UniformType::Float1),
                    UniformDesc::new("u_shockwave", UniformType::Float1),
                    UniformDesc::new("u_blood_moon", UniformType::Float1),
                ],
                ..Default::default()
            },
        )
        .ok();

        Self {
            hit_flash,
            post_process,
        }
    }
}

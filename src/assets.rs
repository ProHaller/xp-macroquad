use color_eyre::{Result, eyre::Context};
use macroquad::{
    audio::{Sound, load_sound},
    experimental::animation::AnimatedSprite,
    prelude::*,
};

use crate::state::Shape;

const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");
const VERTEX_SHADER: &str = include_str!("vertex_shader.glsl");

#[derive(Debug)]
pub struct Assets {
    pub textures: Textures,

    pub sounds: Sounds,
    pub material: Material,
    pub render_target: RenderTarget,
}

#[derive(Debug)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub shape: Shape,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EnemyKind {
    Small,
    Medium,
    Big,
}

impl EnemyKind {
    pub fn random() -> Self {
        Self::from_roll(rand::gen_range(0.0, 1.0))
    }
    pub fn from_roll(roll: f32) -> Self {
        match roll {
            s if s < 0.6 => EnemyKind::Small,
            s if s < 0.9 => EnemyKind::Medium,
            _ => EnemyKind::Big,
        }
    }
    /// Drawn height range, in screen pixels.
    pub fn size_range(self) -> (f32, f32) {
        match self {
            EnemyKind::Small => (16.0, 32.0),
            EnemyKind::Medium => (32.0, 48.0),
            EnemyKind::Big => (48.0, 64.0),
        }
    }

    pub fn frame_size(self) -> Vec2 {
        match self {
            EnemyKind::Small => vec2(34. / 2., 16.),
            EnemyKind::Medium => vec2(64. / 2., 16.),
            EnemyKind::Big => vec2(64. / 2., 32.),
        }
    }
}

impl Assets {
    pub async fn load() -> Result<Self> {
        set_pc_assets_folder("assets");
        let textures = Textures::load().await?;
        let sounds = Sounds::load().await?;
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("iResolution", UniformType::Float2),
                    UniformDesc::new("direction_modifier", UniformType::Float1),
                ],
                ..Default::default()
            },
        )
        .with_context(|| "Couldn't load Material")?;

        build_textures_atlas();

        let render_target = render_target(screen_width() as u32, screen_height() as u32);
        render_target.texture.set_filter(FilterMode::Nearest);

        Ok(Self {
            textures,
            sounds,
            material,
            render_target,
        })
    }
    pub fn render(&self, direction_modifier: f32) {
        self.material
            .set_uniform("iResolution", (screen_width(), screen_height()));
        self.material
            .set_uniform("direction_modifier", direction_modifier);
        gl_use_material(&self.material);
        draw_texture_ex(
            &self.render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Textures {
    pub ship: Texture2D,
    pub enemy_small: Texture2D,
    pub enemy_medium: Texture2D,
    pub enemy_big: Texture2D,
    pub laser: Texture2D,
}

impl Textures {
    pub async fn load() -> Result<Self> {
        let ship: Texture2D = load_texture("ship.png")
            .await
            .with_context(|| "Couldn't load file")?;
        ship.set_filter(FilterMode::Nearest);

        let enemy_small: Texture2D = load_texture("enemy-small.png")
            .await
            .with_context(|| "Couldn't load file")?;
        enemy_small.set_filter(FilterMode::Nearest);

        let enemy_medium: Texture2D = load_texture("enemy-medium.png")
            .await
            .with_context(|| "Couldn't load file")?;
        enemy_medium.set_filter(FilterMode::Nearest);

        let enemy_big: Texture2D = load_texture("enemy-big.png")
            .await
            .with_context(|| "Couldn't load file")?;
        enemy_big.set_filter(FilterMode::Nearest);

        let laser: Texture2D = load_texture("laser-bolts.png")
            .await
            .with_context(|| "Couldn't load file")?;
        laser.set_filter(FilterMode::Nearest);
        Ok(Self {
            ship,
            enemy_small,
            enemy_medium,
            enemy_big,
            laser,
        })
    }
    pub fn enemy(&self, enemy: EnemyKind) -> &Texture2D {
        match enemy {
            EnemyKind::Small => &self.enemy_small,
            EnemyKind::Medium => &self.enemy_medium,
            EnemyKind::Big => &self.enemy_big,
        }
    }
}

#[derive(Debug)]
pub struct Sounds {
    pub theme: Sound,
    pub explosion: Sound,
    pub gameover: Sound,
    pub laser: Sound,
}

impl Sounds {
    pub async fn load() -> Result<Self> {
        let theme = load_sound("8bit-spaceshooter.ogg").await?;
        let explosion = load_sound("explosion.wav").await?;
        let gameover = load_sound("fart_1.wav").await?;
        let laser = load_sound("laser.wav").await?;
        Ok(Self {
            theme,
            explosion,
            gameover,
            laser,
        })
    }
}

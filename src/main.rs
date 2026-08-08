use color_eyre::{
    Result,
    eyre::{Ok, WrapErr},
};
use macroquad::{
    audio::{PlaySoundParams, Sound, load_sound, play_sound},
    experimental::animation::{AnimatedSprite, Animation},
    prelude::*,
    ui::{hash, root_ui, widgets},
};
use macroquad_particles::{self as particles, ColorCurve, Emitter, EmitterConfig};

use ron::de::SpannedError;
use serde::{Deserialize, Serialize};

use std::{fmt::Debug, fs, path::Path, process::exit, str::FromStr, time::SystemTime};

use crate::Mode::MainMenu;

const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");
const VERTEX_SHADER: &str = include_str!("vertex_shader.glsl");
const HIGHSCORE_PATH: &str = "highscore.ron";
const MOVEMENT_SPEED: f32 = 600.0;

#[derive(Debug)]
struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    collided: bool,
}

impl Shape {
    fn collides_with(&self, other: &Self) -> bool {
        self.rect().overlaps(&other.rect())
    }

    fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            w: self.size,
            h: self.size,
        }
    }
}

#[derive(Debug)]
enum Mode {
    MainMenu,
    Playing,
    Paused,
    GameOver,
    Input,
}

#[derive(Debug)]
struct Assets {
    textures: Textures,
    sounds: Sounds,
    material: Material,
    render_target: RenderTarget,
}

impl Assets {
    async fn load() -> Result<Self> {
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
    fn render(&self, direction_modifier: f32) {
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
struct Textures {
    ship: Texture2D,
    enemy_small: Texture2D,
    enemy_medium: Texture2D,
    enemy_big: Texture2D,
    laser: Texture2D,
}
impl Textures {
    async fn load() -> Result<Self> {
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
}

#[derive(Debug)]
struct Sounds {
    theme: Sound,
    explosion: Sound,
    gameover: Sound,
    laser: Sound,
}

impl Sounds {
    async fn load() -> Result<Self> {
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

#[allow(unused)]
struct State {
    player_ship: Shape,
    lasers: Vec<Shape>,
    enemies: Vec<Shape>,

    laser_sprite: AnimatedSprite,
    ship_sprite: AnimatedSprite,
    enemy_small_sprite: AnimatedSprite,
    medium_enemy_sprite: AnimatedSprite,
    large_enemy_sprite: AnimatedSprite,

    explosions: Vec<(Emitter, Vec2)>,
    direction_modifier: f32,
    input_text: String,
    score_board: ScoreBoard,
    score: u32,
    mode: Mode,
}

impl State {
    fn init() -> State {
        let player_ship = Shape {
            size: 32.0,
            speed: MOVEMENT_SPEED,
            x: screen_width() / 2.0,
            y: screen_height() / 3.0,
            collided: false,
        };
        let lasers = vec![];
        let enemies = vec![];
        let explosions: Vec<(Emitter, Vec2)> = vec![];

        let direction_modifier: f32 = 0.0;
        let input_text = String::new();

        let score_board = ScoreBoard::from(HIGHSCORE_PATH);
        let score: u32 = 0;

        let mode = Mode::MainMenu;

        let mut laser_sprite = AnimatedSprite::new(
            16,
            16,
            &[
                Animation {
                    name: "laser".to_string(),
                    row: 0,
                    frames: 2,
                    fps: 12,
                },
                Animation {
                    name: "bolt".to_string(),
                    row: 1,
                    frames: 2,
                    fps: 12,
                },
            ],
            true,
        );
        laser_sprite.set_animation(1);
        let ship_sprite = AnimatedSprite::new(
            16,
            24,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 2,
                    fps: 12,
                },
                Animation {
                    name: "left".to_string(),
                    row: 2,
                    frames: 2,
                    fps: 12,
                },
                Animation {
                    name: "right".to_string(),
                    row: 4,
                    frames: 2,
                    fps: 12,
                },
            ],
            true,
        );

        let enemy_small_sprite = AnimatedSprite::new(
            16,
            24,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 2,
                    fps: 12,
                },
                Animation {
                    name: "left".to_string(),
                    row: 2,
                    frames: 2,
                    fps: 12,
                },
                Animation {
                    name: "right".to_string(),
                    row: 4,
                    frames: 2,
                    fps: 12,
                },
            ],
            true,
        );
        let medium_enemy_sprite = AnimatedSprite::new(
            16,
            24,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 2,
                    fps: 12,
                },
                Animation {
                    name: "left".to_string(),
                    row: 2,
                    frames: 2,
                    fps: 12,
                },
                Animation {
                    name: "right".to_string(),
                    row: 4,
                    frames: 2,
                    fps: 12,
                },
            ],
            true,
        );
        let large_enemy_sprite = AnimatedSprite::new(
            16,
            24,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 2,
                    fps: 12,
                },
                Animation {
                    name: "left".to_string(),
                    row: 2,
                    frames: 2,
                    fps: 12,
                },
                Animation {
                    name: "right".to_string(),
                    row: 4,
                    frames: 2,
                    fps: 12,
                },
            ],
            true,
        );

        State {
            player_ship,
            lasers,
            enemies,
            explosions,
            direction_modifier,
            input_text,
            score_board,
            score,
            mode,
            laser_sprite,
            ship_sprite,
            enemy_small_sprite,
            medium_enemy_sprite,
            large_enemy_sprite,
        }
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("player_ship", &self.player_ship)
            .field("laser", &self.lasers)
            .field("enemies", &self.enemies)
            .field("explosions count", &self.explosions.len())
            .field("direction_modifier", &self.direction_modifier)
            .field("input_text", &self.input_text)
            .field("score_board", &self.score_board)
            .field("score", &self.score)
            .field("mode", &self.mode)
            .finish()
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
struct Score {
    name: String,
    points: u32,
    timestamp: SystemTime,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            name: String::from("Anonymous"),
            points: Default::default(),
            timestamp: SystemTime::now(),
        }
    }
}

impl ScoreBoard {
    fn best(&self) -> Score {
        self.scores.iter().max().cloned().unwrap_or_default()
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Serialize, Deserialize, Debug, Default)]
struct ScoreBoard {
    scores: Vec<Score>,
}

impl FromStr for ScoreBoard {
    type Err = SpannedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ron::from_str(s)
    }
}
impl<T> From<T> for ScoreBoard
where
    T: AsRef<Path>,
{
    fn from(value: T) -> ScoreBoard {
        fs::read_to_string(value).map_or(ScoreBoard::default(), |i| {
            ScoreBoard::from_str(&i).unwrap_or_default()
        })
    }
}
impl ScoreBoard {
    fn save(&self) {
        let serialized = ron::to_string(&self).expect("failed serialization");
        fs::write(HIGHSCORE_PATH, serialized).unwrap();
    }
}

fn particle_explosion() -> particles::EmitterConfig {
    particles::EmitterConfig {
        local_coords: false,
        one_shot: true,
        emitting: true,
        lifetime: 0.6,
        lifetime_randomness: 0.3,
        explosiveness: 0.65,
        initial_direction_spread: 2.0 * std::f32::consts::PI,
        initial_velocity: 300.0,
        initial_velocity_randomness: 0.8,
        size: 3.0,
        size_randomness: 0.3,
        colors_curve: ColorCurve {
            start: RED,
            mid: ORANGE,
            end: RED,
        },
        ..Default::default()
    }
}

#[macroquad::main("xp-macroquad")]
async fn main() -> Result<()> {
    rand::srand(miniquad::date::now() as u64);

    let assets = Assets::load().await?;

    let mut state = State::init();

    play_sound(
        &assets.sounds.theme,
        PlaySoundParams {
            looped: true,
            volume: 1.,
        },
    );

    loop {
        clear_background(BLACK);

        assets.render(state.direction_modifier);

        match state.mode {
            Mode::MainMenu => {
                if is_key_pressed(KeyCode::Escape) || is_key_down(KeyCode::Q) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Space) {
                    state.enemies.clear();
                    state.lasers.clear();
                    state.explosions.clear();
                    state.player_ship.x = screen_width() / 2.0;
                    state.player_ship.y = screen_height() / 3.0;
                    state.score = 0;
                    state.mode = Mode::Playing;
                }
                let text = "Press space";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    WHITE,
                );
            }
            Mode::Playing => {
                let delta_time = get_frame_time();

                state.ship_sprite.set_animation(0);

                // start new game
                if is_key_down(KeyCode::N) {
                    state.score = 0;
                }
                // TODO: Make keybinds configurable in menu
                if is_key_down(KeyCode::Right) || is_key_down(KeyCode::I) {
                    state.player_ship.x += MOVEMENT_SPEED * delta_time;
                    state.direction_modifier += 0.05 * delta_time;
                    state.ship_sprite.set_animation(2);
                }
                if is_key_down(KeyCode::Left) || is_key_down(KeyCode::L) {
                    state.player_ship.x -= MOVEMENT_SPEED * delta_time;
                    state.direction_modifier -= 0.05 * delta_time;
                    state.ship_sprite.set_animation(1);
                }
                if is_key_down(KeyCode::Down) || is_key_down(KeyCode::R) {
                    state.player_ship.y += MOVEMENT_SPEED * delta_time;
                }
                if is_key_down(KeyCode::Up) || is_key_down(KeyCode::T) {
                    state.player_ship.y -= MOVEMENT_SPEED * delta_time;
                }

                if is_key_pressed(KeyCode::Space) {
                    state.lasers.push(Shape {
                        x: state.player_ship.x,
                        y: state.player_ship.y - 24.0,
                        speed: state.player_ship.speed * 2.0,
                        size: 32.0,
                        collided: false,
                    });

                    play_sound(
                        &assets.sounds.laser,
                        PlaySoundParams {
                            looped: false,
                            volume: 1.,
                        },
                    );
                }
                if is_key_pressed(KeyCode::Escape) {
                    state.mode = Mode::Paused;
                }

                // Clamp X and Y to be within the screen
                wrap_around(&mut state.player_ship);

                // Generate a new square
                if rand::gen_range(0, 99) >= 95 {
                    let size = rand::gen_range(16.0, 64.0);
                    state.enemies.push(Shape {
                        size,
                        speed: rand::gen_range(50.0, 150.0),
                        x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                        y: -size,
                        collided: false,
                    });
                }

                // Square Movement
                for square in &mut state.enemies {
                    square.y += square.speed * delta_time;
                }
                // laser Movement
                for laser in &mut state.lasers {
                    laser.y -= laser.speed * delta_time;
                }

                state.ship_sprite.update();
                state.laser_sprite.update();

                // Remove shapes outside of screen
                state
                    .enemies
                    .retain(|square| square.y < screen_height() + square.size);
                state
                    .lasers
                    .retain(|laser| laser.y > 0.0 - laser.size / 2.0);

                // Remove collided shapes
                state.enemies.retain(|square| !square.collided);
                state.lasers.retain(|laser| !laser.collided);

                // Remove old explosions
                state
                    .explosions
                    .retain(|(explosion, _)| explosion.config.emitting);

                // Check for collisions
                if state
                    .enemies
                    .iter()
                    .any(|square| state.player_ship.collides_with(square))
                {
                    play_sound(
                        &assets.sounds.gameover,
                        PlaySoundParams {
                            looped: false,
                            volume: 2.,
                        },
                    );
                    state.mode = Mode::GameOver;
                }
                for square in state.enemies.iter_mut() {
                    for laser in state.lasers.iter_mut() {
                        if laser.collides_with(square) {
                            laser.collided = true;
                            square.collided = true;
                            state.score += square.size.round() as u32;
                            // TODO: handle error
                            state.explosions.push((
                                Emitter::new(EmitterConfig {
                                    amount: square.size.round() as u32 * 2,
                                    ..particle_explosion()
                                }),
                                vec2(square.x, square.y),
                            ));
                            play_sound(
                                &assets.sounds.explosion,
                                PlaySoundParams {
                                    looped: false,
                                    volume: 1.,
                                },
                            );
                        }
                    }
                }

                // Draw everything
                let laser_frame = state.laser_sprite.frame();
                for laser in &state.lasers {
                    draw_texture_ex(
                        &assets.textures.laser,
                        laser.x - laser.size / 2.0,
                        laser.y - laser.size / 2.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(laser.size, laser.size)),
                            source: Some(laser_frame.source_rect),
                            ..Default::default()
                        },
                    );
                }
                let ship_frame = state.ship_sprite.frame();

                let enemy_frame = state.enemy_small_sprite.frame();
                draw_texture_ex(
                    &assets.textures.ship,
                    state.player_ship.x - ship_frame.dest_size.x,
                    state.player_ship.y - ship_frame.dest_size.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(ship_frame.dest_size),
                        source: Some(ship_frame.source_rect),
                        ..Default::default()
                    },
                );
                for enemy in &state.enemies {
                    draw_texture_ex(
                        &assets.textures.enemy_small,
                        enemy.x - enemy_frame.dest_size.x * 2.0,
                        enemy.y - enemy_frame.dest_size.y * 2.0,
                        GREEN,
                        // TODO: Fix enemy size
                        DrawTextureParams {
                            dest_size: Some(enemy_frame.dest_size / 1. + enemy.size),
                            source: Some(enemy_frame.source_rect),
                            ..Default::default()
                        },
                    );
                }
                for (explosion, coords) in state.explosions.iter_mut() {
                    explosion.draw(*coords);
                }
                draw_text(
                    format!("Score: {}", state.score).as_str(),
                    10.0,
                    35.0,
                    25.0,
                    WHITE,
                );
                let highscore_text = format!(
                    "High score: {}: {}",
                    state.score_board.best().name,
                    state.score_board.best().points
                );
                let text_dimensions = measure_text(highscore_text.as_str(), None, 25, 1.0);
                draw_text(
                    highscore_text.as_str(),
                    screen_width() - text_dimensions.width - 10.0,
                    35.0,
                    25.0,
                    WHITE,
                );
            }
            Mode::Paused => {
                if is_key_pressed(KeyCode::Space) {
                    state.mode = Mode::Playing;
                }
                if is_key_pressed(KeyCode::Q) {
                    exit(0)
                }
                let text = "Paused";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    WHITE,
                );
            }
            Mode::GameOver => {
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::N) {
                    state.mode = MainMenu;
                }
                if is_key_pressed(KeyCode::Q) {
                    exit(0)
                }
                let text = "GAME OVER!";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    RED,
                );
                if state.score > state.score_board.best().points {
                    state.mode = Mode::Input
                }
            }
            Mode::Input => {
                widgets::Window::new(
                    hash!(),
                    vec2(screen_width() / 2., screen_height() / 2.),
                    vec2(500., 300.),
                )
                .label("Input")
                .ui(&mut root_ui(), |ui| {
                    ui.input_text(hash!(), "Your Name", &mut state.input_text);
                    if ui.button(None, "Save") {
                        state.score_board.scores.push(Score {
                            name: state.input_text.clone(),
                            points: state.score,
                            timestamp: SystemTime::now(),
                        });
                        state.score_board.save();
                        state.mode = MainMenu;
                    }
                });
            }
        }

        next_frame().await;
    }
}

fn wrap_around(ship: &mut Shape) {
    if ship.x > screen_width() {
        ship.x = 0.;
    }
    if ship.x < 0. {
        ship.x = screen_width()
    }
    if ship.y > screen_height() {
        ship.y = 0.;
    }
    if ship.y < 0. {
        ship.y = screen_height()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_serialize() {
        let score = Score {
            name: "test".to_string(),
            points: 999999,
            timestamp: std::time::SystemTime::now(),
        };
        let score2 = Score {
            name: "test2".to_string(),
            points: 999999,
            timestamp: std::time::SystemTime::now(),
        };
        let scores = vec![score, score2];

        let serialize = ron::to_string(&scores).unwrap();
        fs::write("test.ron", &serialize).unwrap();
        dbg!(&serialize);
        assert_eq!(ron::from_str::<Vec<Score>>(&serialize).unwrap(), scores);
    }

    #[test]
    fn test_load_score() -> Result<()> {
        let scores_str = fs::read_to_string("highscore.ron")?;
        dbg!(&scores_str);
        let parsed: ScoreBoard = ron::from_str(&scores_str)?;
        dbg!(&parsed);
        let scores: Vec<u32> = parsed.scores.iter().map(|s| s.points).collect();
        dbg!(&scores);

        assert!(scores.iter().max().unwrap() > &0u32);
        Ok(())
    }
}

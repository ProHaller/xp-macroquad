use macroquad::audio::{PlaySoundParams, load_sound, play_sound};
use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use macroquad_particles::{self as particles, ColorCurve, Emitter, EmitterConfig};
use ron::de::SpannedError;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use std::cmp::max_by;
use std::fs;
use std::process::exit;
use std::str::FromStr;
use std::time::SystemTime;

use crate::Mode::MainMenu;

const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

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

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
struct Score {
    name: String,
    points: u32,
    timestamp: SystemTime,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            name: Default::default(),
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
impl ScoreBoard {
    fn save(&self) {
        let serialized = ron::to_string(&self).unwrap();
        fs::write("highscore.ron", serialized).unwrap();
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
async fn main() {
    const MOVEMENT_SPEED: f32 = 600.0;

    rand::srand(miniquad::date::now() as u64);
    let mut squares = vec![];
    let mut bullets: Vec<Shape> = vec![];
    let mut player_ship = Shape {
        size: 32.0,
        speed: MOVEMENT_SPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        collided: false,
    };
    let mut score: u32 = 0;
    let mut score_board = fs::read_to_string("highscore.ron")
        .map_or(Ok(ScoreBoard::default()), |i| i.parse::<ScoreBoard>())
        .unwrap_or_default();
    let mut game_state = Mode::MainMenu;

    let mut input_text = String::new();

    let mut direction_modifier: f32 = 0.0;
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);
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
    .unwrap();

    let mut explosions: Vec<(Emitter, Vec2)> = vec![];

    set_pc_assets_folder("assets");
    let ship_texture: Texture2D = load_texture("ship.png").await.expect("Couldn't load file");
    ship_texture.set_filter(FilterMode::Nearest);

    let enemy_texture: Texture2D = load_texture("enemy-small.png")
        .await
        .expect("Couldn't load file");
    enemy_texture.set_filter(FilterMode::Nearest);

    let bullet_texture: Texture2D = load_texture("laser-bolts.png")
        .await
        .expect("Couldn't load file");
    bullet_texture.set_filter(FilterMode::Nearest);
    build_textures_atlas();

    let mut bullet_sprite = AnimatedSprite::new(
        16,
        16,
        &[
            Animation {
                name: "bullet".to_string(),
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
    bullet_sprite.set_animation(1);
    let mut ship_sprite = AnimatedSprite::new(
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

    let enemy_sprite = AnimatedSprite::new(
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

    let theme_music = load_sound("8bit-spaceshooter.ogg").await.unwrap();
    let sound_explosion = load_sound("explosion.wav").await.unwrap();
    let sound_gameover = load_sound("fart_1.wav").await.unwrap();
    let sound_laser = load_sound("laser.wav").await.unwrap();

    play_sound(
        &theme_music,
        PlaySoundParams {
            looped: true,
            volume: 1.,
        },
    );

    loop {
        clear_background(BLACK);

        material.set_uniform("iResolution", (screen_width(), screen_height()));
        material.set_uniform("direction_modifier", direction_modifier);
        gl_use_material(&material);
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();

        match game_state {
            Mode::MainMenu => {
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Escape) | is_key_down(KeyCode::Q) {
                    exit(0)
                }
                if is_key_pressed(KeyCode::Space) {
                    squares.clear();
                    bullets.clear();
                    explosions.clear();
                    player_ship.x = screen_width() / 2.0;
                    player_ship.y = screen_height() / 2.0;
                    score = 0;
                    game_state = Mode::Playing;
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
                ship_sprite.set_animation(0);
                if is_key_down(KeyCode::N) {
                    score = 0;
                }
                if is_key_down(KeyCode::Right) | is_key_down(KeyCode::I) {
                    player_ship.x += MOVEMENT_SPEED * delta_time;
                    direction_modifier += 0.05 * delta_time;
                    ship_sprite.set_animation(2);
                }
                if is_key_down(KeyCode::Left) | is_key_down(KeyCode::L) {
                    player_ship.x -= MOVEMENT_SPEED * delta_time;
                    direction_modifier -= 0.05 * delta_time;
                    ship_sprite.set_animation(1);
                }
                if is_key_down(KeyCode::Down) | is_key_down(KeyCode::R) {
                    player_ship.y += MOVEMENT_SPEED * delta_time;
                }
                if is_key_down(KeyCode::Up) | is_key_down(KeyCode::T) {
                    player_ship.y -= MOVEMENT_SPEED * delta_time;
                }
                if is_key_pressed(KeyCode::Space) {
                    bullets.push(Shape {
                        x: player_ship.x,
                        y: player_ship.y - 24.0,
                        speed: player_ship.speed * 2.0,
                        size: 32.0,
                        collided: false,
                    });

                    play_sound(
                        &sound_laser,
                        PlaySoundParams {
                            looped: false,
                            volume: 1.,
                        },
                    );
                }
                if is_key_pressed(KeyCode::Escape) {
                    game_state = Mode::Paused;
                }

                // Clamp X and Y to be within the screen
                player_ship.x = clamp(player_ship.x, 0.0, screen_width());
                player_ship.y = clamp(player_ship.y, 0.0, screen_height());

                // Generate a new square
                if rand::gen_range(0, 99) >= 95 {
                    let size = rand::gen_range(16.0, 64.0);
                    squares.push(Shape {
                        size,
                        speed: rand::gen_range(50.0, 150.0),
                        x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                        y: -size,
                        collided: false,
                    });
                }

                // Movement
                for square in &mut squares {
                    square.y += square.speed * delta_time;
                }
                for bullet in &mut bullets {
                    bullet.y -= bullet.speed * delta_time;
                }

                ship_sprite.update();
                bullet_sprite.update();

                // Remove shapes outside of screen
                squares.retain(|square| square.y < screen_height() + square.size);
                bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);

                // Remove collided shapes
                squares.retain(|square| !square.collided);
                bullets.retain(|bullet| !bullet.collided);

                // Remove old explosions
                explosions.retain(|(explosion, _)| explosion.config.emitting);

                // Check for collisions
                if squares
                    .iter()
                    .any(|square| player_ship.collides_with(square))
                {
                    play_sound(
                        &sound_gameover,
                        PlaySoundParams {
                            looped: false,
                            volume: 2.,
                        },
                    );
                    game_state = Mode::GameOver;
                }
                for square in squares.iter_mut() {
                    for bullet in bullets.iter_mut() {
                        if bullet.collides_with(square) {
                            bullet.collided = true;
                            square.collided = true;
                            score += square.size.round() as u32;
                            // TODO: handle error
                            explosions.push((
                                Emitter::new(EmitterConfig {
                                    amount: square.size.round() as u32 * 2,
                                    ..particle_explosion()
                                }),
                                vec2(square.x, square.y),
                            ));
                            play_sound(
                                &sound_explosion,
                                PlaySoundParams {
                                    looped: false,
                                    volume: 1.,
                                },
                            );
                        }
                    }
                }

                // Draw everything
                let bullet_frame = bullet_sprite.frame();
                for bullet in &bullets {
                    draw_texture_ex(
                        &bullet_texture,
                        bullet.x - bullet.size / 2.0,
                        bullet.y - bullet.size / 2.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(bullet.size, bullet.size)),
                            source: Some(bullet_frame.source_rect),
                            ..Default::default()
                        },
                    );
                }
                let ship_frame = ship_sprite.frame();

                let enemy_frame = enemy_sprite.frame();
                draw_texture_ex(
                    &ship_texture,
                    player_ship.x - ship_frame.dest_size.x,
                    player_ship.y - ship_frame.dest_size.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(ship_frame.dest_size),
                        source: Some(ship_frame.source_rect),
                        ..Default::default()
                    },
                );
                for square in &squares {
                    draw_texture_ex(
                        &enemy_texture,
                        square.x - enemy_frame.dest_size.x * 2.0,
                        square.y - enemy_frame.dest_size.y * 2.0,
                        GREEN,
                        // TODO: Fix enemy size
                        DrawTextureParams {
                            dest_size: Some(enemy_frame.dest_size / 1. + square.size),
                            source: Some(enemy_frame.source_rect),
                            ..Default::default()
                        },
                    );
                }
                for (explosion, coords) in explosions.iter_mut() {
                    explosion.draw(*coords);
                }
                draw_text(
                    format!("Score: {}", score).as_str(),
                    10.0,
                    35.0,
                    25.0,
                    WHITE,
                );
                let highscore_text = format!(
                    "High score: {}: {}",
                    score_board.best().name,
                    score_board.best().points
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
                    game_state = Mode::Playing;
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
                if is_key_pressed(KeyCode::Space) | is_key_pressed(KeyCode::N) {
                    game_state = Mode::MainMenu;
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
                if score > score_board.best().points {
                    game_state = Mode::Input
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
                    ui.input_text(hash!(), "Your Name", &mut input_text);
                    if ui.button(None, "Save") {
                        score_board.scores.push(Score {
                            name: input_text.clone(),
                            points: score,
                            timestamp: SystemTime::now(),
                        });
                        score_board.save();
                        game_state = MainMenu;
                    }
                });
            }
        }

        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

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
    fn load_score() -> Result<(), Box<dyn Error>> {
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

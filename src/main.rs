use std::{f32::consts::PI, process::exit, time::SystemTime};

use color_eyre::Result;
use macroquad::{
    audio::{PlaySoundParams, play_sound},
    prelude::*,
    ui::{hash, root_ui, widgets},
};
use macroquad_particles::{ColorCurve, Emitter, EmitterConfig};
mod assets;
mod score;
mod state;

use assets::Assets;
use state::{MOVEMENT_SPEED, Mode, Shape, State};

use crate::assets::{Enemy, EnemyKind};

pub const ENEMY_ASPECT: Vec2 = vec2(16.0, 24.0);

fn particle_explosion() -> EmitterConfig {
    EmitterConfig {
        local_coords: false,
        one_shot: true,
        emitting: true,
        lifetime: 0.6,
        lifetime_randomness: 0.3,
        explosiveness: 0.65,
        initial_direction_spread: 2.0 * PI,
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
                main_menu(&mut state);
            }
            Mode::Playing => {
                playing(&mut state, &assets);
            }
            Mode::Paused => {
                paused(&mut state);
            }
            Mode::GameOver => {
                game_over(&mut state);
            }
            Mode::Input => {
                input(&mut state);
            }
        }
        next_frame().await;
    }
}

fn input(state: &mut State) {
    widgets::Window::new(
        hash!(),
        vec2(screen_width() / 2., screen_height() / 2.),
        vec2(500., 300.),
    )
    .label("Input")
    .ui(&mut root_ui(), |ui| {
        ui.input_text(hash!(), "Your Name", &mut state.input_text);
        if ui.button(None, "Save") {
            state.score_board.scores.push(score::Score {
                name: state.input_text.clone(),
                points: state.score,
                timestamp: SystemTime::now(),
            });
            if let Err(e) = state.score_board.save() {
                eprintln!("Score save failed:\n{e}");
            }
            state.mode = Mode::MainMenu;
        }
    });
}

fn game_over(state: &mut State) {
    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::N) {
        state.mode = Mode::MainMenu;
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

fn playing(state: &mut State, assets: &Assets) {
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
            size: vec2(32., 32.),
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

    // Generate a new enemy
    if rand::gen_range(0, 99) >= 95 {
        let kind = EnemyKind::random();
        let (min, max) = kind.size_range();
        let scale = rand::gen_range(min, max) / ENEMY_ASPECT.y; // height-driven
        let size = ENEMY_ASPECT * scale;
        state.enemies.push(Enemy {
            kind,
            shape: Shape {
                size,
                speed: rand::gen_range(50.0, 150.0),
                x: rand::gen_range(size.x / 2.0, screen_width() - size.x / 2.),
                y: -size.y,
                collided: false,
            },
        });
    }

    // Square Movement
    for enemy in &mut state.enemies {
        enemy.shape.y += enemy.shape.speed * delta_time;
    }
    // laser Movement
    for laser in &mut state.lasers {
        laser.y -= laser.speed * delta_time;
    }

    state.ship_sprite.update();
    state.laser_sprite.update();
    state.small_enemy_sprite.update();
    state.medium_enemy_sprite.update();
    state.big_enemy_sprite.update();

    // Remove shapes outside of screen
    state
        .enemies
        .retain(|enemy| enemy.shape.y < screen_height() + enemy.shape.size.y);
    state
        .lasers
        .retain(|laser| laser.y > 0.0 - laser.size.y / 2.0);

    // Remove collided shapes
    state.enemies.retain(|enemy| !enemy.shape.collided);
    state.lasers.retain(|laser| !laser.collided);

    // Remove old explosions
    state
        .explosions
        .retain(|(explosion, _)| explosion.config.emitting);

    // Check for collisions
    if state
        .enemies
        .iter()
        .any(|enemy| state.player_ship.collides_with(&enemy.shape))
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
    for enemy in state.enemies.iter_mut() {
        for laser in state.lasers.iter_mut() {
            if laser.collides_with(&enemy.shape) {
                laser.collided = true;
                enemy.shape.collided = true;
                state.score += enemy.shape.size.y.round() as u32;
                // TODO: handle error
                state.explosions.push((
                    Emitter::new(EmitterConfig {
                        amount: enemy.shape.size.y.round() as u32 * 2,
                        ..particle_explosion()
                    }),
                    vec2(enemy.shape.x, enemy.shape.y),
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

    let laser_frame = state.laser_sprite.frame();
    for laser in &state.lasers {
        draw_texture_ex(
            &assets.textures.laser,
            laser.x - laser.size.x / 2.0,
            laser.y - laser.size.y / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(laser.size.x, laser.size.y)),
                source: Some(laser_frame.source_rect),
                ..Default::default()
            },
        );
    }
    let ship_frame = state.ship_sprite.frame();

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
        let dest = vec2(enemy.shape.size.x, enemy.shape.size.y);
        let enemy_frame = state.sprite(enemy.kind).frame();
        draw_texture_ex(
            assets.textures.enemy(enemy.kind),
            enemy.shape.x - dest.x / 2.0,
            enemy.shape.y - dest.y / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest),
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

fn paused(state: &mut State) {
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

fn main_menu(state: &mut State) {
    if is_key_pressed(KeyCode::Escape) || is_key_down(KeyCode::Q) {
        exit(0);
    }
    if is_key_pressed(KeyCode::Space) {
        state.enemies.clear();
        state.lasers.clear();
        state.explosions.clear();
        state.player_ship.x = screen_width() / 2.0;
        state.player_ship.y = screen_height() / 3.0 * 2.;
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

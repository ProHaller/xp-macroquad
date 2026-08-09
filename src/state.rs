use crate::score::HIGHSCORE_PATH;

use super::score;
use macroquad::{
    experimental::animation::{AnimatedSprite, Animation},
    math::{Rect, Vec2},
    window::{screen_height, screen_width},
};
use std::fmt::Debug;

use macroquad_particles::Emitter;

pub const MOVEMENT_SPEED: f32 = 600.0;

#[derive(Debug)]
pub struct Shape {
    pub size: f32,
    pub speed: f32,
    pub x: f32,
    pub y: f32,
    pub collided: bool,
}

impl Shape {
    pub fn collides_with(&self, other: &Self) -> bool {
        self.rect().overlaps(&other.rect())
    }

    pub fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            w: self.size,
            h: self.size,
        }
    }
}

#[derive(Debug)]
pub enum Mode {
    MainMenu,
    Playing,
    Paused,
    GameOver,
    Input,
}

#[allow(unused)]
pub struct State {
    pub player_ship: Shape,
    pub lasers: Vec<Shape>,
    pub enemies: Vec<Shape>,

    pub laser_sprite: AnimatedSprite,
    pub ship_sprite: AnimatedSprite,
    pub enemy_small_sprite: AnimatedSprite,
    pub medium_enemy_sprite: AnimatedSprite,
    pub large_enemy_sprite: AnimatedSprite,

    pub explosions: Vec<(Emitter, Vec2)>,
    pub direction_modifier: f32,
    pub input_text: String,
    pub score_board: score::ScoreBoard,
    pub score: u32,
    pub mode: Mode,
}

impl State {
    pub fn init() -> State {
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

        let score_board = score::ScoreBoard::load(HIGHSCORE_PATH).unwrap_or_default();
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

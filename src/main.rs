use macroquad::{prelude::*, rand::ChooseRandom};
use std::fs;
use std::process::exit;

const MOVEMENT_SPEED: f32 = 600.0;

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    collided: bool,
    color: Color,
}

pub struct GameState {
    squares: Vec<Shape>,
    circle: Shape,
    gameover: bool,
    bullets: Vec<Shape>,
    score: u32,
    high_score: u32,
}

impl GameState {
    pub fn new() -> GameState {
        let squares = vec![];
        let circle = Shape {
            size: 32.0,
            speed: MOVEMENT_SPEED,
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            collided: false,
            color: YELLOW,
        };
        let gameover = false;
        let bullets: Vec<Shape> = vec![];
        let score = 0;

        GameState {
            squares,
            circle,
            gameover,
            bullets,
            score,
            high_score: 0,
        }
    }

    pub fn reset(&mut self) {
        self.score = 0;
        self.squares.clear();
        self.bullets.clear();
        self.circle.x = screen_width() / 2.0;
        self.circle.y = screen_height() / 2.0;
        self.gameover = false;
    }
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

#[macroquad::main("xp-macroquad")]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let mut state = GameState::new();

    let colors = [RED, GREEN, BLUE, BEIGE, BLACK, BLANK];

    loop {
        clear_background(DARKPURPLE);

        let delta_time = get_frame_time();
        if !state.gameover {
            if is_key_down(KeyCode::Right) | is_key_down(KeyCode::I) {
                state.circle.x += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Left) | is_key_down(KeyCode::L) {
                state.circle.x -= MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Down) | is_key_down(KeyCode::R) {
                state.circle.y += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Up) | is_key_down(KeyCode::T) {
                state.circle.y -= MOVEMENT_SPEED * delta_time;
            }
        }

        // Clamp X and Y to be within the screen
        state.circle.x = clamp(state.circle.x, 0.0, screen_width());
        state.circle.y = clamp(state.circle.y, 0.0, screen_height());

        // Generate a new square
        if rand::gen_range(0, 99) >= 95 {
            let size = rand::gen_range(16.0, 64.0);
            state.squares.push(Shape {
                size,
                speed: rand::gen_range(50.0, 150.0),
                x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                y: -size,
                collided: false,
                color: colors.choose().unwrap().to_owned(),
            });
        }

        // Move squares
        for square in &mut state.squares {
            square.y += square.speed * delta_time;
        }

        for bullet in &mut state.bullets {
            bullet.y -= bullet.speed * delta_time;
        }
        for square in state.squares.iter_mut() {
            for bullet in state.bullets.iter_mut() {
                if bullet.collides_with(square) {
                    bullet.collided = true;
                    square.collided = true;

                    state.score += square.size.round() as u32;
                    state.high_score = state.high_score.max(state.score);
                }
            }
        }

        if state
            .squares
            .iter()
            .any(|square| state.circle.collides_with(square))
        {
            if state.score == state.high_score {
                fs::write("highscore.dat", state.high_score.to_string()).ok();
            }
            state.gameover = true;
        }
        state
            .bullets
            .retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);
        state.squares.retain(|square| !square.collided);
        state.bullets.retain(|bullet| !bullet.collided);

        let score_dimensions = measure_text(state.score.to_string(), None, 50, 1.0);
        draw_text(
            state.score.to_string(),
            screen_width() - 40.0 - score_dimensions.width / 4.0,
            screen_height() - 40.0,
            50.0,
            WHITE,
        );

        if state.gameover && is_key_pressed(KeyCode::Space) {
            state.reset();
        }

        if state.gameover {
            let text = "GAME OVER!";
            let text_dimensions = measure_text(text, None, 50, 1.0);
            draw_text(
                text,
                screen_width() / 2.0 - text_dimensions.width / 2.0,
                screen_height() / 2.0,
                50.0,
                RED,
            );
        }

        if is_key_down(KeyCode::Q) {
            exit(1)
        }
        if is_key_down(KeyCode::N) {
            state.gameover = false;
        }
        if is_key_pressed(KeyCode::Space) {
            state.bullets.push(Shape {
                x: state.circle.x,
                y: state.circle.y,
                speed: state.circle.speed * 2.0,
                size: 5.0,
                collided: false,
                color: BLACK,
            });
        }

        // Remove squares below bottom of screen
        state
            .squares
            .retain(|square| square.y < screen_height() + square.size);

        // Draw everything
        draw_circle(
            state.circle.x,
            state.circle.y,
            state.circle.size / 2.0,
            YELLOW,
        );
        for bullet in &state.bullets {
            draw_circle(bullet.x, bullet.y, bullet.size / 2.0, RED);
        }

        for square in &state.squares {
            draw_rectangle(
                square.x - square.size / 2.0,
                square.y - square.size / 2.0,
                square.size,
                square.size,
                square.color,
            );
        }

        next_frame().await
    }
}

use std::{fs, process::exit};

use macroquad::{
    prelude::*,
    rand::{ChooseRandom, rand},
};

const MOVEMENT_SPEED: f32 = 600.0;

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    collided: bool,
    color: Color,
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
    let mut squares = vec![];
    let mut circle = Shape {
        size: 32.0,
        speed: MOVEMENT_SPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        collided: false,
        color: YELLOW,
    };
    let mut gameover = false;
    let mut bullets: Vec<Shape> = vec![];
    let mut score: u32 = 0;
    let mut high_score: u32 = fs::read_to_string("highscore.dat")
        .map_or(Ok(0), |i| i.parse::<u32>())
        .unwrap_or(0);
    let colors = [RED, GREEN, BLUE, BEIGE, BLACK, BLANK];

    loop {
        clear_background(DARKPURPLE);

        let delta_time = get_frame_time();
        if !gameover {
            if is_key_down(KeyCode::Right) | is_key_down(KeyCode::I) {
                circle.x += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Left) | is_key_down(KeyCode::L) {
                circle.x -= MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Down) | is_key_down(KeyCode::R) {
                circle.y += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Up) | is_key_down(KeyCode::T) {
                circle.y -= MOVEMENT_SPEED * delta_time;
            }
        }

        // Clamp X and Y to be within the screen
        circle.x = clamp(circle.x, 0.0, screen_width());
        circle.y = clamp(circle.y, 0.0, screen_height());

        // Generate a new square
        if rand::gen_range(0, 99) >= 95 {
            let size = rand::gen_range(16.0, 64.0);
            squares.push(Shape {
                size,
                speed: rand::gen_range(50.0, 150.0),
                x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                y: -size,
                collided: false,
                color: colors.choose().unwrap().to_owned(),
            });
        }

        // Move squares
        for square in &mut squares {
            square.y += square.speed * delta_time;
        }

        for bullet in &mut bullets {
            bullet.y -= bullet.speed * delta_time;
        }
        for square in squares.iter_mut() {
            for bullet in bullets.iter_mut() {
                if bullet.collides_with(square) {
                    bullet.collided = true;
                    square.collided = true;

                    score += square.size.round() as u32;
                    high_score = high_score.max(score);
                }
            }
        }

        if squares.iter().any(|square| circle.collides_with(square)) {
            if score == high_score {
                fs::write("highscore.dat", high_score.to_string()).ok();
            }
            gameover = true;
        }
        bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);
        squares.retain(|square| !square.collided);
        bullets.retain(|bullet| !bullet.collided);

        draw_text(
            format!("Score: {}", score).as_str(),
            10.0,
            35.0,
            25.0,
            WHITE,
        );
        let highscore_text = format!("High score: {}", high_score);
        let text_dimensions = measure_text(highscore_text.as_str(), None, 25, 1.0);
        draw_text(
            highscore_text.as_str(),
            screen_width() - text_dimensions.width - 10.0,
            35.0,
            25.0,
            WHITE,
        );

        if gameover && is_key_pressed(KeyCode::Space) {
            squares.clear();
            bullets.clear();
            score = 0;
            circle.x = screen_width() / 2.0;
            circle.y = screen_height() / 2.0;
            gameover = false;
        }

        if gameover {
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
            gameover = false;
        }
        if is_key_pressed(KeyCode::Space) {
            bullets.push(Shape {
                x: circle.x,
                y: circle.y,
                speed: circle.speed * 2.0,
                size: 5.0,
                collided: false,
                color: BLACK,
            });
        }

        // Remove squares below bottom of screen
        squares.retain(|square| square.y < screen_height() + square.size);

        // Draw everything
        draw_circle(circle.x, circle.y, circle.size / 2.0, YELLOW);
        for bullet in &bullets {
            draw_circle(bullet.x, bullet.y, bullet.size / 2.0, RED);
        }

        for square in &squares {
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

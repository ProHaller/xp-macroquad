use macroquad::prelude::*;

#[macroquad::main("xp-macroquad")]
async fn main() {
    loop {
        clear_background(DARKPURPLE);
        next_frame().await
    }
}

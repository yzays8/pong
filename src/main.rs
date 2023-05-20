mod game;

use std::process;

fn main() {
    let mut game = game::Game::build().unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });
    game.run();
}

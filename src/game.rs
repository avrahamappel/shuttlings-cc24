use std::fmt::{self, Display, Formatter};
use std::sync::RwLock;

use actix_web::web::Data;
use actix_web::{get, post, Scope};

enum Tile {
    Empty,
    Cookie,
    Milk,
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Empty => write!(f, "⬛"),
            Self::Cookie => write!(f, "🍪"),
            Self::Milk => write!(f, "🥛"),
        }
    }
}

enum Piece {
    Tile(Tile),
    Wall,
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Tile(tile) => write!(f, "{tile}"),
            Self::Wall => write!(f, "⬜"),
        }
    }
}

struct Game {
    board: Vec<Vec<Piece>>,
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        for line in &self.board {
            for piece in line {
                write!(f, "{piece}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn new_board() -> Vec<Vec<Piece>> {
    use self::Tile::Empty;
    use Piece::{Tile, Wall};

    vec![
        vec![
            Wall,
            Tile(Empty),
            Tile(Empty),
            Tile(Empty),
            Tile(Empty),
            Wall,
        ],
        vec![
            Wall,
            Tile(Empty),
            Tile(Empty),
            Tile(Empty),
            Tile(Empty),
            Wall,
        ],
        vec![
            Wall,
            Tile(Empty),
            Tile(Empty),
            Tile(Empty),
            Tile(Empty),
            Wall,
        ],
        vec![
            Wall,
            Tile(Empty),
            Tile(Empty),
            Tile(Empty),
            Tile(Empty),
            Wall,
        ],
        vec![Wall, Wall, Wall, Wall, Wall, Wall],
    ]
}

impl Game {
    fn new() -> Self {
        let board = new_board();
        Self { board }
    }

    fn reset(&mut self) {
        self.board = new_board();
    }
}

#[get("/board")]
async fn show_board(game: Data<RwLock<Game>>) -> String {
    game.read().unwrap().to_string()
}

#[post("/reset")]
async fn reset_board(game: Data<RwLock<Game>>) -> String {
    let mut game = game.write().unwrap();
    game.reset();
    game.to_string()
}

pub fn scope() -> Scope {
    let game = Data::new(RwLock::new(Game::new())).clone();
    Scope::new("/12")
        .app_data(game)
        .service(show_board)
        .service(reset_board)
}

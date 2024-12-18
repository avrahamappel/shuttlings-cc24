#![allow(clippy::module_name_repetitions)]

use std::fmt::{self, Display, Formatter};

use actix_web::web::{Data, Path};
use actix_web::{get, post, Either, HttpResponse, Scope};
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(untagged, rename_all = "kebab-case")]
enum Piece {
    Cookie,
    Milk,
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Cookie => write!(f, "🍪"),
            Self::Milk => write!(f, "🥛"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Game {
    board: [[Option<Piece>; 4]; 4],
}

enum GameState {
    Winner(Piece),
    Draw,
    NotEnded,
}

impl Game {
    fn new() -> Self {
        let board = Default::default();
        Self { board }
    }

    fn place(&mut self, piece: Piece, column: usize) -> bool {
        if !matches!(self.get_state(), GameState::NotEnded) {
            return false;
        }

        if let Some(last) = self.board[column]
            .iter_mut()
            .take_while(|o| o.is_none())
            .last()
        {
            *last = Some(piece);
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.board = Default::default();
    }

    /// Get game state
    fn get_state(&self) -> GameState {
        // Check columns
        for i in 0..4 {
            let mut winner = self.board[i][0];
            for j in 1..4 {
                if self.board[i][j] != winner {
                    winner = None;
                }
            }
            if let Some(w) = winner {
                return GameState::Winner(w);
            }
        }

        // Check rows
        for i in 0..4 {
            let mut winner = self.board[0][i];
            for j in 1..4 {
                if self.board[j][i] != winner {
                    winner = None;
                }
            }
            if let Some(w) = winner {
                return GameState::Winner(w);
            }
        }

        // Check top-left to bottom-right diagonal
        {
            let mut winner = self.board[0][0];
            for i in 1..4 {
                if self.board[i][i] != winner {
                    winner = None;
                }
            }
            if let Some(w) = winner {
                return GameState::Winner(w);
            }
        }

        // Check top-right to bottom-left
        {
            let mut winner = self.board[3][0];
            for (i, j) in [(2, 1), (1, 2), (0, 3)] {
                if self.board[i][j] != winner {
                    winner = None;
                }
            }
            if let Some(w) = winner {
                return GameState::Winner(w);
            }
        }

        // If board is full, it's a draw, otherwise ongoing
        if self.board.iter().any(|col| col.iter().any(Option::is_none)) {
            GameState::NotEnded
        } else {
            GameState::Draw
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        const WALL: char = '⬜';
        const EMPTY: char = '⬛';

        // Main board
        for i in 0..4 {
            write!(f, "{WALL}")?;
            for j in 0..4 {
                if let Some(piece) = self.board[j][i] {
                    write!(f, "{piece}")?;
                } else {
                    write!(f, "{EMPTY}")?;
                }
            }
            write!(f, "{WALL}")?;
            writeln!(f)?;
        }

        // Bottom
        for _ in 0..6 {
            write!(f, "{WALL}")?;
        }
        writeln!(f)?;

        // Winner state
        match self.get_state() {
            GameState::Winner(w) => {
                writeln!(f, "{w} wins!")?;
            }
            GameState::Draw => {
                writeln!(f, "No winner.")?;
            }
            GameState::NotEnded => (),
        }

        Ok(())
    }
}

pub type SharedGame = Data<RwLock<Game>>;

pub fn new_shared_game() -> SharedGame {
    Data::new(RwLock::new(Game::new()))
}

#[get("/board")]
async fn show_board(game: SharedGame) -> String {
    game.read().await.to_string()
}

#[post("/reset")]
async fn reset_board(game: SharedGame) -> String {
    let mut game = game.write().await;
    game.reset();
    game.to_string()
}

#[derive(Debug, Deserialize)]
struct PlaceParams {
    team: String,
    column: String,
}

#[post("/place/{team}/{column}")]
async fn place_piece(params: Path<PlaceParams>, game: SharedGame) -> Either<String, HttpResponse> {
    let piece_o = match params.team.as_str() {
        "milk" => Some(Piece::Milk),
        "cookie" => Some(Piece::Cookie),
        _ => None,
    };

    if let Some(piece) = piece_o {
        if let Ok(column) = params.column.parse::<usize>() {
            if (1..=4).contains(&column) {
                let mut game = game.write().await;
                eprintln!("GAME STATE");
                eprintln!("{game}");
                eprintln!("Trying to add {piece} to column {column}");

                if game.place(piece, column - 1) {
                    return Either::Left(game.to_string());
                }

                return Either::Right(HttpResponse::ServiceUnavailable().body(game.to_string()));
            }
        }
    }

    Either::Right(HttpResponse::BadRequest().finish())
}

pub fn scope() -> Scope {
    Scope::new("/12")
        .service(show_board)
        .service(reset_board)
        .service(place_piece)
}

#[cfg(test)]
mod tests {
    use super::Piece::*;
    use super::*;

    #[test]
    fn game_displays_correctly() {
        for (game, expected) in [
            (
                Game {
                    board: [[Some(Cookie); 4], [None; 4], [None; 4], [None; 4]],
                },
                "\
⬜🍪⬛⬛⬛⬜
⬜🍪⬛⬛⬛⬜
⬜🍪⬛⬛⬛⬜
⬜🍪⬛⬛⬛⬜
⬜⬜⬜⬜⬜⬜
🍪 wins!
",
            ),
            (
                Game {
                    board: [
                        [Some(Milk), Some(Cookie), Some(Cookie), Some(Cookie)],
                        [Some(Cookie), Some(Milk), Some(Milk), Some(Milk)],
                        [Some(Milk), Some(Cookie), Some(Cookie), Some(Cookie)],
                        [Some(Cookie), Some(Milk), Some(Milk), Some(Milk)],
                    ],
                },
                "\
⬜🥛🍪🥛🍪⬜
⬜🍪🥛🍪🥛⬜
⬜🍪🥛🍪🥛⬜
⬜🍪🥛🍪🥛⬜
⬜⬜⬜⬜⬜⬜
No winner.
",
            ),
            (
                Game {
                    board: [
                        [None, None, None, Some(Cookie)],
                        [None, None, Some(Cookie), Some(Milk)],
                        [None, Some(Cookie), Some(Milk), Some(Milk)],
                        [Some(Cookie), Some(Milk), Some(Milk), Some(Milk)],
                    ],
                },
                "\
⬜⬛⬛⬛🍪⬜
⬜⬛⬛🍪🥛⬜
⬜⬛🍪🥛🥛⬜
⬜🍪🥛🥛🥛⬜
⬜⬜⬜⬜⬜⬜
🍪 wins!
",
            ),
        ] {
            assert_eq!(expected, game.to_string());
        }
    }

    #[test]
    fn can_add_pieces_to_game() {
        let mut game = Game::new();
        game.place(Milk, 1);
        game.place(Cookie, 1);
        game.place(Milk, 2);
        game.place(Cookie, 3);
        game.place(Milk, 2);

        assert_eq!(
            Game {
                board: [
                    [None, None, Some(Cookie), Some(Milk)],
                    [None, None, Some(Milk), Some(Milk)],
                    [None, None, None, Some(Cookie)],
                    [None, None, None, None],
                ]
            },
            game
        );
    }
}

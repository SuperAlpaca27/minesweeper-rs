use std::collections::HashSet;
use std::fmt::Display;

use crossterm::style::Stylize;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
struct Position {
    x: isize,
    y: isize,
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl std::ops::Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug)]
pub struct Minefield<const W: usize, const H: usize> {
    hidden_field: [[Tile; W]; H],
    shown_field: [[TileState; W]; H],
}

impl<const W: usize, const H: usize> Minefield<W, H> {
    pub fn new() -> Self {
        Self {
            hidden_field: [[Default::default(); W]; H],
            shown_field: [[Default::default(); W]; H],
        }
    }

    pub fn width(&self) -> usize {
        W
    }

    pub fn height(&self) -> usize {
        H
    }

    pub fn display_field(&self) {
        print!("   ");
        for x in 0..self.width() {
            print!("{} ", x);
        }
        println!();

        for y in 0..self.height() {
            print!("{: >2} ", y);
            for x in 0..self.width() {
                if self.shown_field[y][x] == TileState::Known {
                    print!("{} ", self.hidden_field[y][x])
                } else {
                    print!("{} ", self.shown_field[y][x])
                }
            }
            println!();
        }
    }

    pub fn display_hidden_field(&self) {
        print!("  ");
        for x in 0..self.width() {
            print!("{} ", x);
        }
        println!();
        for y in 0..self.height() {
            print!("{} ", y);
            for x in 0..self.width() {
                print!("{} ", self.hidden_field[y][x])
            }
            println!();
        }
    }

    pub fn generate_mines(&mut self, num_mines: usize) {
        assert!(num_mines <= self.width() * self.height());
        let mut rng = rand::thread_rng();
        let mine_ind: Vec<usize> =
            rand::seq::index::sample(&mut rng, self.width() * self.height(), num_mines).into_vec();

        for i in mine_ind {
            let y = i / self.width();
            let x = i - (y * self.width());
            let pos = Position {
                x: x.try_into().unwrap(),
                y: y.try_into().unwrap(),
            };

            self.hidden_field[y][x] = Tile::Mine;
            //println!("{}: {}, {}", i, row, col);

            let dirs = [
                Position { x: 0, y: 1 },   // down
                Position { x: 0, y: -1 },  // up
                Position { x: -1, y: 0 },  // left
                Position { x: 1, y: 0 },   // right
                Position { x: 1, y: 1 },   // bottom right
                Position { x: -1, y: 1 },  // bottom left
                Position { x: -1, y: -1 }, // top left
                Position { x: 1, y: -1 },  // top right
            ];

            // TODO: Work on checking adjacent and incrementing
            for dir in dirs {
                let adj = pos + dir;
                if let Some(row) = self.hidden_field.get_mut(adj.y as usize) {
                    if let Some(tile) = row.get_mut(adj.x as usize) {
                        let temp_tile = match tile {
                            Tile::Number(n) => Some(Tile::Number(*n + 1)),
                            Tile::Empty => Some(Tile::Number(1)),
                            _ => None,
                        };

                        if let Some(t) = temp_tile {
                            *tile = t;
                        }
                    }
                }
            }
        }
    }

    pub fn sweep_at(&mut self, x: usize, y: usize) -> Option<&Tile> {
        // Can't sweep if flagged
        if self.shown_field[y][x] == TileState::Unknown(true) {
            return None
        }

        self.shown_field[y][x] = TileState::Known;
        let start_tile = &self.hidden_field[y][x];
        match start_tile {
            Tile::Number(_) => {
                self.shown_field[y][x] = TileState::Known;
                return Some(start_tile);
            }
            Tile::Mine => {
                //TODO: Handle bomb
                self.shown_field[y][x] = TileState::Known;
                return Some(start_tile);
            }
            Tile::Empty => (),
        };

        let mut explored: HashSet<Position> = HashSet::new();
        let mut stack: Vec<Position> = Vec::new();

        let pos = Position {
            x: x.try_into().unwrap(),
            y: y.try_into().unwrap(),
        };
        explored.insert(pos);
        stack.push(pos);

        while let Some(pos) = stack.pop() {
            let x = pos.x as usize;
            let y = pos.y as usize;

            match self.hidden_field[y][x] {
                Tile::Empty => {
                    self.shown_field[y][x] = TileState::Known;
                }
                _ => (),
            }

            let dirs = [
                Position { x: 0, y: 1 },
                Position { x: 0, y: -1 },
                Position { x: -1, y: 0 },
                Position { x: 1, y: 0 },
            ];

            for dir in dirs {
                let p = pos + dir;
                if p.x >= 0
                    && p.x < self.width().try_into().unwrap()
                    && p.y >= 0
                    && p.y < self.height().try_into().unwrap()
                {
                    if !explored.contains(&p) {
                        let x = p.x as usize;
                        let y = p.y as usize;
                        match self.hidden_field[y][x] {
                            Tile::Empty => {
                                stack.push(p);
                                explored.insert(p);
                            }
                            Tile::Number(_) => {
                                explored.insert(p);
                                self.shown_field[y][x] = TileState::Known;
                            }
                            _ => (),
                        }
                    }
                }
            }
        }

        return Some(start_tile);
    }

    pub fn check_win(&self) -> bool {
        for y in 0..self.height() {
            for x in 0..self.width() {
                let shown = &self.shown_field[y][x];
                match *shown {
                    TileState::Unknown(_) => {
                        let hidden = &self.hidden_field[y][x];
                        if *hidden != Tile::Mine {
                            return false;
                        }
                    }
                    _ => (),
                }
            }
        }
        true
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        match self.shown_field[y][x] {
            TileState::Unknown(flag) => {
                self.shown_field[y][x] = TileState::Unknown(!flag);
            }
            _ => (),
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileState {
    Unknown(bool),
    Known,
}

impl Default for TileState {
    fn default() -> Self {
        TileState::Unknown(false)
    }
}

impl Display for TileState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileState::Unknown(false) => write!(f, "{}", "U".grey()),
            TileState::Unknown(true) => write!(f, "{}", "F".green()),
            TileState::Known => write!(f, "{}", "K".yellow()),
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Empty,
    Number(u8),
    Mine,
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            //Tile::Unknown(false) => write!(f, "{}", "U".grey()),
            //Tile::Unknown(true) => write!(f, "{}", "F".green()),
            Tile::Empty => write!(f, "{}", "E".blue()),
            Tile::Number(n) => write!(f, "{}", n.to_string().dark_blue()),
            Tile::Mine => write!(f, "{}", "X".red()),
        }
    }
}

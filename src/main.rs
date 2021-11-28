use crossterm::style::Stylize;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Display;
use std::io;
use std::io::*;

#[derive(Debug, Clone)]
struct Position(usize, usize);

impl Default for Position {
    fn default() -> Self {
        Self(0, 0)
    }
}

#[derive(Debug)]
struct Minefield {
    width: usize,
    height: usize,
    hidden_field: Vec<Tile>,
    shown_field: Vec<TileState>,
}

impl Minefield {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            hidden_field: vec![Default::default(); width * height],
            shown_field: vec![Default::default(); width * height],
        }
    }

    pub fn display_field(&self) {
        print!("   ");
        for x in 0..self.width {
            print!("{} ", x);
        }
        println!();

        for y in 0..self.height {
            print!("{: >2} ", y);
            for x in 0..self.width {
                if self.shown_field[y * self.width + x] == TileState::Known {
                    print!("{} ", self.hidden_field[y * self.width + x])
                } else {
                    print!("{} ", self.shown_field[y * self.width + x])
                }
            }
            println!();
        }
    }

    pub fn display_hidden_field(&self) {
        print!("  ");
        for x in 0..self.width {
            print!("{} ", x);
        }
        println!();
        for y in 0..self.height {
            print!("{} ", y);
            for x in 0..self.width {
                print!("{} ", self.hidden_field[y * self.width + x])
            }
            println!();
        }
    }

    pub fn generate_mines(&mut self, num_mines: usize) {
        assert!(num_mines <= self.width * self.height);
        let mut rng = rand::thread_rng();
        let mine_ind: Vec<usize> =
            rand::seq::index::sample(&mut rng, self.width * self.height, num_mines).into_vec();

        for i in mine_ind {
            self.hidden_field[i] = Tile::Mine;

            let row = i / self.width;
            let col = i - (row * self.width);
            //println!("{}: {}, {}", i, row, col);

            let mut num_indices: Vec<usize> = Vec::new();
            // left and right
            if col != 0 {
                num_indices.push(i - 1);
            }
            if col < self.width - 1 {
                num_indices.push(i + 1);
            }

            // top
            if row != 0 {
                num_indices.push((row - 1) * self.width + col);
                // left diag
                if col != 0 {
                    num_indices.push((row - 1) * self.width + col - 1);
                }
                // right diag
                if col < self.width - 1 {
                    num_indices.push((row - 1) * self.width + col + 1);
                }
            }

            // bottom
            if row + 1 < self.height {
                num_indices.push((row + 1) * self.width + col);

                // left diag
                if col != 0 {
                    num_indices.push((row + 1) * self.width + col - 1);
                }

                // right diag
                if col < self.width - 1 {
                    num_indices.push((row + 1) * self.width + col + 1);
                }
            }

            for ind in num_indices {
                if let Some(tile) = self.hidden_field.get_mut(ind) {
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

    pub fn sweep_at(&mut self, row: usize, col: usize) -> &Tile {
        let ind = row * self.width + col;
        self.shown_field[ind] = TileState::Known;
        let start_tile = &self.hidden_field[ind];
        match start_tile {
            Tile::Number(_) => {
                self.shown_field[ind] = TileState::Known;
                return start_tile;
            }
            Tile::Mine => {
                //TODO: Handle bomb
                self.shown_field[ind] = TileState::Known;
                return start_tile;
            }
            Tile::Empty => (),
        };

        let mut explored: HashSet<(usize, usize)> = HashSet::new();
        let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

        explored.insert((row, col));
        queue.push_back((row, col));

        while let Some(v) = queue.pop_front() {
            let row = v.0;
            let col = v.1;

            let ind = row * self.width + col;

            match self.hidden_field[ind] {
                Tile::Empty => {
                    self.shown_field[ind] = TileState::Known;
                }
                _ => (),
            }

            let mut temp = |ind, w| {
                if !explored.contains(&w) {
                    match self.hidden_field[ind] {
                        Tile::Empty => {
                            queue.push_back(w);
                            explored.insert(w);
                        }
                        Tile::Number(_) => {
                            explored.insert((row, col));
                            self.shown_field[ind] = TileState::Known;
                        }
                        _ => (),
                    }
                }
            };

            // Top
            if row != 0 {
                let w = (row - 1, col);
                let ind = w.0 * self.width + w.1;
                temp(ind, w);
            }
            // Bottom
            if row + 1 < self.height {
                let w = (row + 1, col);
                let ind = w.0 * self.width + w.1;
                temp(ind, w);
            }
            // Left
            if col != 0 {
                let w = (row, col - 1);
                let ind = w.0 * self.width + w.1;
                temp(ind, w);
            }

            // Right
            if col < self.width - 1 {
                let w = (row, col + 1);
                let ind = w.0 * self.width + w.1;
                temp(ind, w);
            }
        }

        return start_tile;
    }

    pub fn check_win(&self) -> bool {
        for y in 0..self.height {
            for x in 0..self.width {
                let shown = &self.shown_field[y * self.width + x];
                match *shown {
                    TileState::Unknown(_) => {
                        let hidden = &self.hidden_field[y * self.width + x];
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

    pub fn toggle_flag(&mut self, row: usize, col: usize) {
        let ind = row * self.width + col;
        match self.shown_field[ind] {
            TileState::Unknown(flag) => {
                self.shown_field[ind] = TileState::Unknown(!flag);
            }
            _ => (),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TileState {
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

#[derive(Debug, Clone, PartialEq)]
enum Tile {
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

fn main() {
    const WIDTH: usize = 10;
    const HEIGHT: usize = 10;
    const MINE_PCT: f64 = 0.1;
    let mut game = Minefield::new(WIDTH, HEIGHT);
    //println!("{}, {}", game.width, game.height);

    game.generate_mines((MINE_PCT * WIDTH as f64 * HEIGHT as f64) as usize);
    while !game.check_win() {
        let mut input = String::new();
        print!("Enter row: ");
        io::stdout().flush();
        io::stdin().read_line(&mut input);
        let row: usize = input.trim().parse().unwrap();
        input = String::new();
        print!("Enter col: ");
        io::stdout().flush();
        io::stdin().read_line(&mut input);
        let col: usize = input.trim().parse().unwrap();
        input = String::new();
        print!("Flag?: ");
        io::stdout().flush();
        io::stdin().read_line(&mut input);
        let flag = input.trim();

        print!("\x1B[2J\x1B[1;1H");
        println!("{}, {}", game.width, game.height);

        if flag == "F" {
            game.toggle_flag(row, col);
        } else {
            match game.sweep_at(row, col) {
                Tile::Mine => {
                    println!("Hit mine!");
                    break;
                }
                _ => (),
            };
        }

        game.display_field();
        println!("{:=<1$}", "", WIDTH * 2 - 1);
        //game.display_hidden_field();
    }
    println!("GAME OVER!")
}

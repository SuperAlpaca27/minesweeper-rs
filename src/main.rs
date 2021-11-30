use std::io;
use std::io::*;

mod minesweeper;
use crate::minesweeper::Minefield;
use crate::minesweeper::Tile;

fn main() {
    const WIDTH: usize = 10;
    const HEIGHT: usize = 10;
    const MINE_PCT: f64 = 0.15;
    let mut game: Minefield<WIDTH, HEIGHT> = Minefield::new();
    //println!("{}, {}", game.width, game.height);

    game.generate_mines((MINE_PCT * WIDTH as f64 * HEIGHT as f64) as usize);
    print!("\x1B[2J\x1B[1;1H");
    println!("{}, {}", game.width(), game.height());
    game.display_field();
    println!("{:=<1$}", "", WIDTH * 2 - 1);
    game.display_hidden_field();

    while !game.check_win() {
        let mut input = String::new();
        print!("Enter x: ");
        io::stdout().flush();
        io::stdin().read_line(&mut input);
        let x: usize = input.trim().parse().unwrap();
        input = String::new();
        print!("Enter y: ");
        io::stdout().flush();
        io::stdin().read_line(&mut input);
        let y: usize = input.trim().parse().unwrap();
        input = String::new();
        print!("Flag?: ");
        io::stdout().flush();
        io::stdin().read_line(&mut input);
        let flag = input.trim();

        print!("\x1B[2J\x1B[1;1H");
        println!("{}, {}", game.width(), game.height());

        if flag == "F" {
            game.toggle_flag(x, y);
        } else {
            if let Some(tile) = game.sweep_at(x, y) {
                match tile {
                    Tile::Mine => {
                        println!("Hit mine!");
                        break;
                    }
                    _ => (),
                };
            }
        }

        game.display_field();
        println!("{:=<1$}", "", WIDTH * 2 - 1);
        game.display_hidden_field();
    }
    println!("GAME OVER!")
}

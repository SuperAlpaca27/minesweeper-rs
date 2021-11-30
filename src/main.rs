use std::io;
use std::io::*;

mod minesweeper;
use crate::minesweeper::Minefield;
use crate::minesweeper::Tile;
use std::str::FromStr;

fn get_user_input<T: FromStr + Default>(message: &str) -> T {
    loop {
        let mut input = String::new();
        print!("{}", message);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        if let Ok(x) = input.trim().parse::<T>() {
            return x;
        } else {
            println!("Invalid value entered! Try again!");
        }
    }
}

fn main() {
    const WIDTH: usize = 10;
    const HEIGHT: usize = 10;
    const MINE_PCT: f64 = 0.2;
    let mut game = Minefield::new(WIDTH, HEIGHT);

    game.generate_mines((MINE_PCT * (WIDTH * HEIGHT) as f64) as usize);
    print!("\x1B[2J\x1B[1;1H");
    println!("{}, {}", game.width(), game.height());
    game.display_field();
    println!("{:=<1$}", "", WIDTH * 2 - 1);
    game.display_hidden_field();

    while !game.check_win() {
        let x: usize = get_user_input("Enter x: ");
        let y: usize = get_user_input("Enter y: ");
        let flag: String = get_user_input("Use flag?: ");

        print!("\x1B[2J\x1B[1;1H");
        println!("{}, {}", game.width(), game.height());

        if flag == "F" {
            game.toggle_flag(x, y);
        } else if let Some(tile) = game.sweep_at(x, y) {
            if tile == &Tile::Mine {
                println!("Hit mine!");
                game.display_field();
                println!("{:=<1$}", "", WIDTH * 2 - 1);
                game.display_hidden_field();
                break;
            };
        }

        game.display_field();
        println!("{:=<1$}", "", WIDTH * 2 - 1);
        game.display_hidden_field();
    }
    println!("GAME OVER!")
}

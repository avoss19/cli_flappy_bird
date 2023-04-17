/*
 *
 * CLI Flappy Bird
 *
 */

use device_query::{DeviceQuery, DeviceState, Keycode};
use rand::Rng;
use std::fs::{create_dir, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::{env, fs, thread, time};
use terminal_size::{terminal_size, Height, Width};

struct Res {
    x: u16,
    y: u16,
}

struct GameState {
    alive: bool,

    x: u16,
    y: f32,

    score: u32,

    clock: u32,

    obstacle_height: u8,
    obstacle_width: u8,
    obstacle_spacing: u8,
    obstacle_inner: u8,
}

fn main() {
    get_flags();
    let game_state = game_loop();
    game_over(game_state);
}

fn start_screen() {
    let c = get_keyboard_input();

    let options = vec!["Start", "Exit"];
    let mut selected: u8 = 0;

    loop {
        let c = get_keyboard_input();

        match c {
            'w' => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            's' => {
                if options.len() - 1 > selected as usize {
                    selected += 1;
                }
            }
            ' ' => {
                game_select_mode(selected);
                break;
            }
            _ => {}
        }

        for (i, option) in options.iter().enumerate() {
            if i == selected as usize {
                println!("> {}", option);
            } else {
                println!("  {}", option);
            }
        }

        screen_clear();

        thread::sleep(time::Duration::from_millis(10));
    }
}

fn game_select_mode(select: u8) {
    match select {
        0 => {
            let game_state = game_loop();
            game_over(game_state);
        }
        1 => {
            screen_clear();
            println!("Exiting...");
            std::process::exit(0);
        }
        _ => {}
    }
}

fn game_loop() -> GameState {
    let res = term_size();
    let res = Res {
        x: res.x,
        y: res.y - 2,
    };

    let mut game_state = GameState {
        alive: true,
        x: 20,
        y: 20.0,
        score: 0,
        clock: 0,
        obstacle_height: 0,
        obstacle_width: 5,
        obstacle_spacing: 50,
        obstacle_inner: 10,
    };

    let sleep_time = time::Duration::from_millis(10);

    let mut screen_grid = vec![vec![' '; res.x as usize]; res.y as usize];

    while game_state.alive {
        game_state = update_player(game_state, &res);
        game_state = update_state(game_state);
        game_state = collision_detection(&screen_grid, game_state);

        screen_grid = update_grid(screen_grid);
        (screen_grid, game_state) = update_obstacles(screen_grid, game_state);
        screen_grid = update_character(screen_grid, &game_state);

        draw_grid(&game_state, &screen_grid);

        screen_grid = remove_character(screen_grid, &game_state);

        thread::sleep(sleep_time);
    }

    return game_state;
}

fn get_flags() -> (bool, bool) {
    let args: Vec<String> = env::args().collect();

    let mut debug = false;
    let mut no_clear = false;

    for arg in args {
        if arg == "--spacing" {
            let spacing = args[args.iter().position(|x| x == &arg).unwrap() + 1]
                .parse::<u8>()
                .unwrap();
        }
        if arg == "--highscore" {
            let highscore = save_highscore(0);
            println!("Highscore: {}", highscore);
            std::process::exit(0);
        }
    }

    return (debug, no_clear);
}

fn update_obstacles(
    mut screen_grid: Vec<Vec<char>>,
    mut game_state: GameState,
) -> (Vec<Vec<char>>, GameState) {
    if game_state.clock % (game_state.obstacle_width + game_state.obstacle_spacing) as u32 == 0 {
        if game_state.obstacle_height != 0 {
            game_state.obstacle_height = 0;
        } else {
            let max_height: u8 = screen_grid.len() as u8 / 2;
            game_state.obstacle_height = rand::thread_rng().gen_range(1..=max_height);
        }
    }

    let x_res = screen_grid[0].len() - 1;
    let w: u32 = (game_state.obstacle_width + game_state.obstacle_spacing) as u32;

    if game_state.clock % w as u32 == 0 {
        game_state.score += 1;
    }

    if game_state.clock % w <= game_state.obstacle_spacing as u32 {
        for y in 0..screen_grid.len() {
            screen_grid[y as usize][x_res] = ' ';
        }
        return (screen_grid, game_state);
    }

    for h in 0..game_state.obstacle_height {
        screen_grid[h as usize][x_res] = 'X';
    }
    for h in game_state.obstacle_height..game_state.obstacle_height + game_state.obstacle_inner {
        screen_grid[h as usize][x_res] = ' ';
    }
    for h in game_state.obstacle_height + game_state.obstacle_inner..screen_grid.len() as u8 {
        screen_grid[h as usize][x_res] = 'X';
    }

    return (screen_grid, game_state);
}

fn remove_character(mut screen_grid: Vec<Vec<char>>, game_state: &GameState) -> Vec<Vec<char>> {
    screen_grid[game_state.y as usize][game_state.x as usize] = ' ';

    return screen_grid;
}

fn update_character(mut screen_grid: Vec<Vec<char>>, game_state: &GameState) -> Vec<Vec<char>> {
    screen_grid[game_state.y as usize][game_state.x as usize] = 'X';

    return screen_grid;
}

fn get_keyboard_input() -> char {
    let device_state = DeviceState::new();
    let keys = device_state.get_keys();

    let mut c = '_';

    for key in keys {
        match key {
            Keycode::W => c = 'w',
            Keycode::A => c = 'a',
            Keycode::S => c = 's',
            Keycode::D => c = 'd',
            Keycode::Space => c = ' ',
            _ => {}
        }
    }

    return c;
}

fn screen_clear() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn update_state(mut game_state: GameState) -> GameState {
    game_state.clock += 1;

    return game_state;
}

fn update_player(mut game_state: GameState, res: &Res) -> GameState {
    let c = get_keyboard_input();
    if c == ' ' {
        game_state.y -= 0.9;
    } else {
        game_state.y += 0.5;
    }

    // Three, that's the magic number.
    // Three. Yes, it is, it's the magic number.
    // Somewhere in this hip-hop soul community
    // Was brin three: Mase, Dove and me
    // And that's the magic number
    // What does it all mean?
    if game_state.y as u16 > res.y - 3 {
        game_state.alive = false;
    }

    return game_state;
}

fn update_grid(mut screen_grid: Vec<Vec<char>>) -> Vec<Vec<char>> {
    for y in 0..screen_grid.len() {
        screen_grid[y as usize].rotate_left(1);
    }

    return screen_grid;
}

fn draw_grid(game_state: &GameState, screen_grid: &Vec<Vec<char>>) {
    screen_clear();
    println!(
        "Score: {}, Time: {}, Pos {} {}",
        game_state.score, game_state.clock, game_state.x, game_state.y
    );
    for y in 0..screen_grid.len() {
        for x in 0..screen_grid[y].len() {
            print!("{}", screen_grid[y as usize][x as usize]);
        }
        println!("");
    }
}

fn collision_detection(screen_grid: &Vec<Vec<char>>, mut game_state: GameState) -> GameState {
    println!("{} {}", game_state.y, game_state.y as u16);
    if screen_grid[game_state.y as usize][game_state.x as usize] != ' ' {
        game_state.alive = false;
    }

    return game_state;
}

fn term_size() -> Res {
    let size = terminal_size();

    if let Some((Width(w), Height(h))) = size {
        return Res { x: w, y: h };
    }

    return Res { x: 0, y: 0 };
}

fn save_highscore(score: u32) -> u32 {
    // Check if cache directory exists
    let home = env::var("HOME").unwrap();
    let path = Path::new(&home).join(".cache").join("cli_flappy");
    if !path.exists() {
        create_dir(&path).unwrap();
    }

    let filename = path.join("highscore.txt");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&filename)
        .unwrap();

    let mut contents = String::new();
    if filename.exists() {
        contents = fs::read_to_string(&filename).unwrap();
    }

    let highscore: u32 = contents.parse().unwrap_or(0);

    if score > highscore {
        file.write_all(score.to_string().as_bytes()).unwrap();
        return score;
    }
    return highscore;
}

fn game_over(game_state: GameState) {
    let highscore = save_highscore(game_state.score);
    screen_clear();
    println!("Game Over!");
    println!("Score: {}", game_state.score);
    println!("Highscore: {}", highscore);

    if game_state.score > 100 {
        println!("Wow, you're pretty good at this!");
        println!("I didn't think anyone would play this far");
    } else if game_state.score > 10 {
        println!("You did pretty well!");
    } else {
        println!("Better luck next time!");
    }
}

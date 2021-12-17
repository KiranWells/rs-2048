extern crate rand;
extern crate termion;
extern crate home;

use std::fs;
use std::io::{stdin, stdout, Read, Write};
use std::iter;
use termion::raw::IntoRawMode;

const ROWS: i32 = 4;

#[derive(std::cmp::PartialEq, Copy, Clone)]
enum Op {
    Up,
    Down,
    Left,
    Right,
    Quit,
    Noop,
    Reset,
    Undo,
    Ok,
}

fn coord(a: i32, b: i32) -> usize {
    (a + b * ROWS) as usize
}

fn get_input(bytes: &mut std::io::Bytes<std::io::StdinLock>) -> Op {
    let b = bytes.next().unwrap().unwrap();

    match b {
        // Directions
        b'w' | 65 => Op::Up,
        b'a' | 68 => Op::Left,
        b's' | 66 => Op::Down,
        b'd' | 67 => Op::Right,
        // Quit
        b'q' => Op::Quit,
        // Reset
        b'r' => Op::Reset,
        b'z' | b'u' => Op::Undo,
        13 => Op::Ok,
        // Base case
        a => {
            print!("{}", a);
            Op::Noop
        }
    }
}

fn move_board(board: &mut Vec<i32>, o: Op, score: &mut u32) {
    if o == Op::Noop {
        return;
    }
    for x in 0..ROWS {
        let mut line = vec![];
        let mut merge = false;
        for y in 0..ROWS {
            // swap directions
            let y = if o == Op::Down || o == Op::Right {
                ROWS - y - 1
            } else {
                y
            };
            // swap rows/cols
            let (x, y) = if o == Op::Left || o == Op::Right {
                (y, x)
            } else {
                (x, y)
            };
            let el = board[coord(x, y)];
            // erase old board
            board[coord(x, y)] = 0;

            if el == 0 {
                continue;
            }
            let len = line.len();
            if len == 0 {
                line.push(el);
                continue;
            }

            let comp = line.get_mut(len - 1).unwrap();
            if el == *comp && merge == false {
                merge = true;
                *comp *= 2;
                *score += *comp as u32;
            } else {
                merge = false;
                line.push(el);
            }
        }
        for (i, &v) in line.iter().enumerate() {
            let i = if o == Op::Down || o == Op::Right {
                ROWS as usize - i - 1
            } else {
                i
            } as i32;
            // swap rows/cols
            let (x, i) = if o == Op::Left || o == Op::Right {
                (i, x)
            } else {
                (x, i)
            };
            board[coord(x, i)] = v;
        }
    }
}

fn print_board<T: std::io::Write>(
    stdout: &mut termion::raw::RawTerminal<T>,
    board: &Vec<i32>,
    score: &u32,
) {
    let (u_width, u_height) = termion::terminal_size().unwrap();
    let (width, height) = (u_width as i32, u_height as i32);
    let top = height / 2 - (ROWS * 3) / 2;
    let left = width / 2 - (ROWS * 6) / 2;

    // ######
    // # 0  #
    // ######
    write!(
        stdout,
        "{}{}2048{}",
        termion::cursor::Goto(u_width / 2 - 2, (top - 4) as u16),
        termion::style::Bold,
        termion::style::Reset
    )
    .unwrap();
    // print score
    write!(
        stdout,
        "{}{}Score:{: >6}{}",
        termion::cursor::Goto(
            u_width / 2 + (5 * ROWS as u16) / 2 - 10,
            u_height / 2 - (3 * ROWS as u16) / 2 - 2
        ),
        termion::color::Fg(termion::color::LightBlack),
        // termion::color::Fg(termion::color::Black),
        score,
        termion::style::Reset
    )
    .unwrap();

    for y in 0..ROWS {
        for x in 0..ROWS {
            print_tile(
                (top + y * 3) as u16,
                (left + x * 6) as u16,
                board[coord(x, y)],
                stdout,
            );
        }
    }
    write!(stdout, "{}", termion::cursor::Goto(0, u_height)).unwrap();
    stdout.flush().unwrap();
}

fn print_tile<T: std::io::Write>(
    top: u16,
    left: u16,
    i: i32,
    stdout: &mut termion::raw::RawTerminal<T>,
) {
    write!(
        stdout,
        "{}{}{}      {}      {}      {}{: ^6}{}",
        termion::cursor::Goto(left, top),
        termion::color::Bg(get_color(i)),
        termion::color::Fg(termion::color::Rgb(40, 40, 40)),
        termion::cursor::Goto(left, top + 1),
        termion::cursor::Goto(left, top + 2),
        termion::cursor::Goto(left, top + 1),
        if i == 0 {
            String::from("")
        } else {
            format!("{}", i)
        },
        termion::style::Reset
    )
    .unwrap();
}

fn get_color(i: i32) -> termion::color::Rgb {
    use termion::color::*;
    match i {
        0 => Rgb(40, 40, 40),
        2 => Rgb(102, 153, 204),
        4 => Rgb(179, 198, 161),
        8 => Rgb(255, 242, 117),
        16 => Rgb(255, 191, 92),
        32 => Rgb(255, 140, 66),
        64 => Rgb(255, 100, 61),
        128 => Rgb(255, 60, 56),
        256 => Rgb(209, 61, 64),
        512 => Rgb(162, 62, 72),
        1024 => Rgb(170, 80, 89),
        2048 => Rgb(178, 96, 104),
        _ => Rgb(178, 96, 104),
    }
}

fn generate_tiles(board: &mut Vec<i32>) {
    // let mut count = 1;//rand::random::<u32>() % 2;
    // print!("{}", count);
    for i in board.iter_mut() {
        if *i == 0 && rand::random::<u32>() % 3 == 0 {
            *i = (2 as i32).pow(rand::random::<u32>() % 2 + 1);
            return;
        }
    }
    for i in board.iter_mut() {
        if *i == 0 {
            *i = (2 as i32).pow(rand::random::<u32>() % 2 + 1);
            return;
        }
    }
}

fn is_lost(board: &Vec<i32>) -> bool {
    let comp = board.clone();
    let mut test = board.clone();
    let mut same = true;
    let mut _fake_score = 0;
    move_board(&mut test, Op::Up, &mut _fake_score);
    if comp != test {
        same = false;
    }
    move_board(&mut test, Op::Down, &mut _fake_score);
    if comp != test {
        same = false;
    }
    move_board(&mut test, Op::Left, &mut _fake_score);
    if comp != test {
        same = false;
    }
    move_board(&mut test, Op::Right, &mut _fake_score);
    if comp != test {
        same = false;
    }
    return same;
}

fn run_game(
    bytes: &mut std::io::Bytes<std::io::StdinLock>,
    stdout: &mut termion::raw::RawTerminal<std::io::StdoutLock>,
    board: Vec<i32>,
    score: u32
) -> (Vec<i32>, u32) {
    let saved = board.len() == 16;
    let mut board: Vec<_> = if board.len() != 16 {
        iter::repeat(0).take(16).collect()
    } else {
        board
    };
    let mut score = score;

    // startup
    write!(stdout, "{}", termion::clear::All).unwrap();
    if !saved {
        generate_tiles(&mut board);
    }
    print_board(stdout, &board, &score);
    // game loop
    let mut last = board.clone();
    let mut last_op = Op::Noop;
    let mut undo_op = Op::Noop;
    let mut last_score = 0;
    loop {
        write!(stdout, "{}", termion::clear::All).unwrap();
        let d = get_input(bytes);
        if d == Op::Quit {
            break;
        }
        if d == Op::Noop {
            continue;
        }
        if d == Op::Reset {
            board = iter::repeat(0).take(16).collect();
            score = 0;
            generate_tiles(&mut board);
            print_board(stdout, &board, &score);
            continue;
        }
        if d == Op::Undo || d == undo_op {
            let temp = board;
            board = last;
            last = temp;
            let temp = score;
            score = last_score;
            last_score = temp;
            if d == Op::Undo {
                undo_op = last_op;
            } else {
                undo_op = Op::Noop;
            }
        } else {
            undo_op = Op::Noop;
            let mut test = board.clone();
            let mut newscore = score;
            move_board(&mut test, d, &mut newscore);
            if test != board {
                last = board.clone();
                last_score = score;
                score = newscore;
                board = test;
                generate_tiles(&mut board);
            }
        }
        let (u_width, u_height) = termion::terminal_size().unwrap();
        if is_lost(&board) {
            // lost, print loss and wait for reset
            write!(
                stdout,
                "{}Game Over",
                termion::cursor::Goto(u_width / 2 - 4, u_height / 2 + 10)
            )
            .unwrap();
        }
        print_board(stdout, &board, &score);
        last_op = d;
    }
    return (board, score);
}

fn print_menu(
    stdout: &mut termion::raw::RawTerminal<std::io::StdoutLock>,
    board: &Vec<i32>,
    highscore: u32,
    index: i32,
) {
    let (u_width, u_height) = termion::terminal_size().unwrap();
    write!(stdout, "{}", termion::clear::All).unwrap();
    if index == 0 {
        if board.len() == 16 {
            write!(
                stdout,
                "{}{}2048{}{}{}{}New Game{}{}Saved Game{}{}High Score: {: >6}{}",
                termion::cursor::Goto(u_width / 2 - 2, u_height / 2 - 6),
                termion::style::Bold,
                termion::cursor::Goto(u_width / 2 - 4, u_height / 2 - 2),
                termion::style::Reset,
                termion::color::Fg(termion::color::Black),
                termion::color::Bg(termion::color::White),
                termion::style::Reset,
                termion::cursor::Goto(u_width / 2 - 5, u_height / 2 - 0),
                termion::cursor::Goto(u_width / 2 - 9, u_height / 2 + 2),
                termion::color::Fg(termion::color::LightBlack),
                highscore,
                termion::style::Reset,
            )
        } else {
            write!(
                stdout,
                "{}{}2048{}{}{}{}New Game{}{}{}Saved Game{}{}{}High Score: {: >6}{}",
                termion::cursor::Goto(u_width / 2 - 2, u_height / 2 - 6),
                termion::style::Bold,
                termion::cursor::Goto(u_width / 2 - 4, u_height / 2 - 2),
                termion::style::Reset,
                termion::color::Fg(termion::color::Black),
                termion::color::Bg(termion::color::White),
                termion::style::Reset,
                termion::cursor::Goto(u_width / 2 - 5, u_height / 2 - 0),
                termion::color::Fg(termion::color::LightBlack),
                termion::style::Reset,
                termion::cursor::Goto(u_width / 2 - 9, u_height / 2 + 2),
                termion::color::Fg(termion::color::LightBlack),
                highscore,
                termion::style::Reset,
            )
        }
    } else {
        write!(
            stdout,
            "{}{}2048{}{}New Game{}{}{}Saved Game{}{}{}High Score: {: >6}{}",
            termion::cursor::Goto(u_width / 2 - 2, u_height / 2 - 6),
            termion::style::Bold,
            termion::cursor::Goto(u_width / 2 - 4, u_height / 2 - 2),
            termion::style::Reset,
            termion::cursor::Goto(u_width / 2 - 5, u_height / 2 - 0),
            termion::color::Fg(termion::color::Black),
            termion::color::Bg(termion::color::White),
            termion::style::Reset,
            termion::cursor::Goto(u_width / 2 - 9, u_height / 2 + 2),
            termion::color::Fg(termion::color::LightBlack),
            highscore,
            termion::style::Reset,
        )
    }
    .unwrap();
    write!(stdout, "{}", termion::cursor::Goto(0, u_height)).unwrap();
    stdout.flush().unwrap();
}

fn main() {
    // set up printing
    let stdin = stdin();
    let stdin = stdin.lock();
    let mut bytes = stdin.bytes();
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    // check cache
    let path = home::home_dir().unwrap().join(".cache/rs-2048");
    let highscorepath = path.join("highscore.txt");
    let savepath = path.join("save.txt");
    let highscore = fs::read_to_string(highscorepath.clone()).unwrap_or(String::new());
    let mut highscore = highscore.parse::<u32>().unwrap_or(0);
    let save = fs::read_to_string(savepath.clone()).unwrap_or(String::new());
    let mut saved_board = vec![];
    let mut saved_score = 0;
    if save != "" {
        // let lines = save.lines();
        let mut lines = save.lines();
        if let Some(first) = lines.next() {
            saved_score = first.parse::<u32>().unwrap_or(0);
        };
        for line in lines {
            let el = line.parse().unwrap_or(-1);
            if el == -1 {
                saved_board.clear();
                break;
            }
            saved_board.push(el);
        }
        if saved_board.len() != 16 {
            saved_board.clear();
        }
    }

    let mut index = 0;
    loop {
        print_menu(&mut stdout, &saved_board, highscore, index);
        let op = get_input(&mut bytes);
        if op == Op::Quit {
            break;
        }
        if op == Op::Up {
            index += 1;
            index %= 2;
        }
        if op == Op::Down {
            index -= 1;
            index = index.abs();
        }
        if index == 1 && saved_board.len() != 16 {
            index = 0;
        }
        if op == Op::Ok {
            let (board, score) = if index == 0 {
                run_game(&mut bytes, &mut stdout, vec![], 0)
            } else {
                run_game(&mut bytes, &mut stdout, saved_board.clone(), saved_score)
            };
            saved_score = score;
            saved_board = board;
            if saved_score > highscore {
                highscore = saved_score;
            }
        }
    }
    fs::write(
        savepath,
        format!(
            "{}\n{}",
            saved_score,
            saved_board
                .iter()
                .map(|i| format!("{}", i))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )
    .or_else(|e| -> Result<(), ()> {
        println!("Failed to save board. {:?}", e);
        Ok(())
    })
    .unwrap();
    fs::write(highscorepath, format!("{}", highscore))
        .or_else(|e| -> Result<(), ()> {
            println!("Failed to save highscore. {:?}", e);
            Ok(())
        })
        .unwrap();
}

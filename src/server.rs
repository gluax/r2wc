/// Client UI file
use std::net::TcpListener;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

extern crate chrono;
use chrono::prelude::*;

extern crate ncurses;
use ncurses::*;
use std::char;

extern crate stopwatch;
use stopwatch::Stopwatch;

mod connection;
use self::connection::Connection;

/// Init ncurses
fn init_ncurses() {
    initscr();
    raw();
    keypad(stdscr(), true);
    start_color();
    init_pair(1, COLOR_GREEN, COLOR_BLACK);
    init_pair(2, COLOR_BLUE, COLOR_BLACK);
    init_pair(3, COLOR_WHITE, COLOR_BLACK);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
}

/// Handle client messages.
fn handle_client_message(
    con: &Connection,
    chat: &mut Vec<(std::string::String, bool)>,
    msg: String,
    sent_time: Stopwatch,
) {
    if msg == "Message Received." {
        let time_in_ms = sent_time.elapsed_ms();
        chat.push((
            format!(
                "Client {}: {} taking {}ms",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                msg,
                time_in_ms
            ),
            true,
        ));
    } else if msg == "Disconnected" {
        chat.push((
            format!(
                "Client {}: Disconnected",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            ),
            true,
        ));
        chat.push((String::from("Waiting for client..."), false));
    } else if msg != "Empty" && msg != "Blocked" {
        chat.push((
            format!(
                "Client {}: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                msg
            ),
            true,
        ));
        con.notify_message_received();
    }
}

/// Handle chat logs.
fn print_chat(chat: &mut Vec<(std::string::String, bool)>, max_y: usize, max_x: usize) {
    while chat.len() >= (max_y + 1) {
        chat.remove(0);
    }

    let mut chat_iter = chat.iter();
    let mut ln = 0;
    loop {
        match chat_iter.next() {
            Some((msg, client)) => {
                mv(ln, 0);
                clrtoeol();
                if *client {
                    attron(COLOR_PAIR(1));
                } else {
                    attron(COLOR_PAIR(2));
                }
                if msg.len() > max_x {
                    let (mut first, mut next) = msg.split_at(max_x);
                    printw(first);
                    while next.len() > max_x {
                        ln += 1;
                        mv(ln, 0);
                        let (f, n) = next.split_at(max_x);
                        first = f;
                        next = n;
                        printw(first);
                    }
                    ln += 1;
                    mv(ln, 0);
                    printw(next);
                } else {
                    printw(msg);
                }
                refresh();
                ln += 1;
            }
            None => break,
        }
    }

    while ln < (max_y as i32) - 1 {
        mv(ln, 0);
        clrtoeol();
        ln += 1;
    }
}

/// Check client is connected.
fn client_check_handler(
    con: &mut connection::Connection,
    server: &TcpListener,
    chat: &mut Vec<(std::string::String, bool)>,
) {
    match con.taken {
        Some(taken_unwrapped) => {
            if !taken_unwrapped {
                con.await_client_timeout(&server);
                let peer = con.get_peer();
                match peer {
                    Some(p) => {
                        chat.push((format!("Client {} connected", p.who()), false));
                    }
                    None => (),
                }
            }
        }
        None => return,
    }
}

/// Handles input.
fn handle_input(
    con: &Connection,
    chat: &mut Vec<(std::string::String, bool)>,
    input: Result<i32, RecvTimeoutError>,
    line: &mut String,
    mut max_y: i32,
    mut max_x: i32,
    sent_time: &mut Stopwatch,
) -> bool {
    match input {
        Ok(c) => {
            match c {
                // enter
                0xA | 13 | KEY_ENTER => {
                    if line == ":quit" {
                        return true;
                    }
                    let (_, time) = con.send_message(line.clone());
                    *sent_time = time;
                    chat.push((
                        format!(
                            "You {}: {}",
                            Local::now().format("%Y-%m-%d %H:%M:%S"),
                            line.clone()
                        ),
                        false,
                    ));
                    line.clear();
                    mv(max_y, 3);
                    clrtoeol();
                }
                // backspace
                0x7f | KEY_BACKSPACE => {
                    &line.pop();
                    mv(max_y, 3);
                    clrtoeol();
                }
                // resize event
                KEY_RESIZE => {
                    clear();
                    getmaxyx(stdscr(), &mut max_y, &mut max_x);
                    max_y -= 1;
                    max_x -= 1;
                    mv(max_y, max_x);
                    mv(max_y, (3 + line.len()) as i32);
                }

                12 => return true,
                // any other key
                _ => {
                    &line.push(char::from_u32(c as u32).unwrap());
                    mv(max_y, 3);
                    clrtoeol();
                }
            }
        }
        Err(_) => return false,
    }

    if line.len() + 3 > max_x as usize {
        let (_, l) = line.split_at(line.len() + 3 - (max_x as usize));
        printw(l);
    } else {
        printw(&line);
    }

    return false;
}

fn main() {
    let (mut con, server) = Connection::new_server_connection(255);

    let mut chat: Vec<(String, bool)> = Vec::new();
    let mut line = String::new();

    init_ncurses();

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr(), &mut max_y, &mut max_x);
    max_y -= 1;
    max_x -= 1;

    let (tx, rx) = mpsc::channel::<i32>();
    thread::spawn(move || loop {
        let c = getch();
        tx.send(c).unwrap();
    });

    let mut sent_time = Stopwatch::start_new();
    chat.push((String::from("Waiting for client..."), false));

    loop {
        con.reject_other_clients(&server);

        let msg = con.receive_message();
        handle_client_message(&con, &mut chat, msg, sent_time);
        print_chat(&mut chat, max_y as usize, max_x as usize);

        mv(max_y, 0);
        attron(COLOR_PAIR(3));
        printw(">> ");
        mv(max_y, (3 + line.len()) as i32);
        refresh();

        client_check_handler(&mut con, &server, &mut chat);

        let input = rx.recv_timeout(Duration::from_millis(100));
        if handle_input(
            &con,
            &mut chat,
            input,
            &mut line,
            max_y,
            max_x,
            &mut sent_time,
        ) {
            break;
        }
    }

    drop(server);
    endwin();
}

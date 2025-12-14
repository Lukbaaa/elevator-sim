use chrono::Local;
use std::collections::VecDeque;
use std::io::{Stdout, Write};
use std::sync::{Mutex, OnceLock, mpsc};
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;

use crate::elevator::{Direction, Elevator, State};
use crate::elevator_controller::ElevatorController;
use crate::person::Person;

pub static DEBUG_SENDER: OnceLock<mpsc::Sender<String>> = OnceLock::new();
static DEBUG_BUFFER: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
const DEBUG_START_X: u16 = 100;
const DEBUG_START_Y: u16 = 5;
const DEBUG_MAX_LINES: usize = 50;

fn safe_goto(x: u16, y: u16) -> termion::cursor::Goto {
    termion::cursor::Goto(std::cmp::max(1, x), std::cmp::max(1, y))
}

pub fn render(
    screen: &mut AlternateScreen<RawTerminal<Stdout>>,
    i: i32,
    tick_duration: u128,
    ec: &ElevatorController,
    persons: &Vec<Person>,
) {
    let tick_speed = 1.0 / (tick_duration as f64 / 1000.0);
    write!(
        screen,
        "{}{}Tick speed {:.2}Hz Ticks {i}",
        termion::clear::All,
        safe_goto(1, 1),
        tick_speed
    )
    .unwrap();

    let start_line = 5;
    let floors = 4;
    let floor_height = 15;

    draw_building(screen, start_line, floors, floor_height);

    for elevator in ec.get_elevators() {
        draw_elevator(
            screen,
            elevator,
            floors as i32,
            start_line,
            floor_height,
            persons,
        );
    }

    draw_floors(screen, start_line, floors, floor_height);

    draw_persons_on_floor(screen, persons, start_line, floor_height);

    draw_debug_area(screen);

    screen.flush().unwrap();
}

fn draw_building(
    screen: &mut AlternateScreen<RawTerminal<Stdout>>,
    start_line: u16,
    floors: u16,
    floor_height: u16,
) {
    for floor in 0..floors {
        write!(
            screen,
            "{}------------------------------------------------------------------------",
            safe_goto(1, start_line + (floor_height * floor))
        )
        .unwrap();
        for i in 0..floor_height {
            write!(
                screen,
                "{}|                    |                    |                    |       |",
                safe_goto(1, start_line + 1 + i + (floor_height * floor))
            )
            .unwrap();
        }
    }
    write!(
        screen,
        "{}------------------------------------------------------------------------",
        safe_goto(1, start_line + floors * floor_height)
    )
    .unwrap();
}

fn draw_elevator(
    screen: &mut AlternateScreen<RawTerminal<Stdout>>,
    elevator: &Elevator,
    floors: i32,
    start_line: u16,
    floor_height: u16,
    persons: &[Person],
) {
    let elevator_state = elevator.elevator_state.lock().unwrap();
    let dir = match elevator_state.direction {
        Direction::Up => -1,
        Direction::Down => 1,
    };

    let lane_width = 21;
    let wall_offset = 3;

    let mut x = (elevator.number * lane_width + wall_offset) as u16;

    let start = start_line;
    let floor_offset = ((floors - 1) - elevator_state.floor) * floor_height as i32;
    let progress_offset =
        (f32::from(floor_height) * elevator_state.floor_progress * dir as f32).round() as i32;

    let y_base = start as i32 + 1 + floor_offset + progress_offset;
    if y_base < 1 {
        return;
    }
    let mut y = y_base as u16;

    let elevator_height = floor_height - 2;

    write!(screen, "{}------------------", safe_goto(x, y)).unwrap();
    match elevator_state.direction {
        Direction::Up => write!(screen, "{}|       UP       |", safe_goto(x, y + 1)).unwrap(),
        Direction::Down => write!(screen, "{}|      DOWN      |", safe_goto(x, y + 1)).unwrap(),
    }

    match elevator_state.state {
        State::Driving => write!(screen, "{}|    Driving     |", safe_goto(x, y + 1)).unwrap(),
        State::Closing => write!(screen, "{}|    Closing     |", safe_goto(x, y + 1)).unwrap(),
        State::Opening => write!(screen, "{}|    Opening     |", safe_goto(x, y + 1)).unwrap(),
        State::Waiting => write!(screen, "{}|     Waiting    |", safe_goto(x, y + 1)).unwrap(),
    }

    write!(screen, "{}|  {}", safe_goto(x, y + 1), elevator_state.floor).unwrap();
    write!(screen, "{}------------------", safe_goto(x, y + 2)).unwrap();

    let door_patterns = [
        "|#######||#######|",
        "|######|  |######|",
        "|#####|    |#####|",
        "|####|      |####|",
        "|###|        |###|",
        "|##|          |##|",
        "|#|            |#|",
        "||              ||",
    ];

    let idx = (elevator_state.door_progress * 7.999).floor() as usize;
    let idx = std::cmp::min(idx, door_patterns.len() - 1);
    let door_pattern = door_patterns[idx];

    for i in 3..elevator_height {
        write!(screen, "{}{door_pattern}", safe_goto(x, y + i)).unwrap();
    }

    write!(
        screen,
        "{}------------------",
        safe_goto(x, y + elevator_height)
    )
    .unwrap();

    let elevator_wall_offset = 3;
    let elevator_ceiling_offset = 4;

    let passengers_in_this_elevator: Vec<&Person> = persons
        .iter()
        .filter(|p| p.in_elevator && p.elevator_id == Some(elevator.number as i32))
        .collect();

    for (i, person) in passengers_in_this_elevator.iter().enumerate() {
        let person_placement = elevator_wall_offset + (i % 6) * 2;
        x = elevator.number as u16 * lane_width as u16
            + wall_offset as u16
            + person_placement as u16;
        let y_pos = start as i32
            + 1
            + floor_offset
            + progress_offset
            + elevator_ceiling_offset
            + (i / 6) as i32;
        if y_pos < 1 {
            continue;
        }
        y = y_pos as u16;

        if door_pattern.as_bytes()[person_placement] != b' ' {
            continue;
        }

        write!(screen, "{}{}", safe_goto(x, y), person.destination).unwrap();
    }
}

fn draw_floors(
    screen: &mut AlternateScreen<RawTerminal<Stdout>>,
    start_line: u16,
    floors: u16,
    floor_height: u16,
) {
    for floor in (0..floors).rev() {
        write!(
            screen,
            "{}------------------------------------------------------------------------",
            safe_goto(1, start_line + (floor_height * floor))
        )
        .unwrap();
    }
    write!(
        screen,
        "{}------------------------------------------------------------------------",
        safe_goto(1, start_line + floors * floor_height)
    )
    .unwrap();
}

fn draw_persons_on_floor(
    screen: &mut AlternateScreen<RawTerminal<Stdout>>,
    persons: &Vec<Person>,
    start_line: u16,
    floor_height: u16,
) {
    let mut persons_floor_counts = [0u16; 10];

    let mut x: u16 = 0;
    let mut y: u16 = 0;

    let hall_offset = 66;
    let floor_roof_offset = 5;

    for person in persons {
        if !person.in_elevator {
            if person.floor >= persons_floor_counts.len() as i32 {
                continue;
            }
            x = hall_offset + (persons_floor_counts[3 - person.floor as usize] % 3) * 2;
            y = start_line
                + floor_height * (3 - person.floor) as u16
                + floor_roof_offset
                + (persons_floor_counts[3 - person.floor as usize] / 3);
            persons_floor_counts[3 - person.floor as usize] += 1;
        }

        write!(screen, "{}{}", safe_goto(x, y), person.destination).unwrap();
    }
}

pub fn debug(msg: impl Into<String>) {
    if let Some(tx) = DEBUG_SENDER.get() {
        let _ = tx.send(msg.into());
    }
}

pub fn drain_debug_messages(rx: &mpsc::Receiver<String>) {
    for msg in rx.try_iter() {
        let mut buffer = DEBUG_BUFFER.lock().unwrap();
        if buffer.len() >= DEBUG_MAX_LINES {
            buffer.pop_front();
        }
        let msg_with_time_stamp = format!("{} {}", Local::now().time(), msg);
        buffer.push_back(msg_with_time_stamp);
    }
}

fn draw_debug_area(screen: &mut AlternateScreen<RawTerminal<Stdout>>) {
    let buffer = DEBUG_BUFFER.lock().unwrap();
    for (idx, line) in buffer.iter().enumerate() {
        let y = DEBUG_START_Y + idx as u16;
        write!(screen, "{}{}", safe_goto(DEBUG_START_X, y), line).unwrap();
    }
}

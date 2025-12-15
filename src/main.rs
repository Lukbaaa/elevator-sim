use std::io::{Write, stdin, stdout};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

use crate::elevator::State;
use crate::elevator_controller::ElevatorController;
use crate::person::Person;
use crate::renderer::{DEBUG_SENDER, debug, drain_debug_messages, render};

mod elevator;
mod elevator_controller;
mod person;
mod renderer;

enum Event {
    Quit,
    Pause,
    SpeedUp,
    SlowDown,
    Reset,
    Manual,
    Spawn(u32, u32),
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let mut screen = std::io::stdout();
        write!(screen, "{}", termion::cursor::Show).unwrap();
        write!(screen, "{}", termion::screen::ToMainScreen).unwrap();
        println!("PANIC: {:?}", info);
        println!("PANIC: {:?}", info.payload_as_str());
    }));

    {
        let mut screen = stdout()
            .into_raw_mode()
            .unwrap()
            .into_alternate_screen()
            .unwrap();
        write!(screen, "{}", termion::cursor::Hide).unwrap();
        screen.flush().unwrap();

        let (tx, rx) = mpsc::channel();
        let (debug_tx, debug_rx) = mpsc::channel::<String>();
        let _ = DEBUG_SENDER.set(debug_tx);

        thread::spawn(move || {
            let stdin = stdin();
            let mut last_num = u32::MAX;
            for c in stdin.keys() {
                match c.unwrap() {
                    Key::Char('q') => tx.send(Event::Quit).unwrap(),
                    Key::Char('+') => tx.send(Event::SpeedUp).unwrap(),
                    Key::Char('-') => tx.send(Event::SlowDown).unwrap(),
                    Key::Char(' ') => tx.send(Event::Pause).unwrap(),
                    Key::Char('r') => tx.send(Event::Reset).unwrap(),
                    Key::Char('m') => tx.send(Event::Manual).unwrap(),
                    Key::Char(c @ '0'..='9') => {
                        let n = c.to_digit(10).unwrap();
                        if last_num == u32::MAX {
                            last_num = n;
                        } else {
                            tx.send(Event::Spawn(last_num - 1, n - 1)).unwrap();
                            last_num = u32::MAX;
                        }
                    }
                    _ => {}
                }
            }
        });

        let mut quit = false;
        const DEFAULT_TICK_DURATION: u64 = 250;
        let mut tick_duration = Duration::from_millis(DEFAULT_TICK_DURATION);
        let mut i = 0;
        let mut pause = false;
        let mut manual = false;

        let mut elevator_controller = ElevatorController::new_with_elevators();

        let mut persons = Vec::new();
        for _ in 0..5 {
            persons.push(Person::new_rnd());
        }

        loop {
            let start = Instant::now();

            for e in rx.try_iter() {
                match e {
                    Event::Quit => quit = true,
                    Event::Pause => {
                        pause = !pause;
                        elevator_controller.set_paused(pause);
                        debug(format!("Paused: {}", pause));
                    }
                    Event::SpeedUp => {
                        if tick_duration.as_millis() > (DEFAULT_TICK_DURATION) as u128 / 2 {
                            tick_duration -= Duration::from_millis(5)
                        }
                        debug(format!("Speeded up simulation"));
                    }
                    Event::SlowDown => {
                        if tick_duration.as_millis() < (DEFAULT_TICK_DURATION) as u128 * 2 {
                            tick_duration += Duration::from_millis(5)
                        }
                        debug(format!("Slowed down simulation"));
                    }
                    Event::Reset => {
                        elevator_controller.reset();
                        elevator_controller.set_paused(pause);
                        persons.clear();
                        for _ in 0..5 {
                            persons.push(Person::new_rnd());
                        }
                        debug(format!("Reset simulation"));
                    }
                    Event::Manual => { 
                        manual = !manual; 
                        debug(format!("Manual mode: {}", manual));
                    }
                    Event::Spawn(floor, destination) => {
                        persons.push(Person::new(floor as i32, destination as i32));
                        debug(format!("Spawned person from floor {} with destination {}", floor, destination));
                    }
                }
            }
            drain_debug_messages(&debug_rx);
            if quit {
                write!(screen, "{}{}", termion::clear::All, termion::cursor::Show).unwrap();
                break;
            }
            if !pause {
                update_simulation(&mut elevator_controller, &mut persons, manual);

                render(
                    &mut screen,
                    i,
                    tick_duration.as_millis(),
                    &elevator_controller,
                    &persons,
                );
                i += 1;
            }
            let elapsed = start.elapsed();
            if elapsed < tick_duration {
                std::thread::sleep(tick_duration - elapsed);
            }
        }
    }
}

fn update_simulation(controller: &mut ElevatorController, persons: &mut Vec<Person>, manual: bool) {
    controller.update();

    let mut rng = rand::rng();
    use rand::Rng;
    if persons.len() < 30 && rng.random_bool(0.15) && !manual {
        persons.push(Person::new_rnd());
    }

    let mut to_remove = Vec::new();

    for (i, person) in persons.iter_mut().enumerate() {
        if person.in_elevator {
            if let Some(elevator_id) = person.elevator_id {
                let elevator = controller.get_elevator(elevator_id);
                let state = elevator.elevator_state.lock().unwrap();

                if state.floor == person.destination && matches!(state.state, State::Waiting) {
                    drop(state);
                    elevator.remove_passenger();

                    to_remove.push(i);
                }
            }
        } else {
            let mut entered = false;
            for elevator in controller.get_elevators() {
                let state = elevator.elevator_state.lock().unwrap();
                if state.floor == person.floor
                    && matches!(
                        state.state,
                        State::Waiting | State::Opening | State::Closing
                    )
                {
                    drop(state);
                    if elevator.add_passenger() {
                        debug(format!("Added passenger from floor {}", person.floor));
                        person.in_elevator = true;
                        person.elevator_id = Some(elevator.number as i32);
                        elevator.add_request(person.destination);
                        entered = true;
                        break;
                    }
                }
            }

            if !entered {
                person.press_button_up_or_down(controller);
            }
        }
    }

    for i in to_remove.into_iter().rev() {
        persons.remove(i);
    }
}

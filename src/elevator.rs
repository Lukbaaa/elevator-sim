use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use crate::renderer::debug;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Driving,
    Waiting,
    Closing,
    Opening,
}

pub struct ElevatorState {
    pub floor: i32,
    pub destination: i32,
    pub requests: Vec<i32>,
    pub floor_progress: f32,
    pub direction: Direction,
    pub state: State,
    pub door_progress: f32,
    pub passenger_count: i32,
    pub wait_timer: u32,
    pub entry_cooldown: u32,
}

impl ElevatorState {
    fn pick_nearest_destination(&mut self) {
        if let Some(&nearest) = self
            .requests
            .iter()
            .min_by_key(|&&r| (r - self.floor).abs())
        {
            self.destination = nearest;
        }
    }

    pub fn step(&mut self) {
        if self.entry_cooldown > 0 {
            self.entry_cooldown -= 1;
        }

        match self.state {
            State::Driving => {
                if self.floor_progress == 0.0 && self.requests.contains(&self.floor) {
                    self.requests.retain(|&x| x != self.floor);
                    if !self.requests.is_empty() {
                        self.pick_nearest_destination();
                    } else {
                        self.destination = self.floor;
                    }
                    self.state = State::Opening;
                    return;
                }

                if self.floor != self.destination {
                    if self.destination > self.floor {
                        self.direction = Direction::Up;
                    } else {
                        self.direction = Direction::Down;
                    }

                    self.floor_progress += 0.05;
                    if self.floor_progress >= 1.0 {
                        self.floor_progress = 0.0;
                        match self.direction {
                            Direction::Up => self.floor += 1,
                            Direction::Down => self.floor -= 1,
                        }
                    }
                } else {
                    self.state = State::Opening;
                }
            }
            State::Opening => {
                if self.door_progress < 1.0 {
                    self.door_progress += 0.05;
                } else {
                    self.door_progress = 1.0;
                    self.state = State::Waiting;
                    self.wait_timer = 50;
                }
            }
            State::Closing => {
                if self.passenger_count > 2 {
                    self.state = State::Opening;
                    return;
                }
                if self.door_progress > 0.0 {
                    self.door_progress -= 0.05;
                } else {
                    self.door_progress = 0.0;
                    self.state = State::Driving;
                    debug(format!("Elevator on floor {} starts driving", self.floor));
                }
            }
            State::Waiting => {
                if self.passenger_count > 2 {
                    return;
                }
                if self.destination == self.floor && !self.requests.is_empty() {
                    self.pick_nearest_destination();
                }
                if self.wait_timer > 0 {
                    self.wait_timer -= 1;
                } else if self.floor != self.destination || !self.requests.is_empty() {
                    self.state = State::Closing;
                }
            }
        }
    }
}

pub struct Elevator {
    pub number: usize,
    pub elevator_state: Arc<Mutex<ElevatorState>>,
    paused: Arc<AtomicBool>,
}

impl Elevator {
    pub fn new(number: usize) -> Self {
        let elevator_state = ElevatorState {
            floor: 0,
            destination: 0,
            requests: Vec::new(),
            floor_progress: 0.0,
            direction: Direction::Up,
            state: State::Waiting,
            door_progress: 1.0,
            passenger_count: 0,
            wait_timer: 0,
            entry_cooldown: 0,
        };

        let shared_state = Arc::new(Mutex::new(elevator_state));
        let thread_state = Arc::clone(&shared_state);
        let paused_flag = Arc::new(AtomicBool::new(false));
        let paused_for_thread = Arc::clone(&paused_flag);

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(40));
                if paused_for_thread.load(Ordering::Relaxed) {
                    continue;
                }
                {
                    let mut state = thread_state.lock().unwrap();
                    state.step();
                }
            }
        });

        Elevator {
            number,
            elevator_state: shared_state,
            paused: paused_flag,
        }
    }

    pub fn add_request(&self, floor: i32) {
        let mut es = self.elevator_state.lock().unwrap();
        if !es.requests.contains(&floor) {
            es.requests.push(floor);
        }
        es.pick_nearest_destination();

        if es.floor != es.destination {
            match es.state {
                State::Waiting => {
                    es.wait_timer = 50;
                    es.state = State::Closing;
                }
                State::Opening => {
                    es.wait_timer = 50;
                }
                State::Closing => {
                    es.wait_timer = 0;
                }
                State::Driving => { }
            }
        }
    }

    pub fn add_passenger(&self) -> bool {
        let mut es = self.elevator_state.lock().unwrap();
        if es.passenger_count >= 2 {
            return false;
        }
        if es.entry_cooldown > 0 {
            return false;
        }
        es.entry_cooldown = 10;
        es.passenger_count += 1;
        if let State::Waiting = es.state {
            es.wait_timer = 50;
        }

        if let State::Closing = es.state {
            es.state = State::Opening;
        }
        true
    }

    pub fn remove_passenger(&self) {
        let mut es = self.elevator_state.lock().unwrap();
        if es.passenger_count > 0 {
            es.passenger_count -= 1;
        }
        if let State::Waiting = es.state {
            es.wait_timer = 50;
        }
    }

    pub fn reset(&self) {
        let mut es = self.elevator_state.lock().unwrap();
        es.floor = 0;
        es.destination = 0;
        es.requests.clear();
        es.floor_progress = 0.0;
        es.direction = Direction::Up;
        es.state = State::Waiting;
        es.door_progress = 1.0;
        es.passenger_count = 0;
        es.wait_timer = 0;
        es.entry_cooldown = 0;
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }
}

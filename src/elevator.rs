use crate::floor::Floor;
use crate::passenger::Passenger;
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::time::Duration;

#[derive(Debug)]
pub enum ElelvatorStates {
    UP,
    DOWN,
    WAIT,
}

#[derive(Debug)]
pub struct Elevator {
    number: i32,
    pub queue: Vec<Passenger>,
    pub current_floor: i32,
    pub next_floor: i32,
    door_closed: bool,
    pub passenger_count: i32,
    pub max_passenger_count: i32,
    pub current_state: ElelvatorStates,
}
impl Elevator {
    pub fn new(
        number: i32,
        vector: Vec<Passenger>,
        current_floor: i32,
        door_closed: bool,
        passenger_count: i32,
    ) -> Self {
        Elevator {
            number: number,
            queue: vector,
            current_floor: current_floor,
            door_closed: door_closed,
            passenger_count: passenger_count,
            next_floor: 0,
            max_passenger_count: 10,
            current_state: ElelvatorStates::WAIT,
        }
    }

    fn next_floor(&mut self) {
        if let Some(last_passenger) = self.queue.pop() {
            self.current_floor = last_passenger.info.1;
            println!(
                "Fahrstuhl {} fährt in die Etage {} ...",
                self.number, self.current_floor
            );
            thread::sleep(Duration::from_millis(200));
            println!(
                "Fahrstuhl {} ist in Etage {} angekommen.",
                self.number, self.current_floor
            );
            self.passenger_count -= 1;
        }
    }

    pub fn enter_passenger(&mut self, new_pass: Passenger) {
        println!(
            "{} ist in Fahrstuhl {} eingestiegen und will in Etage {}",
            new_pass.info.0, self.number, new_pass.info.1
        );
        self.queue.push(new_pass);
        self.passenger_count += 1;
    }

    pub fn remove_passengers(&mut self) {
        while let Some(passenger) = self.queue.iter().next() {
            let mut i = 0;
            while i < self.queue.len() {
                let passenger = &self.queue[i];

                if passenger.info.1 == self.current_floor {
                    self.queue.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }

    fn open_door(&mut self) {
        println!("Tür von Fahrstuhl {} öffnet.", self.number);
        thread::sleep(Duration::from_millis(200));
        self.door_closed = false;
    }

    fn close_door(&mut self) {
        println!("Tür von Fahrstuhl {} schließt.", self.number);
        thread::sleep(Duration::from_millis(200));
        self.door_closed = true;
    }

    pub fn set_state(&mut self, new_state: ElelvatorStates) {
        self.current_state = new_state;
    }

    pub fn start_elevator(elevator: Arc<Mutex<Elevator>>, floor: Arc<Mutex<Floor>>) {
        thread::spawn(move || {
            loop {
                {
                    let mut floor = floor.lock().unwrap();
                    let mut elev = elevator.lock().unwrap();
                    elev.open_door();
                    if let Some(passenger) = floor.waiting_passengers.pop() {
                        elev.enter_passenger(passenger);
                    }
                    elev.close_door();
                }

                {
                    let mut elev = elevator.lock().unwrap();
                    elev.next_floor();
                }

                thread::sleep(Duration::from_millis(300));
            }
        });
    }
}

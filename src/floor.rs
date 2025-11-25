use crate::passenger::Passenger;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::time::Duration;
#[derive(Clone, Debug)]
pub struct Floor {
    floor_number: i32,
    pub waiting_passengers: Vec<Passenger>,
    pub up_button: i32,
    pub down_button: i32,
}

impl Floor {
    pub fn new(_floor_number: i32, _waiting_passengers: Vec<Passenger>) -> Self {
        Floor {
            floor_number: _floor_number,
            waiting_passengers: _waiting_passengers,
            up_button: 0,
            down_button: 0,
        }
    }
    fn passenger_generator(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(1..=3);
        let new_passenger = Passenger {
            info: (String::from("Mensch"), x),
        };

        if new_passenger.info.1 > self.floor_number {
            self.up_button += 1;
            self.waiting_passengers.push(new_passenger);
        } else if new_passenger.info.1 < self.floor_number {
            self.down_button += 1;
            self.waiting_passengers.push(new_passenger);
        } else {
            drop(new_passenger);
        }
    }
    pub fn start_floor(floor: Arc<Mutex<Floor>>) {
        thread::spawn(move || {
            loop {
                {
                    Floor::passenger_generator(&mut floor.lock().unwrap());
                }
                let mut rng = rand::thread_rng();
                let random_time = rng.gen_range(100..=300);
                thread::sleep(Duration::from_millis(random_time));
            }
        });
    }
}

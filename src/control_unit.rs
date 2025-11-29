use crate::elevator::{self, ElelvatorStates, Elevator};
use crate::floor::Floor;
use crate::passenger::{self, Passenger};
use std::f64::consts::E;
use std::sync::{Arc, Mutex};

const CARRY_MAX: i32 = 10;

#[derive(Debug)]
pub struct ControlUnit {
    elevators: Vec<Arc<Mutex<Elevator>>>,
    floors: Vec<Arc<Mutex<Floor>>>,
}

impl ControlUnit {
    pub fn new(elvs: Vec<Arc<Mutex<Elevator>>>, fls: Vec<Arc<Mutex<Floor>>>) -> Self {
        ControlUnit {
            elevators: elvs,
            floors: fls,
        }
    }

    fn elevator_to_floor(elevator: &mut Elevator, floor: i32) {
        if elevator.current_floor < floor {
            elevator.set_state(ElelvatorStates::UP);
            elevator.current_floor = floor;
        } else if elevator.current_floor > floor {
            elevator.set_state(ElelvatorStates::DOWN);
            elevator.current_floor = floor;
        }
    }

    fn get_passengers(elevator: &mut Elevator, passengers: &mut Vec<Passenger>) {
        elevator.set_state(ElelvatorStates::WAIT);
        while let Some(passenger) = passengers.iter().next() {
            let mut i = 0;

            while i < passengers.len() && elevator.passenger_count <= elevator.max_passenger_count {
                let passenger = &passengers[i];

                if elevator.next_floor > elevator.current_floor
                    && passenger.info.1 > elevator.current_floor
                {
                    let passenger = passengers.remove(i);
                    elevator.enter_passenger(passenger);
                } else {
                    i += 1; // i muss nicht immer erhöht werden, weil der Vektor schrumpft!
                }
            }
        }
    }

    fn exit_passengers(elevator: &mut Elevator) {
        elevator.set_state(ElelvatorStates::WAIT);
        elevator.remove_passengers();
    }

    fn start_elevators(&mut self) {}
}

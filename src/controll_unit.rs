use crate::elevator::{self, Elevator};
use crate::floor::Floor;
use crate::passenger::{self, Passenger};
use std::sync::{Arc, Mutex};

const CARRY_MAX: i32 = 10;

#[derive(Debug)]
pub struct ControllUnit {
    elevators: Vec<Arc<Mutex<Elevator>>>,
    floors: Vec<Arc<Mutex<Floor>>>,
}

impl ControllUnit {
    pub fn new(elvs: Vec<Arc<Mutex<Elevator>>>, fls: Vec<Arc<Mutex<Floor>>>) -> Self {
        ControllUnit {
            elevators: elvs,
            floors: fls,
        }
    }

    fn elevator_to_floor(elevator: &mut Elevator, floor: i32) {
        elevator.current_floor = floor;
    }

    fn get_passengers(elevator: &mut Elevator, passengers: &mut Vec<Passenger>) {
        let mut carry_count = 0;
        while let Some(passenger) = passengers.iter().next()
            && carry_count <= CARRY_MAX
        {
            let mut i = 0;

            while i < passengers.len() && elevator.passenger_count <= elevator.max_passenger_count {
                let passenger = &passengers[i];

                if elevator.next_floor > elevator.current_floor
                    && passenger.info.1 > elevator.current_floor
                {
                    let passenger = passengers.remove(i);
                    elevator.enter_passenger(passenger);
                    carry_count += 1;
                } else {
                    i += 1; // i muss nicht immer erhöht werden, weil der Vektor schrumpft!
                }
            }
        }
    }

    fn start_elevators(&mut self) {}
}

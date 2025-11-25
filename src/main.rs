extern crate rand;
mod controll_unit;
mod elevator;
mod floor;
mod passenger;
use crate::controll_unit::ControllUnit;
use crate::elevator::Elevator;
use crate::floor::Floor;
use crate::passenger::Passenger;
use std::sync::{Arc, Mutex};

fn main() {
    let building: Vec<Arc<Mutex<Floor>>> = vec![
        Arc::new(Mutex::new(Floor::new(1, Vec::<Passenger>::new()))),
        Arc::new(Mutex::new(Floor::new(2, Vec::<Passenger>::new()))),
        Arc::new(Mutex::new(Floor::new(3, Vec::<Passenger>::new()))),
    ];

    let elevators: Vec<Arc<Mutex<Elevator>>> = vec![
        Arc::new(Mutex::new(Elevator::new(1, Vec::new(), 1, true, 0))),
        Arc::new(Mutex::new(Elevator::new(2, Vec::new(), 1, true, 0))),
        Arc::new(Mutex::new(Elevator::new(3, Vec::new(), 1, true, 0))),
    ];

    let controll_unit = ControllUnit::new(elevators.clone(), building.clone());

    println!("Starte den Floorgenerator");

    for i in 0..3 {
        Floor::start_floor(Arc::clone(&building[i]));
    }
    for i in 0..3 {
        Elevator::start_elevator(Arc::clone(&elevators[i]), Arc::clone(&building[i]));
    }
    loop {}
}

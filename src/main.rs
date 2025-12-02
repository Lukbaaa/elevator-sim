extern crate rand;
mod control_unit;
mod elevator;
mod floor;
mod message_types;
mod passenger;

use crate::control_unit::ControlUnit;
use crate::elevator::Elevator;
use crate::floor::Floor;
use crate::message_types::*;
use crate::passenger::Passenger;
use std::sync::{Arc, Mutex, mpsc::channel};

fn main() {
    // Floor -> ControlUnit
    let (floorrequest_s, floorrequest_r) = channel::<FloorRequest>();
    let floorrequest_r = Arc::new(Mutex::new(floorrequest_r)); // ← Wrappen! 

    // ControlUnit -> Elevator
    let (e_s0, e_r0) = channel::<ElevatorCommand>();
    let (e_s1, e_r1) = channel::<ElevatorCommand>();
    let (e_s2, e_r2) = channel::<ElevatorCommand>();
    let elevator_command_sender = vec![e_s0, e_s1, e_s2];

    // Receiver wrappen für die Elevators
    let elevator_receivers = vec![
        Arc::new(Mutex::new(e_r0)),
        Arc::new(Mutex::new(e_r1)),
        Arc::new(Mutex::new(e_r2)),
    ];

    // Floors erstellen
    let building: Vec<Arc<Mutex<Floor>>> = vec![
        Arc::new(Mutex::new(Floor::new(
            0, // ← 0-basiert!  (wichtig für Array-Zugriff)
            Vec::<Passenger>::new(),
            floorrequest_s.clone(),
        ))),
        Arc::new(Mutex::new(Floor::new(
            1,
            Vec::<Passenger>::new(),
            floorrequest_s.clone(),
        ))),
        Arc::new(Mutex::new(Floor::new(
            2,
            Vec::<Passenger>::new(),
            floorrequest_s.clone(),
        ))),
    ];

    // Elevators erstellen (ohne Receiver im Konstruktor)
    let elevators: Vec<Arc<Mutex<Elevator>>> = vec![
        Arc::new(Mutex::new(Elevator::new(0, Vec::new(), 0, true, 0))),
        Arc::new(Mutex::new(Elevator::new(1, Vec::new(), 0, true, 0))),
        Arc::new(Mutex::new(Elevator::new(2, Vec::new(), 0, true, 0))),
    ];

    // ControlUnit erstellen
    let mut control_unit =
        ControlUnit::new(elevator_command_sender, floorrequest_r, elevators.clone());

    println!("Starte die Simulation");

    // Floors starten
    for i in 0..3 {
        Floor::start_floor(Arc::clone(&building[i]));
    }

    // Elevators starten
    for i in 0..3 {
        Elevator::start_elevator(
            Arc::clone(&elevators[i]),
            building.clone(),                   // ← Alle Floors
            Arc::clone(&elevator_receivers[i]), // ← Der passende Receiver
        );
    }

    // ControlUnit starten - WICHTIG!
    control_unit.start_elevators();

    // Hauptthread am Leben halten
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}


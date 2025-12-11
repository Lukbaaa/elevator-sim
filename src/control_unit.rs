use crate::elevator::{self, Elevator};
use crate::floor::Floor;
use crate::message_types::{Direction, ElevatorCommand, ElevatorCommandType, FloorRequest};
use crate::passenger::{self, Passenger};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, spawn};
use std::time::Duration;

const CARRY_MAX: i32 = 10;

#[derive(Debug)]
pub struct ControlUnit {
    elevator_sender: Vec<Sender<ElevatorCommand>>,
    floor_reciver: Arc<Mutex<Receiver<FloorRequest>>>,
    pending_requests: Vec<FloorRequest>,
    elevators: Vec<Arc<Mutex<Elevator>>>,
}

impl ControlUnit {
    pub fn new(
        elv_sender: Vec<Sender<ElevatorCommand>>,
        fls_reciver: Arc<Mutex<Receiver<FloorRequest>>>,
        _elevators: Vec<Arc<Mutex<Elevator>>>,
    ) -> Self {
        ControlUnit {
            elevator_sender: elv_sender,
            floor_reciver: fls_reciver,
            elevators: _elevators,
            pending_requests: Vec::new(),
        }
    }

    fn find_best_elevator(
        elevators: &Vec<Arc<Mutex<Elevator>>>,
        pending_floors: &Vec<Vec<usize>>,
        target_floor: usize,
    ) -> usize {
        let mut best_id = 0;
        let mut best_score = i32::MAX;

        for (id, elevator) in elevators.iter().enumerate() {
            let elev = elevator.lock().unwrap();
            let distance = (elev.current_floor - target_floor as i32).abs();

            // Score berechnen
            let busy_penalty = if elev.is_busy { 50 } else { 0 };
            let queue_penalty = (pending_floors[id].len() as i32) * 10; // Mehr Aufträge = schlechter

            let score = distance + busy_penalty + queue_penalty;

            if score < best_score {
                best_score = score;
                best_id = id;
            }
        }

        best_id
    }
    pub fn start_elevators(&mut self) {
        let receiver_clone = Arc::clone(&self.floor_reciver);
        let sender_clone = self.elevator_sender.clone();
        let elevators_clone = self.elevators.clone();
        let elevator_count = self.elevators.len();

        thread::spawn(move || {
            // Jeder Fahrstuhl hat seine eigene Auftragsliste
            let mut pending_floors: Vec<Vec<usize>> = vec![Vec::new(); elevator_count];

            loop {
                // 1. Alle neuen Requests einsammeln
                loop {
                    let request = {
                        let receiver = receiver_clone.lock().unwrap();
                        receiver.try_recv()
                    };

                    match request {
                        Ok(floor_request) => {
                            // Besten Fahrstuhl für diesen Auftrag finden
                            let elevator_id = Self::find_best_elevator(
                                &elevators_clone,
                                &pending_floors,
                                floor_request.floor,
                            );

                            // Auftrag zur Liste dieses Fahrstuhls hinzufügen
                            if !pending_floors[elevator_id].contains(&floor_request.floor) {
                                pending_floors[elevator_id].push(floor_request.floor);
                                println!(
                                    "ControlUnit: Etage {} ist Fahrstuhl {} (Queue: {:?})",
                                    floor_request.floor, elevator_id, pending_floors[elevator_id]
                                );
                            }
                        }
                        Err(_) => break,
                    }
                }

                // 2. Wenn alle Queues leer, warte auf ersten Request
                let all_empty = pending_floors.iter().all(|q| q.is_empty());
                if all_empty {
                    let floor_request = { receiver_clone.lock().unwrap().recv().unwrap() };

                    let elevator_id = Self::find_best_elevator(
                        &elevators_clone,
                        &pending_floors,
                        floor_request.floor,
                    );

                    pending_floors[elevator_id].push(floor_request.floor);
                    println!(
                        "ControlUnit: Etage {} ist Fahrstuhl {} (Queue: {:?})",
                        floor_request.floor, elevator_id, pending_floors[elevator_id]
                    );
                }

                // 3.  Für jeden Fahrstuhl: Aufträge sortieren und nächsten abarbeiten
                for elevator_id in 0..elevator_count {
                    if pending_floors[elevator_id].is_empty() {
                        continue;
                    }

                    // Ist dieser Fahrstuhl frei?
                    let is_busy = elevators_clone[elevator_id].lock().unwrap().is_busy;
                    if is_busy {
                        continue; // Später nochmal versuchen
                    }

                    // Sortieren nach aktueller Position
                    let current_floor =
                        { elevators_clone[elevator_id].lock().unwrap().current_floor as usize };

                    pending_floors[elevator_id].sort_by(|a, b| {
                        let a_above = *a >= current_floor;
                        let b_above = *b >= current_floor;

                        match (a_above, b_above) {
                            (true, true) => a.cmp(b),
                            (false, false) => b.cmp(a),
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                        }
                    });

                    // Nächsten Auftrag abarbeiten
                    let next_floor = pending_floors[elevator_id].remove(0);

                    println!(
                        "ControlUnit: Fahrstuhl {} fährt zu Etage {} (verbleibend: {:?})",
                        elevator_id, next_floor, pending_floors[elevator_id]
                    );

                    sender_clone[elevator_id]
                        .send(ElevatorCommand {
                            elevator_id,
                            command: ElevatorCommandType::MoveTo(next_floor),
                        })
                        .unwrap();

                    sender_clone[elevator_id]
                        .send(ElevatorCommand {
                            elevator_id,
                            command: ElevatorCommandType::OpenDoors,
                        })
                        .unwrap();

                    sender_clone[elevator_id]
                        .send(ElevatorCommand {
                            elevator_id,
                            command: ElevatorCommandType::CloseDoors,
                        })
                        .unwrap();
                }

                // Kurz warten bevor wir wieder prüfen
                thread::sleep(Duration::from_millis(100));
            }
        });
    }
}

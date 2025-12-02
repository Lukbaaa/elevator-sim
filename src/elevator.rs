use crate::floor::Floor;
use crate::message_types::{Direction, ElevatorCommand, ElevatorCommandType};
use crate::passenger::Passenger;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::time::Duration;

#[derive(Debug)]
pub struct Elevator {
    number: i32,
    pub queue: Vec<Passenger>,
    pub current_floor: i32,
    door_closed: bool,
    pub passenger_count: i32,
    pub max_passenger_count: i32,
    pub current_state: Direction,
    pub is_busy: bool,
}
impl Elevator {
    pub fn new(
        _number: i32,
        _queue: Vec<Passenger>,
        _current_floor: i32,
        _door_closed: bool,
        _passenger_count: i32,
    ) -> Self {
        Elevator {
            number: _number,
            queue: _queue,
            current_floor: _current_floor,
            door_closed: _door_closed,
            passenger_count: _passenger_count,
            max_passenger_count: 10,
            current_state: Direction::Wait,
            is_busy: false,
        }
    }

    fn move_to(&mut self, target_floor: i32) {
        if self.current_floor < target_floor {
            self.set_state(Direction::Up);
            println!("Fahrstuhl {} fährt hoch.. .", self.number);
        } else if self.current_floor > target_floor {
            self.set_state(Direction::Down);
            println!("Fahrstuhl {} fährt runter.. .", self.number);
        }

        self.set_state(Direction::Wait);
        println!(
            "Fahrstuhl {} angekommen in Etage {}",
            self.number, self.current_floor
        );
    }

    pub fn enter_passenger(&mut self, current_floor: &mut Floor) {
        while let Some(passenger) = current_floor.waiting_passengers.pop() {
            if self.passenger_count < self.max_passenger_count {
                println!(
                    "{} ist in Fahrstuhl {} eingestiegen und will in Etage {}",
                    passenger.info.0, self.number, passenger.info.1
                );
                self.queue.push(passenger);
                self.passenger_count += 1;
            } else {
                current_floor.waiting_passengers.push(passenger);
                break;
            }
        }
    }

    pub fn remove_passengers(&mut self) {
        let mut i = 0;
        while i < self.queue.len() {
            let passenger = &self.queue[i];
            if passenger.info.1 == self.current_floor {
                println!(
                    "{} steigt in Etage {} aus Fahrstuhl {} aus",
                    passenger.info.0, self.current_floor, self.number
                );
                self.queue.remove(i);
                self.passenger_count -= 1;
            } else {
                i += 1;
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

    pub fn set_state(&mut self, new_state: Direction) {
        self.current_state = new_state;
    }

    pub fn start_elevator(
        elevator: Arc<Mutex<Elevator>>,
        floors: Vec<Arc<Mutex<Floor>>>,
        command_receiver: Arc<Mutex<Receiver<ElevatorCommand>>>,
    ) {
        thread::spawn(move || {
            loop {
                let command = { command_receiver.lock().unwrap().recv().unwrap() };

                match command.command {
                    ElevatorCommandType::MoveTo(floor) => {
                        let (current, number) = {
                            let elev = elevator.lock().unwrap();
                            (elev.current_floor, elev.number)
                        };
                        let target = floor as i32;

                        if current != target {
                            println!(
                                "Fahrstuhl {} fährt von Etage {} zu Etage {}",
                                number, current, target
                            );
                            thread::sleep(Duration::from_millis(500));
                            elevator.lock().unwrap().current_floor = target;
                            println!("Fahrstuhl {} angekommen in Etage {}", number, target);
                        }
                    }
                    ElevatorCommandType::OpenDoors => {
                        let mut elev = elevator.lock().unwrap();
                        elev.open_door();

                        let mut current_floor = floors[elev.current_floor as usize].lock().unwrap();

                        elev.remove_passengers();

                        elev.enter_passenger(&mut current_floor);
                    }
                    ElevatorCommandType::CloseDoors => {
                        elevator.lock().unwrap().close_door();
                    }
                    ElevatorCommandType::Idle => {
                        elevator.lock().unwrap().set_state(Direction::Wait);
                    }
                    ElevatorCommandType::Shutdown => {
                        break;
                    }
                }
            }
        });
    }
}

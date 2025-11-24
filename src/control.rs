use crate::elevator::{self, Elevator};
use crate::floor::{self, Floor};
use crate::passenger::Passenger;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
pub struct Control {
    elevators: [Elevator; 3],
    floors: Arc<Mutex<[Floor; 4]>>,
}

impl Control {
    pub fn new() -> Self {
        Self {
            floors: Arc::new(Mutex::new([
                Floor::new(0),
                Floor::new(1),
                Floor::new(2),
                Floor::new(3),
            ])),
            elevators: [
                Elevator::new(0, 0),
                Elevator::new(1, 0),
                Elevator::new(2, 0),
            ],
        }
    }

    pub fn start_simulation(self) {
        let mut handlers = vec![];
        
        for floor_id in 0..4 {
            let floors = Arc::clone(&self.floors);
            let handle = thread::spawn(move || {
                println!("Floor {} thread started", floor_id);
                loop {
                    // generate new passengers at random intervals
                    let sec: u64 = rand::thread_rng().gen_range(1..15);
                    thread::sleep(Duration::from_secs(sec));
                    let mut floors = floors.lock().unwrap();

                    generate_passengers(&mut floors[floor_id]);

                    drop(floors);
                }
            });
            handlers.push(handle);
        }

        for mut elevator in self.elevators {
            let floors = Arc::clone(&self.floors);
            let handle = thread::spawn(move || {
                println!("Elevator {} thread started", elevator.get_id());
                loop {
                    let current_floor = elevator.get_current_floor();
                    
                    passenger_leaves_elevator(&mut elevator, current_floor);

                    let mut floors_guard = floors.lock().unwrap();
                    let floors_len = floors_guard.len();
                    let current_floor_obj = &mut floors_guard[current_floor as usize];

                    passenger_boards_elevator(
                        &mut elevator,
                        current_floor_obj,
                        current_floor,
                        floors_len,
                    );

                    start_elevator(&mut elevator, &floors_guard);

                    drop(floors_guard);

                    if *elevator.get_door_state() == elevator::DoorState::Open {
                        elevator.set_door_state(elevator::DoorState::Closed);
                    }
                    elevator.next_floor();

                    thread::sleep(Duration::from_secs(2));
                }
            });
            handlers.push(handle);
        }

        for handle in handlers {
            handle.join().unwrap();
        }
    }
}

fn generate_passengers(floor: &mut Floor) {
    let new_passenger: Passenger;

    if floor.get_id() > 0 && floor.get_id() < 3 {
        let direction: bool = rand::thread_rng().gen_bool(0.5);
        new_passenger = Passenger::new(direction);
    } else if floor.get_id() == 0 {
        new_passenger = Passenger::new(true);
    } else {
        new_passenger = Passenger::new(false);
    }

    floor.push_passenger(new_passenger);

    println!(
        "Floor {} has {} Passengers",
        floor.get_id(),
        floor.get_passengers().len()
    );
}

fn passenger_leaves_elevator(elevator: &mut Elevator, current_floor: u32) {
    if let Some(index) = elevator
        .get_passengers()
        .iter()
        .enumerate()
        .find_map(|(index, p)| {
            if p.get_destination_floor().unwrap_or(100 /* some number outside of floor range */) == current_floor {
                Some(index)
            } else {
                None
            }
        })
    {
        elevator.set_door_state(elevator::DoorState::Open);
        elevator.remove_passenger(index);

        println!(
            "Passenger left elevator {} at floor {}",
            elevator.get_id(),
            current_floor
        );
    }
}

fn passenger_boards_elevator(
    elevator: &mut Elevator,
    current_floor: &mut Floor,
    current_floor_id: u32,
    floors_len: usize,
) {
    if let Some(boarding_passenger_index) = current_floor.get_passengers().iter().position(|p| {
        p.get_destination() == elevator.get_direction().unwrap_or(p.get_destination()) /*use unwrap or because otherwise if in 0 or 3 dont get picked up*/
    }) 
    && elevator.get_passengers().len() < 2
    {
        elevator.set_door_state(elevator::DoorState::Open);

        let boarding_passenger = current_floor.remove_passenger(boarding_passenger_index);

        let destination_floor: u32;
        if boarding_passenger.get_destination() {
            destination_floor = rand::thread_rng().gen_range(current_floor_id+1..floors_len as u32);
        } else {
            destination_floor = rand::thread_rng().gen_range(0..current_floor_id);
        }

        elevator.board_passenger(boarding_passenger, destination_floor);

        println!(
            "Passenger boarded elevator in floor {}",
            current_floor_id,
        );
    }
}

fn start_elevator(elevator: &mut Elevator, floors_guard: &std::sync::MutexGuard<'_, [Floor; 4]>) {
    if elevator.get_passengers().is_empty() {
        if let Some((floor_index, _)) = floors_guard
            .iter()
            .enumerate()
            .find(|(_, floor)| !matches!(floor.get_request(), floor::FloorRequest::Idle))
        {
            let direction = (floor_index as u32) > elevator.get_current_floor();
            elevator.set_direction(Some(direction));
        }
    }
}
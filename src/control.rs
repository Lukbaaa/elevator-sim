use crate::elevator::{self, Elevator};
use crate::floor::{self, Floor};
use crate::passenger::Passenger;
use crate::visualizer::Visualizer;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
pub struct Control {
    elevators: Arc<Mutex<[Elevator; 3]>>,
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
            elevators: Arc::new(Mutex::new([
                Elevator::new( 0),
                Elevator::new( 0),
                Elevator::new( 0),
            ])),
        }
    }

    pub fn start_simulation(self) {
        let mut handlers = vec![];
        
        for floor_id in 0..4 {
            let floors = Arc::clone(&self.floors);
            let handle = thread::spawn(move || {
                loop {
                    // generate new passengers at random intervals
                    let sec: u64 = rand::thread_rng().gen_range(100..1500);
                    thread::sleep(Duration::from_millis(sec));
                    let mut floors = floors.lock().unwrap();

                    generate_passengers(&mut floors[floor_id]);

                    drop(floors);
                }
            });
            handlers.push(handle);
        }

        for elevator_id in 0..3 {
            let floors = Arc::clone(&self.floors);
            let elevators = Arc::clone(&self.elevators);
            let handle = thread::spawn(move || {
                loop {
                    let mut elevators_guard = elevators.lock().unwrap();
                    let elevator = &mut elevators_guard[elevator_id];
                    let current_floor = elevator.get_current_floor();
                    
                    passenger_leaves_elevator(elevator, current_floor);

                    let mut floors_guard = floors.lock().unwrap();
                    let floors_len = floors_guard.len();
                    let current_floor_obj = &mut floors_guard[current_floor as usize];

                    passenger_boards_elevator(
                        elevator,
                        current_floor_obj,
                        current_floor,
                        floors_len,
                    );

                    start_elevator(elevator, &floors_guard);

                    drop(floors_guard);

                    if *elevator.get_door_state() == elevator::DoorState::Open {
                        elevator.set_door_state(elevator::DoorState::Closed);
                    }

                    // Only move if elevator has a direction
                    elevator.next_floor();

                    drop(elevators_guard);

                    thread::sleep(Duration::from_millis(500));
                }
            });
            handlers.push(handle);
        }

        // Visualization thread
        let floors = Arc::clone(&self.floors);
        let elevators = Arc::clone(&self.elevators);
        let vis_handle = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(500));
                let elevators_guard = elevators.lock().unwrap();
                let floors_guard = floors.lock().unwrap();
                Visualizer::draw(&elevators_guard[..], &floors_guard);
                drop(elevators_guard);
                drop(floors_guard);
            }
        });
        handlers.push(vis_handle);

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

}

fn passenger_leaves_elevator(elevator: &mut Elevator, current_floor: u32) {
    
    while let Some(index) = elevator
        .get_passengers()
        .iter()
        .position(|p| p.get_destination_floor().unwrap_or(100) == current_floor)
    {
        elevator.set_door_state(elevator::DoorState::Open);
        elevator.remove_passenger(index);
    }
}

fn passenger_boards_elevator(
    elevator: &mut Elevator,
    current_floor: &mut Floor,
    current_floor_id: u32,
    floors_len: usize,
) {
    while let Some(boarding_passenger_index) = current_floor.get_passengers().iter().position(|p| {
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
        } else {
            // No passengers and no requests - become idle
            elevator.set_direction(None);
        }
    }
}
use crate::{
    elevator::{Direction, Elevator, State},
    debug,
};

struct PickupRequest {
    floor: i32,
    direction: Direction,
    assigned_elevator: Option<usize>,
}

pub struct ElevatorController {
    elevators: [Elevator; 3],
    pickup_requests: Vec<PickupRequest>,
}

const MAX_CAPACITY: i32 = 2;

impl ElevatorController {
    pub fn new_with_elevators() -> Self {
        let elevator1 = Elevator::new(0);
        let elevator2 = Elevator::new(1);
        let elevator3 = Elevator::new(2);

        ElevatorController {
            elevators: [elevator1, elevator2, elevator3],
            pickup_requests: Vec::new(),
        }
    }

    pub fn request_elevator(&mut self, floor: i32, direction: Direction) {
        debug(format!(
            "Request on floor {floor} with direction {direction:?}"
        ));
        if !self
            .pickup_requests
            .iter()
            .any(|r| r.floor == floor && r.direction == direction)
        {
            self.pickup_requests.push(PickupRequest {
                floor,
                direction,
                assigned_elevator: None,
            });
        } else {
            debug(format!("({floor}, {direction:?}) already contained"));
        }
    }

    pub fn get_elevator(&self, idx: i32) -> &Elevator {
        &self.elevators[idx as usize]
    }

    pub fn get_elevators(&self) -> &[Elevator; 3] {
        &self.elevators
    }

    pub fn update(&mut self) {
        let mut handled_indices = Vec::new();

        for (i, req) in self.pickup_requests.iter_mut().enumerate() {
            let floor = req.floor;

            if let Some(idx) = req.assigned_elevator {
                let state = self.elevators[idx].elevator_state.lock().unwrap();
                let at_floor = state.floor == floor
                    && matches!(state.state, State::Waiting | State::Opening | State::Closing);
                let full = state.passenger_count >= MAX_CAPACITY;
                drop(state);

                if full && !at_floor {
                    req.assigned_elevator = None;
                }
            }

            if req.assigned_elevator.is_none() {
                let mut best_elevator = None;
                let mut min_distance = i32::MAX;

                for (idx, elevator) in self.elevators.iter().enumerate() {
                    let state = elevator.elevator_state.lock().unwrap();
                    if state.passenger_count >= MAX_CAPACITY {
                        continue;
                    }

                    if state.floor == floor
                        && matches!(
                            state.state,
                            State::Waiting | State::Opening | State::Closing
                        )
                    {
                        best_elevator = Some(idx);
                        break;
                    }

                    let dist = (state.floor - floor).abs();
                    let is_idle = state.requests.is_empty() && (state.state == State::Waiting);

                    if is_idle {
                        if dist < min_distance {
                            min_distance = dist;
                            best_elevator = Some(idx);
                        }
                    } else if state.state == State::Driving {
                        match state.direction {
                            Direction::Up => {
                                if state.floor < floor && dist < min_distance {
                                    min_distance = dist;
                                    best_elevator = Some(idx);
                                }
                            }
                            Direction::Down => {
                                if state.floor > floor && dist < min_distance {
                                    min_distance = dist;
                                    best_elevator = Some(idx);
                                }
                            }
                        }
                    }
                }

                if let Some(elevator_idx) = best_elevator {
                    req.assigned_elevator = Some(elevator_idx);
                    self.elevators[elevator_idx].add_request(floor);
                }
            }

            if let Some(elevator_idx) = req.assigned_elevator {
                let state = self.elevators[elevator_idx].elevator_state.lock().unwrap();
                let full = state.passenger_count >= MAX_CAPACITY;
                let elevator_at_floor = state.floor == floor
                    && matches!(
                        state.state,
                        State::Waiting | State::Opening | State::Closing
                    );
                drop(state);

                if elevator_at_floor && !full {
                    handled_indices.push(i);
                }
            }
        }

        for i in handled_indices.into_iter().rev() {
            self.pickup_requests.remove(i);
        }
    }

    pub fn reset(&mut self) {
        for elevator in &self.elevators {
            elevator.reset();
        }
        self.pickup_requests.clear();
    }

    pub fn set_paused(&self, paused: bool) {
        for elevator in &self.elevators {
            elevator.set_paused(paused);
        }
    }
}

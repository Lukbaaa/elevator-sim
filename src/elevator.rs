use crate::passenger::Passenger;

#[derive(Copy, Clone, PartialEq)]
pub enum DoorState {
    Open,
    Closed,
    Opening,
    Closing,
}
#[derive(Clone)]
pub struct Elevator {
    id: u32,
    current_floor: u32,
    direction: Option<bool>, // true for up, false for down, none for idle
    door_state: DoorState,
    passengers: Vec<Passenger>,
}

impl Elevator {
    pub fn new(id: u32, floor_id: u32) -> Self {
        Self {
            id,
            current_floor: floor_id,
            door_state: DoorState::Closed,
            passengers: Vec::new(),
            direction: None,
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_passengers(&self) -> &Vec<Passenger> {
        &self.passengers
    }

    pub fn remove_passenger(&mut self, index: usize) -> Passenger {
        self.passengers.remove(index)
    }

    pub fn get_current_floor(&self) -> u32 {
        self.current_floor
    }

    pub fn next_floor(&mut self) {
        if let Some(direction) = self.direction {
            if direction {
                if self.current_floor < 3 {
                    self.current_floor += 1;
                }
            } else {
                if self.current_floor > 0 {
                    self.current_floor -= 1;
                }
            }
        }
    }

    pub fn set_door_state(&mut self, state: DoorState) {
        self.door_state = state;
    }

    pub fn get_door_state(&self) -> &DoorState {
        &self.door_state
    }

    pub fn board_passenger(&mut self, mut passenger: Passenger, destination: u32) {
        passenger.set_destination_floor(destination);
        self.passengers.push(passenger);
        self.direction = Some(destination > self.current_floor);
    }

    pub fn get_direction(&self) -> Option<bool> {
        self.direction
    }

    pub fn set_direction(&mut self, direction: Option<bool>) {
        self.direction = direction;
    }
}

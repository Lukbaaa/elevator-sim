use crate::passenger::Passenger;
#[derive(Clone)]
pub enum FloorRequest {
    Up,
    Down,
    Idle,
}
#[derive(Clone)]
pub struct Floor {
    id: u32,
    passengers: Vec<Passenger>,
    request: FloorRequest,
}

impl Floor {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            passengers: Vec::new(),
            request: FloorRequest::Idle,
        }
    }

    pub fn get_passengers(&self) -> &Vec<Passenger> {
        &self.passengers
    }

    pub fn remove_passenger(&mut self, index: usize) -> Passenger {
        let passenger = self.passengers.remove(index);
        if self.passengers.is_empty() {
            self.request = FloorRequest::Idle;
        }
        return passenger;
    }

    pub fn push_passenger(&mut self, passenger: Passenger) {
        let destination = passenger.get_destination();
        self.passengers.push(passenger);
        self.request = match destination {
            true => FloorRequest::Up,
            false => FloorRequest::Down,
        };
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_request(&self) -> &FloorRequest {
        &self.request
    }
}

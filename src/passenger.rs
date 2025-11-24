#[derive(Clone)]
pub struct Passenger {
    destination: bool, // true = up, false = down
    destination_floor: Option<u32>,
}

impl Passenger {
    pub fn new(destination: bool) -> Self {
        Self {
            destination,
            destination_floor: None,
        }
    }

    pub fn get_destination(&self) -> bool {
        self.destination
    }

    pub fn get_destination_floor(&self) -> Option<u32> {
        self.destination_floor
    }

    pub fn set_destination_floor(&mut self, floor_id: u32) {
        self.destination_floor = Some(floor_id);
    }
}

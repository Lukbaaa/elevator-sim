use rand::Rng;

use crate::elevator_controller::ElevatorController;

pub struct Person {
    pub floor: i32,
    pub destination: i32,
    pub in_elevator: bool,
    pub elevator_id: Option<i32>,
}

impl Person {
    pub fn new(floor: i32, destination: i32) -> Self {
        Person {
            floor,
            destination,
            in_elevator: false,
            elevator_id: None,
        }
    }

    pub fn new_rnd() -> Self {
        let mut rng = rand::rng();

        let floors = 4;

        let floor = rng.random_range(0..floors);

        let mut destination = rng.random_range(0..floors);
        while destination == floor {
            destination = rng.random_range(0..floors);
        }

        Person {
            floor,
            destination,
            in_elevator: false,
            elevator_id: None,
        }
    }

    pub fn press_button_up_or_down(&mut self, elevator_controller: &mut ElevatorController) {
        if self.destination > self.floor {
            elevator_controller.request_elevator(self.floor, crate::elevator::Direction::Up);
        } else {
            elevator_controller
                .request_elevator(self.floor, crate::elevator::Direction::Down);
        }
    }

    pub fn enter_elevator(&mut self, idx: i32, elevator_controller: &mut ElevatorController) {
        elevator_controller.get_elevator(idx).add_passenger();
        self.elevator_id = Some(idx);
    }

    pub fn leave_elevator(&mut self, idx: i32, elevator_controller: &mut ElevatorController) {
        elevator_controller.get_elevator(idx).remove_passenger();
        self.elevator_id = None;
    }

    pub fn press_floor_button(&self, elevator_controller: &mut ElevatorController) {
        elevator_controller
            .get_elevator(self.elevator_id.unwrap())
            .add_request(self.destination);
    }
}

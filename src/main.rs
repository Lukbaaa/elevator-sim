struct Passanger {
    info: (String, i32), //Name, zielstockwerk
}

struct Elevator {
    number: i32,
    queue: Vec<(Passanger)>,
    current_flor: i32,
    door_closed: bool,
    passanger_count: i32,
}

struct ControllUnit {
    elevators: [Elevator; 3],
}

impl ControllUnit {
    fn next_floor(elevator: &mut Elevator) {
        let last_passanger = elevator.queue.pop().unwrap();
        print!(
            "{} steigt in Etage {} aus",
            last_passanger.info.0, last_passanger.info.1
        );
        elevator.passanger_count -= 1;
    }

    fn enter_passanger(elevator: &mut Elevator, new_pass: Passanger) {
        print!(
            "{} ist eingestiegen und will in Etage {}",
            new_pass.info.0, new_pass.info.1
        );
        elevator.queue.push(new_pass);
        elevator.passanger_count += 1;
    }

    fn open_door(elevator: &mut Elevator) {
        elevator.door_closed = false;
    }

    fn close_door(elevator: &mut Elevator) {
        elevator.door_closed = true;
    }
}

fn main() {
    todo!();
}

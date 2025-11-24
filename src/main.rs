extern crate rand;
use core::num;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread::{self, spawn};
use std::time::Duration;

#[derive(Clone)]
struct Passanger {
    info: (String, i32), //Name, zielstockwerk
}

struct Elevator {
    number: i32,
    queue: Vec<Passanger>,
    current_floor: i32,
    door_closed: bool,
    passanger_count: i32,
}

struct ControllUnit {
    elevators: [Elevator; 3],
}

#[derive(Clone)]
struct Floor {
    floor_number: i32,
    waiting_passangers: Vec<Passanger>,
}

impl Floor {
    fn new(floor_number: i32, waiting_passangers: Vec<Passanger>) -> Self {
        Floor {
            floor_number: floor_number,
            waiting_passangers: waiting_passangers,
        }
    }
    fn passanger_generator(&mut self) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(1..=3);
        let new_passanger = Passanger {
            info: (String::from("Mensch"), x),
        };
        self.waiting_passangers.push(new_passanger);
    }
    fn start_floor(building: Arc<Mutex<Vec<Floor>>>, floor_id: usize) {
        thread::spawn(move || {
            loop {
                {
                    let mut floors = building.lock().unwrap();
                    floors[floor_id].passanger_generator();
                }
                let mut rng = rand::thread_rng();
                let random_time = rng.gen_range(200..=500);
                thread::sleep(Duration::from_millis(random_time));
            }
        });
    }
}

impl Elevator {
    fn new(
        number: i32,
        vector: Vec<Passanger>,
        current_floor: i32,
        door_closed: bool,
        passanger_count: i32,
    ) -> Self {
        Elevator {
            number: number,
            queue: vector,
            current_floor: current_floor,
            door_closed: door_closed,
            passanger_count: passanger_count,
        }
    }

    fn next_floor(&mut self) {
        if let Some(last_passanger) = self.queue.pop() {
            self.current_floor = last_passanger.info.1;
            println!(
                "Fahrstuhl {} fährt in die Etage {} ...",
                self.number, self.current_floor
            );
            thread::sleep(Duration::from_millis(200));
            println!(
                "Fahrstuhl {} ist in Etage {} angekommen.",
                self.number, self.current_floor
            );
            self.passanger_count -= 1;
        }
    }

    fn enter_passanger(&mut self, new_pass: Passanger) {
        println!(
            "{} ist in Fahrstuhl {} eingestiegen und will in Etage {}",
            new_pass.info.0, self.number, new_pass.info.1
        );
        self.queue.push(new_pass);
        self.passanger_count += 1;
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

    fn start_elevator(elevator: Arc<Mutex<Elevator>>, building: Arc<Mutex<Vec<Floor>>>) {
        thread::spawn(move || {
            loop {
                {
                    let mut floors = building.lock().unwrap();
                    let mut elev = elevator.lock().unwrap();
                    let floor_index = (elev.current_floor - 1) as usize;
                    elev.open_door();
                    if let Some(passanger) = floors[floor_index].waiting_passangers.pop() {
                        //elev.queue.push(passanger);
                        elev.enter_passanger(passanger);
                    }
                    elev.close_door();
                }

                {
                    let mut elev = elevator.lock().unwrap();
                    elev.next_floor();
                }

                thread::sleep(Duration::from_millis(300));
            }
        });
    }
}

fn main() {
    let building = Arc::new(Mutex::new(vec![
        Floor {
            floor_number: 1,
            waiting_passangers: Vec::<Passanger>::new(),
        },
        Floor {
            floor_number: 2,
            waiting_passangers: Vec::<Passanger>::new(),
        },
        Floor {
            floor_number: 3,
            waiting_passangers: Vec::<Passanger>::new(),
        },
    ]));

    let elevators: Vec<Arc<Mutex<Elevator>>> = vec![
        Arc::new(Mutex::new(Elevator::new(1, Vec::new(), 1, true, 0))),
        Arc::new(Mutex::new(Elevator::new(2, Vec::new(), 1, true, 0))),
        Arc::new(Mutex::new(Elevator::new(3, Vec::new(), 1, true, 0))),
    ];

    // let mut controll_unit: ControllUnit = ControllUnit {
    //     elevators: [elevator_one, elevator_two, elevator_three],
    // };

    println!("Starte den Floorgenerator");

    for i in 0..3 {
        Floor::start_floor(Arc::clone(&building), i);
    }
    for i in 0..3 {
        Elevator::start_elevator(Arc::clone(&elevators[i]), Arc::clone(&building));
    }

    let building_clone = Arc::clone(&building);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(600));

            let mut floors = building_clone.lock().unwrap();
            if let Some(passanger) = floors[0].waiting_passangers.pop() {
                println!("Passagier abgeholt: {}", passanger.info.0);
            }
        }
    })
    .join()
    .unwrap();
}

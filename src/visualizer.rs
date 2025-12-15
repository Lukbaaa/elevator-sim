use crate::elevator::{DoorState, Elevator};
use crate::floor::Floor;
use std::sync::MutexGuard;

pub struct Visualizer;

impl Visualizer {
    pub fn clear_screen() {
        print!("\x1B[2J\x1B[1;1H");
    }

    pub fn draw(elevators: &[Elevator], floors: &MutexGuard<'_, [Floor; 4]>) {
        Self::clear_screen();

        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║           ELEVATOR SIMULATION - BUILDING VIEW                  ║");
        println!("╚════════════════════════════════════════════════════════════════╝");
        println!();

        // Draw from top floor (3) to bottom floor (0)
        for floor_num in (0..=3).rev() {
            Self::draw_floor(floor_num, elevators, &floors[floor_num as usize]);
            if floor_num > 0 {
                println!("├─────┼─────────┼─────────┼─────────┼──────────────────────┤");
            }
        }

        println!("└─────┴─────────┴─────────┴─────────┴──────────────────────┘");
        Self::draw_legend();
    }

    fn draw_floor(floor_num: u32, elevators: &[Elevator], floor: &Floor) {
        let floor_passengers = floor.get_passengers();

        // Draw floor number and elevator shafts
        print!("│ F{} │", floor_num);

        for (_, elevator) in elevators.iter().enumerate() {
            if elevator.get_current_floor() == floor_num {
                let passenger_count = elevator.get_passengers().len();

                // Direction indicator
                let direction = match elevator.get_direction() {
                    Some(true) => "↑",
                    Some(false) => "↓",
                    None => "○",
                };

                // Door state with distinct visual representations
                let door = match elevator.get_door_state() {
                    DoorState::Open => "╔═╗",    // Fully open
                    DoorState::Closed => "║█║",  // Closed
                };

                print!(
                    " {}{}{} {} │",
                    direction,
                    passenger_count,
                    if passenger_count < 10 { " " } else { "" },
                    door
                );
            } else {
                print!("    │    │");
            }
        }

        // Draw waiting passengers with better spacing
        let waiting_up = floor_passengers
            .iter()
            .filter(|p| p.get_destination())
            .count();
        let waiting_down = floor_passengers
            .iter()
            .filter(|p| !p.get_destination())
            .count();

        print!(" ");
        if waiting_up > 0 && waiting_down > 0 {
            print!("↑{} ↓{:<2}", waiting_up, waiting_down);
        } else if waiting_up > 0 {
            print!("↑{:<5}", waiting_up);
        } else if waiting_down > 0 {
            print!("↓{:<5}", waiting_down);
        } else {
            print!("------");
        }
        println!(" │");
    }

    fn draw_legend() {
        println!();
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║ LEGEND                                                           ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║ Elevator Status:                                                 ║");
        println!("║   Direction: ↑ (up) | ↓ (down) | ○ (idle)                        ║");
        println!("║   Number: Passengers inside                                      ║");
        println!("║                                                                  ║");
        println!("║ Door States:                                                     ║");
        println!("║   ╔═╗  Doors fully OPEN                                          ║");
        println!("║   ╔─╗  Doors OPENING...                                          ║");
        println!("║   ║─║  Doors CLOSING...                                          ║");
        println!("║   ║█║  Doors fully CLOSED                                        ║");
        println!("║                                                                  ║");
        println!("║ Waiting Passengers (right column):                              ║");
        println!("║   ↑N = N passengers waiting to go up                             ║");
        println!("║   ↓N = N passengers waiting to go down                           ║");
        println!("╚══════════════════════════════════════════════════════════════════╝");
    }
}

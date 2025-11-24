mod floor;
mod passenger;
mod elevator;
mod control;   

fn main() {
    let control = control::Control::new();
    control.start_simulation();
}

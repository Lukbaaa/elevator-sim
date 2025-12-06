mod floor;
mod passenger;
mod elevator;
mod control;
mod visualizer;

fn main() {
    let control = control::Control::new();
    control.start_simulation();
}

// Nachrichtentypen für die Simulation (Channels).
// Dieses Modul ist intentionally unabhängig von `elevator`/`floor`-Modulen,
// sodass keine zyklischen Abhängigkeiten entstehen.

use std::time::SystemTime;

/// Richtung einer Anforderung (Taste oben/unten).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Wait,
}

/// Anfrage, die ein Floor an die Control-Unit schicken kann.
/// - `floor`: Index (0-basiert oder 1-basiert, konsistent im Projekt verwenden)
/// - `direction`: Up/Down
/// - `requested_at`: Zeitstempel (optional nützlich für Metriken)
#[derive(Debug, Clone)]
pub struct FloorRequest {
    pub floor: usize,
    pub direction: Direction,
}

/// Typ der Kommandos, die die Control an einen Aufzug schicken kann.
#[derive(Debug, Clone)]
pub enum ElevatorCommandType {
    MoveTo(usize), // Ziel-Etage
    OpenDoors,
    CloseDoors,
    Idle,
    Shutdown,
}

/// Envelope für Commands an einen konkreten Aufzug.
#[derive(Debug, Clone)]
pub struct ElevatorCommand {
    pub elevator_id: usize,
    pub command: ElevatorCommandType,
}

/// Vereinfachte Aufzugszustände für Statusmeldungen (unabhängig von interner Implementierung).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElevatorState {
    Up,
    Down,
    Wait,
}
/// Statusmeldung vom Aufzug an die Control-Unit.
/// - `elevator_id`: ID/Index des Aufzugs
/// - `current_floor`: aktuelle Etage
/// - `passenger_count`: aktuelle Personenzahl
/// - `state`: aktueller Zustand
/// - `reported_at`: Zeitstempel der Meldung
#[derive(Debug, Clone)]
pub struct ElevatorStatus {
    pub elevator_id: usize,
    pub current_floor: usize,
    pub passenger_count: usize,
    pub state: ElevatorState,
    pub reported_at: SystemTime,
}

/// Optionaler gemeinsamer Message-Envelope, falls du einen Channel für mehrere
/// Message-Typen nutzen möchtest. Alternativ kannst du separate Channels verwenden,
/// was oft sauberer ist (z. B. FloorRequest-Channel, ElevatorCommand-Channel,
/// ElevatorStatus-Channel).
#[derive(Debug, Clone)]
pub enum Message {
    FloorRequest(FloorRequest),
    ElevatorCommand(ElevatorCommand),
    ElevatorStatus(ElevatorStatus),
}

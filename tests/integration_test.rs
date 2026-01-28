use std::thread;
use std::time::Duration;
use common_explorer::{ExplorerAI, ExplorerBagContent};
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use crossbeam_channel::unbounded;
use explorer_nico::Explorer;

#[test]
fn create_explorer(){
    // 0. Channels
    let (tx_eto, rx_eto) = unbounded::<ExplorerToOrchestrator<ExplorerBagContent>>();
    let (_tx_etp, _rx_etp) = unbounded::<ExplorerToPlanet>();
    let (_tx_pte, rx_pte) = unbounded::<PlanetToExplorer>();
    let (tx_ote, rx_ote) = unbounded::<OrchestratorToExplorer>();

    // 1. Create Explorer
    let mut explorer = Explorer::new(
        266,
        tx_eto,
        rx_ote,
        rx_pte,
        Duration::from_millis(500),
    );

    // 2. Spawn thread
    let handle = thread::spawn(move || {
        explorer.run().expect("Explorer thread panicked");
    });

    // 3. Send Start
    tx_ote.send(OrchestratorToExplorer::StartExplorerAI).expect("Error while sending to the Explorer");

    // 4. Check response
    let response = rx_eto.recv().expect("Error while receiving from Explorer");
    let expected : ExplorerToOrchestrator<ExplorerBagContent> = ExplorerToOrchestrator::StartExplorerAIResult {explorer_id : 266};
    assert_eq!(format!("{response:?}"), format!("{expected:?}"));

    // 4. Send Kill
    tx_ote.send(OrchestratorToExplorer::KillExplorer).expect("Error while sending to the Explorer");

    // 5. Check response
    let response = rx_eto.recv().expect("Error while receiving from Explorer");
    let expected : ExplorerToOrchestrator<ExplorerBagContent> = ExplorerToOrchestrator::KillExplorerResult {explorer_id : 266};
    assert_eq!(format!("{response:?}"), format!("{expected:?}"));

    // Last. Join thread
    handle.join().expect("Error while joining explorer thread");
}
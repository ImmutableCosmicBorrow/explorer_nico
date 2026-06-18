use std::time::{Duration, Instant};
use common_explorer::{ExplorerAI, ExplorerBagContent};
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::planet_explorer::PlanetToExplorer;
use common_game::utils::ID;
use crossbeam_channel::{select, Receiver, Sender};
use crate::brain::Brain;
use crate::logging::{log_debug, log_trace};
use crate::payload;
use crate::galaxy::planet_stats::PlanetStats;

mod lifecycle;
pub(crate) mod handlers;
mod communication;

pub struct Explorer {
    id: ID,
    brain: Brain,
    planet_stats: PlanetStats,
    orchestrator_sender: Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
    orchestrator_receiver: Receiver<OrchestratorToExplorer>,
    planet_receiver: Receiver<PlanetToExplorer>,
    game_step: Duration,
    manual_mode: bool,
    path: Vec<ID>,
}

impl Explorer {
    /// Creates a Nico Explorer.
    /// - `id`: The ID of the explorer
    /// - `tx_explorer_to_orchestrator`: The Sender to send messages from the Explorer to the Orchestrator
    /// - `rx_orchestrator_to_explorer`: The Receiver to receive messages from the Orchestrator to the Explorer
    /// - `rx_planet_to_explorer`: The Receiver to receive messages from the Planets to the Explorer
    /// - `game_step`: The game step
    ///
    /// Returns a Nico Explorer instance.
    #[must_use]
    pub fn new(
        id: ID,
        tx_explorer_to_orchestrator: Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
        rx_orchestrator_to_explorer: Receiver<OrchestratorToExplorer>,
        rx_planet_to_explorer: Receiver<PlanetToExplorer>,
        game_step: Duration,
    ) -> Self {
        let brain = Brain::new(game_step);
        log_debug(payload!(action : "Nico ExplorerAI ready", explorer_id : id));
        Explorer {
            id,
            brain,
            orchestrator_sender: tx_explorer_to_orchestrator,
            orchestrator_receiver: rx_orchestrator_to_explorer,
            planet_receiver: rx_planet_to_explorer,
            planet_stats: PlanetStats::new(),
            game_step,
            manual_mode: true,
            path: Vec::new(),
        }
    }
}

impl ExplorerAI for Explorer {
    /// Runs the Explorer.
    ///
    /// Returns an error if an error occurred during execution.
    fn run(&mut self) -> Result<(), String> {
        let mut next_tick = Instant::now() + self.game_step;
        loop {
            let now = Instant::now();
            let timeout = next_tick.saturating_duration_since(now);
            select! {
                recv(self.orchestrator_receiver) -> msg => {
                    let msg = msg.expect("Error while receiving from Orchestrator");
                    log_trace(payload!(
                        action : "Nico received from Orchestrator",
                        explorer_id : self.id,
                        msg : format!("{msg:?}")
                    ));
                    let kill = self.handle_orchestrator_message(msg)?;
                    if kill {
                        return Ok(());
                    }
                }
                recv(self.planet_receiver) -> msg => {
                    let msg = msg.expect("Error while receiving from Planet");
                    log_debug(payload!(
                        action : "Nico received from Planet",
                        explorer_id : self.id,
                        planet : format!("{:?}", self.planet_stats.id()),
                        msg : format!("{msg:?}")
                    ));
                    self.handle_planet_message(msg)?;
                }
                default(timeout) => {
                    if !self.manual_mode{
                        self.execute_intention()?;
                    }
                    next_tick = Instant::now() + self.game_step;
                }
            }
        }
    }
}
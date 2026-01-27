use crate::payload;
use common_explorer::{ExplorerAI, ExplorerBag, ExplorerBagContent};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::logging::{ActorType, Channel, EventType};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer, OrchestratorToExplorerKind,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
/*
pub(crate) struct PlanetStats {
    supported_resources: HashSet<BasicResourceType>,
    supported_combinations: HashSet<BasicResourceType>,
}
*/
pub struct Explorer {
    pub(crate) id: ID,
    pub(crate) bag: ExplorerBag,
    pub(crate) current_planet_id: ID,
    orchestrator_sender: Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
    orchestrator_receiver: Receiver<OrchestratorToExplorer>,
    pub(crate) planet_sender: Sender<ExplorerToPlanet>,
    planet_receiver: Receiver<PlanetToExplorer>,
    pub(crate) pending: Vec<OrchestratorToExplorerKind>,
    pub(crate) planets_supported_resources: HashMap<ID, HashSet<BasicResourceType>>,
    pub(crate) planets_supported_combinations: HashMap<ID, HashSet<ComplexResourceType>>,
    game_step : Duration,
}

impl Explorer {
    #[must_use]
    pub fn new(
        id: ID,
        planet_id : ID,
        planet_sender: Sender<ExplorerToPlanet>,
        tx_explorer_to_orchestrator: Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
        rx_orchestrator_to_explorer: Receiver<OrchestratorToExplorer>,
        rx_planet_to_explorer: Receiver<PlanetToExplorer>,
        game_step : Duration,
    ) -> Self {
        Explorer {
            id,
            bag: ExplorerBag::new(),
            current_planet_id: planet_id,
            orchestrator_sender: tx_explorer_to_orchestrator,
            orchestrator_receiver: rx_orchestrator_to_explorer,
            planet_sender,
            planet_receiver: rx_planet_to_explorer,
            pending: Vec::new(),
            planets_supported_resources: HashMap::new(),
            planets_supported_combinations: HashMap::new(),
            game_step,
        }
    }
    pub(crate) fn to_orchestrator(
        &self,
        msg: ExplorerToOrchestrator<ExplorerBagContent>,
    ) -> Result<(), String> {
        self.log_msg_to(
            Channel::Trace,
            EventType::MessageExplorerToOrchestrator,
            (ActorType::Orchestrator, 0u32),
            payload!(
                message : format!("{msg:?}")
            ),
        );
        self.orchestrator_sender
            .send(msg)
            .map_err(|err| err.to_string())
    }
    pub(crate) fn to_planet(&self, msg: ExplorerToPlanet) -> Result<(), String> {
        self.log_msg_to(
            Channel::Trace,
            EventType::MessageExplorerToPlanet,
            (ActorType::Planet, self.current_planet_id),
            payload!(
                    message : format!("{msg:?}")
                ),
        );
        self.planet_sender.send(msg).map_err(|err| err.to_string())
    }

    pub(crate) fn wait_for_start(&self) -> Result<bool, String> {
        loop {
            let message = self.orchestrator_receiver.recv();

            match message {
                Ok(OrchestratorToExplorer::StartExplorerAI) => {
                    self.to_orchestrator(ExplorerToOrchestrator::StartExplorerAIResult {explorer_id : self.id})?;
                    return Ok(false)
                },
                Ok(OrchestratorToExplorer::KillExplorer) => {
                    self.to_orchestrator(ExplorerToOrchestrator::KillExplorerResult {
                        explorer_id: self.id,
                    })?;
                    return Ok(true);
                }
                Ok(_) => {
                    // ignore
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    pub(crate) fn move_to(&mut self, planet_id: ID, new_sender: Option<Sender<ExplorerToPlanet>>) {
        if let Some(sender) = new_sender {
            self.current_planet_id = planet_id;
            self.planet_sender = sender;
        }
    }
}

impl ExplorerAI for Explorer {
    fn run(&mut self) -> Result<(), String> {
        let kill = self.wait_for_start()?;
        if kill {
            return Ok(());
        }

        loop {
            if let Ok(message) = self
                .orchestrator_receiver
                .recv_timeout(Duration::from_millis(100))
            {
                let kill = self.handle_orchestrator_message(message)?;
                if kill {
                    return Ok(());
                }
            }

            if let Ok(message) = self
                .planet_receiver
                .recv_timeout(Duration::from_millis(100))
            {
                self.handle_planet_message(message);
            }
        }
    }
}

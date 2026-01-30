use crate::genetics::{Brain, Intention};
use crate::logging_utils::{log_debug, log_error, log_info, log_trace, log_warning};
use crate::payload;
use crate::planet_stats::PlanetStats;
use common_explorer::{ExplorerAI, ExplorerBagContent};
use common_game::components::resource::GenericResource;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender, select};
use std::thread;
use std::time::{Duration, Instant};

pub struct Explorer {
    id: ID,
    brain: Brain,
    planet_stats: PlanetStats,
    orchestrator_sender: Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
    orchestrator_receiver: Receiver<OrchestratorToExplorer>,
    planet_receiver: Receiver<PlanetToExplorer>,
    game_step: Duration,
    manual_mode: bool,
}

impl Explorer {
    #[must_use]
    pub fn new(
        id: ID,
        tx_explorer_to_orchestrator: Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
        rx_orchestrator_to_explorer: Receiver<OrchestratorToExplorer>,
        rx_planet_to_explorer: Receiver<PlanetToExplorer>,
        game_step: Duration,
    ) -> Self {
        let brain = Brain::new();
        log_debug(
            payload!(action : "Nico ExplorerAI ready", explorer_id : id, genome : format!("{:?}", brain.get_genome())),
        );
        Explorer {
            id,
            brain,
            orchestrator_sender: tx_explorer_to_orchestrator,
            orchestrator_receiver: rx_orchestrator_to_explorer,
            planet_receiver: rx_planet_to_explorer,
            planet_stats: PlanetStats::new(),
            game_step,
            manual_mode: true,
        }
    }
    pub(crate) fn to_orchestrator(
        &self,
        msg: ExplorerToOrchestrator<ExplorerBagContent>,
    ) -> Result<(), String> {
        let res = self
            .orchestrator_sender
            .send(msg)
            .map_err(|err| err.to_string());
        if res.is_err() {
            log_error(payload!(
                action : "Nico got an error while trying to send to the Orchestrator",
                explorer_id : self.id
            ));
        }
        res
    }
    pub(crate) fn to_planet(&self, msg: ExplorerToPlanet) -> Result<(), String> {
        if let Some(ref sender) = self.planet_stats.sender() {
            sender.send(msg).map_err(|err| err.to_string())
        } else {
            log_warning(
                payload!(action : "Nico does not have a Planet sender", explorer_id : self.id),
            );
            Err("Planet sender is None".into())
        }
    }
    pub(crate) fn try_move_to(
        &mut self,
        planet_id: ID,
        new_sender: Option<Sender<ExplorerToPlanet>>,
    ) -> Result<bool, String> {
        // Tell the Orchestrator the result
        self.to_orchestrator(ExplorerToOrchestrator::MovedToPlanetResult {
            explorer_id: self.id,
            planet_id,
        })?;

        // Move only if the sender is Some
        if let Some(sender) = new_sender {
            // Reset the Planet stats and update id and sender.
            self.planet_stats.reset();
            self.planet_stats.update_id_and_sender(planet_id, sender);
            self.brain.on_move();

            // Ask Planet for its supported resources and combinations
            self.to_planet(ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: self.id,
            })?;
            self.to_planet(ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: self.id,
            })?;
            // Ask Orchestrator for neighbors of the current Planet
            self.to_orchestrator(ExplorerToOrchestrator::NeighborsRequest {
                explorer_id: self.id,
                current_planet_id: planet_id,
            })?;
        }

        Ok(false)
    }

    pub(crate) fn handle_planet_message(
        &mut self,
        message: PlanetToExplorer,
    ) -> Result<(), String> {
        match message {
            PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                // Send to Orchestrator only if it is in manual mode
                if self.manual_mode {
                    self.to_orchestrator(ExplorerToOrchestrator::SupportedResourceResult {
                        explorer_id: self.id,
                        supported_resources: resource_list.clone(),
                    })?;
                }
                self.planet_stats.update_resources(resource_list);
            }
            PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                // Send to Orchestrator only if it is in manual mode
                if self.manual_mode {
                    self.to_orchestrator(ExplorerToOrchestrator::SupportedCombinationResult {
                        explorer_id: self.id,
                        combination_list: combination_list.clone(),
                    })?;
                }
                self.planet_stats.update_combinations(combination_list);
            }
            PlanetToExplorer::GenerateResourceResponse { resource } => {
                let generated = if let Some(r) = resource {
                    log_debug(
                        payload!(action : "Nico generated a basic resource", explorer_id : self.id, basic_resource : format!("{:?}", r.get_type()),others_in_bag : format!("{:?}", self.brain.get_bag_content())),
                    );
                    self.brain
                        .insert_resource(GenericResource::BasicResources(r));
                    Ok(())
                } else {
                    Err("Planet did not generate resource".into())
                };
                // Send to Orchestrator only if it is in manual mode
                if self.manual_mode {
                    self.to_orchestrator(ExplorerToOrchestrator::GenerateResourceResponse {
                        explorer_id: self.id,
                        generated,
                    })?;
                }
            }
            PlanetToExplorer::CombineResourceResponse { complex_response } => {
                let generated = match complex_response {
                    Ok(r) => {
                        log_debug(
                            payload!(action : "Nico generated a complex resource", explorer_id : self.id, basic_resource : format!("{:?}", r.get_type()),others_in_bag : format!("{:?}", self.brain.get_bag_content())),
                        );
                        self.brain
                            .insert_resource(GenericResource::ComplexResources(r));
                        Ok(())
                    }
                    Err((_error, r1, r2)) => {
                        self.brain.reinsert_resource(r1);
                        self.brain.reinsert_resource(r2);
                        Err("Planet did not create resource".into())
                    }
                };
                // Send to Orchestrator only if it is in manual mode
                if self.manual_mode {
                    self.to_orchestrator(ExplorerToOrchestrator::GenerateResourceResponse {
                        explorer_id: self.id,
                        generated,
                    })?;
                }
            }
            PlanetToExplorer::AvailableEnergyCellResponse { .. } | PlanetToExplorer::Stopped => {}
        }

        Ok(())
    }
    pub(crate) fn handle_orchestrator_message(
        &mut self,
        message: OrchestratorToExplorer,
    ) -> Result<bool, String> {
        match message {
            OrchestratorToExplorer::StartExplorerAI => self.start(),
            OrchestratorToExplorer::ResetExplorerAI => self.reset(),
            OrchestratorToExplorer::KillExplorer => self.kill(),
            OrchestratorToExplorer::StopExplorerAI => self.stop(),
            OrchestratorToExplorer::MoveToPlanet {
                planet_id,
                sender_to_new_planet,
            } => self.try_move_to(planet_id, sender_to_new_planet),
            OrchestratorToExplorer::CurrentPlanetRequest => {
                self.to_orchestrator(ExplorerToOrchestrator::CurrentPlanetResult {
                    explorer_id: self.id,
                    planet_id: self
                        .planet_stats
                        .id()
                        .ok_or("Explorer is not in a Planet".to_string())?,
                })?;
                Ok(false)
            }
            OrchestratorToExplorer::SupportedResourceRequest => {
                self.handle_supported_resources_request()
            }
            OrchestratorToExplorer::SupportedCombinationRequest => {
                self.handle_supported_combination_request()
            }
            OrchestratorToExplorer::GenerateResourceRequest { to_generate } => {
                self.to_planet(ExplorerToPlanet::GenerateResourceRequest {
                    explorer_id: self.id,
                    resource: to_generate,
                })?;
                Ok(false)
            }
            OrchestratorToExplorer::CombineResourceRequest { to_generate } => {
                if let Some(request) = self.brain.try_combination_request(to_generate) {
                    self.to_planet(ExplorerToPlanet::CombineResourceRequest {
                        explorer_id: self.id,
                        msg: request,
                    })?;
                }
                Ok(false)
            }
            OrchestratorToExplorer::BagContentRequest => {
                self.to_orchestrator(ExplorerToOrchestrator::BagContentResponse {
                    explorer_id: self.id,
                    bag_content: self.brain.get_bag_content(),
                })?;
                Ok(false)
            }
            OrchestratorToExplorer::NeighborsResponse { neighbors } => {
                self.planet_stats.update_neighbors(neighbors);
                Ok(false)
            }
        }
    }

    fn handle_supported_resources_request(&mut self) -> Result<bool, String> {
        if let Some(list) = self.planet_stats.resources() {
            self.to_orchestrator(ExplorerToOrchestrator::SupportedResourceResult {
                explorer_id: self.id,
                supported_resources: list.clone(),
            })?;
        } else {
            self.to_planet(ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: self.id,
            })?;
        }
        Ok(false)
    }
    fn handle_supported_combination_request(&mut self) -> Result<bool, String> {
        if let Some(list) = self.planet_stats.combinations() {
            self.to_orchestrator(ExplorerToOrchestrator::SupportedCombinationResult {
                explorer_id: self.id,
                combination_list: list.clone(),
            })?;
        } else {
            self.to_planet(ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: self.id,
            })?;
        }
        Ok(false)
    }
    fn execute_intention(&mut self) -> Result<(), String> {
        let intention = self.brain.decide(&mut self.planet_stats);
        log_trace(payload!(
            intention : format!("{intention:?}"),
        ));

        match intention {
            Intention::Generate(Some(resource)) => {
                self.to_planet(ExplorerToPlanet::GenerateResourceRequest {
                    explorer_id: self.id,
                    resource,
                })?;
                Ok(())
            }
            Intention::Combine(Some(resource)) => {
                if let Some(msg) = self.brain.try_combination_request(resource) {
                    self.to_planet(ExplorerToPlanet::CombineResourceRequest {
                        explorer_id: self.id,
                        msg,
                    })?;
                }
                Ok(())
            }
            Intention::Move(Some(dest)) => {
                if let Some(curr) = self.planet_stats.id() {
                    self.to_orchestrator(ExplorerToOrchestrator::TravelToPlanetRequest {
                        explorer_id: self.id,
                        current_planet_id: curr,
                        dst_planet_id: dest,
                    })?;
                }
                Ok(())
            }
            Intention::Move(None) => {
                log_debug(payload!(
                    action : "Nico could not find a Planet to move into",
                    explorer_id : self.id,
                    known_neighbors : format!("{:?}",self.planet_stats.neighbors())
                ));
                if let Some(planet_id) = self.planet_stats.id() {
                    self.to_orchestrator(ExplorerToOrchestrator::NeighborsRequest {
                        explorer_id: self.id,
                        current_planet_id: planet_id,
                    })?;
                }
                Ok(())
            }
            _ => {
                self.brain.on_no_action();
                Ok(())
            }
        }
    }
    fn reset(&mut self) -> Result<bool, String> {
        // TODO: actually reset AI
        self.manual_mode = true;
        log_info(payload!(
            action : "Nico ExplorerAI correctly reset",
            explorer_id : self.id
        ));
        self.to_orchestrator(ExplorerToOrchestrator::ResetExplorerAIResult {
            explorer_id: self.id,
        })?;
        Ok(false)
    }
    fn stop(&mut self) -> Result<bool, String> {
        self.manual_mode = true;
        log_info(payload!(
            action : "Nico Explorer switched to manual mode",
            explorer_id : self.id,
        ));
        self.to_orchestrator(ExplorerToOrchestrator::StopExplorerAIResult {
            explorer_id: self.id,
        })?;
        Ok(false)
    }
    fn start(&mut self) -> Result<bool, String> {
        self.manual_mode = false;
        self.to_orchestrator(ExplorerToOrchestrator::StartExplorerAIResult {
            explorer_id: self.id,
        })?;
        log_info(payload!(action : "Nico ExplorerAI correctly started", explorer_id : self.id));
        Ok(false)
    }
    fn kill(&mut self) -> Result<bool, String> {
        log_info(payload!(
            action : "Nico has been killed, bye bye :(",
            explorer_id : self.id,
            performance : self.brain.get_performance(),
            genome : format!("{:?}",self.brain.get_genome()),
            bag_content : format!("{:?}", self.brain.get_bag_content()),
        ));
        thread::sleep(Duration::from_millis(5));
        self.to_orchestrator(ExplorerToOrchestrator::KillExplorerResult {
            explorer_id: self.id,
        })?;
        Ok(true)
    }
}

impl ExplorerAI for Explorer {
    fn run(&mut self) -> Result<(), String> {
        let timeout = Duration::from_millis(100);
        loop {
            let start = Instant::now();

            select! {
                recv(self.orchestrator_receiver) -> msg => {
                    let msg = msg.expect("Error while receiving from Orchestrator");
                    let kill = self.handle_orchestrator_message(msg)?;
                    if kill {
                        return Ok(());
                    }
                }
                recv(self.planet_receiver) -> msg => {
                    let msg = msg.expect("Error while receiving from Orchestrator");
                    self.handle_planet_message(msg)?;
                }
                default(timeout) => {
                    if !self.manual_mode{
                        self.execute_intention()?;
                    }

                }
            }

            let elapsed = start.elapsed();
            if elapsed < self.game_step {
                thread::sleep(self.game_step - elapsed);
            }
        }
    }
}

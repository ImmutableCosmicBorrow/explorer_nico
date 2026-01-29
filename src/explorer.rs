use crate::genetics::Intention;
use crate::payload;
use crate::planet_stats::PlanetStats;
use common_explorer::{ExplorerAI, ExplorerBag, ExplorerBagContent};
use common_game::components::resource::{BasicResourceType, ComplexResourceType, GenericResource};
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender, select};
use rand::RngExt;
use std::thread;
use std::time::{Duration, Instant};

pub struct Explorer {
    genome: Vec<u8>,
    gene_step: usize,
    id: ID,
    bag: ExplorerBag,
    current_planet_id: Option<ID>,
    orchestrator_sender: Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
    orchestrator_receiver: Receiver<OrchestratorToExplorer>,
    planet_sender: Option<Sender<ExplorerToPlanet>>,
    planet_receiver: Receiver<PlanetToExplorer>,
    planet_stats: PlanetStats,
    game_step: Duration,
    manual_mode: bool,
    move_chance : u8,
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
        let mut rng = rand::rng();
        let genome = (0..20).map(|_| rng.random_range(0..32)).collect();

        Self::log_internal(
            Channel::Debug,
            payload!(
                action : "Nico ExplorerAI ready",
                explorer_id : id,
                genome : format!("{genome:?}"),
            ),
        );

        Explorer {
            genome,
            gene_step: 0,
            id,
            bag: ExplorerBag::new(),
            current_planet_id: None,
            orchestrator_sender: tx_explorer_to_orchestrator,
            orchestrator_receiver: rx_orchestrator_to_explorer,
            planet_sender: None,
            planet_receiver: rx_planet_to_explorer,
            planet_stats: PlanetStats::new(),
            game_step,
            manual_mode: true,
            move_chance : 0,
        }
    }
    pub(crate) fn to_orchestrator(
        &self,
        msg: ExplorerToOrchestrator<ExplorerBagContent>,
    ) -> Result<(), String> {
        thread::sleep(Duration::from_secs(1));
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
        if let Some(ref sender) = self.planet_sender {
            self.log_msg_to(
                Channel::Trace,
                EventType::MessageExplorerToPlanet,
                (ActorType::Planet, self.current_planet_id.unwrap()),
                payload!(
                    message : format!("{msg:?}")
                ),
            );

            sender.send(msg).map_err(|err| err.to_string())
        } else {
            Err("Planet sender is None".into())
        }
    }
    pub(crate) fn move_to(
        &mut self,
        planet_id: ID,
        new_sender: Option<Sender<ExplorerToPlanet>>,
    ) -> Result<(), String> {
        self.planet_stats = PlanetStats::new();
        self.current_planet_id = Some(planet_id);
        self.planet_sender = new_sender;
        self.move_chance = 0;

        self.to_planet(ExplorerToPlanet::SupportedResourceRequest {
            explorer_id: self.id,
        })?;
        self.to_planet(ExplorerToPlanet::SupportedCombinationRequest {
            explorer_id: self.id,
        })?;
        self.to_orchestrator(ExplorerToOrchestrator::NeighborsRequest {
            explorer_id: self.id,
            current_planet_id: planet_id,
        })?;
        Ok(())
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
                    Self::log_internal(
                        Channel::Trace,
                        payload!(
                            action : "Nico generated a basic resource",
                            basic_resource : format!("{:?}", r.get_type()),
                            others_in_bag : format!("{:?}", self.bag.to_content()),
                        ),
                    );
                    self.bag.insert(GenericResource::BasicResources(r));
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
                        Self::log_internal(
                            Channel::Trace,
                            payload!(
                                action : "Nico crafted a complex resource",
                                complex_resource : format!("{:?}",r.get_type()),
                                others_in_bag : format!("{:?}", self.bag.to_content()),
                            ),
                        );
                        self.bag.insert(GenericResource::ComplexResources(r));
                        Ok(())
                    }
                    Err((_error, r1, r2)) => {
                        self.bag.insert(r1);
                        self.bag.insert(r2);
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
    #[allow(clippy::too_many_lines)]
    pub(crate) fn handle_orchestrator_message(
        &mut self,
        message: OrchestratorToExplorer,
    ) -> Result<bool, String> {
        match message {
            OrchestratorToExplorer::StartExplorerAI => {
                self.manual_mode = false;
                self.to_orchestrator(ExplorerToOrchestrator::StartExplorerAIResult {
                    explorer_id: self.id,
                })?;
                Self::log_internal(
                    Channel::Info,
                    payload!(
                        action : "Nico ExplorerAI correctly started",
                        explorer_id : self.id
                    ),
                );
                Ok(false)
            }
            OrchestratorToExplorer::ResetExplorerAI => {
                // TODO: actually reset AI
                self.manual_mode = true;
                Self::log_internal(
                    Channel::Info,
                    payload!(
                        action : "Nico ExplorerAI correctly resetted",
                        explorer_id : self.id
                    ),
                );
                self.to_orchestrator(ExplorerToOrchestrator::ResetExplorerAIResult {
                    explorer_id: self.id,
                })?;
                Ok(false)
            }
            OrchestratorToExplorer::KillExplorer => {
                self.to_orchestrator(ExplorerToOrchestrator::KillExplorerResult {
                    explorer_id: self.id,
                })?;
                Ok(true)
            }
            OrchestratorToExplorer::StopExplorerAI => {
                self.manual_mode = true;
                Self::log_internal(
                    Channel::Info,
                    payload!(
                        action : "Nico Explorer switched to manual mode",
                        explorer_id : self.id,
                    ),
                );
                Ok(false)
            }
            OrchestratorToExplorer::MoveToPlanet {
                planet_id,
                sender_to_new_planet,
            } => {
                self.to_orchestrator(ExplorerToOrchestrator::MovedToPlanetResult {
                    explorer_id: self.id,
                    planet_id,
                })?;
                self.move_to(planet_id, sender_to_new_planet)?;
                Ok(false)
            }
            OrchestratorToExplorer::CurrentPlanetRequest => {
                self.to_orchestrator(ExplorerToOrchestrator::CurrentPlanetResult {
                    explorer_id: self.id,
                    planet_id: self
                        .current_planet_id
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
                if let Some(request) = self.bag.create_combination_request(to_generate) {
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
                    bag_content: self.bag.to_content(),
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
        if let Some(list) = self.planet_stats.get_resources() {
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
        if let Some(list) = self.planet_stats.get_combinations() {
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
    fn decide(&mut self) -> Intention {
        let gene = self.genome[self.gene_step % self.genome.len()];
        self.gene_step += 1;

        match gene % (10 + self.move_chance) {
            0..6 => Intention::Generate(self.decide_generation(gene)),
            6..9 => Intention::Combine(self.decide_combination(gene)),
            _ => Intention::Move(self.decide_move(gene)),
        }
    }

    fn decide_generation(&self, gene: u8) -> Option<BasicResourceType> {
        self.planet_stats
            .get_resources()
            .as_ref()
            .filter(|resources| {!resources.is_empty()})
            .map(|resources| {
                *resources.iter().collect::<Vec<&BasicResourceType>>()
                    [gene as usize % resources.len()]
            })
    }
    fn decide_combination(&self, gene: u8) -> Option<ComplexResourceType> {
        self.planet_stats
            .get_combinations()
            .as_ref()
            .filter(|resources| {!resources.is_empty()})
            .map(|resources| {
                *resources.iter().collect::<Vec<&ComplexResourceType>>()
                    [gene as usize % resources.len()]
            })
    }
    fn decide_move(&self, gene: u8) -> Option<ID> {
        self.planet_stats
            .get_neighbors()
            .as_ref()
            .filter(|resources| {!resources.is_empty()})
            .map(|neighbors| {
                neighbors[gene as usize % neighbors.len()]
            })
    }

    pub fn log_internal(channel: Channel, payload: Payload) {
        LogEvent::system(EventType::InternalExplorerAction, channel, payload).emit();
    }

    /// Creates a log event with itself as sender
    pub fn log_msg_to(
        &self,
        channel: Channel,
        event_type: EventType,
        to: (ActorType, ID),
        payload: Payload,
    ) {
        LogEvent::new(
            Some(Participant::new(ActorType::Explorer, self.id)),
            Some(Participant::new(to.0, to.1)),
            event_type,
            channel,
            payload,
        )
        .emit();
    }
    fn execute_intention(&mut self) -> Result<(), String> {
        let intention = self.decide();

        match intention {
            Intention::Generate(Some(resource)) => {
                self.to_planet(ExplorerToPlanet::GenerateResourceRequest {
                    explorer_id: self.id,
                    resource,
                })?;
                Ok(())
            }
            Intention::Combine(Some(resource)) => {
                if let Some(msg) = self.bag.create_combination_request(resource) {
                    self.to_planet(ExplorerToPlanet::CombineResourceRequest {
                        explorer_id: self.id,
                        msg,
                    })?;
                }
                Ok(())
            }
            Intention::Move(Some(dest)) => {
                if let Some(curr) = self.current_planet_id {
                    self.to_orchestrator(ExplorerToOrchestrator::TravelToPlanetRequest {
                        explorer_id: self.id,
                        current_planet_id: curr,
                        dst_planet_id: dest,
                    })?;
                }
                Ok(())
            }
            _ => {
                self.move_chance += 1;
                Ok(())
            },
        }
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

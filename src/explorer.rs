use crate::payload;
use common_explorer::{ExplorerAI, ExplorerBag, ExplorerBagContent};
use common_game::components::resource::{BasicResourceType, ComplexResourceType, GenericResource};
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;
use rand::RngExt;
use crate::genetics::Intention;
/*
pub(crate) struct PlanetStats {
    supported_resources: HashSet<BasicResourceType>,
    supported_combinations: HashSet<BasicResourceType>,
}
*/
pub struct Explorer {
    genome: Vec<u8>,
    gene_step : usize,
    id: ID,
    bag: ExplorerBag,
    current_planet_id: Option<ID>,
    orchestrator_sender: Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
    orchestrator_receiver: Receiver<OrchestratorToExplorer>,
    planet_sender: Option<Sender<ExplorerToPlanet>>,
    planet_receiver: Receiver<PlanetToExplorer>,
    planets_supported_resources: HashMap<ID, HashSet<BasicResourceType>>,
    planets_supported_combinations: HashMap<ID, HashSet<ComplexResourceType>>,
    game_step: Duration,
    planet_neighbors : Vec<ID>,
    manual_mode : bool,
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
            planets_supported_resources: HashMap::new(),
            planets_supported_combinations: HashMap::new(),
            game_step,
            planet_neighbors : Vec::new(),
            manual_mode: true,
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
    pub(crate) fn move_to(&mut self, planet_id: ID, new_sender: Option<Sender<ExplorerToPlanet>>) {
        self.current_planet_id = Some(planet_id);
        self.planet_sender = new_sender;
    }

    pub(crate) fn handle_planet_message(
        &mut self,
        message: PlanetToExplorer,
    ) -> Result<(), String> {
        match message {
            PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                self.planets_supported_resources.insert(
                    self.current_planet_id
                        .ok_or("Explorer is not in a Planet".to_string())?,
                    resource_list.clone(),
                );
                // Send to Orchestrator only if it is in manual mode
                if self.manual_mode {
                    self.to_orchestrator(ExplorerToOrchestrator::SupportedResourceResult {explorer_id : self.id, supported_resources : resource_list})?;
                }
            }
            PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                self.planets_supported_combinations.insert(
                    self.current_planet_id
                        .ok_or("Explorer is not in a Planet".to_string())?,
                    combination_list.clone(),
                );
                // Send to Orchestrator only if it is in manual mode
                if self.manual_mode {
                    self.to_orchestrator(ExplorerToOrchestrator::SupportedCombinationResult {explorer_id : self.id, combination_list})?;
                }
            }
            PlanetToExplorer::GenerateResourceResponse { resource } => {
                let generated = if resource.is_some() { Ok(())} else { Err("Planet did not create resource".into()) };
                if let Some(r) = resource {
                    self.bag.insert(GenericResource::BasicResources(r));
                }
                // Send to Orchestrator only if it is in manual mode
                if self.manual_mode {
                    self.to_orchestrator(ExplorerToOrchestrator::GenerateResourceResponse {explorer_id : self.id, generated })?;
                }
            }
            PlanetToExplorer::CombineResourceResponse { complex_response } => {
                let generated = if complex_response.is_ok() { Ok(())} else { Err("Planet did not create resource".into()) };
                match complex_response {
                    Ok(r) => {
                        self.bag.insert(GenericResource::ComplexResources(r));
                    }
                    Err((_error, r1, r2)) => {
                        // TODO: log the error
                        self.bag.insert(r1);
                        self.bag.insert(r2);
                    }
                }
                // Send to Orchestrator only if it is in manual mode
                if self.manual_mode {
                    self.to_orchestrator(ExplorerToOrchestrator::GenerateResourceResponse {explorer_id : self.id, generated })?;
                }
            }
            PlanetToExplorer::AvailableEnergyCellResponse { .. }
            | PlanetToExplorer::Stopped => {}
        }

        Ok(())
    }
    pub(crate) fn handle_orchestrator_message(
        &mut self,
        message: OrchestratorToExplorer,
    ) -> Result<bool, String> {
        match message {
            OrchestratorToExplorer::StartExplorerAI => {
                self.to_orchestrator(ExplorerToOrchestrator::StartExplorerAIResult {
                    explorer_id: self.id,
                })?;
                Self::log_internal(
                    Channel::Info,
                    payload!(
                        action : "Nico ExplorerAI started correctly",
                        explorer_id : self.id
                    ),
                );
                Ok(false)
            }
            OrchestratorToExplorer::ResetExplorerAI => {
                // TODO: actually reset AI
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
                Ok(false)
            }
            OrchestratorToExplorer::MoveToPlanet {
                planet_id,
                sender_to_new_planet,
            } => {
                self.move_to(planet_id, sender_to_new_planet);
                Self::log_internal(
                    Channel::Debug,
                    payload!(
                        action : "Nico ExplorerAI correctly moved to Planet",
                        explorer_id : self.id,
                        destination_planet : planet_id,
                    ),
                );
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
                self.planet_neighbors = neighbors;
                Ok(false)
            }
        }
    }

    fn handle_supported_resources_request(&mut self) -> Result<bool, String> {
        if let Some(list) = self.planets_supported_resources.get(
            &self
                .current_planet_id
                .ok_or("Explorer is not in a Planet".to_string())?,
        ) {
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
        if let Some(list) = self.planets_supported_combinations.get(
            &self
                .current_planet_id
                .ok_or("Explorer is not in a Planet".to_string())?,
        ) {
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
    pub(crate) fn decide(&mut self) -> Intention {
        let gene = self.genome[self.gene_step % self.genome.len()];
        self.gene_step += 1;

        match gene % 10 {
            0..6 => Intention::Generate(self.decide_generation(gene)),
            6..9 => Intention::Combine(self.decide_combination(gene)),
            _ => Intention::Move(self.decide_move(gene)),
        }
    }

    fn decide_generation(&self, gene : u8) -> Option<BasicResourceType> {
        let all_supported  = &self.planets_supported_resources;

        if let Some(planet_id) = self.current_planet_id && let Some(resources) = all_supported.get(&planet_id){
            Some(*resources.iter().collect::<Vec<&BasicResourceType>>()[gene as usize & resources.len()])
        } else {
            None
        }
    }

    fn decide_combination(&self, gene : u8) -> Option<ComplexResourceType> {
        let all_supported = &self.planets_supported_combinations;

        if let Some(planet_id) = self.current_planet_id && let Some(resources) = all_supported.get(&planet_id){
            Some(*resources.iter().collect::<Vec<&ComplexResourceType>>()[gene as usize & resources.len()])
        } else {
            None
        }
    }

    fn decide_move(&self, gene : u8) -> Option<ID> {
        let len = self.planet_neighbors.len();
        if len > 0 {
            Some(self.planet_neighbors[gene as usize % len])
        } else {
            None
        }
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
}

impl ExplorerAI for Explorer {
    fn run(&mut self) -> Result<(), String> {
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
                self.handle_planet_message(message)?;
            }
            thread::sleep(self.game_step);
        }
    }


}

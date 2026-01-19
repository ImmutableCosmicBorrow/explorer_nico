use std::time::Duration;
use common_game::components::resource::{GenericResource};
use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};
use dummy_explorer::{ExplorerAI, ExplorerBag, ExplorerBagContent};

pub struct Explorer{
    id : ID,
    bag : ExplorerBag,
    current_planet_id : ID,
    to_orchestrator : Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
    from_orchestrator: Receiver<OrchestratorToExplorer>,
    to_planet : Sender<ExplorerToPlanet>,
    from_planet: Receiver<PlanetToExplorer>,

}


impl Explorer{
    #[must_use] pub fn new(
        id : ID,
        current_planet_id : ID,
        tx_explorer_to_orchestrator: Sender<ExplorerToOrchestrator<ExplorerBagContent>>,
        rx_orchestrator_to_explorer : Receiver<OrchestratorToExplorer>,
        tx_explorer_to_planet : Sender<ExplorerToPlanet>,
        rx_planet_to_explorer : Receiver<PlanetToExplorer>,
    ) -> Self{

        Explorer{
            id,
            bag : ExplorerBag::new(),
            current_planet_id,
            to_orchestrator : tx_explorer_to_orchestrator,
            from_orchestrator : rx_orchestrator_to_explorer,
            to_planet : tx_explorer_to_planet,
            from_planet: rx_planet_to_explorer,
        }
    }
    fn to_orchestrator(&self, msg : ExplorerToOrchestrator<ExplorerBagContent>) -> Result<(), String> {
        self.to_orchestrator.send(msg).map_err(|err| err.to_string())
    }
    #[allow(dead_code)]
    fn to_planet(&self, msg : ExplorerToPlanet) -> Result<(), String> {
        self.to_planet.send(msg).map_err(|err| err.to_string())
    }

    fn wait_for_start(&self) -> Result<bool, String>{
        loop{
            let message = self.from_orchestrator.recv();

            match message{
                Ok(OrchestratorToExplorer::StartExplorerAI)=>{
                    return Ok(false)
                },
                Ok(OrchestratorToExplorer::KillExplorer)=>{
                    self.to_orchestrator(ExplorerToOrchestrator::KillExplorerResult {explorer_id: self.id})?;
                    return Ok(true)
                },
                Ok(_) => {
                    // ignore
                },
                Err(e) => return Err(e.to_string())
            }
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
            if let Ok(message) = self.from_orchestrator.recv_timeout(Duration::from_millis(100)) {
                match message {
                    OrchestratorToExplorer::StartExplorerAI => {
                        self.to_orchestrator(ExplorerToOrchestrator::StartExplorerAIResult { explorer_id: self.id })?;
                    }
                    OrchestratorToExplorer::ResetExplorerAI => {
                        // TODO: actually reset AI
                        self.to_orchestrator(ExplorerToOrchestrator::ResetExplorerAIResult { explorer_id: self.id })?;
                    }
                    OrchestratorToExplorer::KillExplorer => {
                        self.to_orchestrator(ExplorerToOrchestrator::KillExplorerResult { explorer_id: self.id })?;
                        return Ok(())
                    }
                    OrchestratorToExplorer::StopExplorerAI => {
                        let kill = self.wait_for_start()?;
                        if kill {
                            return Ok(());
                        }
                    }
                    OrchestratorToExplorer::MoveToPlanet { .. } => {
                        todo!()
                    }
                    OrchestratorToExplorer::CurrentPlanetRequest => {
                        self.to_orchestrator(ExplorerToOrchestrator::CurrentPlanetResult { explorer_id: self.id, planet_id: self.current_planet_id })?;
                    }
                    OrchestratorToExplorer::SupportedResourceRequest => {
                        todo!()
                    }
                    OrchestratorToExplorer::SupportedCombinationRequest => {
                        todo!()
                    }
                    OrchestratorToExplorer::GenerateResourceRequest { .. } => {
                        todo!()
                    }
                    OrchestratorToExplorer::CombineResourceRequest { .. } => {
                        todo!()
                    }
                    OrchestratorToExplorer::BagContentRequest => {
                        self.to_orchestrator(ExplorerToOrchestrator::BagContentResponse { explorer_id: self.id, bag_content: self.bag.to_content() })?;
                    }
                    OrchestratorToExplorer::NeighborsResponse { .. } => {}
                }
            }


            if let Ok(message) = self.from_planet.recv_timeout(Duration::from_millis(100)) {
                match message {
                    PlanetToExplorer::SupportedResourceResponse { .. } => {
                        todo!()
                    }
                    PlanetToExplorer::SupportedCombinationResponse { .. } => {
                        todo!()
                    }
                    PlanetToExplorer::GenerateResourceResponse { resource } => {
                        if let Some(r) = resource {
                            self.bag.insert(GenericResource::BasicResources(r));
                        }
                    }
                    PlanetToExplorer::CombineResourceResponse { complex_response } => {
                        match complex_response {
                            Ok(r) => {
                                self.bag.insert(GenericResource::ComplexResources(r));
                            },
                            Err((_error, r1, r2)) => {
                                // TODO: log the error
                                self.bag.insert(r1);
                                self.bag.insert(r2);
                            }
                        }
                    }
                    PlanetToExplorer::AvailableEnergyCellResponse { .. } => {
                        todo!()
                    }
                    PlanetToExplorer::Stopped => {}
                }
            }
        }
    }
}

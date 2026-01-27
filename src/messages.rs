use crate::Explorer;
use common_game::components::resource::GenericResource;
use common_game::protocols::orchestrator_explorer::ExplorerToOrchestrator::{
    SupportedCombinationResult, SupportedResourceResult,
};
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer, OrchestratorToExplorerKind,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};

impl Explorer {
    pub(crate) fn handle_planet_message(
        &mut self,
        message: PlanetToExplorer,
    ){
        match message {
            PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                self.planets_supported_resources.insert(
                    self.current_planet_id,
                    resource_list,
                );
                // TODO: if pending request from orchestrator, send back
            }
            PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                self.planets_supported_combinations.insert(
                    self.current_planet_id,
                    combination_list,
                );
                // TODO: if pending request from orchestrator, send back
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
                    }
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
    pub(crate) fn handle_orchestrator_message(
        &mut self,
        message: OrchestratorToExplorer,
    ) -> Result<bool, String> {
        match message {
            OrchestratorToExplorer::StartExplorerAI => {
                self.to_orchestrator(ExplorerToOrchestrator::StartExplorerAIResult {
                    explorer_id: self.id,
                })?;
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
                let kill = self.wait_for_start()?;
                Ok(kill)
            }
            OrchestratorToExplorer::MoveToPlanet {
                planet_id,
                sender_to_new_planet,
            } => {
                self.move_to(planet_id, sender_to_new_planet);
                Ok(false)
            }
            OrchestratorToExplorer::CurrentPlanetRequest => {
                self.to_orchestrator(ExplorerToOrchestrator::CurrentPlanetResult {
                    explorer_id: self.id,
                    planet_id: self
                        .current_planet_id,
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
                self.pending
                    .push(OrchestratorToExplorerKind::GenerateResourceRequest);
                self.to_planet(ExplorerToPlanet::GenerateResourceRequest {
                    explorer_id: self.id,
                    resource: to_generate,
                })?;
                Ok(false)
            }
            OrchestratorToExplorer::CombineResourceRequest { to_generate } => {
                if let Some(request) = self.bag.create_combination_request(to_generate) {
                    self.pending
                        .push(OrchestratorToExplorerKind::CombineResourceRequest);
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
            OrchestratorToExplorer::NeighborsResponse { .. } => {
                todo!()
            }
        }
    }

    fn handle_supported_resources_request(&mut self) -> Result<bool, String> {
        if let Some(list) = self.planets_supported_resources.get(
            &self
                .current_planet_id,
        ) {
            self.to_orchestrator(SupportedResourceResult {
                explorer_id: self.id,
                supported_resources: list.clone(),
            })?;
        } else {
            self.pending
                .push(OrchestratorToExplorerKind::SupportedResourceRequest);
            self.to_planet(ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: self.id,
            })?;
        }
        Ok(false)
    }

    fn handle_supported_combination_request(&mut self) -> Result<bool, String> {
        if let Some(list) = self.planets_supported_combinations.get(
            &self
                .current_planet_id,
        ) {
            self.to_orchestrator(SupportedCombinationResult {
                explorer_id: self.id,
                combination_list: list.clone(),
            })?;
        } else {
            self.pending
                .push(OrchestratorToExplorerKind::SupportedCombinationRequest);
            self.to_planet(ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: self.id,
            })?;
        }
        Ok(false)
    }
}

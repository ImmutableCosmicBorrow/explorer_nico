use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use common_game::utils::ID;
use crossbeam_channel::Sender;
use crate::Explorer;

impl Explorer {
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
        self.brain.on_move();

        // Move only if the sender is Some
        if let Some(sender) = new_sender {
            self.path.push(planet_id);

            // Reset the Planet stats and update id and sender.
            self.planet_stats.update_planet(planet_id, sender);

            if !self.manual_mode {
                // Ask Planet for its supported resources and combinations
                self.to_planet(ExplorerToPlanet::SupportedResourceRequest {
                    explorer_id: self.id,
                })?;

                self.to_planet(ExplorerToPlanet::SupportedCombinationRequest {
                    explorer_id: self.id,
                })?;
            }

            // Ask Orchestrator for neighbors of the current Planet
            self.to_orchestrator(ExplorerToOrchestrator::NeighborsRequest {
                explorer_id: self.id,
                current_planet_id: planet_id,
            })?;
        } else {
            self.to_orchestrator(ExplorerToOrchestrator::NeighborsRequest { explorer_id: self.id, current_planet_id: self.planet_stats.id().expect("Nico is not in a Planet") })?;
        }

        Ok(false)
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
                if self.manual_mode {
                    self.to_planet(ExplorerToPlanet::GenerateResourceRequest {
                        explorer_id: self.id,
                        resource: to_generate,
                    })?;
                }
                Ok(false)
            }
            OrchestratorToExplorer::CombineResourceRequest { to_generate } => {
                if self.manual_mode {
                    if let Some(request) = self.brain.try_combination_request(to_generate) {
                        self.to_planet(ExplorerToPlanet::CombineResourceRequest {
                            explorer_id: self.id,
                            msg: request,
                        })?;
                    } else {
                        self.to_orchestrator(ExplorerToOrchestrator::CombineResourceResponse {
                            explorer_id: self.id,
                            generated: Err("Explorer did not have ingredients".into()),
                        })?;
                    }
                }
                Ok(false)
            }
            OrchestratorToExplorer::BagContentRequest => {
                self.to_orchestrator(ExplorerToOrchestrator::BagContentResponse {
                    explorer_id: self.id,
                    bag_content: self.brain.bag_content(),
                })?;
                Ok(false)
            }
            OrchestratorToExplorer::NeighborsResponse { neighbors } => {
                /* TODO
                if neighbors.is_empty() {
                    self.brain.got_blocked();
                } else {
                    self.brain.unblock();
                }*/
                self.planet_stats.update_neighbors(neighbors.clone());
                self.brain
                    .set_planet_neighbors(self.planet_stats.id().unwrap_or(0), neighbors);
                Ok(false)
            }
        }
    }

    fn handle_supported_resources_request(&mut self) -> Result<bool, String> {
        if self.manual_mode {
            self.to_planet(ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: self.id,
            })?;
        }
        Ok(false)
    }

    fn handle_supported_combination_request(&mut self) -> Result<bool, String> {
        if self.manual_mode {
            self.to_planet(ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: self.id,
            })?;
        }
        Ok(false)
    }
}
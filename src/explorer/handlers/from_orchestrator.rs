use crate::Explorer;
use common_game::components::resource::ResourceType;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use common_game::utils::ID;
use crossbeam_channel::Sender;

impl Explorer {
    /// Handles the received Orchestrator message.
    ///
    /// Returns an error if any send of a message fails.
    /// Otherwise, returns a `Ok(bool)`, indicating whether the Explorer has been killed or not.
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
                    self.brain.set_needs(ResourceType::Basic(to_generate));

                    self.to_planet(ExplorerToPlanet::GenerateResourceRequest {
                        explorer_id: self.id,
                        resource: to_generate,
                    })?;
                }
                Ok(false)
            }
            OrchestratorToExplorer::CombineResourceRequest { to_generate } => {
                if self.manual_mode {
                    if to_generate.is_aipartner() {
                        self.brain.reset_needs();
                    } else {
                        self.brain.set_needs(ResourceType::Complex(to_generate));
                    }

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
                self.planet_stats.update_neighbors(neighbors.clone());
                self.brain
                    .set_planet_neighbors(self.planet_stats.id().unwrap_or(0), neighbors);
                Ok(false)
            }
        }
    }

    /// Tries to move to a new Planet. This succeeds if `new_sender` is `Some`.
    ///
    /// Returns an error if any send of a message fails.
    /// Otherwise, returns `Ok(false)` indicating that the Explorer has not been killed.
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
            // Ask for neighbors again, since a Planet might have been killed
            self.to_orchestrator(ExplorerToOrchestrator::NeighborsRequest {
                explorer_id: self.id,
                current_planet_id: self.planet_stats.id().expect("Nico is not in a Planet"),
            })?;
        }

        Ok(false)
    }

    /// Handles a `SupportedResourceRequest`
    ///
    /// Returns an error if any send of a message fails.
    /// Otherwise, returns `Ok(false)` indicating that the Explorer has not been killed.
    fn handle_supported_resources_request(&mut self) -> Result<bool, String> {
        if self.manual_mode {
            self.to_planet(ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: self.id,
            })?;
        } else {
            let supported_resources = self
                .brain
                .supported_resources(self.planet_stats.id().expect("Explorer is not in a Planet"));
            self.to_orchestrator(ExplorerToOrchestrator::SupportedResourceResult {
                explorer_id: self.id,
                supported_resources,
            })?;
        }
        Ok(false)
    }

    /// Handles a `SupportedCombinationRequest`
    ///
    /// Returns an error if any send of a message fails.
    /// Otherwise, returns `Ok(false)` indicating that the Explorer has not been killed.
    fn handle_supported_combination_request(&mut self) -> Result<bool, String> {
        if self.manual_mode {
            self.to_planet(ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: self.id,
            })?;
        } else {
            let combination_list = self.brain.supported_combinations(
                self.planet_stats.id().expect("Explorer is not in a Planet"),
            );
            self.to_orchestrator(ExplorerToOrchestrator::SupportedCombinationResult {
                explorer_id: self.id,
                combination_list,
            })?;
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_explorer::{ExplorerAI, ExplorerBag, ExplorerBagContent};
    use common_game::components::resource::{BasicResourceType, ComplexResourceType};
    use common_game::protocols::planet_explorer::PlanetToExplorer;
    use crossbeam_channel::unbounded;
    use std::collections::HashSet;
    use std::thread;
    use std::time::Duration;

    const EXPLORER_ID: u32 = 1;
    const PLANET_ID: u32 = 2;

    #[test]
    fn test_bag_content_request() {
        // 0. Channels
        let (tx_eto, rx_eto) = unbounded::<ExplorerToOrchestrator<ExplorerBagContent>>();
        let (tx_etp, _rx_etp) = unbounded::<ExplorerToPlanet>();
        let (_tx_pte, rx_pte) = unbounded::<PlanetToExplorer>();
        let (tx_ote, rx_ote) = unbounded::<OrchestratorToExplorer>();

        // 1. Create Explorer
        let mut explorer = Explorer::new(
            EXPLORER_ID,
            tx_eto,
            rx_ote,
            rx_pte,
            Duration::from_millis(500),
        );

        // 2. Spawn thread
        let handle = thread::spawn(move || {
            explorer.run().expect("Explorer thread panicked");
        });

        // 3. Send MoveToPlanet
        tx_ote
            .send(OrchestratorToExplorer::MoveToPlanet {
                planet_id: PLANET_ID,
                sender_to_new_planet: Some(tx_etp),
            })
            .expect("Error while sending to the Explorer");

        // 4. Explorer should respond with MovedToPlanetResult
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::MovedToPlanetResult {
                explorer_id: EXPLORER_ID,
                planet_id: PLANET_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // 4.5 Explorer asks neighbors
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::NeighborsRequest {
                explorer_id: EXPLORER_ID,
                current_planet_id: PLANET_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // 5. Orchestrator asks for bag content
        tx_ote
            .send(OrchestratorToExplorer::BagContentRequest)
            .expect("Error while sending to the Explorer");

        // 6. Explorer responds with an empty set
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::BagContentResponse {
                explorer_id: EXPLORER_ID,
                bag_content: ExplorerBag::new().to_content(),
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // Last-2. Send Kill
        tx_ote
            .send(OrchestratorToExplorer::KillExplorer)
            .expect("Error while sending to the Explorer");

        // Last-1. Check response
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::KillExplorerResult {
                explorer_id: EXPLORER_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // Last. Join thread
        handle.join().expect("Error while joining explorer thread");
    }

    #[test]
    fn test_supported_combinations_request() {
        // 0. Channels
        let (tx_eto, rx_eto) = unbounded::<ExplorerToOrchestrator<ExplorerBagContent>>();
        let (tx_etp, rx_etp) = unbounded::<ExplorerToPlanet>();
        let (tx_pte, rx_pte) = unbounded::<PlanetToExplorer>();
        let (tx_ote, rx_ote) = unbounded::<OrchestratorToExplorer>();

        // 1. Create Explorer
        let mut explorer = Explorer::new(
            EXPLORER_ID,
            tx_eto,
            rx_ote,
            rx_pte,
            Duration::from_millis(500),
        );

        // 2. Spawn thread
        let handle = thread::spawn(move || {
            explorer.run().expect("Explorer thread panicked");
        });

        // 3. Send MoveToPlanet
        tx_ote
            .send(OrchestratorToExplorer::MoveToPlanet {
                planet_id: PLANET_ID,
                sender_to_new_planet: Some(tx_etp),
            })
            .expect("Error while sending to the Explorer");

        // 4. Explorer should respond with MovedToPlanetResult
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::MovedToPlanetResult {
                explorer_id: EXPLORER_ID,
                planet_id: PLANET_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // 4.5 Explorer asks neighbors
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::NeighborsRequest {
                explorer_id: EXPLORER_ID,
                current_planet_id: PLANET_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // 5. Send SupportedCombinationRequest
        tx_ote
            .send(OrchestratorToExplorer::SupportedCombinationRequest {})
            .expect("Error while sending to the Explorer");

        // 6. Explorer asks Planet supported combinations
        let request = rx_etp.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToPlanet = ExplorerToPlanet::SupportedCombinationRequest {
            explorer_id: EXPLORER_ID,
        };
        assert_eq!(format!("{request:?}"), format!("{expected:?}"));

        // 7. Planet responds to the explorer
        let mut resources = HashSet::new();
        resources.insert(ComplexResourceType::Diamond);
        tx_pte
            .send(PlanetToExplorer::SupportedCombinationResponse {
                combination_list: resources.clone(),
            })
            .expect("Error while sending to the Explorer");

        // 8. Explorer responds to the Orchestrator with the resource list
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::SupportedCombinationResult {
                explorer_id: EXPLORER_ID,
                combination_list: resources,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // Last-2. Send Kill
        tx_ote
            .send(OrchestratorToExplorer::KillExplorer)
            .expect("Error while sending to the Explorer");

        // Last-1. Check response
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::KillExplorerResult {
                explorer_id: EXPLORER_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // Last. Join thread
        handle.join().expect("Error while joining explorer thread");
    }

    #[test]
    fn test_supported_resources_request() {
        // 0. Channels
        let (tx_eto, rx_eto) = unbounded::<ExplorerToOrchestrator<ExplorerBagContent>>();
        let (tx_etp, rx_etp) = unbounded::<ExplorerToPlanet>();
        let (tx_pte, rx_pte) = unbounded::<PlanetToExplorer>();
        let (tx_ote, rx_ote) = unbounded::<OrchestratorToExplorer>();

        // 1. Create Explorer
        let mut explorer = Explorer::new(
            EXPLORER_ID,
            tx_eto,
            rx_ote,
            rx_pte,
            Duration::from_millis(500),
        );

        // 2. Spawn thread
        let handle = thread::spawn(move || {
            explorer.run().expect("Explorer thread panicked");
        });

        // 3. Send MoveToPlanet
        tx_ote
            .send(OrchestratorToExplorer::MoveToPlanet {
                planet_id: PLANET_ID,
                sender_to_new_planet: Some(tx_etp),
            })
            .expect("Error while sending to the Explorer");

        // 4. Explorer should respond with MovedToPlanetResult
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::MovedToPlanetResult {
                explorer_id: EXPLORER_ID,
                planet_id: PLANET_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // 4.5 Explorer asks neighbors
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::NeighborsRequest {
                explorer_id: EXPLORER_ID,
                current_planet_id: PLANET_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // 5. Send SupportedResourceRequest
        tx_ote
            .send(OrchestratorToExplorer::SupportedResourceRequest {})
            .expect("Error while sending to the Explorer");

        // 6. Explorer asks Planet supported resources
        let request = rx_etp.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToPlanet = ExplorerToPlanet::SupportedResourceRequest {
            explorer_id: EXPLORER_ID,
        };
        assert_eq!(format!("{request:?}"), format!("{expected:?}"));

        // 7. Planet responds to the explorer
        let mut resources = HashSet::new();
        resources.insert(BasicResourceType::Carbon);
        tx_pte
            .send(PlanetToExplorer::SupportedResourceResponse {
                resource_list: resources.clone(),
            })
            .expect("Error while sending to the Explorer");

        // 8. Explorer responds to the Orchestrator with the resource list
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::SupportedResourceResult {
                explorer_id: EXPLORER_ID,
                supported_resources: resources,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // Last-2. Send Kill
        tx_ote
            .send(OrchestratorToExplorer::KillExplorer)
            .expect("Error while sending to the Explorer");

        // Last-1. Check response
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::KillExplorerResult {
                explorer_id: EXPLORER_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // Last. Join thread
        handle.join().expect("Error while joining explorer thread");
    }
}

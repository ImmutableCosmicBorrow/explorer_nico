use common_game::components::resource::GenericResource;
use common_game::protocols::orchestrator_explorer::ExplorerToOrchestrator;
use common_game::protocols::planet_explorer::PlanetToExplorer;
use crate::{payload, Explorer};
use crate::logging::log_debug;

impl Explorer {
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
                self.brain
                    .set_planet_basic_resources(self.planet_stats.id().unwrap_or(0), &resource_list);
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
                self.brain.set_planet_complex_resources(
                    self.planet_stats.id().unwrap_or(0),
                    &combination_list,
                );
                self.planet_stats
                    .update_combinations(combination_list);
            }
            PlanetToExplorer::GenerateResourceResponse { resource } => {
                let generated = if let Some(r) = resource {
                    let resource_type = r.get_type();
                    self.brain
                        .insert_resource(GenericResource::BasicResources(r));
                    log_debug(
                        payload!(action : "Nico generated a basic resource", explorer_id : self.id, basic_resource : format!("{:?}", resource_type)),
                    );
                    Ok(())
                } else {
                    //self.brain.on_no_action();
                    log_debug(
                        payload!(action : "Planet did not generate basic resource for Nico", explorer_id : self.id),
                    );
                    Ok(())
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
                            payload!(action : "Nico generated a complex resource", explorer_id : self.id, basic_resource : format!("{:?}", r.get_type())),
                        );
                        self.brain
                            .insert_resource(GenericResource::ComplexResources(r));
                        Ok(())
                    }
                    Err((error, r1, r2)) => {
                        self.brain.reinsert_resource(r1);
                        self.brain.reinsert_resource(r2);
                        //self.brain.on_no_action();
                        log_debug(
                            payload!(action : "Planet did not combine complex resource for Nico", explorer_id : self.id),
                        );
                        Err(error)
                    }
                };
                // Send to Orchestrator only if it is in manual mode
                if self.manual_mode {
                    self.to_orchestrator(ExplorerToOrchestrator::CombineResourceResponse {
                        explorer_id: self.id,
                        generated,
                    })?;
                }
            }
            // Ignore others
            PlanetToExplorer::AvailableEnergyCellResponse { .. } | PlanetToExplorer::Stopped => {}
        }

        Ok(())
    }
}
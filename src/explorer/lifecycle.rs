use common_game::protocols::orchestrator_explorer::ExplorerToOrchestrator;
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use crate::{payload, Explorer};
use crate::brain::intention::Intention;
use crate::logging::{log_debug, log_info};

impl Explorer {
    /// Thinks of the next intention and executes it.
    ///
    /// Returns an error if any send of a message fails.
    pub(crate) fn execute_intention(&mut self) -> Result<(), String> {
        let intention = self.brain.think(self.planet_stats.id().unwrap_or(0));

        log_debug(payload!(
            intention : format!("Nico wants to: {intention:?}"),
            explorer_id: self.id,
        ));
        match intention {
            Intention::Generate(Some(resource)) => {
                self.to_planet(ExplorerToPlanet::GenerateResourceRequest {
                    explorer_id: self.id,
                    resource,
                })?;
                Ok(())
            }
            Intention::Combine(Some(request)) => {
                self.to_planet(ExplorerToPlanet::CombineResourceRequest {
                    explorer_id: self.id,
                    msg: request,
                })?;
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
                    action : "Nico could not find a Planet to move into. Feels lonely here.",
                    explorer_id : self.id
                ));
                //self.brain.got_blocked();
                // Try asking for neighbors again, maybe we are not updated
                if !self.planet_stats.has_neighbors()
                    && let Some(planet_id) = self.planet_stats.id()
                {
                    self.to_orchestrator(ExplorerToOrchestrator::NeighborsRequest {
                        explorer_id: self.id,
                        current_planet_id: planet_id,
                    })?;
                } else {
                    self.brain.set_planet_neighbors(
                        self.planet_stats.id().unwrap_or(0),
                        self.planet_stats.neighbors().cloned().unwrap_or_default(),
                    );
                }
                Ok(())
            }
            _ => {
                //self.brain.on_no_action();
                Ok(())
            }
        }
    }

    /// Resets the Explorer
    ///
    /// Returns an error if any send of a message fails.
    /// Otherwise, returns `Ok(false)`, indicating that the Explorer has not been killed.
    pub(crate) fn reset(&mut self) -> Result<bool, String> {
        // TODO: actually reset AI
        self.manual_mode = true;
        self.path.clear();
        self.to_orchestrator(ExplorerToOrchestrator::ResetExplorerAIResult {
            explorer_id: self.id,
        })?;
        log_info(payload!(
            action : "Nico ExplorerAI correctly reset",
            explorer_id : self.id
        ));
        Ok(false)
    }

    /// Stops the Explorer, transitioning it to manual mode.
    ///
    /// Returns an error if any send of a message fails.
    /// Otherwise, returns `Ok(false)`, indicating that the Explorer has not been killed.
    pub(crate) fn stop(&mut self) -> Result<bool, String> {
        self.manual_mode = true;
        self.to_orchestrator(ExplorerToOrchestrator::StopExplorerAIResult {
            explorer_id: self.id,
        })?;
        log_info(payload!(
            action : "Nico switched to manual mode",
            explorer_id : self.id,
        ));
        Ok(false)
    }

    /// Starts the Explorer, transitioning it to AI mode.
    ///
    /// Returns an error if any send of a message fails.
    /// Otherwise, returns `Ok(false)`, indicating that the Explorer has not been killed.
    pub(crate) fn start(&mut self) -> Result<bool, String> {
        self.manual_mode = false;
        self.to_orchestrator(ExplorerToOrchestrator::StartExplorerAIResult {
            explorer_id: self.id,
        })?;
        log_info(payload!(action : "Nico ExplorerAI correctly started", explorer_id : self.id));

        // Ask Planet for its supported resources and combinations
        self.to_planet(ExplorerToPlanet::SupportedResourceRequest {
            explorer_id: self.id,
        })?;

        self.to_planet(ExplorerToPlanet::SupportedCombinationRequest {
            explorer_id: self.id,
        })?;

        Ok(false)
    }

    /// Kills the Explorer
    ///
    /// Returns an error if any send of a message fails.
    /// Otherwise, returns `Ok(true)`, indicating that the Explorer has been killed.
    pub(crate) fn kill(&mut self) -> Result<bool, String> {
        log_info(payload!(
            action : "Nico has been killed, bye bye :(",
            explorer_id : self.id,
            performance : self.brain.performance(),
            bag_content : format!("{:?}", self.brain.bag_content()),
        ));
        self.path.clear();
        self.to_orchestrator(ExplorerToOrchestrator::KillExplorerResult {
            explorer_id: self.id,
        })?;
        Ok(true)
    }
}
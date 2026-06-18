use crate::logging::{log_error, log_to_orchestrator, log_to_planet, log_warning};
use crate::{Explorer, payload};
use common_explorer::ExplorerBagContent;
use common_game::protocols::orchestrator_explorer::ExplorerToOrchestrator;
use common_game::protocols::planet_explorer::ExplorerToPlanet;

impl Explorer {
    /// Sends a `ExplorerToOrchestrator` message to the Orchestrator.
    ///
    /// Returns an error if the send fails.
    pub(crate) fn to_orchestrator(
        &self,
        msg: ExplorerToOrchestrator<ExplorerBagContent>,
    ) -> Result<(), String> {
        log_to_orchestrator(
            self.id,
            payload!(
                action : "Nico sending to the Orchestrator",
                explorer_id : self.id,
                msg : format!("{msg:?}")
            )
        );
        let res = self
            .orchestrator_sender
            .send(msg)
            .map_err(|err| err.to_string());
        if let Err(ref e) = res {
            log_error(payload!(
                action : "Nico got an error while trying to send to the Orchestrator",
                explorer_id : self.id,
                error : e,
            ));
        }
        res
    }

    /// Sends a `ExplorerToPlanet` message to the current Planet.
    ///
    /// Returns an error if the send fails, or if Nico does not have a Planet sender.
    pub(crate) fn to_planet(&self, msg: ExplorerToPlanet) -> Result<(), String> {
        if let Some(ref sender) = self.planet_stats.sender() {
            log_to_planet(
                self.id,
                self.planet_stats.id().unwrap_or(0),
                payload!(
                    action : "Nico sending to Planet",
                    explorer_id : self.id,
                    planet : format!("{:?}", self.planet_stats.id()),
                    msg : format!("{msg:?}")
                )
            );
            sender.send(msg).map_err(|err| err.to_string())
        } else {
            log_warning(
                payload!(action : "Nico does not have a Planet sender", explorer_id : self.id),
            );
            Ok(())
        }
    }
}

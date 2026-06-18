use common_game::protocols::planet_explorer::ExplorerToPlanet;
use common_game::utils::ID;
use crossbeam_channel::Sender;

pub(crate) struct PlanetStats {
    id: Option<ID>,
    sender: Option<Sender<ExplorerToPlanet>>,
    neighbors: Option<Vec<ID>>,
}

impl PlanetStats {
    pub(crate) fn new() -> Self {
        Self {
            id: None,
            sender: None,
            neighbors: None,
        }
    }

    /// Updates the neighbors of the Planet
    pub(crate) fn update_neighbors(&mut self, neighbors: Vec<ID>) {
        self.neighbors = Some(neighbors);
    }

    /// Updates the ID and the Sender of the Planet
    pub(crate) fn update_planet(&mut self, id: ID, sender: Sender<ExplorerToPlanet>) {
        self.id = Some(id);
        self.sender = Some(sender);
        self.neighbors = None;
    }

    /// Returns an `Option<&Vec<ID>>` of the Planet neighbors.
    pub(crate) fn neighbors(&self) -> Option<&Vec<ID>> {
        self.neighbors.as_ref()
    }

    /// Returns the Planet ID.
    pub(crate) fn id(&self) -> Option<ID> {
        self.id
    }

    /// Returns `true` if neighbors is `Some`, otherwise, `false`
    pub(crate) fn has_neighbors(&self) -> bool {
        self.neighbors.is_some()
    }

    /// Returns the Planet Sender
    pub(crate) fn sender(&self) -> Option<Sender<ExplorerToPlanet>> {
        self.sender.clone()
    }
}

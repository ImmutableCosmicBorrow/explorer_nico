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

    pub(crate) fn update_neighbors(&mut self, neighbors: Vec<ID>) {
        self.neighbors = Some(neighbors);
    }

    pub(crate) fn update_planet(&mut self, id: ID, sender: Sender<ExplorerToPlanet>) {
        self.id = Some(id);
        self.sender = Some(sender);
        self.neighbors = None;
    }

    pub(crate) fn neighbors(&self) -> Option<&Vec<ID>> {
        self.neighbors.as_ref()
    }

    pub(crate) fn id(&self) -> Option<ID> {
        self.id
    }

    pub(crate) fn has_neighbors(&self) -> bool {
        self.neighbors.is_some()
    }

    pub(crate) fn sender(&self) -> Option<Sender<ExplorerToPlanet>> {
        self.sender.clone()
    }
}

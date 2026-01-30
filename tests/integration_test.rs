mod tests {
    use common_explorer::{ExplorerAI, ExplorerBagContent};
    use common_game::protocols::orchestrator_explorer::{
        ExplorerToOrchestrator, OrchestratorToExplorer,
    };
    use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
    use common_game::utils::ID;
    use crossbeam_channel::unbounded;
    use explorer_nico::Explorer;
    use std::thread;
    use std::time::Duration;
    static EXPLORER_ID: ID = 266;
    static PLANET_ID: ID = 222;
    static PLANET_2_ID: ID = 333;

    #[test]
    fn test_creation_start_kill() {
        // 0. Channels
        let (tx_eto, rx_eto) = unbounded::<ExplorerToOrchestrator<ExplorerBagContent>>();
        let (_tx_etp, _rx_etp) = unbounded::<ExplorerToPlanet>();
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

        // 3. Send Start
        tx_ote
            .send(OrchestratorToExplorer::StartExplorerAI)
            .expect("Error while sending to the Explorer");

        // 4. Check response
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::StartExplorerAIResult {
                explorer_id: EXPLORER_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // 4. Send Kill
        tx_ote
            .send(OrchestratorToExplorer::KillExplorer)
            .expect("Error while sending to the Explorer");

        // 5. Check response
        let response = rx_eto.recv().expect("Error while receiving from Explorer");
        let expected: ExplorerToOrchestrator<ExplorerBagContent> =
            ExplorerToOrchestrator::KillExplorerResult {
                explorer_id: EXPLORER_ID,
            };
        assert_eq!(format!("{response:?}"), format!("{expected:?}"));

        // Last. Join thread
        handle.join().expect("Error while joining explorer thread");
    }

    mod manual_mode {
        use super::*;

        #[test]
        fn test_move_to_planet() {
            // 0. Channels
            let (tx_eto, rx_eto) = unbounded::<ExplorerToOrchestrator<ExplorerBagContent>>();
            let (tx_etp, rx_etp) = unbounded::<ExplorerToPlanet>();
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

            // 5. Explorer should ask Orchestrator for neighbors and Planet for its resources and combinations
            let response1 = rx_eto.recv().expect("Error while receiving from Explorer");
            let response2 = rx_etp.recv().expect("Error while receiving from Explorer");
            let response3 = rx_etp.recv().expect("Error while receiving from Explorer");

            let expected1: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::NeighborsRequest {
                    explorer_id: EXPLORER_ID,
                    current_planet_id: PLANET_ID,
                };
            let expected2: ExplorerToPlanet = ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: EXPLORER_ID,
            };
            let expected3: ExplorerToPlanet = ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: EXPLORER_ID,
            };

            assert_eq!(format!("{response1:?}"), format!("{expected1:?}"));
            assert_eq!(format!("{response2:?}"), format!("{expected2:?}"));
            assert_eq!(format!("{response3:?}"), format!("{expected3:?}"));

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

    mod ai_mode {
        use super::*;
        use common_game::components::resource::BasicResourceType;
        use crossbeam_channel::select;
        use std::collections::HashSet;

        #[test]
        fn test_generate_intentions() {
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

            // 4. Send Start
            tx_ote
                .send(OrchestratorToExplorer::StartExplorerAI)
                .expect("Error while sending to the Explorer");

            // 5. Check response
            let response = rx_eto
                .recv()
                .expect("Error while receiving StartExplorerResult from Explorer");
            let expected: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::StartExplorerAIResult {
                    explorer_id: EXPLORER_ID,
                };
            assert_eq!(format!("{response:?}"), format!("{expected:?}"));

            // 6. Send MoveToPlanet
            tx_ote
                .send(OrchestratorToExplorer::MoveToPlanet {
                    planet_id: PLANET_ID,
                    sender_to_new_planet: Some(tx_etp),
                })
                .expect("Error while sending to the Explorer");

            // 7. Explorer should respond with MovedToPlanetResult
            let response = rx_eto
                .recv()
                .expect("Error while receiving MovedToPlanetResult from Explorer");
            let expected: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::MovedToPlanetResult {
                    explorer_id: EXPLORER_ID,
                    planet_id: PLANET_ID,
                };
            assert_eq!(format!("{response:?}"), format!("{expected:?}"));

            // 8. Explorer should ask Orchestrator for neighbors and Planet for its resources and combinations
            let response1 = rx_eto.recv().expect("Error while receiving from Explorer");
            let response2 = rx_etp.recv().expect("Error while receiving from Explorer");
            let response3 = rx_etp.recv().expect("Error while receiving from Explorer");

            let expected1: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::NeighborsRequest {
                    explorer_id: EXPLORER_ID,
                    current_planet_id: PLANET_ID,
                };
            let expected2: ExplorerToPlanet = ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: EXPLORER_ID,
            };
            let expected3: ExplorerToPlanet = ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: EXPLORER_ID,
            };

            assert_eq!(format!("{response1:?}"), format!("{expected1:?}"));
            assert_eq!(format!("{response2:?}"), format!("{expected2:?}"));
            assert_eq!(format!("{response3:?}"), format!("{expected3:?}"));

            // 9. Planet tells Explorer that it can craft Carbon
            let mut resource_list = HashSet::new();
            resource_list.insert(BasicResourceType::Carbon);
            tx_pte
                .send(PlanetToExplorer::SupportedResourceResponse { resource_list })
                .expect("Error while sending to the Explorer");

            // 9. Wait for Explorer Intention. It should be a Generate(Carbon), so it should ask the Planet to generate carbon
            let expected = ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: EXPLORER_ID,
                resource: BasicResourceType::Carbon,
            };

            select! {
                recv(rx_eto) -> msg => panic!("Expected to receive from Planet, but received {msg:?}"),
                recv(rx_etp) -> msg => assert_eq!(format!("{:?}", msg.expect("Error while receiving from Explorer")), format!("{expected:?}")),
                default(Duration::from_secs(50)) => panic!{"Explorer message reception timed out"},
            }

            // Last-2. Send Kill
            tx_ote
                .send(OrchestratorToExplorer::KillExplorer)
                .expect("Error while sending to the Explorer");

            // Last-1. Check response
            let response = rx_eto
                .recv()
                .expect("Error while receiving KillExplorerResult from Explorer");
            let expected: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::KillExplorerResult {
                    explorer_id: EXPLORER_ID,
                };
            assert_eq!(format!("{response:?}"), format!("{expected:?}"));

            // Last. Join thread
            handle.join().expect("Error while joining explorer thread");
        }
        #[test]
        fn test_move_intentions() {
            // 0. Channels
            let (tx_eto, rx_eto) = unbounded::<ExplorerToOrchestrator<ExplorerBagContent>>();
            let (tx_etp, rx_etp) = unbounded::<ExplorerToPlanet>();
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

            // 4. Send Start
            tx_ote
                .send(OrchestratorToExplorer::StartExplorerAI)
                .expect("Error while sending to the Explorer");

            // 5. Check response
            let response = rx_eto
                .recv()
                .expect("Error while receiving StartExplorerResult from Explorer");
            let expected: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::StartExplorerAIResult {
                    explorer_id: EXPLORER_ID,
                };
            assert_eq!(format!("{response:?}"), format!("{expected:?}"));

            // 6. Send MoveToPlanet
            tx_ote
                .send(OrchestratorToExplorer::MoveToPlanet {
                    planet_id: PLANET_ID,
                    sender_to_new_planet: Some(tx_etp),
                })
                .expect("Error while sending to the Explorer");

            // 7. Explorer should respond with MovedToPlanetResult
            let response = rx_eto
                .recv()
                .expect("Error while receiving MovedToPlanetResult from Explorer");
            let expected: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::MovedToPlanetResult {
                    explorer_id: EXPLORER_ID,
                    planet_id: PLANET_ID,
                };
            assert_eq!(format!("{response:?}"), format!("{expected:?}"));

            // 8. Explorer should ask Orchestrator for neighbors and Planet for its resources and combinations
            let response1 = rx_eto.recv().expect("Error while receiving from Explorer");
            let response2 = rx_etp.recv().expect("Error while receiving from Explorer");
            let response3 = rx_etp.recv().expect("Error while receiving from Explorer");

            let expected1: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::NeighborsRequest {
                    explorer_id: EXPLORER_ID,
                    current_planet_id: PLANET_ID,
                };
            let expected2: ExplorerToPlanet = ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: EXPLORER_ID,
            };
            let expected3: ExplorerToPlanet = ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: EXPLORER_ID,
            };

            assert_eq!(format!("{response1:?}"), format!("{expected1:?}"));
            assert_eq!(format!("{response2:?}"), format!("{expected2:?}"));
            assert_eq!(format!("{response3:?}"), format!("{expected3:?}"));

            // 9. Orchestrator tells Explorer the neighbors of the Planet
            let neighbors = vec![PLANET_2_ID];
            tx_ote
                .send(OrchestratorToExplorer::NeighborsResponse { neighbors })
                .expect("Error while sending to the Explorer");

            // 9. Wait for Explorer Intention. It should be a Move(PLANET_2_ID), so it should ask the Orchestrator to move
            let expected: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::TravelToPlanetRequest {
                    explorer_id: EXPLORER_ID,
                    current_planet_id: PLANET_ID,
                    dst_planet_id: PLANET_2_ID,
                };

            select! {
                recv(rx_eto) -> msg => assert_eq!(format!("{:?}", msg.expect("Error while receiving from Explorer")), format!("{expected:?}")),
                recv(rx_etp) -> msg => panic!("Expected to receive from Planet, but received {msg:?}"),
                default(Duration::from_secs(50)) => panic!{"Explorer message reception timed out"},
            }

            // Last-2. Send Kill
            tx_ote
                .send(OrchestratorToExplorer::KillExplorer)
                .expect("Error while sending to the Explorer");

            // Last-1. Check response
            let response = rx_eto
                .recv()
                .expect("Error while receiving KillExplorerResult from Explorer");
            let expected: ExplorerToOrchestrator<ExplorerBagContent> =
                ExplorerToOrchestrator::KillExplorerResult {
                    explorer_id: EXPLORER_ID,
                };
            assert_eq!(format!("{response:?}"), format!("{expected:?}"));

            // Last. Join thread
            handle.join().expect("Error while joining explorer thread");
        }
    }
}

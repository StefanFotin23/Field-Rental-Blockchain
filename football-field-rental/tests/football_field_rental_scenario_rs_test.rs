use multiversx_sc_scenario::imports::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("mxsc:output/football-field-rental.mxsc.json", football_field_rental::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/football_field_rental.scen.json");
}

#[test]
fn full_scenario_rs() {
    world().run("scenarios/full_scenario.scen.json");
}

#[test]
fn cancel_scenario_rs() {
    let mut world = world();
    world.current_block().block_timestamp(0); // Set block timestamp for early cancellation
    world.run("scenarios/cancel_scenario.scen.json");
}

#[test]
fn fail_cancel_scenario_rs() {
    world().run("scenarios/fail_cancel_scenario.scen.json");
}

#[test]
fn late_cancel_penalty_scenario_rs() {
    let mut world = world();
    world.current_block().block_timestamp(900); // Set block timestamp for late cancellation
    world.run("scenarios/late_cancel_penalty_scenario.scen.json");
}

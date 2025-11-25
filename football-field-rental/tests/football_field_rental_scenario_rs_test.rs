use multiversx_sc_scenario::imports::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    // blockchain.set_current_dir_from_workspace("relative path to your workspace, if applicable");
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

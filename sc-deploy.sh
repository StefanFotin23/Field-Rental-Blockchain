#!/bin/bash

cd football-field-rental
sc-meta all build

source devnet.snippets.sh
deploySC

cp output/football_field_rental_proxy.rs interactor/src/proxy.rs
cd interactor
sc-meta all snippets --overwrite
cargo run deploy
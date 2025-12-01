#!/bin/bash

set -e

cd football-field-rental
sc-meta all build

source devnet.snippets.sh
deploySC

# Extract the contract address from deploy.txt
SC_ADDRESS=$(grep '"contractAddress"' ../deploy.txt | awk -F'"' '{print $$4}')
echo "Deployed Smart Contract Address: $SC_ADDRESS"

# Update the Makefile with the new SC_ADDRESS
# Need to go back to the root directory to modify the Makefile
cd ..
sed -i 's/^SC_ADDRESS = ".*"$$/SC_ADDRESS = "'"$SC_ADDRESS"'"/' Makefile

# Go back to football-field-rental for interactor deployment
cd football-field-rental

cp output/football_field_rental_proxy.rs interactor/src/proxy.rs
cd interactor
sc-meta all snippets --overwrite
cargo run deploy

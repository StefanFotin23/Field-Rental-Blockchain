# Makefile for football-field-rental smart contract

# ----------------------------
# Variables
# ----------------------------

# Wallet PEM file
WALLET = "/mnt/c/Users/andre/Desktop/BPDA/wallet.pem"

# Proxy for devnet
PROXY = "https://devnet-gateway.multiversx.com"

# Smart Contract Address (replace with your deployed contract address)
SC_ADDRESS = "erd1qqqqqqqqqqqqqpgqz9wac2msv85jfvwfguv9tcr2rt6tvz09n2jsktsdk6"

# ----------------------------
# Build
# ----------------------------

.PHONY: build
build:
	cd football-field-rental && sc-meta all build

# ----------------------------
# Owner Interactions
# ----------------------------

.PHONY: owner-withdraw
owner-withdraw:
	mxpy --verbose contract call $(SC_ADDRESS) \
		--function="withdraw" \
		--pem=$(WALLET) \
		--proxy=$(PROXY) \
		--send

.PHONY: owner-set-manager
owner-set-manager:
	mxpy --verbose contract call $(SC_ADDRESS) \
		--function="setFootballFieldManager" \
		--arguments $(MANAGER_ADDRESS) \
		--pem=$(WALLET) \
		--proxy=$(PROXY) \
		--send

.PHONY: owner-set-court-cost
owner-set-court-cost:
	mxpy --verbose contract call $(SC_ADDRESS) \
		--function="setFootballCourtCost" \
		--arguments $(COST) \
		--pem=$(WALLET) \
		--proxy=$(PROXY) \
		--send

.PHONY: owner-confirm-slot
owner-confirm-slot:
	mxpy --verbose contract call $(SC_ADDRESS) \
		--function="confirmSlot" \
		--pem=$(WALLET) \
		--proxy=$(PROXY) \
		--send

# ----------------------------
# User Interactions
# ----------------------------

.PHONY: user-create-slot
user-create-slot:
	mxpy --verbose contract call $(SC_ADDRESS) \
		--function="createFootballSlot" \
		--arguments $(START) $(END) \
		--value 100000000000000000 \
		--pem=$(WALLET) \
		--proxy=$(PROXY) \
		--send

.PHONY: user-participate
user-participate:
	mxpy --verbose contract call $(SC_ADDRESS) \
		--function="participateToFootballSlot" \
		--value 100000000000000000 \
		--pem=$(WALLET) \
		--proxy=$(PROXY) \
		--send

.PHONY: user-cancel
user-cancel:
	mxpy --verbose contract call $(SC_ADDRESS) \
		--function="cancelFootballSlot" \
		--pem=$(WALLET) \
		--proxy=$(PROXY) \
		--send

.PHONY: user-pay-court
user-pay-court:
	mxpy --verbose contract call $(SC_ADDRESS) \
		--function="payCourt" \
		--pem=$(WALLET) \
		--proxy=$(PROXY) \
		--send

# ----------------------------
# Views
# ----------------------------

.PHONY: get-slot-status
get-slot-status:
	mxpy --verbose contract query $(SC_ADDRESS) \
		--function="getSlotStatus" \
		--proxy=$(PROXY)
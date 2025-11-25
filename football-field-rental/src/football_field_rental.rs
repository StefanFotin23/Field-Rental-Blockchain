#![no_std]

multiversx_sc::imports!();

pub const DEPOSIT_AMOUNT: u64 = 100_000_000_000_000_000; // 0.1 EGLD

#[multiversx_sc::contract]
pub trait FootballFieldRental {
    #[init]
    fn init(&self) {
        self.slot_is_reserved().set(false);
    }

    #[upgrade]
    fn upgrade(&self) {}

    // Storage
    #[storage_mapper("footballFieldManagerAddress")]
    fn football_field_manager_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("footballCourtCost")]
    fn football_court_cost(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("participants")]
    fn participants(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("slotIsReserved")]
    fn slot_is_reserved(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("slotStart")]
    fn slot_start(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("slotEnd")]
    fn slot_end(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("slotInitiator")]
    fn slot_initiator(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("slotAmountCollected")]
    fn slot_amount_collected(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("slotConfirmed")]
    fn slot_confirmed(&self) -> SingleValueMapper<bool>;

    // Events
    #[event("slotCreated")]
    fn slot_created_event(
        &self,
        #[indexed] initiator: &ManagedAddress,
        #[indexed] start: u64,
        #[indexed] end: u64,
    );

    #[event("participantAdded")]
    fn participant_added_event(
        &self,
        #[indexed] participant: &ManagedAddress,
    );

    #[event("slotCancelled")]
    fn slot_cancelled_event(&self);

    #[event("managerSet")]
    fn manager_set_event(
        &self,
        #[indexed] manager_address: &ManagedAddress,
    );

    #[event("courtCostSet")]
    fn court_cost_set_event(
        &self,
        #[indexed] cost: &BigUint,
    );

    #[event("slotConfirmed")]
    fn slot_confirmed_event(&self);

    #[event("paymentMade")]
    fn payment_made_event(
        &self,
        #[indexed] manager_address: &ManagedAddress,
        #[indexed] amount: &BigUint,
    );

    // Endpoints
    #[payable("EGLD")]
    #[endpoint]
    fn createFootballSlot(&self, start: u64, end: u64) {
        let payment = self.call_value().egld().clone_value();
        require!(payment == BigUint::from(DEPOSIT_AMOUNT), "Deposit must be 0.1 EGLD");
        require!(!self.slot_is_reserved().get(), "A slot is already reserved");

        let caller = self.blockchain().get_caller();
        self.slot_is_reserved().set(true);
        self.slot_start().set(start);
        self.slot_end().set(end);
        self.slot_initiator().set(caller.clone());
        self.slot_amount_collected().set(payment);
        self.slot_confirmed().set(false);

        self.participants().insert(caller.clone());
        self.slot_created_event(&caller, start, end);
    }

    #[payable("EGLD")]
    #[endpoint]
    fn participateToFootballSlot(&self) {
        let payment = self.call_value().egld().clone_value();
        require!(payment == BigUint::from(DEPOSIT_AMOUNT), "Deposit must be 0.1 EGLD");
        require!(self.slot_is_reserved().get(), "No slot is currently reserved");

        let caller = self.blockchain().get_caller();
        require!(!self.participants().contains(&caller), "You are already a participant");

        self.slot_amount_collected().update(|amount| *amount += payment);
        self.participants().insert(caller.clone());

        self.participant_added_event(&caller);
    }

    #[endpoint]
    fn cancelFootballSlot(&self) {
        require!(self.slot_is_reserved().get(), "No slot to cancel");
        let caller = self.blockchain().get_caller();
        let initiator = self.slot_initiator().get();
        require!(caller == initiator, "Only the initiator can cancel the slot");
        require!(!self.slot_confirmed().get(), "Cannot cancel a confirmed slot");

        let participants_mapper = self.participants();
        let amount_per_participant = BigUint::from(DEPOSIT_AMOUNT);

        for participant in participants_mapper.iter() {
            self.send().direct_egld(&participant, &amount_per_participant);
        }

        self.clear_slot();
        self.slot_cancelled_event();
    }

    #[endpoint]
    #[only_owner]
    fn setFootballFieldManager(&self, manager_address: ManagedAddress) {
        self.football_field_manager_address().set(&manager_address);
        self.manager_set_event(&manager_address);
    }

    #[endpoint]
    fn payCourt(&self) {
        require!(self.slot_is_reserved().get(), "No slot to pay for");
        require!(self.slot_confirmed().get(), "Slot is not confirmed");

        let manager = self.football_field_manager_address().get();
        let cost = self.football_court_cost().get();
        require!(!manager.is_zero(), "Football field manager is not set");
        require!(cost > 0, "Football court cost is not set");

        let amount_collected = self.slot_amount_collected().get();
        require!(amount_collected >= cost, "Not enough funds collected to pay the court");

        self.send().direct_egld(&manager, &cost);

        // Refund remaining amount to participants
        let remaining_amount = amount_collected - &cost;
        if remaining_amount > 0 {
            let participants_mapper = self.participants();
            let num_participants = participants_mapper.len() as u64;
            if num_participants > 0 {
                let refund_per_participant = remaining_amount / num_participants;
                if refund_per_participant > 0 {
                    for participant in participants_mapper.iter() {
                        self.send().direct_egld(&participant, &refund_per_participant);
                    }
                }
            }
        }

        self.payment_made_event(&manager, &cost);
        self.clear_slot();
    }

    #[endpoint]
    #[only_owner]
    fn setFootballCourtCost(&self, cost: BigUint) {
        self.football_court_cost().set(&cost);
        self.court_cost_set_event(&cost);
    }

    #[endpoint]
    fn confirmSlot(&self) {
        let caller = self.blockchain().get_caller();
        let owner = self.blockchain().get_owner_address();
        let manager = self.football_field_manager_address().get();
        require!(caller == owner || caller == manager, "Only owner or manager can confirm");

        require!(self.slot_is_reserved().get(), "No slot to confirm");
        require!(!self.slot_confirmed().get(), "Slot is already confirmed");

        self.slot_confirmed().set(true);
        self.slot_confirmed_event();
    }

    #[view(getSlotStatus)]
    fn get_slot_status(&self) -> (u64, u64, ManagedAddress, BigUint, bool, ManagedVec<ManagedAddress>) {
        if !self.slot_is_reserved().get() {
            return (0, 0, ManagedAddress::zero(), BigUint::zero(), false, ManagedVec::new());
        }

        (
            self.slot_start().get(),
            self.slot_end().get(),
            self.slot_initiator().get(),
            self.slot_amount_collected().get(),
            self.slot_confirmed().get(),
            self.participants().iter().collect(),
        )
    }

    #[endpoint]
    fn clear_slot(&self) {
        self.slot_is_reserved().set(false);
        self.slot_start().clear();
        self.slot_end().clear();
        self.slot_initiator().clear();
        self.slot_amount_collected().clear();
        self.slot_confirmed().clear();
        self.participants().clear();
    }
}

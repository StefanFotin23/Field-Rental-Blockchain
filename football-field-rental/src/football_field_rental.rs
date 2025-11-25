#![no_std]

multiversx_sc::imports!();

pub const DEPOSIT_AMOUNT: u64 = 100_000_000_000_000_000; // 0.1 EGLD

#[derive(TypeAbi, NestedEncode, NestedDecode, TopEncode, TopDecode)]
pub struct Slot<M: ManagedTypeApi> {
    pub start: u64,
    pub end: u64,
    pub initiator: ManagedAddress<M>,
    pub amount_collected: BigUint<M>,
    pub confirmed: bool,
}

#[multiversx_sc::contract]
pub trait FootballFieldRental {
    #[init]
    fn init(&self) {
        self.reserved_slot().set_if_empty(&None);
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

    #[storage_mapper("reservedSlot")]
    fn reserved_slot(&self) -> SingleValueMapper<Option<Slot<Self::Api>>>;

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
        let payment: BigUint = self.call_value().egld_value().clone_value();
        require!(payment == BigUint::from(DEPOSIT_AMOUNT), "Deposit must be 0.1 EGLD");
        require!(self.reserved_slot().get().is_none(), "A slot is already reserved");

        let caller = self.blockchain().get_caller();
        let slot = Slot {
            start,
            end,
            initiator: caller.clone(),
            amount_collected: payment,
            confirmed: false,
        };

        self.reserved_slot().set(Some(slot));
        self.participants().insert(caller.clone());

        self.slot_created_event(&caller, start, end);
    }

    #[payable("EGLD")]
    #[endpoint]
    fn participateToFootballSlot(&self) {
        let payment: BigUint = self.call_value().egld_value().clone_value();
        require!(payment == BigUint::from(DEPOSIT_AMOUNT), "Deposit must be 0.1 EGLD");
        let optional_slot = self.reserved_slot().get();
        require!(optional_slot.is_some(), "No slot is currently reserved");

        let caller = self.blockchain().get_caller();
        require!(!self.participants().contains(&caller), "You are already a participant");

        let mut slot = optional_slot.unwrap();
        slot.amount_collected += &payment;
        self.reserved_slot().set(Some(slot));
        self.participants().insert(caller.clone());

        self.participant_added_event(&caller);
    }

    #[endpoint]
    fn cancelFootballSlot(&self) {
        let optional_slot = self.reserved_slot().get();
        require!(optional_slot.is_some(), "No slot to cancel");
        let slot = optional_slot.unwrap();
        let caller = self.blockchain().get_caller();
        require!(caller == slot.initiator, "Only the initiator can cancel the slot");
        require!(!slot.confirmed, "Cannot cancel a confirmed slot");

        let participants_mapper = self.participants();
        let amount_per_participant = BigUint::from(DEPOSIT_AMOUNT);

        for participant in participants_mapper.iter() {
            self.send().direct_egld(&participant, &amount_per_participant);
        }

        self.reserved_slot().clear();
        self.participants().clear();
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
        let optional_slot = self.reserved_slot().get();
        require!(optional_slot.is_some(), "No slot to pay for");
        let slot = optional_slot.unwrap();
        require!(slot.confirmed, "Slot is not confirmed");

        let manager = self.football_field_manager_address().get();
        let cost = self.football_court_cost().get();
        require!(!manager.is_zero(), "Football field manager is not set");
        require!(cost > 0, "Football court cost is not set");
        require!(slot.amount_collected >= cost, "Not enough funds collected to pay the court");

        self.send().direct_egld(&manager, &cost);

        // Refund remaining amount to participants
        let remaining_amount = slot.amount_collected - &cost;
        if remaining_amount > 0 {
            let participants_mapper = self.participants();
            let num_participants = participants_mapper.len() as u64;
            if num_participants > 0 {
                let refund_per_participant = remaining_amount / num_participants;
                for participant in participants_mapper.iter() {
                     self.send().direct_egld(&participant, &refund_per_participant);
                }
            }
        }


        self.payment_made_event(&manager, &cost);
        self.reserved_slot().clear();
        self.participants().clear();
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

        let optional_slot = self.reserved_slot().get();
        require!(optional_slot.is_some(), "No slot to confirm");
        let mut slot = optional_slot.unwrap();
        require!(!slot.confirmed, "Slot is already confirmed");

        slot.confirmed = true;
        self.reserved_slot().set(Some(slot));
        self.slot_confirmed_event();
    }

    #[view(getSlotStatus)]
    fn get_slot_status(&self) -> (Option<Slot<Self::Api>>, ManagedVec<ManagedAddress>, BigUint, bool) {
        let optional_slot = self.reserved_slot().get();
        if optional_slot.is_none() {
            (None, ManagedVec::new(), BigUint::zero(), false)
        } else {
            let slot = optional_slot.clone().unwrap();
            let participants = self.participants().iter().collect();
            let amount = slot.amount_collected.clone();
            let confirmed = slot.confirmed;
            (optional_slot, participants, amount, confirmed)
        }
    }
}

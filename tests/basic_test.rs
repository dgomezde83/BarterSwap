use exchange_place::*;
use multiversx_sc::types::Address;
use multiversx_sc_scenario::api::SingleTxApi;
use multiversx_sc_scenario::{scenario_model::*, *};

//Comments:
//When using .put_account, many times we use esdt_balance(tokenName: BytesKey, amount: BigUint). We could have used esdt_nft_balance(tokenName: BytesKey, nonce:u64, amount: BigUint) instead
//To simulate the balance of a certain NFT or SFT with a nonce
/*-------------------------------------------------------------------------*
*                                                                          *
*-------------------------------------------------------------------------*/
const EXCHANGE_PLACE_PATH_EXPR: &str = "file:output/exchange_place.wasm";
//const M_FEE: u64 = 20000000000000000;
const M_FEE: u64 = 0;
/*-------------------------------------------------------------------------*
*  World function to setup the scenario for the tests.                     *
*-------------------------------------------------------------------------*/
fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(EXCHANGE_PLACE_PATH_EXPR, exchange_place::ContractBuilder);
    blockchain
}
/*-------------------------------------------------------------------------*
* Structure to contain the elements necessary to call SC functions.        *
*-------------------------------------------------------------------------*/
#[derive(Debug, PartialEq)] // Automatically implement Debug and PartialEq traits
struct TestData {
    str_token_id: String,
    nonce: u64,

    amount: u64,
    amount_hex: String,
    amount_length_ascii_hex: String,

    price: u64,
    price_hex: String,
    price_length_ascii_hex: String,

    offer_id: u64,

    //Bech 32 string addresses: erd1...
    bidder_bech32_str: String,
    taker_bech32_str: String,

    //Bech32 encoded Address objects
    bidder_bech32: Address,
    taker_bech32: Address,

    //String hex addresses
    bidder_str: String,
    taker_str: String,
}
/*-------------------------------------------------------------------------*
* Constructor for the precedent struct.                                    *
*-------------------------------------------------------------------------*/
impl TestData {
    fn new(p_str_token_id: &str, nonce: u64, amount: u64, price: u64, offer_id: u64, p_bidder_bech32_str: &str, p_taker_bech32_str: &str) -> Self {

        let str_token_id = p_str_token_id.to_string();
        let bidder_bech32_str = p_bidder_bech32_str.to_string();
        let taker_bech32_str = p_taker_bech32_str.to_string();

        let amount_hex: String = format!("{:x}",amount);
        let amount_length_ascii_hex = amount_hex.len().to_string().chars().map(|c| format!("{:x}", c as u8)).collect::<String>();

        let price_hex: String = format!("{:x}",price);
        let price_length_ascii_hex = price_hex.len().to_string().chars().map(|c| format!("{:x}", c as u8)).collect::<String>();

        let bidder_bech32: Address = bech32::decode(p_bidder_bech32_str);
        let taker_bech32: Address = bech32::decode(p_taker_bech32_str);

        let bidder_str = multiversx_sc::formatter::hex_util::encode_bytes_as_hex(bidder_bech32.as_bytes()).to_string();
        let taker_str = multiversx_sc::formatter::hex_util::encode_bytes_as_hex(taker_bech32.as_bytes()).to_string();

        Self {
            str_token_id,
            nonce,
            amount,
            amount_hex,
            amount_length_ascii_hex,
            price,
            price_hex,
            price_length_ascii_hex,
            offer_id,
            bidder_bech32_str,
            taker_bech32_str,
            bidder_bech32,
            taker_bech32,
            bidder_str,
            taker_str,
        }
    }
}
/*-------------------------------------------------------------------------*
* Calls the init function of the smart contract from the provided address. *
*-------------------------------------------------------------------------*/
fn deploy_step(p_owner_address: &str, p_code_expression: &BytesValue) -> ScDeployStep
{
    ScDeployStep::new() //Create a new step of type ScDeployState: https://docs.rs/multiversx-sc-scenario/latest/multiversx_sc_scenario/scenario/model/struct.ScDeployStep.html
    .from(p_owner_address) //Address of the deployer
    .code(p_code_expression) //Can be set a code of value BytesValue
}
/*-------------------------------------------------------------------------*
* Calls the createOffer function from the SC.                              *
*-------------------------------------------------------------------------*/
fn call_create_offer(p_test_data: &TestData, p_fee: u64) -> ScCallStep
{
    ScCallStep::new()
    .from(AddressKey::from(&p_test_data.bidder_bech32)) //Address of the caller
    .to("sc:exchangeplace") //destination of the call (the smart contract)
    .esdt_transfer(BytesKey::from(p_test_data.str_token_id.clone().into_bytes()),p_test_data.nonce,BigUintValue::from(p_test_data.amount)) //Transfer the NFT or SFT
    .egld_value(BigUintValue::from(p_fee)) //Pay the fee
    .function("createOffer") //The name of the function
    .argument(BytesValue::from(p_test_data.offer_id.to_be_bytes().as_ref())) //ID of the offer
    .argument(BytesValue::from(p_test_data.price.to_be_bytes().as_ref())) //Price of the offer
    .argument(BytesValue::from(p_test_data.taker_bech32.as_bytes())) //Address of the taker
}
/*-------------------------------------------------------------------------*
* Calls the takeOffer function from the SC.                                *
*-------------------------------------------------------------------------*/
fn call_take_offer(p_test_data: &TestData, p_fee: u64) -> ScCallStep
{
    ScCallStep::new()
    .from(AddressKey::from(&p_test_data.taker_bech32)) //AddressValue of the caller
    .to("sc:exchangeplace") //destination of the call (the smart contract)
    .egld_value(BigUintValue::from(p_test_data.price + p_fee))
    .function("takeOffer") //The name of the function
    .argument(BytesValue::from(p_test_data.offer_id.to_be_bytes().as_ref())) //ID of the offer
    .argument(BytesValue::from(p_test_data.bidder_bech32.as_bytes())) //Address of the bidder
}
/*-------------------------------------------------------------------------*
* Calls the refundOffer function from the SC.                              *
*-------------------------------------------------------------------------*/
fn call_refund_offer(p_test_data: &TestData) -> ScCallStep
{
    ScCallStep::new()
    .from(AddressKey::from(&p_test_data.bidder_bech32)) //AddressValue of the caller
    .to("sc:exchangeplace") //Destination of the call (the smart contract)
    .function("refundOffer") //The name of the function
    .argument(BytesValue::from(p_test_data.offer_id.to_be_bytes().as_ref())) //ID of the offer
    .argument(BytesValue::from(p_test_data.taker_bech32.as_bytes())) //Address of the taker
}
/*-------------------------------------------------------------------------*
* Initialization test: deploy the contract.                                *
*-------------------------------------------------------------------------*/
#[test]
fn init_unit_test() {
    let t_exchanger = exchange_place::contract_obj::<SingleTxApi>();
    t_exchanger.init();
}
/*-------------------------------------------------------------------------*
* Create an offer and then take it.                                        *
*-------------------------------------------------------------------------*/
#[test]
fn create_and_take_offer_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         )
        .sc_call(
            call_take_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
        );
}
/*-------------------------------------------------------------------------*
* Place an offer with a huge price.                                        *
*-------------------------------------------------------------------------*/
#[test]
fn create_offer_max_price_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = u64::MAX;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         );
}
/*-------------------------------------------------------------------------*
* Place an offer with a huge ID.                                           *
*-------------------------------------------------------------------------*/
#[test]
fn create_offer_max_id_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = u64::MAX;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         );
}
/*-------------------------------------------------------------------------*
* Place an offer with a huge ID.                                           *
*-------------------------------------------------------------------------*/
#[test]
fn create_offer_max_nonce_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = u64::MAX;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_nft_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),t_nonce,BigUintValue::from(t_amount),Some(""))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         );
}
/*-------------------------------------------------------------------------*
* Create two times the same offer (same id and taker address).             *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "Element already present. Try with different ID.")]
fn create_offer_duplicated_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id_1 : &str = "PROPO-123456";
    let t_str_token_id_2 : &str = "PROPO-179101";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data_1 = TestData::new(t_str_token_id_1, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);
    let t_test_data_2 = TestData::new(t_str_token_id_2, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id_1.as_bytes().to_vec()),BigUintValue::from(t_amount)).esdt_balance(BytesKey::from(t_str_token_id_2.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data_1,M_FEE).expect(TxExpect::ok().no_result())
         )
        .sc_call(
            call_create_offer(&t_test_data_2,M_FEE).expect(TxExpect::ok().no_result())
        );
}
/*-------------------------------------------------------------------------*
* The bidder places a bid, and the taker asks for a refund on an offer     *
* that he didn't make (bad bidder address).                                *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "Refund offer not found.")]
fn refund_offer_bad_address_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);
    let t_refund_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_taker_address, t_taker_address); //TAKER makes the call on an offer he didn't create: SHOULD PANIC

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         )
        .sc_call(
            call_refund_offer(&t_refund_data)
        );
}
/*-------------------------------------------------------------------------*
* The bidder places a bid, and the taker asks for a refund on an offer     *
* that doesn't exist (bad offer ID).                                       *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "Refund offer not found.")]
fn refund_offer_bad_offerid_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);
    let t_refund_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id + 1, t_bidder_address, t_taker_address); //Calls with offerid + 1, an offer that doesn't exist

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         )
        .sc_call(
            call_refund_offer(&t_refund_data)
        );
}
/*-------------------------------------------------------------------------*
* The bidder places a bid, and the taker asks to take an offer with an     *
* unexistant offer id.                                                     *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "Take offer not found.")]
fn take_offer_bad_offerid_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;
    let t_offer_id_nonexistant: u64 = 2;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);
    let t_taker_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id_nonexistant, t_bidder_address, t_taker_address); //ID of the offer (unexistant one, should panic)

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         )
        .sc_call(
            call_take_offer(&t_taker_data,M_FEE)
        );
}
/*-------------------------------------------------------------------------*
* Try to take an offer, but providing a bad bidder address.                *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "Take offer not found.")]
fn take_offer_bad_address_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);
    let t_taker_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_taker_address, t_taker_address); //Provide taker's address as bidder. Generates error

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         )
        .sc_call(
            call_take_offer(&t_taker_data,M_FEE)
        );
}
/*-------------------------------------------------------------------------*
* Take an offer, but provided an incorrect payment in EGLD.                *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "Incorrect payment provided.")]
fn take_offer_invalid_payment_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);
    let t_taker_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price - 1, t_offer_id, t_bidder_address, t_taker_address); //Provide EGLD quantity - 1 (so not ENOUGH, PANIC)

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         )
        .sc_call(
            call_take_offer(&t_taker_data,M_FEE)
        );
}
/*-------------------------------------------------------------------------*
* Try to place an offer from a smart contract address (bidder). Normally   *
* the contract is deployed as Non-payable by smart contract, so this is for*
* pure testing.                                                            *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "Bidder address is from a smart contract.")]
fn create_offer_bidder_sc_address_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1qqqqqqqqqqqqqpgq5cfxcvq5dqp290j2q9gw5yc8fcremmlqplkqtly3rs";
    
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).code(EXCHANGE_PLACE_PATH_EXPR).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         );
}
/*-------------------------------------------------------------------------*
* Try to place an offer for a smart contract address (taker). Normally the *
* contract is deployed as Non-payable by smart contract, so this is for    *
* pure testing.                                                            *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "Taker address is from a smart contract.")]
fn create_offer_taker_sc_address_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd1qqqqqqqqqqqqqpgq5cfxcvq5dqp290j2q9gw5yc8fcremmlqplkqtly3rs";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).code(EXCHANGE_PLACE_PATH_EXPR).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         );
}
/*-------------------------------------------------------------------------*
* Create an offer and then take it.                                        *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "incorrect number of ESDT transfers")]
fn create_and_take_offer_no_nft_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            ScCallStep::new()
                .from(AddressKey::from(&t_test_data.bidder_bech32)) //Address of the caller
                .to("sc:exchangeplace") //destination of the call (the smart contract)
                //.esdt_transfer(BytesKey::from(t_test_data.str_token_id.clone().into_bytes()),t_test_data.nonce,BigUintValue::from(t_test_data.amount)) //Don't transfer the NFT or SFT
                .egld_value(BigUintValue::from(M_FEE)) //Pay the fee
                .function("createOffer") //The name of the function
                .argument(BytesValue::from(t_test_data.offer_id.to_be_bytes().as_ref())) //ID of the offer
                .argument(BytesValue::from(t_test_data.price.to_be_bytes().as_ref())) //Price of the offer
                .argument(BytesValue::from(t_test_data.taker_bech32.as_bytes())) //Address of the taker.expect(TxExpect::ok().no_result())
            )
        .sc_call(
            call_take_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
        );
}
/*-------------------------------------------------------------------------*
* Simultaneous call for taking and refunding an offer.                     *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "Refund offer not found.")]
fn create_and_take_and_refund_offer_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            call_create_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
         )
        .sc_call(
            call_take_offer(&t_test_data,M_FEE).expect(TxExpect::ok().no_result())
        )
        .sc_call(
            call_refund_offer(&t_test_data).expect(TxExpect::ok().no_result())
        );
}
/*-------------------------------------------------------------------------*
* Forbidden multi_esdt transfer on create.                                 *
*-------------------------------------------------------------------------*/
#[test]
#[should_panic(expected = "incorrect number of ESDT transfers")]
fn create_multi_esdt_offer_unit_test() {
    //std::env::set_var("RUST_BACKTRACE", "full");

    let t_owner_address : &str = "address:owner";

    let t_sc_address : &str = "sc:exchangeplace";

    let mut world = world();
    let exchange_place_code = world.code_expression(EXCHANGE_PLACE_PATH_EXPR); //BytesValue representing the wasm code

    let t_str_token_id : &str = "PROPO-123456";
    let t_nonce: u64 = 0;

    //BigUint amount
    let t_amount: u64 = 100000000000;

    //BigUint price
    let t_price: u64 = 700000000000;
    let t_offer_id: u64 = 1;

    //Addresses
    let t_bidder_address : &str = "erd1suej7d7yl5x95quuh38ur9x0vj2tdvy3rzuqx9n4dnulskyxvl0q0ec3n0";
    let t_taker_address : &str = "erd16jruked88jgtsar78ej85hjp3qsd9jkjcw4swsn7k0teqh3wgcqqgyrupq";

    let t_test_data = TestData::new(t_str_token_id, t_nonce, t_amount, t_price, t_offer_id, t_bidder_address, t_taker_address);

    let t_set_step = SetStateStep::new()
    .put_account(t_owner_address, Account::new().nonce(1)) //define address expression (str) and Account struct
    .new_address(t_owner_address, 1, t_sc_address) //define creator address expression (str), creator nonce (u64) and new address expression
    .put_account(AddressKey::from(&bech32::decode(t_bidder_address)), Account::new().nonce(0).esdt_balance(BytesKey::from(t_str_token_id.as_bytes().to_vec()),BigUintValue::from(2*t_amount))) //Into bytes must be used in order for the VM to correctly parse the token ID string
    .put_account(AddressKey::from(&bech32::decode(t_taker_address)), Account::new().nonce(0).balance(BigUintValue::from(t_price + M_FEE)));        

    // Create two instances of TxESDT
    let tx1 = TxESDT {
        esdt_token_identifier: BytesValue::from(BytesKey::from(t_test_data.str_token_id.clone().into_bytes())),
        nonce: U64Value::from(t_test_data.nonce),
        esdt_value: BigUintValue::from(BigUintValue::from(t_test_data.amount)),
    };

    let tx2 = TxESDT {
        esdt_token_identifier: BytesValue::from(BytesKey::from(t_test_data.str_token_id.clone().into_bytes())),
        nonce: U64Value::from(t_test_data.nonce),
        esdt_value: BigUintValue::from(BigUintValue::from(t_test_data.amount)),
    };

    // Create a vector and initialize it with tx1 and tx2
    let tx_vec: Vec<TxESDT> = vec![tx1, tx2];

    world.set_state_step(
            t_set_step
        )
        .sc_deploy( //deploy a step
            deploy_step(t_owner_address, &exchange_place_code).expect(TxExpect::ok().no_result()) //expect a TxExpect struct
        )
         .sc_call( //First call: place the bid
            ScCallStep::new()
            .from(AddressKey::from(&t_test_data.bidder_bech32)) //Address of the caller
            .to("sc:exchangeplace") //destination of the call (the smart contract)
            .multi_esdt_transfer(tx_vec) //Transfer the NFT or SFT
            .egld_value(BigUintValue::from(M_FEE)) //Pay the fee
            .function("createOffer") //The name of the function
            .argument(BytesValue::from(t_test_data.offer_id.to_be_bytes().as_ref())) //ID of the offer
            .argument(BytesValue::from(t_test_data.price.to_be_bytes().as_ref())) //Price of the offer
            .argument(BytesValue::from(t_test_data.taker_bech32.as_bytes())) //Address of the taker
         );
}

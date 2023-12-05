#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

//use multiversx_sc::types::heap::String;

mod structure_elements;

// A struct has to be annotated with the following to be serializable: https://docs.multiversx.com/developers/developer-reference/serialization-format
// Encoding:
// token id: 2 bytes for the length of the token (which is always 12 in decimal, so 'C' in hex, because ABCDE-123456 is 12 bytes long) + 12 bytes (1 bytes for each char of the token)
// Biguint: 2 bytes for the length of the big integer + n bytes (specified just before) for the actual big integer
// Managed address: 2 bytes for the length of the big integer + n bytes (specified just before) for the actual big integer

use structure_elements::{KeyElement,MarketplaceElement};

// FEE for creating and taking offers
const M_FEE: u64 = 0;

#[multiversx_sc::contract]
pub trait ExchangePlace {    
    // In the init, we don't need to initialize anything
    // https://docs.multiversx.com/developers/developer-reference/sc-annotations
    /*-------------------------------------------------------------------------*
    *                                                                          *
    *-------------------------------------------------------------------------*/
    #[init]
    fn init(&self){}
    /*-------------------------------------------------------------------------*
    *                                                                          *
    *-------------------------------------------------------------------------*/
    // Callable functions
    /*-------------------------------------------------------------------------*
    * List a certain amount of a token with a unique buyer address. Payable in *
    * any token (ESDT, NFT, SFT).                                              *
    * Input:                                                                   *
    * The offer id (u64) agreed by the bidder and taker.                       *
    * Price of the offer in EGLD.                                              *
    * Address of the taker of the offer.                                       *
    *-------------------------------------------------------------------------*/
    #[payable("*")]
    #[endpoint(createOffer)]
    fn create_offer(&self, p_offer_id: u64, p_price: BigUint, p_taker_address: ManagedAddress)
    {
        // Get received token. Signals an error if no transfer of ESDT/NFT/SFT has been done ("incorrect number of ESDT transfers")
        let t_esdt_structure: EsdtTokenPayment = self.call_value().single_esdt();

        // Get bidder address (the bidder is the caller)
        let t_bidder_address: ManagedAddress = self.blockchain().get_caller();

        // Insert new element into the map
        self.insert_element(t_esdt_structure, p_price, p_offer_id, t_bidder_address, p_taker_address);
    }
    /*-------------------------------------------------------------------------*
    * Refund an offer to the bidder. Should be called by the bidder of the     *
    * offer with the id he provided when creating it as well as the taker addr.*
    * Input:                                                                   *
    * u64 representing the ID.                                                 *
    * Address of the taker of the offer.                                       *
    *-------------------------------------------------------------------------*/
    #[endpoint(refundOffer)]
    fn refund_offer(&self, p_offer_id: u64, p_taker_address: ManagedAddress)
    {
        // Get caller address
        let t_bidder_address: ManagedAddress = self.blockchain().get_caller();

        // Search for the element
        match self.remove_element_by_key(p_offer_id.clone(), t_bidder_address.clone(), p_taker_address.clone()) {
            Some(t_marketplace_element) => {
                // Send the esdt token to the bidder
                self.send().direct_esdt(&t_bidder_address, &t_marketplace_element.get_collection_id(), t_marketplace_element.get_nonce(), &t_marketplace_element.get_amount()); 
            }
            None => {
                // Handle the case when the Option is empty
                sc_panic!("Refund offer not found.");
            }
        }
    }
    /*-------------------------------------------------------------------------*
    * Take an offer by ID. Should be called by the taker of the offer with the *
    * id the bidder defined, as well as the id the bidder gave to the offer.   *
    * Input:                                                                   *
    * u64 representing the ID.                                                 *
    * Address of the bidder of the offer.                                      *
    *-------------------------------------------------------------------------*/
    #[payable("EGLD")]
    #[endpoint(takeOffer)]
    fn take_offer(&self, p_offer_id: u64, p_bidder_address: ManagedAddress)
    {
        // Get caller address
        let t_taker_address: ManagedAddress = self.blockchain().get_caller();

        // Remove the element from the MapMapper and perform the transactions 
        match self.remove_element_by_key(p_offer_id.clone(), p_bidder_address.clone(), t_taker_address.clone()) {
            Some(t_removed_marketplace_element) => {
                require!((*self.call_value().egld_value()).eq(&(t_removed_marketplace_element.get_price().add(&BigUint::from(M_FEE)))), "Incorrect payment provided.");
                // Send the esdt token to the taker
                self.send().direct_esdt(&t_taker_address, &t_removed_marketplace_element.get_collection_id(), t_removed_marketplace_element.get_nonce(), &t_removed_marketplace_element.get_amount());
                // Send the EGLD to the bidder
                self.send().direct_egld(&p_bidder_address, &self.call_value().egld_value());
                // Send the fee to the contract deployer
                self.send().direct_egld(&self.blockchain().get_owner_address(),&BigUint::from(M_FEE));
            }
            None => {
                // Handle the case when the Option is empty
                sc_panic!("Take offer not found.");
            }
        }
    }
    /*-------------------------------------------------------------------------*
    * Finds an element in the hashmap provided the bidder address, the taker   *
    * address, and the id. This constitutes a key.                             *
    * Input:                                                                   *
    * u64 representing the ID.                                                 *
    * ManagedAddress representing the address of bidder.                       *
    * ManagedAddress representing the address to refund.                       *
    * Output:                                                                  *
    * Monad of the MarketplaceElement.                                         *
    *-------------------------------------------------------------------------*/
    fn find_element_by_key(&self, p_offer_id: u64, p_bidder_address: ManagedAddress, p_taker_address: ManagedAddress)-> Option<MarketplaceElement<Self::Api>>
    {
        // Create the key
        let t_key = KeyElement::new(            
            p_offer_id,
            p_bidder_address,
            p_taker_address,
        );

        // Return monad
        self.marketplace_elements().get(&t_key)
    }
    /*-------------------------------------------------------------------------*
    * Removes an element in the hashmap provided the bidder address, the taker *
    * address, and the id. This constitutes a key.                             *
    * Input:                                                                   *
    * u64 representing the ID.                                                 *
    * ManagedAddress representing the address of bidder.                       *
    * ManagedAddress representing the address to refund.                       *
    * Output:                                                                  *
    * Monad of the MarketplaceElement.                                         *
    *-------------------------------------------------------------------------*/
    fn remove_element_by_key(&self, p_offer_id: u64, p_bidder_address: ManagedAddress, p_taker_address: ManagedAddress)-> Option<MarketplaceElement<Self::Api>>
    {
        // Create the key
        let t_key = KeyElement::new(            
            p_offer_id,
            p_bidder_address,
            p_taker_address,
        );

        // Return monad
        self.marketplace_elements().remove(&t_key)
    }
    /*-------------------------------------------------------------------------*
    * Inserts new element into the KeyMap.                                     *
    * Input:                                                                   *
    * EsdtTokenPayment structure representing the token we want to bid.        *
    * BigUint representing the price we want to bid at.                        *
    * u64 representing the offer id.                                           *
    * ManagedAddress representing the address of bidder.                       *
    * ManagedAddress representing the address to refund.                       *
    * Output:                                                                  *
    * True or false depending on the success of the operation.                 *
    *-------------------------------------------------------------------------*/
    fn insert_element(&self, p_esdt_structure: EsdtTokenPayment, p_price: BigUint, p_offer_id: u64, p_bidder_address: ManagedAddress, p_taker_address: ManagedAddress)
    {        
        // Verify both taker and bidder address are payable addresses not belonging to a smart contract
        require!(!self.blockchain().is_smart_contract(&p_bidder_address) ,"Bidder address is from a smart contract.");
        require!(!self.blockchain().is_smart_contract(&p_taker_address) ,"Taker address is from a smart contract.");

        // Create the element
        let t_new_element = MarketplaceElement::new(
            p_esdt_structure.token_identifier,
            p_esdt_structure.token_nonce,
            p_esdt_structure.amount,
            p_price,
        );

        // Create the key
        let t_new_key = KeyElement::new(            
            p_offer_id,
            p_bidder_address,
            p_taker_address,
        );

        // Verify if the element is not already present
        require!(!self.marketplace_elements().insert(t_new_key, t_new_element).is_some(), "Element already present. Try with different ID.")   
    }
    /*-------------------------------------------------------------------------*
    *                                                                          *
    *-------------------------------------------------------------------------*/

    //Storage
    /*-------------------------------------------------------------------------*
    *  Unordered set to store all the marketplace elements.                    *
    *-------------------------------------------------------------------------*/
    //See storage mappers: https://docs.multiversx.com/developers/developer-reference/sc-annotations
    #[view(getMarketplaceElements)]
    #[storage_mapper("marketplaceElements")]
    fn marketplace_elements(&self) -> MapMapper<KeyElement<Self::Api>, MarketplaceElement<Self::Api>>;
}

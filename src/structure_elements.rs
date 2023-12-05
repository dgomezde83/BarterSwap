multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// A key containing the offerid, the bidder address and the taker address, that uniquely defines an offer
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct KeyElement<M: ManagedTypeApi>
{
    offer_id: u64,
    bidder_address: ManagedAddress<M>,
    taker_address: ManagedAddress<M>,
}    
impl<M: ManagedTypeApi> KeyElement<M> {
    pub fn new(       
        offer_id: u64,
        bidder_address: ManagedAddress<M>,
        taker_address: ManagedAddress<M>,
    ) -> Self {
        KeyElement {            
            offer_id,
            bidder_address,
            taker_address,
        }
    }
    pub fn get_offer_id(&self)->u64{
        self.offer_id
    }
    pub fn get_bidder_address(&self)->&ManagedAddress<M>{
        &self.bidder_address
    }
    pub fn get_taker_address(&self)->&ManagedAddress<M>{
        &self.taker_address
    }  
}
// A marketplace element containing the token (collection id and nonce), the amount of the token, and the price (in EGLD) of this token
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct MarketplaceElement<M: ManagedTypeApi>
{
    collection_id: TokenIdentifier<M>,
    nonce: u64,
    amount: BigUint<M>,
    price: BigUint<M>,
}
impl<M: ManagedTypeApi> MarketplaceElement<M> {
    pub fn new(
        collection_id: TokenIdentifier<M>,
        nonce: u64,
        amount: BigUint<M>,
        price: BigUint<M>,
    ) -> Self {
        MarketplaceElement {
            collection_id,
            nonce,
            amount,
            price,
        }
    }
    pub fn get_collection_id(&self)->&TokenIdentifier<M>{
        &self.collection_id
    }
    pub fn get_nonce(&self)->u64{
        self.nonce
    }
    pub fn get_amount(&self)->&BigUint<M>{
        &self.amount
    }
    pub fn get_price(&self)->&BigUint<M>{
        &self.price
    }    
}

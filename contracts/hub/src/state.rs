use cosmwasm_std::{Addr};
use cw_storage_plus::{
    Item, UniqueIndex, IndexList, Index, IndexedMap
};
use cosmwasm_schema::cw_serde;

////////////////////////////////////////////////////////////////////////////

pub const TICKER: Item<u32> = Item::new("ticker");

pub const CONFIGURATION: Item<Configuration> = Item::new("ticket_hub_config");

#[cw_serde]
pub struct Configuration {
    pub admin: Addr,
    pub ticket_nft_addr: Option<Addr>,
}


#[cw_serde]
pub struct TicketInfo {
    pub ticket_id: String,
    pub block_used: u64
}

pub struct UsedTicketIndexes<'a> {
    pub ticket_id: UniqueIndex<'a, String, TicketInfo, (&'a Addr, String)>,
}

impl IndexList<TicketInfo> for UsedTicketIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TicketInfo>> + '_> {
        let v: Vec<&dyn Index<TicketInfo>> = vec![
            &self.ticket_id,
        ];
        Box::new(v.into_iter())
    }
}

pub fn used_tickets<'a>() -> IndexedMap<'a, (&'a Addr, String), TicketInfo, UsedTicketIndexes<'a>> {
    let indexes = UsedTicketIndexes {
        ticket_id: UniqueIndex::new(|ticket| ticket.ticket_id.clone(), "ticket__id"),
    };

    IndexedMap::new("used_tickets_im", indexes)
}








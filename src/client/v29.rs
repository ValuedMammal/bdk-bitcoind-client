use corepc_types::{bitcoin::BlockHash, v29::GetBlockFilter};

use jsonrpc::serde_json::json;

use crate::{Client, Error};

impl Client {
    /// Retrieve the `basic` BIP 157 content filter for a particular block
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block whose filter is requested
    ///
    /// # Returns
    ///
    /// The `GetBlockFilter` structure containing the filter data
    pub fn get_block_filter(&self, block_hash: &BlockHash) -> Result<GetBlockFilter, Error> {
        let block_filter: GetBlockFilter = self.call("getblockfilter", &[json!(block_hash)])?;
        Ok(block_filter)
    }
}

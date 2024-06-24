//! Extra functions for the electrs rpc
//!

use std::thread;
use std::time::Duration;

use electrum_client::{tapyrus::Txid, ElectrumApi};

use crate::ElectrsD;

impl ElectrsD {
    /// wait up to a minute the electrum server has indexed up to the given height.
    pub fn wait_height(&self, height: usize) {
        for _ in 0..600 {
            match self.client.block_header_raw(height) {
                Ok(_) => break,
                Err(_) => thread::sleep(Duration::from_millis(100)),
            }
        }
    }

    /// wait up to a minute the electrum server has indexed the given transaction
    pub fn wait_tx(&self, txid: &Txid) {
        'main_loop: for _ in 0..600 {
            match self.client.transaction_get(txid) {
                Ok(tx) => {
                    // having the raw tx doesn't mean the scripts has been indexed
                    let txid = tx.txid();
                    if let Some(output) = tx.output.first() {
                        let history = self
                            .client
                            .script_get_history(&output.script_pubkey)
                            .unwrap();
                        for el in history {
                            if el.tx_hash == txid {
                                // the tx has to be updated atomically, so founding one is enough
                                return;
                            }
                        }
                        // the tx output has not been yet found
                        continue 'main_loop;
                    }
                    // the tx has 0 ouptut, no need to ensure script_pubkey are indexed
                    return;
                }
                Err(_) => thread::sleep(Duration::from_millis(100)),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test::setup_nodes;
    use electrum_client::{tapyrus::Amount, ElectrumApi};
    use tapyrusd::get_private_key;
    use tapyrusd::tapyruscore_rpc::RpcApi;

    #[test]
    fn test_wait_height() {
        let (_, tapyrusd, electrsd) = setup_nodes();
        let header = electrsd.client.block_headers_subscribe().unwrap();
        assert_eq!(header.height, 1);
        let address = tapyrusd
            .client
            .get_new_address(None)
            .unwrap()
            .assume_checked();
        tapyrusd
            .client
            .generate_to_address(100, &address, get_private_key())
            .unwrap();
        electrsd.wait_height(101);
        let header = electrsd.client.block_headers_subscribe().unwrap();
        assert_eq!(header.height, 101);
    }

    #[test]
    fn test_wait_tx() {
        let (_, tapyrusd, electrsd) = setup_nodes();
        let header = electrsd.client.block_headers_subscribe().unwrap();
        assert_eq!(header.height, 1);
        let generate_address = tapyrusd
            .client
            .get_new_address(None)
            .unwrap()
            .assume_checked();
        tapyrusd
            .client
            .generate_to_address(100, &generate_address, get_private_key())
            .unwrap();

        let address = tapyrusd
            .client
            .get_new_address(None)
            .unwrap()
            .assume_checked();
        let txid = tapyrusd
            .client
            .send_to_address(
                &address,
                Amount::from_tap(10000),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        electrsd.wait_tx(&txid);
        let history = electrsd
            .client
            .script_get_history(&address.script_pubkey())
            .unwrap();
        assert_eq!(history.len(), 1);
    }
}

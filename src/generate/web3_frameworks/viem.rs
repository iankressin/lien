use crate::types::{ContractMetadata, IntermediateContracts};

use super::UpdateFiles;
use std::{collections::HashMap, error, fs, path::PathBuf, u64};

type ContractAddresses = HashMap<u64, HashMap<String, String>>;

pub struct Viem {
    abi_dir: PathBuf,
    addresses_dir: PathBuf,
    intermidiate_contracts: IntermediateContracts,
}

impl UpdateFiles for Viem {
    fn update_files(&self) -> Result<(), Box<dyn error::Error>> {
        self.update_addresses_file()?;

        // TODO: this call is overrinding the ABIs generated by the previous call
        self.intermidiate_contracts
            .iter()
            .for_each(|(_, contracts)| {
                self.update_abi_files(&contracts).unwrap();
            });

        Ok(())
    }

    fn update_addresses_file(&self) -> Result<(), Box<dyn error::Error>> {
        let mut contract_addresses_hashmap: ContractAddresses = std::collections::HashMap::new();

        self.intermidiate_contracts
            .iter()
            .for_each(|(chain_id, contracts)| {
                let mut chain_contracts: HashMap<String, String> = std::collections::HashMap::new();
                contracts.iter().for_each(|contract| {
                    chain_contracts.insert(contract.name.clone(), contract.address.clone());
                });

               contract_addresses_hashmap.insert(*chain_id, chain_contracts);
            });

        fs::write(
            self.addresses_dir.join("contract-addresses.ts"), 
            Viem::json_to_ts(contract_addresses_hashmap)
        ).expect("Failed to write contract addresses file");

        Ok(())
    }

    fn update_abi_files( &self, contract_metadata: &Vec<ContractMetadata>) -> Result<(), Box<dyn error::Error>> {
        contract_metadata.iter().for_each(|contract| {
            let abi_file_path = self.abi_dir.join(format!("{}.ts", contract.name));
            let abi_json = serde_json::to_string_pretty(&contract.abi).unwrap();

            let abi_ts = format!("export default {abi_json} as const;\n");
            fs::write(abi_file_path, abi_ts).expect("Failed to write abi file");
        });

        Ok(())
    }
}

impl Viem {
    pub fn new(
        addresses_dir: PathBuf,
        abi_dir: PathBuf,
        intermidiate_contracts: IntermediateContracts,
    ) -> Viem {
        Viem {
            abi_dir,
            addresses_dir,
            intermidiate_contracts,
        }
    }

    pub fn json_to_ts(contract_addresses: ContractAddresses) -> String {
        let mut contract_addresses_ts = "export const ContractAddress = {\n".to_owned();

        for (chain_id, contracts) in contract_addresses {
            contract_addresses_ts.push_str(&format!("  {}: {{\n", chain_id));

            for (contract_name, address) in contracts {
                contract_addresses_ts
                    .push_str(&format!("    \"{}\": \"{}\",\n", contract_name, address));
            }

            contract_addresses_ts.push_str("  },\n");
        }

        contract_addresses_ts.push_str("} as const;\n");

        contract_addresses_ts
    }
}

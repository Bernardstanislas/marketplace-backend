use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::info;
use starknet::{
    accounts::{Account, Call},
    core::{
        types::{AddTransactionResult, FieldElement},
        utils::{cairo_short_string_to_felt, get_selector_from_name},
    },
};

use super::models::ContractUpdateStatus;
use crate::model::pullrequest;

pub struct GithubOracle<'a, A: Account + Sync> {
    oracle_contract_address: FieldElement,
    account: &'a A,
}

fn oracle_contract_address() -> FieldElement {
    let registry_contract_address =
        std::env::var("METADATA_ADDRESS").expect("METADATA_ADDRESS must be set");
    FieldElement::from_hex_be(&registry_contract_address)
        .expect("Invalid value for METADATA_ADDRESS")
}

#[async_trait]
pub trait Oracle {
    async fn add_contribution(&self, pr: &pullrequest::PullRequest)
        -> Result<ContractUpdateStatus>;

    fn make_add_contribution_call(&self, pr: &pullrequest::PullRequest) -> Call;

    async fn send_transaction(&self, calls: &Vec<Call>) -> Result<AddTransactionResult>;
}

impl<'a, A: Account + Sync> GithubOracle<'a, A> {
    pub fn new(account: &'a A) -> Self {
        Self {
            oracle_contract_address: oracle_contract_address(),
            account,
        }
    }
}

#[async_trait]
impl<'a, A: Account + Sync> Oracle for GithubOracle<'a, A> {
    async fn add_contribution(
        &self,
        pr: &pullrequest::PullRequest,
    ) -> Result<ContractUpdateStatus> {
        info!(
            "Register contribution #{} by {} ({})",
            pr.id, pr.author, pr.status
        );

        let transaction_result = self
            .send_transaction(&vec![self.make_add_contribution_call(&pr)])
            .await?;

        Ok(ContractUpdateStatus::new(
            pr.id.clone(),
            format!("0x{:x}", transaction_result.transaction_hash),
        ))
    }

    fn make_add_contribution_call(&self, pr: &pullrequest::PullRequest) -> Call {
        Call {
            to: self.oracle_contract_address,
            selector: get_selector_from_name("add_contribution_from_handle").unwrap(),
            calldata: vec![
                FieldElement::from_dec_str(&pr.author).unwrap(), // github identifier
                cairo_short_string_to_felt("").unwrap(),         // owner
                cairo_short_string_to_felt(&pr.repository_id).unwrap(), // repo
                FieldElement::from_dec_str(&pr.id).unwrap(),     // PR ID
                FieldElement::from_dec_str(&pr.status.to_string()).unwrap(), // PR status (merged)
            ],
        }
    }

    async fn send_transaction(&self, calls: &Vec<Call>) -> Result<AddTransactionResult> {
        info!("Sending transactions with {} calls", calls.len());

        match self.account.execute(calls).send().await {
            Ok(transaction_result) => Ok(transaction_result),
            Err(error) => Err(anyhow!(error.to_string())),
        }
    }
}
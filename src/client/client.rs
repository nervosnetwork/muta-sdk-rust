use muta_protocol::traits as muta_traits;
use muta_protocol::types as muta_types;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::convert::TryInto;
use std::str::FromStr;

use super::request::{
    GET_BLOCK, GET_BLOCK_QUERY, GET_RECEIPT, GET_RECEIPT_QUERY, GET_TRANSACTION,
    GET_TRANSACTION_QUERY, SEND_TRANSACTION, SEND_TRANSACTION_MUTATION, SERVICE, SERVICE_QUERY,
};
use super::rpc_types::{Block, Hash, Receipt, RpcError, ServiceResponse, SignedTransaction};
use crate::util::{random_nonce, u64_to_hex};

pub struct Config {
    pub url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8000/graphql".to_owned(),
        }
    }
}

pub struct HttpRpcClient {
    config: Config,
    client: reqwest::Client,
}

impl Default for HttpRpcClient {
    fn default() -> Self {
        let config = Config::default();
        Self::new(config)
    }
}

impl HttpRpcClient {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    pub async fn raw<T: Serialize + ?Sized, U: DeserializeOwned>(
        &self,
        q: &T,
        method: &str,
    ) -> Result<U, RpcError> {
        let mut resp: Value = self
            .client
            .post(&self.config.url)
            .json(&q)
            .send()
            .await?
            .json()
            .await?;

        if let Some(errs) = resp.get("errors") {
            return Err(RpcError::GraphQLError(errs.to_string()));
        }
        Ok(serde_json::from_value(
            resp.get_mut("data")
                .ok_or(RpcError::DataIsNone)?
                .get_mut(method)
                .ok_or(RpcError::DataIsNone)?
                .take(),
        )?)
    }

    pub async fn get_block(&self, height: Option<u64>) -> Result<muta_types::Block, RpcError> {
        let q = json!({
            "query": GET_BLOCK_QUERY,
            "variables": {
                "h": height.map(u64_to_hex),
            },
        });
        let rpc_block: Block = self.raw(&q, GET_BLOCK).await?;
        Ok(rpc_block.try_into()?)
    }

    pub async fn get_transaction(
        &self,
        tx_hash: muta_types::Hash,
    ) -> Result<muta_types::SignedTransaction, RpcError> {
        let q = json!({
            "query": GET_TRANSACTION_QUERY,
            "variables": {
                "txHash": tx_hash.as_hex(),
            },
        });
        let rpc_transaction: SignedTransaction = self.raw(&q, GET_TRANSACTION).await?;
        Ok(rpc_transaction.try_into()?)
    }

    pub async fn get_receipt(
        &self,
        tx_hash: muta_types::Hash,
    ) -> Result<muta_types::Receipt, RpcError> {
        let q = json!({
            "query": GET_RECEIPT_QUERY,
            "variables": {
                "txHash": tx_hash.as_hex(),
            },
        });
        let rpc_receipt: Receipt = self.raw(&q, GET_RECEIPT).await?;
        Ok(rpc_receipt.try_into()?)
    }

    pub async fn query_service(
        &self,
        height: Option<u64>,
        cycles_limit: Option<u64>,
        cycles_price: Option<u64>,
        caller: Option<muta_types::Address>,
        service_name: String,
        method: String,
        payload: String,
    ) -> Result<muta_traits::ServiceResponse<String>, RpcError> {
        let q = json!({
            "query": SERVICE_QUERY,
            "variables": {
                "height": height.map(u64_to_hex),
                "cyclesLimit": cycles_limit.map(u64_to_hex),
                "cyclesPrice": cycles_price.map(u64_to_hex),
                "caller": caller.unwrap_or(muta_types::Address::from_hex("0x0000000000000000000000000000000000000000")?),
                "serviceName": service_name,
                "method": method,
                "payload": payload,
            },
        });
        let rpc_service: ServiceResponse = self.raw(&q, SERVICE).await?;
        Ok(rpc_service.try_into()?)
    }

    pub async fn generate_raw_transaction(
        &self,
        chain_id: String,
        timeout_gap: Option<u64>,
        cycles_limit: Option<u64>,
        cycles_price: Option<u64>,
        sender: String,
        service_name: String,
        method: String,
        payload: String,
    ) -> Result<muta_types::RawTransaction, RpcError> {
        let block = self.get_block(None).await?;
        let timeout = block.header.height + timeout_gap.unwrap_or(20);
        let nonce = random_nonce();
        Ok(muta_types::RawTransaction {
            chain_id: muta_types::Hash::from_hex(chain_id.as_str())?,
            nonce,
            timeout,
            sender: muta_types::Address::from_str(sender.as_str())?,
            cycles_price: cycles_price.unwrap_or(1),
            cycles_limit: cycles_limit.unwrap_or(1000000),
            request: muta_types::TransactionRequest {
                service_name,
                method,
                payload,
            },
        })
    }

    pub async fn send_transaction(
        &self,
        tx: muta_types::SignedTransaction,
    ) -> Result<muta_types::Hash, RpcError> {
        let q = json!({
            "query": SEND_TRANSACTION_MUTATION,
            "variables": {
                "input_raw": {
                    "chainId": tx.raw.chain_id.as_hex(),
                    "cyclesLimit": u64_to_hex(tx.raw.cycles_limit),
                    "cyclesPrice": u64_to_hex(tx.raw.cycles_price),
                    "nonce": tx.raw.nonce.as_hex(),
                    "timeout": u64_to_hex(tx.raw.timeout),
                    "serviceName": tx.raw.request.service_name,
                    "method": tx.raw.request.method,
                    "payload": tx.raw.request.payload,
                    "sender": tx.raw.sender.to_string(),
                },
                "input_encryption": {
                    "txHash": tx.tx_hash.as_hex(),
                    "pubkey": "0x".to_owned() + &hex::encode(rlp::encode_list::<Vec<u8>, _>(&[tx.pubkey.to_vec()])),
                    "signature": "0x".to_owned() + &hex::encode(rlp::encode_list::<Vec<u8>, _>(&[tx.signature.to_vec()]))
                }
            },
        });

        let rpc_hash: Hash = self.raw(&q, SEND_TRANSACTION).await?;
        Ok(muta_types::Hash::from_hex(&rpc_hash)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Account;
    use std::{thread, time};
    #[tokio::test]
    async fn client_get_block_works() {
        let client = HttpRpcClient::default();
        let res = client.get_block(None).await.unwrap();
        assert_eq!(
            "0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
            res.header.chain_id.as_hex()
        );
    }

    #[tokio::test]
    async fn client_query_service() {
        let client = HttpRpcClient::default();
        let payload = r#"{"asset_id": "0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c", "user": "muta14e0lmgck835vm2dfm0w3ckv6svmez8fdgdl705"}"#;
        let res = client
            .query_service(
                None,
                None,
                None,
                None,
                "asset".to_owned(),
                "get_balance".to_owned(),
                payload.to_owned(),
            )
            .await
            .unwrap();
        assert_eq!(0, res.code);
    }

    #[tokio::test]
    async fn client_send_transaction() {
        let client = HttpRpcClient::default();
        let account =
            Account::from_hex("5ec982173d54d830b6789cbbbe43eaa2853a5ff752d1ebc1b266cf9790314f8a")
                .unwrap();
        let payload = r#"{"asset_id": "0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c","to": "muta1tdw5mnyk5s3lngz2mcjd2rse4htrnu6679pr23","value": 1}"#;
        let raw = client
            .generate_raw_transaction(
                "0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036".to_owned(),
                None,
                None,
                None,
                "muta14e0lmgck835vm2dfm0w3ckv6svmez8fdgdl705".to_owned(),
                "asset".to_owned(),
                "transfer".to_owned(),
                payload.to_owned(),
            )
            .await
            .unwrap();

        let signed_transaction = account.sign_raw_tx(raw).unwrap();

        let tx_hash = client.send_transaction(signed_transaction).await.unwrap();
        println!("{:?}", tx_hash);

        let duration = time::Duration::from_secs(1);
        let mut i: u64 = 0;
        while i < 10 {
            thread::sleep(duration);
            i += 1;

            match client.get_transaction(tx_hash.clone()).await {
                Ok(tx) => assert_eq!(tx_hash, tx.tx_hash),
                Err(_e) => continue,
            }
            match client.get_receipt(tx_hash.clone()).await {
                Ok(tx) => {
                    assert_eq!(tx_hash, tx.tx_hash);
                    assert_eq!(0, tx.response.response.code);
                    break;
                }
                Err(_e) => continue,
            }
        }
        assert_ne!(10, i);
    }
}

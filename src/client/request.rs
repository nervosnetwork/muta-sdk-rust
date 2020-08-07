pub const GET_TRANSACTION: &str = "getTransaction";
pub const GET_TRANSACTION_QUERY: &str = r#"
query RpcTransaction($txHash: Hash!) {
    getTransaction(txHash: $txHash) {
      chainId
      cyclesLimit
      cyclesPrice
      nonce
      timeout
      sender
      serviceName
      method
      payload
      txHash
      pubkey
      signature 
    }
  }
"#;

pub const GET_BLOCK: &str = "getBlock";
pub const GET_BLOCK_QUERY: &str = r#"
query RpcBlock($height: Uint64) {
    getBlock(height: $height) {
      header {
        chainId
        height
        execHeight
        prevHash
        timestamp
        orderRoot
        orderSignedTransactionsHash
        confirmRoot
        stateRoot
        receiptRoot
        cyclesUsed
        proposer
        proof {
          height
          round
          blockHash
          signature
          bitmap
        }
        validatorVersion
        validators {
          pubkey
          proposeWeight
          voteWeight
        }
      }
      orderedTxHashes
      hash
    }
  }
"#;

pub const GET_RECEIPT: &str = "getReceipt";
pub const GET_RECEIPT_QUERY: &str = r#"
query RpcReceipt($txHash: Hash!) {
    getReceipt(txHash: $txHash) {
      stateRoot
      height    
      txHash   
      cyclesUsed
      events {
        service
        name
        data
      }   
      response {
        serviceName
        method
        response {
          code
          succeedData
          errorMessage
        }
      }
    }
  }
"#;

pub const SERVICE: &str = "queryService";
pub const SERVICE_QUERY: &str = r#"
query RpcService($height: Uint64, $cyclesLimit: Uint64, $cyclesPrice: Uint64, $caller: Address!, $serviceName: String!, $method: String!, $payload: String!) {
    queryService(height: $height, cyclesLimit: $cyclesLimit, cyclesPrice: $cyclesPrice, caller: $caller, serviceName: $serviceName, method: $method, payload: $payload) {
      code
      succeedData
      errorMessage   
    }
  }
"#;

pub const SEND_TRANSACTION: &str = "sendTransaction";
pub const SEND_TRANSACTION_MUTATION: &str = r#"
mutation RpcSendTransaction($input_raw: InputRawTransaction!, $input_encryption: InputTransactionEncryption!) {
  sendTransaction(inputRaw: $input_raw, inputEncryption: $input_encryption) 
}
"#;

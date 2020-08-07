use std::{thread, time};

#[tokio::main]
async fn main() {
    println!("start ");
    let my_account = create_account();
    let client = muta_sdk::client::client::HttpRpcClient::default();
    let chain_account = muta_sdk::account::Account::from_hex(
        "5ec982173d54d830b6789cbbbe43eaa2853a5ff752d1ebc1b266cf9790314f8a",
    )
    .unwrap();
    transfer(&client, &chain_account, &my_account).await;
    let duration = time::Duration::from_secs(10);
    thread::sleep(duration);
    get_balance(&client, &my_account).await;

    transfer(&client, &my_account, &chain_account).await;
    let duration = time::Duration::from_secs(10);
    thread::sleep(duration);
    get_balance(&client, &my_account).await;
}

fn create_account() -> muta_sdk::account::Account {
    let wallet = muta_sdk::wallet::Wallet::generate("");
    let private_key = wallet.derive_privatekey(0).unwrap();
    muta_sdk::account::Account::new(private_key)
}

async fn transfer(
    client: &muta_sdk::client::client::HttpRpcClient,
    from_account: &muta_sdk::account::Account,
    to_account: &muta_sdk::account::Account,
) {
    let from_address = from_account.get_address().to_string();
    let to_address = to_account.get_address().to_string();
    let payload = format!(
        r#"{{"asset_id": "0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c","to": "{}","value": 10000000}}"#,
        to_address
    );
    let raw = client
        .generate_raw_transaction(
            "0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036".to_owned(),
            None,
            None,
            None,
            from_address,
            "asset".to_owned(),
            "transfer".to_owned(),
            payload.to_owned(),
        )
        .await
        .unwrap();
    let signed_transaction = from_account.sign_raw_tx(raw).unwrap();

    let tx_hash = client.send_transaction(signed_transaction).await.unwrap();
    println!("{:?}", tx_hash);
}

async fn get_balance(
    client: &muta_sdk::client::client::HttpRpcClient,
    account: &muta_sdk::account::Account,
) {
    let address = account.get_address().to_string();
    let payload = format!(
        r#"{{"asset_id": "0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c", "user": "{}"}}"#,
        address
    );
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
    println!("{:?}", res);
}

// src/bin/deploy_drex_contract.rs
use ethers::prelude::*;
use ethers::contract::ContractFactory;

#[tokio::main]
async fn main() -> Result<(), String> {
    let provider = Provider::<Http>::try_from("https://drex-testnet.bcb.gov.br")
        .map_err(|e| format!("Provider error: {}", e))?;
    let wallet: LocalWallet = std::env::var("DREX_PRIVATE_KEY")
        .unwrap_or_default()
        .parse()
        .map_err(|e| format!("Wallet error: {}", e))?;
    let client = SignerMiddleware::new(provider, wallet);

    // In a real scenario you would include compiled files
    // let bytecode = include_bytes!("../contracts/drex/RoyaltySplitter.bin").to_vec();
    // let abi = include_bytes!("../contracts/drex/RoyaltySplitter.abi").to_vec();
    let bytecode = Bytes::from(vec![]);
    let abi: ethers::abi::Abi = serde_json::from_str("[]").unwrap();

    let factory = ContractFactory::new(abi, bytecode, client.into());

    let drex_token: Address = std::env::var("DREX_TOKEN_ADDRESS")
        .unwrap_or_default()
        .parse()
        .map_err(|e| format!("Address error: {}", e))?;
    let recipients: Vec<Address> = vec![
        "0x0000000000000000000000000000000000000001".parse().unwrap(),
        "0x0000000000000000000000000000000000000002".parse().unwrap(),
    ];
    let shares: Vec<U256> = vec![7000.into(), 3000.into()];

    // Deploy
    let contract = factory.deploy((drex_token, recipients, shares))
        .map_err(|e| format!("Deploy error: {}", e))?
        .send().await
        .map_err(|e| format!("Send error: {}", e))?;
    println!("✅ Contrato RoyaltySplitter deployado em: {:?}", contract.address());

    Ok(())
}
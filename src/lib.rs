use webhook_flows::{create_endpoint, request_handler, send_response, route::{get, post, route, RouteError, Router}};
use flowsnet_platform_sdk::logger;
use serde_json::Value;
use serde_json::json;
// use std::fs;
use std::collections::HashMap;
use std::str::FromStr;
use ethers_signers::{LocalWallet, Signer};
use ethers_core::{k256::elliptic_curve::rand_core::block, types::{NameOrAddress, U256, H160}};
use ethers_core::abi::Token;
use ethers_core::rand::thread_rng;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use std::sync::Arc;

pub mod ether_lib;
use ether_lib::*;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

static BLOCK_NUMBER: Lazy<Mutex<U256>> = Lazy::new(|| {
    Mutex::new(U256::from(0))
});

#[request_handler]
async fn handler(_headers: Vec<(String, String)>, _subpath: String, _qry: HashMap<String, Value>, _body: Vec<u8>) {
    let mut router = Router::new();
    router
        .insert(
            "/trigger",
            vec![post(trigger)],
        )
        .unwrap();

    if let Err(e) = route(router).await {
        match e {
            RouteError::NotFound => {
                send_response(404, vec![], b"No route matched".to_vec());
            }
            RouteError::MethodNotAllowed => {
                send_response(405, vec![], b"Method not allowed".to_vec());
            }
        }
    }
}

fn init_rpc(path: &str, _qry: &HashMap<String, Value>) -> (String, u64, NameOrAddress, LocalWallet){
    logger::init();
    log::info!("{} Query -- {:?}", path, _qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://mainnet.cybermiles.io".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("18".to_string()).parse::<u64>().unwrap_or(18u64);
    let contract_address = NameOrAddress::from(H160::from_str(std::env::var("CONTRACT_ADDRESS").unwrap().as_str()).unwrap());
    let mut wallet: LocalWallet = LocalWallet::new(&mut thread_rng());
    
    let private_key = std::env::var("PRIVATE_KEY").unwrap();

	wallet = private_key.as_str()
	.parse::<LocalWallet>()
	.unwrap()
	.with_chain_id(chain_id);

    return (rpc_node_url, chain_id, contract_address, wallet);
}

async fn trigger(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    log::info!("Headers -- {:#?} ", _headers);
    let (rpc_node_url, chain_id, contract_address, wallet) = init_rpc("submit_request", &_qry);
    let contract_address = format!("{:?}", contract_address.as_address().unwrap());
    let content = String::from_utf8(_body.clone()).unwrap();
    log::info!("Content -- {:#?} ", content);
    let log = get_latest_log(&rpc_node_url, &contract_address, json!(["0x5d994db479791b5e3ca06f8955d4fd321623157fcec560abc14bd1b0087e2e3e"])).await.unwrap();
    log::info!("log -- {:#?} ", log);
    if log.as_array().unwrap().len() == 0 {
        send_response(
            200,
            vec![(String::from("content-type"), String::from("text/html"))],
            "OK".to_string().into_bytes().to_vec(),
        );
        return;
    }
    let now_block_number = U256::from_str(log.get(0).unwrap()["blockNumber"].as_str().unwrap()).unwrap();
    let mut block_number = BLOCK_NUMBER.lock().await;
    log::info!("old block -- {:#?} ", block_number);
    if *block_number >= now_block_number {
        send_response(
            200,
            vec![(String::from("content-type"), String::from("text/html"))],
            "OK".to_string().into_bytes().to_vec(),
        );
        return;
    }else{
        *block_number = now_block_number;
    }
    log::info!("Do strategy {} {}", block_number, now_block_number);
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        "OK".to_string().into_bytes().to_vec(),
    );
}


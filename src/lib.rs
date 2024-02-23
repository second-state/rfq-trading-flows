use webhook_flows::{create_endpoint, request_handler, send_response, route::{get, post, route, RouteError, Router}};
use flowsnet_platform_sdk::logger;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use ethers_signers::{LocalWallet, Signer};
use ethers_core::types::{H160, U256};
use ethers_core::abi::{Token, AbiEncode};
use store_flows;
use std::time::{SystemTime, UNIX_EPOCH};
use ethers_core::utils::hex;
use ethers_core::rand;
use ethers_core::rand::Rng;
use std::cmp;

pub mod ether_lib;
use ether_lib::*;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}


#[request_handler]
async fn handler(_headers: Vec<(String, String)>, _subpath: String, _qry: HashMap<String, Value>, _body: Vec<u8>) {
    let mut router = Router::new();
    router
        .insert(
            "/trigger",
            vec![post(trigger)],
        )
        .unwrap();
    router
        .insert(
            "/random-response",
            vec![post(random_response)],
        )
        .unwrap();

    router
        .insert(
            "/reset-state",
            vec![get(reset_state)],
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

fn init_rpc(_qry: &HashMap<String, Value>) -> (String, u64, String, LocalWallet){
    logger::init();
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://mainnet.cybermiles.io".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("18".to_string()).parse::<u64>().unwrap_or(18u64);
    let contract_address = std::env::var("CONTRACT_ADDRESS").unwrap().to_string();
    
    let private_key = std::env::var("PRIVATE_KEY").unwrap();

	let wallet = private_key.as_str()
	.parse::<LocalWallet>()
	.unwrap()
	.with_chain_id(chain_id);

    return (rpc_node_url, chain_id, contract_address, wallet);
}

fn init_variable() -> (U256, U256, U256, U256, Vec<U256>, String, String, u128, u128, u128, u128, U256, U256, bool, U256, U256) {
    let quantity = std::env::var("QUANTITY").unwrap_or("100".to_string()).parse::<U256>().unwrap();
    let exchange_quantity = std::env::var("EXCHANGE_QUANTITY").unwrap_or("99".to_string()).parse::<U256>().unwrap();
    let profit_spread = std::env::var("PROFIT_SPREAD").unwrap_or("1".to_string()).parse::<U256>().unwrap();
    let last_block_number = U256::from_str(store_flows::get("last_block_number").unwrap_or(json!("0")).as_str().unwrap()).unwrap();
    let request_list = store_flows::get("request_list").unwrap_or(json!([])).as_array().unwrap().clone()
    .iter()
    .map(|value| U256::from_str(value.as_str().unwrap()).unwrap())
    .collect::<Vec<U256>>();
    let base = std::env::var("BASE").unwrap_or("0x0Fa9C7e8430103Cd88823e574FC575eb62603e4C".to_string()).to_lowercase();
    let quote = std::env::var("QUOTE").unwrap_or("0x6C90Ab407aBa1917F366860C4F0836d4B24fB95E".to_string()).to_lowercase();
    let min_base_quantity = std::env::var("MIN_BASE_QUANTITY").unwrap_or("98".to_string()).parse::<u128>().unwrap();
    let max_base_quantity = std::env::var("MAX_BASE_QUANTITY").unwrap_or("102".to_string()).parse::<u128>().unwrap();
    let min_quote_quantity = std::env::var("MIN_QUOTE_QUANTITY").unwrap_or("98".to_string()).parse::<u128>().unwrap();
    let max_quote_quantity = std::env::var("MAX_QUOTE_QUANTITY").unwrap_or("102".to_string()).parse::<u128>().unwrap();
    let cooling_time = U256::from_dec_str(std::env::var("COOLING_TIME").unwrap_or("300".to_string()).as_str()).unwrap();
    let last_time = U256::from_str(store_flows::get("last_time").unwrap_or(Value::from("0")).as_str().unwrap()).unwrap();
    let is_lock = store_flows::get("is_lock").unwrap_or(json!(false)).as_bool().unwrap();
    let request_id = U256::from_str(store_flows::get("request_id").unwrap_or(json!("0")).as_str().unwrap()).unwrap();
    let response_id = U256::from_str(store_flows::get("response_id").unwrap_or(json!("0")).as_str().unwrap()).unwrap();
    (quantity, exchange_quantity, profit_spread, last_block_number,
    request_list, base, quote, min_base_quantity, max_base_quantity,
    min_quote_quantity, max_quote_quantity, cooling_time, last_time,
    is_lock, request_id, response_id)
}

fn store_state(last_block_number: Option<U256>, request_list: Option<Vec<U256>>, now_time: Option<U256>, response_id: Option<U256>, request_id: Option<U256>, is_lock: Option<bool>) {
    if let Some(last_block_number) = last_block_number {
        store_flows::set("last_block_number", json!(last_block_number), None);
    }
    if let Some(request_list) = request_list{
        store_flows::set("request_list", json!(request_list), None);
    }
    if let Some(now_time) = now_time{
        store_flows::set("last_time", json!(now_time), None);
    }
    if let Some(response_id) = response_id{
        store_flows::set("response_id", json!(response_id), None);
    }
    if let Some(request_id) = request_id{
        store_flows::set("request_id", json!(request_id), None);
    }
    if let Some(is_lock) = is_lock{
        store_flows::set("is_lock", json!(is_lock), None);
    }
}

fn lock(name: &str) -> bool {
    let lock_name = format!("{}_{}", name, "lock");
    let is_lock = store_flows::get(&lock_name).unwrap_or(json!(false)).as_bool().unwrap();
    if !is_lock {
        store_flows::set(&lock_name, json!(true), None);
        return true;
    }
    false
}

fn unlock(name: &str) {
    let lock_name = format!("{}_{}", name, "lock");
    store_flows::set(&lock_name, json!(false), None);
}

async fn reset_state(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>) {
    store_flows::set("last_block_number", json!("0"), None);
    store_flows::set("request_list", json!([]), None);
    store_flows::set("last_time", json!(0), None);
    store_flows::set("response_id", json!(0), None);
    store_flows::set("request_id", json!(0), None);
    store_flows::set("is_lock", json!(false), None);
    store_flows::set("trigger_lock", json!(false), None);
    store_flows::set("random_lock", json!(false), None);
}

async fn trigger(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){


    if !lock("trigger") {
        send_response(
            200,
            vec![(String::from("content-type"), String::from("text/html"))],
            "Locked".to_string().into_bytes().to_vec(),
        );
        return;
    }

    let (rpc_node_url, chain_id, contract_address, wallet) = init_rpc(&_qry);

	let (quantity, exchange_quantity,  profit_spread, last_block_number, mut request_list, base, quote,
         _, _, _, _, _, _, _, _, _) = init_variable();

	let now_block_number = get_block_number(&rpc_node_url).await.unwrap();

	let mut response_log = get_log_from(&rpc_node_url, &contract_address, json!(["0x04a02541703318cd8f9e95f53f8a3e93327acfef4a41ba01dfd2bdd5623cfb6a"]),
								format!("{:#x}", last_block_number + 1).as_str() , format!("{:#x}", now_block_number).as_str()).await.unwrap();
	let response_list = response_log.as_array_mut().unwrap();
	response_list.sort_by_key(| now | U256::from_str(&(now["topics"][1].to_string()).trim_matches('"')[26..]).unwrap());
	let len = response_log.as_array().unwrap().len();
	let mut request_idx = 0;
    let now_time:U256 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs()
		.into();

	// Check if accept response  
	for idx in 0..len{
		let now = response_log.get(idx).unwrap();
		let response_id = U256::from_str(&(now["topics"][1].to_string()).trim_matches('"')[26..]).unwrap();
		// let buyer = format!("0x{}", &now["data"].as_str().unwrap()[26..66]);
		let request_id = U256::from_str(&(now["topics"][2].to_string()).trim_matches('"')[2..]).unwrap();
		let amount_in = U256::from_str(&now["data"].as_str().unwrap()[90..130]).unwrap();
		let expire_time = U256::from_str(&now["data"].as_str().unwrap()[131..194]).unwrap();
		if expire_time < now_time {
			continue;
		}
		while request_idx < request_list.len() && request_id > request_list[request_idx] {
			request_idx += 1;
		}
		if request_idx == request_list.len(){
			break;
		}
		let request = get_log(&rpc_node_url, &contract_address, 
			json!(["0x5d994db479791b5e3ca06f8955d4fd321623157fcec560abc14bd1b0087e2e3e", null, request_id.encode_hex()]))
			.await
			.unwrap();
        let now = request.get(0).unwrap();
		let token_out = format!("0x{}", &now["data"].as_str().unwrap()[26..66]);
		let token_in = format!("0x{}", &now["data"].as_str().unwrap()[90..130]);
		// let amount_out = U256::from_str(&now["data"].as_str().unwrap()[131..194]).unwrap();
		let expire_time = U256::from_str(&now["data"].as_str().unwrap()[195..258]).unwrap();
		if expire_time < now_time {
			continue;
		}
		let mut accept = false;
        // 1. quote(quantity) <-> base(exchange_quantity)
        // 2. base(exchange_quantity) <-> quote(quantity + profit_spread)
        // Profix = profit_spread
		if token_out == quote && token_in == base {
			if amount_in >= exchange_quantity {
				accept = true;
			}
		}else if token_out == base && token_in == quote {
			if amount_in >= quantity + profit_spread {
				accept = true;
			}
		}

		if accept {
			let accept_bid_params = vec![Token::Uint(request_id.into()), Token::Uint(response_id.into())];
			let _ = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "acceptBid", accept_bid_params, &contract_address).await;
			request_list.remove(request_idx);
            let withdraw_params = vec![Token::Uint(request_id.into())];
			let _ = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "withdraw", withdraw_params, &contract_address).await;
		}
		
		// Create reverse request
		if token_out == quote && token_in == base && accept{
			let withdraw_params = vec![Token::Uint(request_id.into())];
			let _ = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "withdraw", withdraw_params, &contract_address).await;
            let approve_params = vec![Token::Address(H160::from_str(&contract_address).unwrap()), Token::Uint((amount_in).into())];
            let _ = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "approve", approve_params, &base).await.unwrap();
			let submit_request_params = vec![Token::Address(H160::from_str(&token_in).unwrap()), Token::Address(H160::from_str(&token_out).unwrap()), Token::Uint((amount_in).into()), Token::Uint(0.into())];
			let result = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "submitRequest", submit_request_params, &contract_address).await.unwrap();
			for idx in 0..result["logs"].as_array().unwrap().len(){
                if result["logs"][idx]["topics"][0].to_string().trim_matches('"') == "0x5d994db479791b5e3ca06f8955d4fd321623157fcec560abc14bd1b0087e2e3e" {
                    let new_request_id = U256::from_str(&(result["logs"][idx]["topics"][2].to_string()).trim_matches('"')[2..]).unwrap();
                    request_list.push(new_request_id);
                    break;
                }else{
                    continue;
                }
            }
		}
	} 

	// Create request used remaining token
    loop {
        let balance_of_params = vec![Token::Address(wallet.address().into())];
        let balance = U256::from_str(call_function(&rpc_node_url, "balanceOf", balance_of_params, &quote).await.unwrap().as_str()).unwrap();
        if balance < quantity {
            break;
        }
        let approve_params = vec![Token::Address(H160::from_str(&contract_address).unwrap()), Token::Uint((quantity).into())];
        let _ = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "approve", approve_params, &quote).await.unwrap();
        let submit_request_params = vec![Token::Address(H160::from_str(&quote).unwrap()), Token::Address(H160::from_str(&base).unwrap()), Token::Uint((quantity).into()), Token::Uint(0.into())];
        let result = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "submitRequest", submit_request_params, &contract_address).await.unwrap();
        for idx in 0..result["logs"].as_array().unwrap().len(){
            if result["logs"][idx]["topics"][0].to_string().trim_matches('"') == "0x5d994db479791b5e3ca06f8955d4fd321623157fcec560abc14bd1b0087e2e3e" {
                let new_request_id = U256::from_str(&(result["logs"][idx]["topics"][2].to_string()).trim_matches('"')[2..]).unwrap();
                request_list.push(new_request_id);
                break;
            }else{
                continue;
            }
        }
    }
    
    store_state(Some(now_block_number), Some(request_list), None, None, None, None);

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        "OK".to_string().into_bytes().to_vec(),
    );

    unlock("trigger");

}

async fn random_response(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    
    if !lock("random") {
        send_response(
            200,
            vec![(String::from("content-type"), String::from("text/html"))],
            "Locked".to_string().into_bytes().to_vec(),
        );
        return;
    }

    let (rpc_node_url, chain_id, contract_address, wallet) = init_rpc(&_qry);
	let (_, _,  _, _, _,
         base, quote,
          min_base_quantity, max_base_quantity, min_quote_quantity, max_quote_quantity,
           cooling_time, last_time, mut is_lock, mut request_id, mut response_id) = init_variable();
    
    let mut now_time:U256 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs()
		.into();
    
    if now_time - last_time <= cooling_time {
        println!("Waiting accept");
        return;
    }
    // withdraw old response
    let response_log = get_log(&rpc_node_url, &contract_address, json!(["0x04a02541703318cd8f9e95f53f8a3e93327acfef4a41ba01dfd2bdd5623cfb6a", response_id.to_string().as_str(), request_id.to_string().as_str()]),).await.unwrap();
    if is_lock {
        if let Some(now_request) = response_log.get(0) {
            let buyer = format!("0x{}", &now_request["data"].as_str().unwrap()[26..66]);
            if buyer == format!("{:?}", wallet.address()) {
                let withdraw_params = vec![Token::Uint(request_id.into()), Token::Uint(response_id.into())];
                let _ = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "withdraw", withdraw_params, &contract_address).await;
                is_lock = false;
            }
        } 
    }

    // create new response
    let get_request_length_params = vec![];
    let request_length = U256::from_str(call_function(&rpc_node_url, "getRequestLength", get_request_length_params, &contract_address).await.unwrap().as_str()).unwrap();
    if request_length == U256::from(0) {
        println!("No request");
        return;
    }
    let get_request_params = vec![Token::Uint((request_length - 1).into())];
    if let Ok(now_request) = call_function(&rpc_node_url, "getRequest", get_request_params, &contract_address).await {
        let decode_result = decode_output("getRequest", &hex::decode(now_request).unwrap())
        .unwrap()[0]
        .clone().into_tuple()
        .unwrap();
        let (_owner, _token_out, token_in, _lock_amount, _deposit_size, _expire_time, finish, _buyer) =
        (decode_result[0].clone().into_address().unwrap(), decode_result[1].clone().into_address().unwrap(),
        decode_result[2].clone().into_address().unwrap(), decode_result[3].clone().into_uint().unwrap(),
        decode_result[4].clone().into_uint().unwrap(), decode_result[5].clone().into_uint().unwrap(),
        decode_result[6].clone().into_bool().unwrap(), decode_result[7].clone().into_uint().unwrap());
        if !finish {
            let mut rng = rand::thread_rng();
            let mut exchange_quantity = 0;
			let mut token_address = String::from("");
            let balance_of_params = vec![Token::Address(wallet.address().into())];
            if token_in == H160::from_str(&base).unwrap() {
				token_address = base.clone();
                let base_balance = U256::from_str(call_function(&rpc_node_url, "balanceOf", balance_of_params.clone(), &base).await.unwrap().as_str()).unwrap();
                if base_balance > U256::from(min_base_quantity) {
					exchange_quantity = rng.gen_range(min_base_quantity..cmp::min(base_balance.as_u128(), max_base_quantity));
                }
            }else if token_in == H160::from_str(&quote).unwrap() {
				token_address = quote.clone();
				let quote_balance = U256::from_str(call_function(&rpc_node_url, "balanceOf", balance_of_params.clone(), &quote).await.unwrap().as_str()).unwrap();
                if quote_balance > U256::from(min_quote_quantity) {
					exchange_quantity = rng.gen_range(min_quote_quantity..cmp::min(quote_balance.as_u128(), max_quote_quantity));
                }
            }
            if exchange_quantity != 0 {
                let approve_params = vec![Token::Address(H160::from_str(&contract_address).unwrap()), Token::Uint((exchange_quantity).into())];
                let _ = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "approve", approve_params, &token_address).await.unwrap();
                let submit_response_params = vec![Token::Uint((request_length - 1).into()), Token::Uint((exchange_quantity).into()), Token::Uint((cmp::max(U256::from(0), cooling_time - 10)).into())];
                let result = call_function_and_wait(&rpc_node_url, chain_id, wallet.clone(), "submitResponse", submit_response_params, &contract_address).await.unwrap();
                for idx in 0..result["logs"].as_array().unwrap().len(){
                    if result["logs"][idx]["topics"][0].to_string().trim_matches('"') == "0x04a02541703318cd8f9e95f53f8a3e93327acfef4a41ba01dfd2bdd5623cfb6a" {
                        let new_response_id = U256::from_str(&(result["logs"][idx]["topics"][1].to_string()).trim_matches('"')[2..]).unwrap();
                        let new_request_id = U256::from_str(&(result["logs"][idx]["topics"][2].to_string()).trim_matches('"')[2..]).unwrap();
                        request_id = new_request_id;
                        response_id = new_response_id;
                        is_lock = true;
                        break;
                    }else{
                        continue;
                    }
                }
            }
        }
    }
    
    now_time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs()
    .into();

    store_state(None, None, Some(now_time), Some(response_id), Some(request_id), Some(is_lock));

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        "OK".to_string().into_bytes().to_vec(),
    );

    unlock("random");
}
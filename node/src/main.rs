//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod benchmarking;
mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

// @@@
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::thread;
use tokio;

use frame_metadata::RuntimeMetadataPrefixed;
use parity_scale_codec::Decode;

use utils::rpc_to_localhost;

use std::str::FromStr;
// use encoding_rs::WINDOWS_1252;
// use encoding_rs_io::DecodeReaderBytesBuilder;
use parity_scale_codec::{Compact, Encode};
use sp_core::{blake2_256, H256};
use sp_keyring::AccountKeyring;
use sp_runtime::{generic::Era, MultiAddress, MultiSignature};
use sp_version::RuntimeVersion;
// @@@


#[get("/")]
async fn hello() -> impl Responder {
	// See https://www.jsonrpc.org/specification for more information on
	// the JSON RPC 2.0 format that we use here to talk to nodes.
	let res = rpc_to_localhost("state_getMetadata", ()).await.unwrap();
	let metadata_hex = res.as_str().unwrap();
	let metadata_bytes = hex::decode(&metadata_hex.trim_start_matches("0x")).unwrap();

	// Fortunately, we know what type the metadata is, so we are able to decode our SCALEd bytes to it:
	let decoded = RuntimeMetadataPrefixed::decode(&mut metadata_bytes.as_slice()).unwrap();
	// let decoded = RuntimeMetadataPrefixed::
	// We'll finally re-encode to JSON to make prettier output.
	let output = serde_json::to_string_pretty(&decoded).unwrap();
	// println!("{}", serde_json::to_string_pretty(&body).unwrap());
	// println!("{}", output);
	let body = output;
	HttpResponse::Ok().body(body)
}
#[get("/update")]
async fn update() -> impl Responder {
	// pallet loc
	let pallet_index: u8 = 7;
	let call_index: u8 = 0;

	// the account that operate used
	let from = AccountKeyring::Alice.to_account_id();
	let alice_nonce = get_nonce(&from).await;
	// get tor consensus
	let res = reqwest::get("http://localhost:7000/tor/status-vote/current/consensus.z")
		.await
		.unwrap();
	// println!("Headers:\n{:#?}", res.headers());
	match res.headers().get("content-type") {
		None => {
			return HttpResponse::Ok()
				.body("Get consensus failed. Please try again after a moment.");
		},
		Some(_) => {},
	}

	let text = res.text().await.unwrap();
	// println!("text: {text:?}");

	let body = text.clone();
	let new_info: Vec<u8> = body.into();

	let call = (pallet_index, call_index, new_info);
	let extra = (
		// How long should this call "last" in the transaction pool before
		// being deemed "out of date" and discarded?
		Era::Immortal,
		// How many prior transactions have occurred from this account? This
		// Helps protect against replay attacks or accidental double-submissions.
		Compact(alice_nonce),
		// This is a tip, paid to the block producer (and in part the treasury)
		// to help incentive it to include this transaction in the block. Can be 0.
		Compact(500u128),
	);
	let runtime_version = get_runtime_version().await;
	let genesis_hash = get_genesis_hash().await;
	let additional = (
		// This TX won't be valid if it's not executed on the expected runtime version:
		runtime_version.spec_version,
		runtime_version.transaction_version,
		// Genesis hash, so TX is only valid on the correct chain:
		genesis_hash,
		// The block hash of the "checkpoint" block. If the transaction is
		// "immortal", use the genesis hash here. If it's mortal, this block hash
		// should be equal to the block number provided in the Era information,
		// so that the signature can verify that we're looking at the expected block.
		// (one thing that this can help prevent is your transaction executing on the
		// wrong fork; same genesis hash but likely different block hash for mortal tx).
		genesis_hash,
	);
	let signature = {
		// Combine this data together and SCALE encode it:
		let full_unsigned_payload = (&call, &extra, &additional);
		let full_unsigned_payload_scale_bytes = full_unsigned_payload.encode();

		// If payload is longer than 256 bytes, we hash it and sign the hash instead:
		if full_unsigned_payload_scale_bytes.len() > 256 {
			AccountKeyring::Alice.sign(&blake2_256(&full_unsigned_payload_scale_bytes)[..])
		} else {
			AccountKeyring::Alice.sign(&full_unsigned_payload_scale_bytes)
		}
	};
	let signature_to_encode = Some((
		// The account ID that's signing the payload:
		MultiAddress::Id::<_, u32>(from),
		// The actual signature, computed above:
		MultiSignature::Sr25519(signature),
		// Extra information to be included in the transaction:
		extra,
	));
	let payload_scale_encoded = encode_extrinsic(signature_to_encode, call);
	let payload_hex = format!("0x{}", hex::encode(&payload_scale_encoded));

	// Submit it!
	// println!("Submitting this payload: {}", payload_hex);
	println!("Submitting the extrinsic...");
	rpc_to_localhost("author_submitExtrinsic", [payload_hex]).await.unwrap();

	// The result from this call is the hex value for the extrinsic hash.
	// println!("{:?}", res);

	HttpResponse::Ok().body(text)
}


fn main() -> sc_cli::Result<()> {
	let srv = HttpServer::new(|| {
		App::new()
			.service(hello)
			.service(update)
			.default_service(web::to(|| HttpResponse::NotFound()))
	})
	.bind(("localhost", 8080))
	.unwrap()
	.run();
	let t1 = thread::spawn(|| {
		tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.build()
			.unwrap()
			.block_on(async {
				srv.await;
			});
	});

	command::run()?;
	println!("after command::run()");
	t1.join();
	println!("Server exited");
	// let srv_handle = srv.handle();

	Ok(())
	// srv_handle.stop(false).await;
}

/// Fetch the genesis hash from the node.
async fn get_genesis_hash() -> H256 {
	let genesis_hash_json = rpc_to_localhost("chain_getBlockHash", [0]).await.unwrap();
	let genesis_hash_hex = genesis_hash_json.as_str().unwrap();
	H256::from_str(genesis_hash_hex).unwrap()
}

/// Fetch runtime information from the node.
async fn get_runtime_version() -> RuntimeVersion {
	let runtime_version_json = rpc_to_localhost("state_getRuntimeVersion", ()).await.unwrap();
	serde_json::from_value(runtime_version_json).unwrap()
}

/// How many transactions has this account already made?
async fn get_nonce(account: &sp_runtime::AccountId32) -> u32 {
	let nonce_json = rpc_to_localhost("system_accountNextIndex", (account,)).await.unwrap();
	serde_json::from_value(nonce_json).unwrap()
}

/// Encode the extrinsic into the expected format. De-optimised a little
/// for simplicity, and taken from sp_runtime/src/generic/unchecked_extrinsic.rs
fn encode_extrinsic<S: Encode, C: Encode>(signature: Option<S>, call: C) -> Vec<u8> {
	let mut tmp: Vec<u8> = vec![];

	// 1 byte for version ID + "is there a signature".
	// The top bit is 1 if signature present, 0 if not.
	// The remaining 7 bits encode the version number (here, 4).
	const EXTRINSIC_VERSION: u8 = 4;
	match signature.as_ref() {
		Some(s) => {
			tmp.push(EXTRINSIC_VERSION | 0b1000_0000);
			// Encode the signature itself now if it's present:
			s.encode_to(&mut tmp);
		},
		None => {
			tmp.push(EXTRINSIC_VERSION & 0b0111_1111);
		},
	}

	// Encode the call itself after this version+signature stuff.
	call.encode_to(&mut tmp);

	// We'll prefix the encoded data with it's length (compact encoding):
	let compact_len = Compact(tmp.len() as u32);

	// So, the output will consist of the compact encoded length,
	// and then the version+"is there a signature" byte,
	// and then the signature (if any),
	// and then encoded call data.
	let mut output: Vec<u8> = vec![];
	compact_len.encode_to(&mut output);
	output.extend(tmp);

	output
}

// fn latin1_to_string(s: &[u8]) -> String {
//     s.iter().map(|&c| c as char).collect()
// }

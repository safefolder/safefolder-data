extern crate argparse;
extern crate xid;
extern crate serde_yaml;
extern crate colored;
extern crate lazy_static;
extern crate tokio;
use lingua::{LanguageDetector, LanguageDetectorBuilder};

pub mod commands;
pub mod storage;
pub mod planet;
pub mod functions;

// use ipfs_api::IpfsClient;
// use std::io::Cursor;

// use bip32::{Mnemonic, Prefix, XPrv};
// use bip32::secp256k1::ecdsa::{
//     signature::{Signer, Verifier},
//     Signature
// };
// use rand_core::OsRng;

// use async_std::task;
// use libp2p::{
//     Multiaddr,
//     swarm::{Swarm, SwarmEvent},
//     PeerId,
//     identity,
//     development_transport
// };
// use libp2p::kad::{
//     Kademlia,
//     KademliaConfig,
//     KademliaEvent,
//     GetClosestPeersError,
//     QueryResult,
// };
// use libp2p::kad::record::store::MemoryStore;
// use std::{env, error::Error, str::FromStr, time::Duration};

use argparse::{ArgumentParser, StoreTrue, Store};
use std::collections::HashMap;

use crate::commands::CommandRunner;
use crate::commands::folder::Command;
use crate::planet::{PlanetContext, Context, ContextSource};
use planet::constants::*;

// async fn swarm() {
//     const BOOTNODES: [&'static str; 4] = [
//         "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
//         "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
//         "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
//         "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt"
//     ];
//     let local_key = identity::Keypair::generate_ed25519();
//     eprintln!("swarm :: local_key: {:?}", &local_key.public());
//     let local_peer_id = PeerId::from(local_key.public());
//     eprintln!("swarm :: local_peer_id: {:?}", &local_peer_id);
//     let transport = development_transport(local_key).await.unwrap();
//     let swarm = {
//         // Create a Kademlia behaviour.
//         let mut cfg = KademliaConfig::default();
//         cfg.set_query_timeout(Duration::from_secs(5 * 60));
//         let store = MemoryStore::new(local_peer_id);
//         let mut behaviour = Kademlia::with_config(local_peer_id, store, cfg);

//         // Add the bootnodes to the local routing table. `libp2p-dns` built
//         // into the `transport` resolves the `dnsaddr` when Kademlia tries
//         // to dial these nodes.
//         let bootaddr = Multiaddr::from_str("/dnsaddr/bootstrap.libp2p.io").unwrap();
//         for peer in &BOOTNODES {
//             behaviour.add_address(&PeerId::from_str(peer).unwrap(), bootaddr.clone());
//         }

//         Swarm::new(transport, behaviour, local_peer_id)
//     };
//     //eprintln!("swarm :: swarm: {:?}", swarm);
// }


#[tokio::main]
async fn main() {
    // achiever run command ...
    // achiever run action ...
    // achiever run journey ...

    let mut verbose = false;
    let mut account_id: String = String::from("");
    let mut space_id: String = String::from("");
    let mut path_yaml = String::from("");
    let mut command = String::from("");
    let mut op = String::from("run");
    let mut scope = String::from("");
    // println!("account_id: {}", hex::encode_upper(account_id));
    let _: LanguageDetector = LanguageDetectorBuilder::from_languages(&LANGUAGES).with_preloaded_language_models().build();

    { // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Achiever Client Tool");
        ap.refer(&mut verbose).add_option(
            &["-v", "--verbose"], 
            StoreTrue,
            "Be verbose");
        ap.refer(&mut account_id).add_option(
            &["-a", "--accountid"], Store,
            "Account Id");
        ap.refer(&mut space_id).add_option(
            &["-s", "--spaceid"], Store,
            "Space Id");
        ap.refer(&mut path_yaml).add_option(
            &["-c", "--path_yaml"], Store,
            "Path for YAML config file");
        ap.refer(&mut op).add_argument("op", Store, "Operation: run, etc...");
        ap.refer(&mut scope).add_argument("scope", Store, "Scope: command, action, journey");
        ap.refer(&mut command).add_argument("name", Store, "Command name, action name or journey name");
        ap.parse_args_or_exit();
    }

    // Get planet context. I embed planet context into components
    // let planet_context_source, planet_context: PlanetContext = PlanetContext::import_context().unwrap();

    let planet_context_source = PlanetContext::import_context().unwrap();
    let home_path = &planet_context_source.home_path.unwrap();
    let planet_context = PlanetContext{
        mission: &planet_context_source.mission,
        home_path: Some(&home_path.as_str()),
    };

    // Context: This is TEMP, simply context struct, but in future will come from shell, or we create a new one
    // let space_id = hex::decode(space_id.unwrap()).unwrap();
    // let account_id = hex::decode(account_id.unwrap()).unwrap();
    // let space_id = space_id.unwrap().as_bytes();
    let context_source: ContextSource = ContextSource{
        id: None,
        data: Some(HashMap::new()),
        space_id: Some(String::from("private")),
        account_id: None,
        box_id: Some(String::from("base")),
        site_id: None,
    };
    let account_id = &context_source.account_id.unwrap_or_default();
    let space_id = &context_source.space_id.unwrap_or_default();
    let box_id = &context_source.box_id.unwrap_or_default();
    let site_id_wrap = context_source.site_id.clone();
    let site_id = &context_source.site_id.unwrap_or_default();
    let mut context = Context{
        id: None,
        data: None,
        account_id: Some(account_id),
        space_id: Some(space_id),
        box_id: Some(box_id),
        site_id: None,
    };
    if site_id_wrap.is_some() {
        context.site_id = Some(site_id)
    }
    eprintln!("main.rs :: context: {:#?}", &context);

    // // bip32 and encryption
    // let mnemonic = Mnemonic::random(&mut OsRng, Default::default());
    // let seed = mnemonic.to_seed("password");
    // // let root_xprv = XPrv::new(&seed).unwrap();
    // // println!("root_xprv: {:?}", root_xprv);
    // // println!("private key: {:?}", root_xprv.to_string(Prefix::XPRV));
    // // println!("public key: {:?}", root_xprv.public_key().to_string(Prefix::XPUB));
    // let child_path = "m/0/2147483647'/1/2147483646'";
    // let child_xprv = XPrv::derive_from_path(
    //     &seed, &child_path.parse().unwrap()).unwrap();
    // let child_xpub = child_xprv.public_key();
    // let child_xprv_str = &child_xprv.to_string(Prefix::XPRV).to_string();
    // let child_xpub_str = &child_xpub.to_string(Prefix::XPUB);
    // let child_xprv_array = &child_xprv.to_bytes();
    // println!("child private key: {:?} [{}]", &child_xprv_str, &child_xprv_str.len());
    // println!("child public key: {:?} [{}]", &child_xpub_str, &child_xpub_str.len());
    // println!("child private key array: {:?} [{}]", &child_xprv_array, &child_xprv_array.len());
    // let signing_key = child_xprv.private_key();
    // let verification_key = child_xpub.public_key();
    // let example_msg = b"Hello, worlds!";
    // let signature: Signature = signing_key.sign(example_msg);
    // // println!("Signature: {:?}", signature);
    // println!("Veritifcation signature: {:?}", 
    //     verification_key.verify(example_msg, &signature).is_ok(), 
    // );

    // IPFS
    // let mine = swarm();

    // IPFS API (5001)
    // println!("IPFS API....");
    // let client = IpfsClient::default();
    // let data = Cursor::new("Hello World!");
    // match client.add(data).await {
    //     Ok(res) => println!("{}", res.hash),
    //     Err(e) => eprintln!("error adding file: {}", e)
    // }    

    if op.to_lowercase() == "run" && &scope.to_lowercase() == "command" {
        let command_runner =  CommandRunner{
            planet_context: &planet_context,
            context: &context,
            command: &command,
            path_yaml: Some(&path_yaml)
        };
        run_command(command_runner).unwrap();
    }
}

fn run_command(runner: CommandRunner) -> Result<String, String> {
    // CommandRunner: command, account_id, space_id, path_yaml, possible command_file (when get from dir), planet context
    // I also need to create a context if not informed.
    let runner = runner.clone();
    let runner_path_yaml = &runner.path_yaml;
    if runner_path_yaml.is_some() {
        let path_yaml = format!("{}", runner_path_yaml.clone().unwrap());
        let match_option = *&runner.command.as_str();
        match match_option {
            "CREATE FOLDER" => commands::folder::schema::CreateFolder::runner(&runner, &path_yaml),
            "INSERT INTO FOLDER" => commands::folder::data::InsertIntoFolder::runner(&runner, &path_yaml),
            "GET FROM FOLDER" => commands::folder::data::GetFromFolder::runner(&runner, &path_yaml),
            "SELECT FROM FOLDER" => commands::folder::data::SelectFromFolder::runner(&runner, &path_yaml),
            _ => println!("default")
        }
        Ok("Command executed".to_string())
    } else {
        return Err("Path to YAML command not informed".to_string());
    }
}
use blockfrost::{load, BlockFrostApi};
use clap::Parser;
use error_chain::error_chain;
use std::{io::Read, collections::HashSet};
use std::env;
use serde::{Deserialize, Serialize};

fn build_api() -> blockfrost::Result<BlockFrostApi> {
    let configurations = load::configurations_from_env()?;
    let project_id = configurations["project_id"].as_str().unwrap();
    let api = BlockFrostApi::new(project_id, Default::default());
    Ok(api)
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    name: Option<String>,
}

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[derive(Serialize, Deserialize)]
struct CollectionEntry {
    collection_id: String,
    description: String,
    blockchain: String,
    network: String,
}

#[derive(Serialize, Deserialize)]
struct CollectionBody {
    #[serde(rename = "type")]
    collection_type: String,
    data: Vec<CollectionEntry>,
}

fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    let cli = Cli::parse();

    let api = build_api().unwrap();

    let mut res = reqwest::blocking::get("https://api.book.io/api/v0/collections")?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    println!("Body:\n{}", body);

    let status_u16 = res.status().as_u16();
    let mut policy_ids : HashSet<String> = HashSet::new();
    if status_u16 >= 200 && status_u16 < 300 {
        let parsed: CollectionBody = serde_json::from_str(&body).unwrap();
        for data_item in parsed.data {
            policy_ids.insert(data_item.collection_id);
        }
    }

    println!("Policy Ids:\n{:#?}", policy_ids);

    let mut policy_id = String::from("");
    if cli.name.is_some() {
        policy_id = cli.name.unwrap();
        let exists = policy_ids.contains(&policy_id);
        println!("Item {} is a policy_id: {}", policy_id, exists);

        if !exists {
            return Err(format!("policy_id not found: {}", policy_id).into());
        }
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let asset_policy = rt.block_on(api.assets_policy_by_id(&policy_id)).unwrap();
    println!("Asset Policy: {:#?}", asset_policy);

    let mut asset_policies: Vec<String> = Vec::new();
    for ap in asset_policy {
        if ap.asset != policy_id {
            asset_policies.push(ap.asset);
        }
    }

    println!("Asset Policies to inspect: {:#?}", asset_policies);

    for asset_id in asset_policies {
        let asset = rt.block_on(api.assets_by_id(&asset_id)).unwrap();
        println!("Asset Details: {:#?}", asset);
    }

    Ok(())
}

use blockfrost::{load, BlockFrostApi, IpfsApi, IpfsSettings};
use clap::Parser;
use error_chain::error_chain;
use std::fs::{File, self};
use std::io::Write;
use std::path::Path;
use std::{io::Read, collections::HashSet};
use std::env;
use serde::{Deserialize, Serialize};

fn build_api() -> blockfrost::Result<BlockFrostApi> {
    let configurations = load::configurations_from_env()?;
    let project_id = configurations["project_id"].as_str().unwrap();
    let api = BlockFrostApi::new(project_id, Default::default());
    Ok(api)
}

fn build_ipfs() -> blockfrost::Result<IpfsApi> {
    let configurations = load::configurations_from_env()?;
    let project_id = configurations["ipfs_id"].as_str().unwrap();
    let api = IpfsApi::new(project_id, IpfsSettings::new());
    Ok(api)
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The policy ID to look for. Lists all policies if this is omitted.
    name: Option<String>,

    /// The path to save the files in
    #[arg(short, long, default_value_t = String::from("."))]
    path: String,

    /// The maximum number of images to download per policy ID
    #[arg(short, long, default_value_t = 10)]
    quota: u16,
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

#[derive(Serialize, Deserialize)]
struct AssetFile {
    #[serde(rename = "mediaType")]
    media_type: String,
    name: String,
    src: String,
}

// #[derive(Serialize, Deserialize)]
// struct ImageHashHistory {
//     image_hash: String,
//     status: String,
// }

fn main() -> Result<()> {
    //env::set_var("RUST_BACKTRACE", "1");
    let cli = Cli::parse();

    let api = build_api().unwrap();

    let mut res = reqwest::blocking::get("https://api.book.io/api/v0/collections")?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    let status_u16 = res.status().as_u16();
    if status_u16 < 200 || status_u16 >= 300 {
        return Err(format!("Listing policy id's yielded status {}", status_u16).into());
    }

    if cli.name.is_none() {
        // Without a policy_id, user is asking to list what's available, so show the body & exit with Ok
        println!("Body:\n{}", body);
        return Ok(());
    }

    // get the policy_id's
    let mut policy_ids : HashSet<String> = HashSet::new();
    let parsed: CollectionBody = serde_json::from_str(&body).unwrap();
    for data_item in parsed.data {
        policy_ids.insert(data_item.collection_id);
    }

    println!("Policy Ids:\n{:#?}", policy_ids);

    // does the specified policy exist?
    let policy_id = cli.name.unwrap();
    let exists = policy_ids.contains(&policy_id);
    println!("Item {} is a policy_id: {}", policy_id, exists);

    if !exists {
        return Err(format!("policy_id not found: {}", policy_id).into());
    }

    // get the asset policies for the policy id
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

    // Loop below is SLOW, so show a notice so user doesn't think we froze or crashed
    println!("Computing image sources...");

    // get the IPFS src hashes for each asset_id
    let mut image_srcs: HashSet<String> = HashSet::new();
    for asset_id in asset_policies {
        let asset = rt.block_on(api.assets_by_id(&asset_id)).unwrap();
 
        if asset.onchain_metadata.is_some() {
            let asset_map = asset.onchain_metadata.unwrap();
            if asset_map.contains_key("files") {
                let asset_files : Vec<AssetFile> = serde_json::from_value(asset_map["files"].clone()).unwrap();
                for asset_file in asset_files {
                    // If the src starts with ipfs://, then trim it off, else just add it as-is
                    let src_option = asset_file.src.strip_prefix("ipfs://");
                    if src_option.is_some() {
                        image_srcs.insert(src_option.unwrap().to_string());
                    } else {
                        image_srcs.insert(asset_file.src);
                    }

                    if image_srcs.len() as u16 >= cli.quota {
                        break;
                    }
                }
            }
        }

        if image_srcs.len() as u16 >= cli.quota {
            break;
        }
    }

    println!("Image srcs: {:#?}", image_srcs);

    // TODO: Make json file to track list of image hashes & their download status. This file goes in the directory marked by cli.path
    //
    // let policy_path = Path::new(".policy");
    // if !policy_path.exists() {
    //     let create_result = fs::create_dir(policy_path);
    //     if create_result.is_err() {
    //         return Err(format!("unable to make path: {}", policy_path.as_os_str().to_str().unwrap()).into());
    //     }
    // }
    // let policy_file = policy_id + &String::from(".json");


    // TODO: Code to download & clobber incomplete png files from the list of hashes
    //
    // let ipfs_api = build_ipfs().unwrap();

    // let path = Path::new(&cli.path);
    // let png_extension = String::from(".png");

    // for image_src in image_srcs {
    //     let gateway = rt.block_on(ipfs_api.gateway(&image_src)).unwrap();
    //     let fileName = image_src+&png_extension;
    //     let filePath = path.join(fileName);
    //     if (filePath.exists()) {
    //         let rr = fs::remove_file(filePath);
    //         // check rr
    //     }
    //     let mut file_s = File::create(filePath);
    //     if (file_s.is_ok()) {
    //         let mut file = file_s.unwrap();
    //         file.write_all(&gateway);
    //     }
    // }

    Ok(())
}

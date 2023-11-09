### Getting started

All instructions were tested on my Ubuntu 22.04.3 desktop. Your mileage may vary on other OS's. I might consider making a docker or containerd dev-env container to simplify setup, especially for this first section.

Install rust (and family) if you don't have it already:
```bash
$ curl https://sh.rustup.rs -sSf | sh
```

Install tools that cargo needs:
```bash
$ sudo apt-get install gcc
$ sudo apt install build-essential
```
Install pkg-config to find openSSL:
```bash
$ sudo apt install pkg-config libssl-dev
```

### Project id's and toml

On this site (https://blockfrost.io/) add two new projects: One project for network Cardano Mainnet, and one project for network IPFS. You may give these projects any names you want.

Next, Take careful note of the project ID's mentioned for these. Suppose they are mainnetXyz91Uvw8Rst7Pqr6Mno5Jkl4Ghi3Def2 and mainnetAbc1Def2Ghi3Jkl4Mno5Pqr6Stu7Vwx8 for the Cardano and IPFS projects respectively (no, these aren't my actual project ID's...).

Make a file called .blockfrost.toml within this repo (it's among the .gitignore files). The contents of this file should look like:
```toml
project_id = "mainnetXyz91Uvw8Rst7Pqr6Mno5Jkl4Ghi3Def2"
cardano_network = "https://cardano-mainnet.blockfrost.io/api/v0"
ipfs_id = "mainnetAbc1Def2Ghi3Jkl4Mno5Pqr6Stu7Vwx8"
ipfs_network = "https://ipfs.blockfrost.io/api/v0"
```

### Building and running

Build the project via cargo:
```bash
$ cargo build
```

If you wish to list the valid policy id's, do:
```bash
$ ./target/debug/book_images
```

KEEP IN MIND: A collection_id is a policy_id. So look at the output from previous step for a policy_id you wish to get images for.

Suppose you decide you want to download images at relative directory "images" for policy id 6a1388037f4a58d3acd4c121a94a6ebb0ca428a53d4321ce1f7ac28d, then you do:
```bash
$ ./target/debug/book_images 6a1388037f4a58d3acd4c121a94a6ebb0ca428a53d4321ce1f7ac28d -p images
```

If you're unsure of which arguments to try, you can view the help page:
```bash
$ ./target/debug/book_images -h
```

### Under development

- Downloading images (I need a valid project ID to test with for a network IPFS). I have the code written, just need to try it & see if it works.
- Testing non-happy path scenarios, like if JSON from sites do not parse correctly, etc
- Suppose for async downloading of multiple files at same time. For now, just attempting to download 1 file at a time in blocking manner.
- Barely started: Outputting a json files into the user specified path to track download status of image files. The idea here is this: Each json file is in format `<policy-id>.json` and contains a list of image id's we're intending to download, and the status on if we finished downloading them or not. If this json file exists, we can skip trying to locate the 10 image hashes for the policy-id. Next, for the images in this file, we check if the file exists and its status was set to download completed. If yes, we skip, otherwise, we know we still need to download the file & possibly clobber the older copy if that download was incomplete.

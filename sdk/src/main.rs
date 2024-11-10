pub mod utils;
pub mod api;

use aws_config::{meta::region::RegionProviderChain, Region};
use aws_sdk_ec2::{
    types::{InstanceType},
    Client as EC2Client,
};
use utils::{create_key_pair, get_latest_ami_id, get_public_ip, save_private_key, setup_connection};
use std::{
    error::Error,
    net::Ipv4Addr,
    thread,
    time::Duration,
};
use api::EC2Impl;

const INSTANCE_NAME: &str = "MVP";
const KEY_PAIR_NAME: &str = "mvp-key-pair";
const SECURITY_GROUP_NAME: &str = "mvp-security-group";
const SSM_ROLE_NAME: &str = "EC2SSMRole";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let region_provider =
        RegionProviderChain::default_provider().or_else(Region::new("eu-north-1"));
    let config = aws_config::from_env().region(region_provider).load().await;
    let ec2_client = EC2Client::new(&config);
    let ec2 = EC2Impl::new(ec2_client);

    run(&ec2).await?;
    setup_connection(&ec2).await?;
    Ok(())
}



async fn run(ec2: &EC2Impl) -> Result<(), Box<dyn Error>> {
    //Create Security Group
    let security_group = ec2
        .create_security_group_if_not_exists(SECURITY_GROUP_NAME, "Mvp Security Group for SSH")
        .await?;
    println!("Created Security Group: {:#?}", security_group);

    //Create Key Pair
    let key_pair_name = "mvp-key-pair";
    let (key_pair_info, private_key) = create_key_pair(&ec2).await?;
    println!("Created Key Pair: {:#?}", key_pair_info);
    println!("Private Key Material: \n{}", private_key);

    //Save Private Key
    save_private_key(&key_pair_name, &private_key)?;

    //Open Ports 22 and 8000
    let public_ip = get_public_ip().await?;
    println!("Public Ip: {}", public_ip);
    let ingress_ip: Ipv4Addr = public_ip.parse().unwrap();
    let cidr_ip = format!("{}/32", ingress_ip);
    ec2.add_ingress_rule_if_not_exists(&security_group.group_id().unwrap(), ingress_ip, 22)
        .await?;
    ec2.add_ingress_rule_if_not_exists(&security_group.group_id().unwrap(), ingress_ip, 8000)
        .await?;
    println!("Authorized SSH ingress from IP: {}", cidr_ip);

    //Create Instance
    let ami_id = get_latest_ami_id().await?;
    let instance_type = InstanceType::D3enXlarge;
    let instance_id = ec2
        .create_instance(
            ami_id.as_str(),
            instance_type,
            &key_pair_info,
            vec![&security_group],
            INSTANCE_NAME,
            Some(SSM_ROLE_NAME),
        )
        .await?;
    println!("Launched EC2 instance with ID: {}", instance_id);
    // ec2.wait_for_instance_ready(&instance_id, Some(Duration::from_secs(120)))
    //     .await?;
    thread::sleep(Duration::from_secs(10));
    println!("EC2 Instance is ready!");

    Ok(())
}


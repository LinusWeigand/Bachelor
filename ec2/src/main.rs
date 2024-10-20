mod util;

use aws_sdk_ec2::{types::InstanceType, Client as EC2Client};
use aws_sdk_ssm::Client as SSMClient;
use util::EC2Impl;
use std::{
    error::Error,
    fs::{self, create_dir_all, Permissions},
    io::Write,
    net::{Ipv4Addr},
};


#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let config = aws_config::load_from_env().await;
    let ec2_client = EC2Client::new(&config);
    let ec2 = EC2Impl::new(ec2_client);

    let key_pair_name = "mvp-key-pair".to_string();
    let mut key_pair_id: Option<String> = None;
    let key_pairs = ec2.list_key_pair().await?;
    for key_pair in &key_pairs {
        if key_pair.key_name == Some(key_pair_name.clone()) {
            if let Some(id) = &key_pair.key_pair_id {
                key_pair_id = Some(id.clone());
            }
        }
        println!("Key_pair: {:?}", key_pair);
    }
    if let Some(id) = key_pair_id {
        match ec2.delete_key_pair(&id).await {
            Err(e) => println!("Key Pair does not already exist"),
            Ok(_) => println!("Key Pair does already exist: Deleting.."),
        };
    }
    let (key_pair_info, private_key) = ec2.create_key_pair(key_pair_name.clone()).await?;
    println!("Created Key Pair: {:#?}", key_pair_info);
    println!("Private Key Material: \n{}", private_key);
    save_private_key(&key_pair_name, &private_key)?;

    let security_group = ec2
        .create_security_group_if_not_exists("mvp-security-group", "Mvp Security Group for SSH")
        .await?;
    println!("Created Security Group: {:#?}", security_group);

    let public_ip = get_public_ip().await?;
    println!("Public Ip: {}", public_ip);


    let ingress_ip: Ipv4Addr = public_ip.parse().unwrap();
    let cidr_ip = format!("{}/32", ingress_ip);
    ec2.add_ssh_ingress_rule_if_not_exists(&security_group.group_id().unwrap(), ingress_ip)
        .await?;
    println!("Authorized SSH ingress from IP: {}", cidr_ip);

    let ami_id = get_latest_ami_id().await?;
    let instance_type = InstanceType::T4gNano;
    let instance_id = ec2
        .create_instance(ami_id.as_str(), instance_type, &key_pair_info, vec![&security_group], "MVP")
        .await?;

    println!("Launched EC2 instance with ID: {}", instance_id);
    ec2.wait_for_instance_ready(&instance_id, None).await?;
    println!("EC2 Instance is ready!");

    Ok(())
}


pub async fn get_public_ip() -> Result<String, Box<dyn Error>> {
    let response = reqwest::get("https://api.ipify.org").await?;
    let ip_address = response.text().await?;
    Ok(ip_address)
}

pub fn save_private_key(key_name: &str, private_key: &str) -> Result<(), Box<dyn Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let key_path = home_dir.join(".ssh").join(format!("{}.pem", key_name));

    create_dir_all(key_path.parent().unwrap())?;

    let mut file = std::fs::File::create(&key_path)?;
    file.write_all(private_key.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = Permissions::from_mode(0o600);
        fs::set_permissions(&key_path, permissions)?;
    }

    println!("Private key saved to: {:?}", key_path);
    Ok(())
}


pub async fn get_latest_ami_id() -> Result<String, Box<dyn Error>> {
    let config = aws_config::load_from_env().await;
    let ssm_client = SSMClient::new(&config);

    let ami_param_name = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-arm64";
    let ami_param = ssm_client
        .get_parameter()
        .name(ami_param_name)
        .send()
        .await?;

    let ami_id = ami_param.parameter.unwrap().value.unwrap();
    Ok(ami_id)
}


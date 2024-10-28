mod util;

use aws_config::{meta::region::RegionProviderChain, Region};
use aws_sdk_ec2::{
    types::{InstanceType, KeyPairInfo},
    Client as EC2Client,
};
use aws_sdk_ssm::Client as SSMClient;
use std::{
    error::Error,
    fs::{self, create_dir_all, OpenOptions, Permissions},
    io::Write,
    net::Ipv4Addr,
    thread,
    time::Duration,
};
use util::EC2Impl;

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

async fn setup_connection(ec2: &EC2Impl) -> Result<(), Box<dyn Error>> {
    let instance_id = ec2
        .get_instance_id_by_name_if_running(INSTANCE_NAME)
        .await?
        .unwrap();

    let ec2_ip = ec2.get_instance_public_ip(&instance_id).await?.unwrap();

    //Configure connect.sh
    let file_path = "./connect.sh";
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open(file_path)?;
    writeln!(
        file,
        "{}",
        format!("ssh -i ~/.ssh/{}.pem ec2-user@{}", KEY_PAIR_NAME, ec2_ip)
    )?;
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
    let instance_type = InstanceType::T4gNano;
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
    thread::sleep(Duration::from_secs(20));
    println!("EC2 Instance is ready!");

    Ok(())
}

async fn create_key_pair(ec2: &EC2Impl) -> Result<(KeyPairInfo, String), Box<dyn Error>> {
    let mut key_pair_id: Option<String> = None;
    let key_pairs = ec2.list_key_pair().await?;
    for key_pair in &key_pairs {
        if key_pair.key_name == Some(KEY_PAIR_NAME.to_string()) {
            if let Some(id) = &key_pair.key_pair_id {
                key_pair_id = Some(id.clone());
            }
        }
        println!("Key_pair: {:?}", key_pair);
    }
    if let Some(id) = key_pair_id {
        match ec2.delete_key_pair(&id).await {
            Err(e) => println!("Key Pair does not already exist: {:?}", e),
            Ok(_) => println!("Key Pair does already exist: Deleting.."),
        };
    }
    let (key_pair_info, private_key) = ec2.create_key_pair(KEY_PAIR_NAME.to_string()).await?;
    Ok((key_pair_info, private_key.to_string()))
}

async fn get_public_ip() -> Result<String, Box<dyn Error>> {
    let response = reqwest::get("https://api.ipify.org").await?;
    let ip_address = response.text().await?;
    Ok(ip_address)
}

fn save_private_key(key_name: &str, private_key: &str) -> Result<(), Box<dyn Error>> {
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

async fn get_latest_ami_id() -> Result<String, Box<dyn Error>> {
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

async fn setup_instance_via_ssm(
    ssm_client: &SSMClient,
    instance_id: &str,
) -> Result<(), Box<dyn Error>> {
    let binary_url = "https://github.com/LinusWeigand/Bachelor/releases/download/mvp-1.0.0/mvp";
    let wget_command = format!("wget {} -O ~/mvp", binary_url);

    let commands = vec![
        "sudo yum update -y",
        "sudo yum install -y wget",
        wget_command.as_str(),
        "chmod +x ~/mvp",
        "sudo mkdir -p /mnt/ebs",
        "sudo mount /dev/xvdb /mnt/ebs",
        "~/mvp",
    ];

    let document_name = "AWS-RunShellScript";

    let send_command_output = ssm_client
        .send_command()
        .instance_ids(instance_id)
        .document_name(document_name)
        .parameters("commands", commands.iter().map(|s| s.to_string()).collect())
        .send()
        .await?;

    let command_id = send_command_output
        .command()
        .and_then(|cmd| cmd.command_id())
        .ok_or("Failed to get command ID")?;

    println!("Sent SSM command with ID: {}", command_id);
    Ok(())
}

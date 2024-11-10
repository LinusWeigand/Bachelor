use aws_sdk_ec2::types::KeyPairInfo;
use aws_sdk_ssm::Client as SSMClient;
use std::{error::Error, fs::{create_dir_all, set_permissions, OpenOptions, Permissions}, io::Write};

use crate::{api::EC2Impl, INSTANCE_NAME, KEY_PAIR_NAME};

pub async fn create_key_pair(ec2: &EC2Impl) -> Result<(KeyPairInfo, String), Box<dyn Error>> {
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
        set_permissions(&key_path, permissions)?;
    }

    println!("Private key saved to: {:?}", key_path);
    Ok(())
}

pub async fn get_latest_ami_id() -> Result<String, Box<dyn Error>> {
    let config = aws_config::load_from_env().await;
    let ssm_client = SSMClient::new(&config);

    // let ami_param_name = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-arm64"; // ARM
    let ami_param_name = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64"; //x_85
    let ami_param = ssm_client
        .get_parameter()
        .name(ami_param_name)
        .send()
        .await?;

    let ami_id = ami_param.parameter.unwrap().value.unwrap();
    Ok(ami_id)
}

pub async fn setup_instance_via_ssm(
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

pub async fn setup_connection(ec2: &EC2Impl) -> Result<(), Box<dyn Error>> {
    let instance_id = ec2
        .get_instance_id_by_name_if_running(INSTANCE_NAME)
        .await?
        .unwrap();

    let ec2_ip = ec2.get_instance_public_ip(&instance_id).await?.unwrap();

    //Configure connect.sh
    let file_path = "./ip.sh";
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open(file_path)?;
    writeln!(file, "{}", format!("export IP={}", ec2_ip))?;
    Ok(())
}

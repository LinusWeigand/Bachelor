// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// This code is modified

use std::{net::Ipv4Addr, time::Duration};

use aws_sdk_ec2::{
    client::Waiters,
    error::ProvideErrorMetadata,
    operation::{
        allocate_address::AllocateAddressOutput, associate_address::AssociateAddressOutput,
    },
    types::{
        BlockDeviceMapping, DomainType, EbsBlockDevice, Filter, Image, Instance, InstanceStateName, InstanceType, IpPermission, IpRange, KeyPairInfo, SecurityGroup, Tag
    },
    Client as EC2Client,
};
use aws_sdk_ssm::types::Parameter;
use aws_smithy_runtime_api::client::waiters::error::WaiterError;

#[derive(Clone)]
pub struct EC2Impl {
    pub client: EC2Client,
}

impl EC2Impl {
    pub fn new(client: EC2Client) -> Self {
        EC2Impl { client }
    }


    //Method added
    pub async fn get_instance_public_ip(
        &self,
        instance_id: &str,
    ) -> Result<Option<String>, EC2Error> {
        let response = self
            .client
            .describe_instances()
            .instance_ids(instance_id)
            .send()
            .await?;

        let reservations = response.reservations();
        if let Some(instance) = reservations
            .iter()
            .flat_map(|r| r.instances())
            .find(|i| i.instance_id().unwrap() == instance_id)
        {
            return Ok(instance.public_ip_address().map(|ip| ip.to_string()));
        }

        Err(EC2Error::new(format!(
            "No public IP found for instance {}",
            instance_id
        )))
    }

    //Method added
    pub async fn add_ssh_ingress_rule_if_not_exists(
        &self,
        group_id: &str,
        ingress_ip: Ipv4Addr,
    ) -> Result<(), EC2Error> {
        let describe_output = self
            .client
            .describe_security_groups()
            .group_ids(group_id)
            .send()
            .await?;
        let mut security_group_exists = false;
        let security_groups = describe_output.security_groups();
        if let Some(security_group) = security_groups.first() {
            for permission in security_group.ip_permissions() {
                if Some(22) == permission.from_port()
                    && Some(22) == permission.to_port()
                    && Some("tcp") == permission.ip_protocol().as_deref()
                {
                    for range in permission.ip_ranges() {
                        if range.cidr_ip().as_deref() == Some(&format!("{}/32", ingress_ip)) {
                            security_group_exists = true;
                        }
                    }
                }
            }
        }

        if security_group_exists {
            println!("SSH ingress rule already exists");
            return Ok(());
        }

        self.authorize_security_group_ssh_ingress(group_id, vec![ingress_ip])
            .await?;
        Ok(())
    }

    //Method added
    pub async fn create_security_group_if_not_exists(
        &self,
        group_name: &str,
        description: &str,
    ) -> Result<SecurityGroup, EC2Error> {
        let existing_group = self.describe_security_group_by_name(group_name).await?;

        if let Some(group) = existing_group {
            println!(
                "Security group '{}' already exists: {:#?}",
                group_name, group
            );
            return Ok(group);
        }

        let security_group = self.create_security_group(group_name, description).await?;
        Ok(security_group)
    }

    //Method added
    pub async fn describe_security_group_by_name(
        &self,
        group_name: &str,
    ) -> Result<Option<SecurityGroup>, EC2Error> {
        let name_filter = Filter::builder()
            .name("group-name")
            .values(group_name)
            .build();

        let describe_output = self
            .client
            .describe_security_groups()
            .filters(name_filter)
            .send()
            .await?;

        let groups = describe_output.security_groups.unwrap_or_default();
        if groups.is_empty() {
            Ok(None)
        } else {
            Ok(Some(groups[0].clone()))
        }
    }

    //Method added
    pub async fn get_instance_id_by_name_if_running(
        &self,
        instance_name: &str
    ) -> Result<Option<String>, EC2Error> {
        let name_filter = Filter::builder()
            .name("tag:Name")
            .values(instance_name)
            .build();

        let response = self
            .client
            .describe_instances()
            .filters(name_filter)
            .send()
            .await?;

            //TODO
        if let Some(instance) = response
            .reservations()
            .iter()
            .flat_map(|res| res.instances())
            .find(|instance| {
                if let Some(state) = instance.state() {
                    if let Some(state_name) = state.name() {
                        state_name == &InstanceStateName::Running
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        {
            if let Some(instance_id) = instance.instance_id() {
                return Ok(Some(instance_id.to_string()))
            }
        }


        Ok(None)
    }

    // snippet-start:[ec2.rust.create_key.impl]
    pub async fn create_key_pair(&self, name: String) -> Result<(KeyPairInfo, String), EC2Error> {
        tracing::info!("Creating key pair {name}");
        let output = self.client.create_key_pair().key_name(name).send().await?;
        let info = KeyPairInfo::builder()
            .set_key_name(output.key_name)
            .set_key_fingerprint(output.key_fingerprint)
            .set_key_pair_id(output.key_pair_id)
            .build();
        let material = output
            .key_material
            .ok_or_else(|| EC2Error::new("Create Key Pair has no key material"))?;
        Ok((info, material))
    }
    // snippet-end:[ec2.rust.create_key.impl]

    // snippet-start:[ec2.rust.list_keys.impl]
    pub async fn list_key_pair(&self) -> Result<Vec<KeyPairInfo>, EC2Error> {
        let output = self.client.describe_key_pairs().send().await?;
        Ok(output.key_pairs.unwrap_or_default())
    }
    // snippet-end:[ec2.rust.list_keys.impl]

    // snippet-start:[ec2.rust.delete_key.impl]
    pub async fn delete_key_pair(&self, key_pair_id: &str) -> Result<(), EC2Error> {
        let key_pair_id: String = key_pair_id.into();
        tracing::info!("Deleting key pair {key_pair_id}");
        self.client
            .delete_key_pair()
            .key_pair_id(key_pair_id)
            .send()
            .await?;
        Ok(())
    }
    // snippet-end:[ec2.rust.delete_key.impl]

    // snippet-start:[ec2.rust.create_security_group.impl]
    pub async fn create_security_group(
        &self,
        name: &str,
        description: &str,
    ) -> Result<SecurityGroup, EC2Error> {
        tracing::info!("Creating security group {name}");
        let create_output = self
            .client
            .create_security_group()
            .group_name(name)
            .description(description)
            .send()
            .await
            .map_err(EC2Error::from)?;

        let group_id = create_output
            .group_id
            .ok_or_else(|| EC2Error::new("Missing security group id after creation"))?;

        let group = self
            .describe_security_group(&group_id)
            .await?
            .ok_or_else(|| {
                EC2Error::new(format!("Could not find security group with id {group_id}"))
            })?;

        tracing::info!("Created security group {name} as {group_id}");

        Ok(group)
    }
    // snippet-end:[ec2.rust.create_security_group.impl]

    // snippet-start:[ec2.rust.describe_security_group.impl]
    /// Find a single security group, by ID. Returns Err if multiple groups are found.
    pub async fn describe_security_group(
        &self,
        group_id: &str,
    ) -> Result<Option<SecurityGroup>, EC2Error> {
        let group_id: String = group_id.into();
        let describe_output = self
            .client
            .describe_security_groups()
            .group_ids(&group_id)
            .send()
            .await?;

        let mut groups = describe_output.security_groups.unwrap_or_default();

        match groups.len() {
            0 => Ok(None),
            1 => Ok(Some(groups.remove(0))),
            _ => Err(EC2Error::new(format!(
                "Expected single group for {group_id}"
            ))),
        }
    }
    // snippet-end:[ec2.rust.describe_security_group.impl]

    // snippet-start:[ec2.rust.authorize_security_group_ssh_ingress.impl]
    /// Add an ingress rule to a security group explicitly allowing IPv4 address
    /// as {ip}/32 over TCP port 22.
    pub async fn authorize_security_group_ssh_ingress(
        &self,
        group_id: &str,
        ingress_ips: Vec<Ipv4Addr>,
    ) -> Result<(), EC2Error> {
        tracing::info!("Authorizing ingress for security group {group_id}");
        self.client
            .authorize_security_group_ingress()
            .group_id(group_id)
            .set_ip_permissions(Some(
                ingress_ips
                    .into_iter()
                    .map(|ip| {
                        IpPermission::builder()
                            .ip_protocol("tcp")
                            .from_port(22)
                            .to_port(22)
                            .ip_ranges(IpRange::builder().cidr_ip(format!("{ip}/32")).build())
                            .build()
                    })
                    .collect(),
            ))
            .send()
            .await?;
        Ok(())
    }
    // snippet-end:[ec2.rust.authorize_security_group_ssh_ingress.impl]

    // snippet-start:[ec2.rust.delete_security_group.impl]
    pub async fn delete_security_group(&self, group_id: &str) -> Result<(), EC2Error> {
        tracing::info!("Deleting security group {group_id}");
        self.client
            .delete_security_group()
            .group_id(group_id)
            .send()
            .await?;
        Ok(())
    }
    // snippet-end:[ec2.rust.delete_security_group.impl]

    // snippet-start:[ec2.rust.list_images.impl]
    pub async fn list_images(&self, ids: Vec<Parameter>) -> Result<Vec<Image>, EC2Error> {
        let image_ids = ids.into_iter().filter_map(|p| p.value).collect();
        let output = self
            .client
            .describe_images()
            .set_image_ids(Some(image_ids))
            .send()
            .await?;

        let images = output.images.unwrap_or_default();
        if images.is_empty() {
            Err(EC2Error::new("No images for selected AMIs"))
        } else {
            Ok(images)
        }
    }
    // snippet-end:[ec2.rust.list_images.impl]

    // snippet-start:[ec2.rust.list_instance_types.impl]
    /// List instance types that match an image's architecture and are free tier eligible.
    pub async fn list_instance_types(&self, image: &Image) -> Result<Vec<InstanceType>, EC2Error> {
        let architecture = format!(
            "{}",
            image.architecture().ok_or_else(|| EC2Error::new(format!(
                "Image {:?} does not have a listed architecture",
                image.image_id()
            )))?
        );
        let free_tier_eligible_filter = Filter::builder()
            .name("free-tier-eligible")
            .values("false")
            .build();
        let supported_architecture_filter = Filter::builder()
            .name("processor-info.supported-architecture")
            .values(architecture)
            .build();
        let response = self
            .client
            .describe_instance_types()
            .filters(free_tier_eligible_filter)
            .filters(supported_architecture_filter)
            .send()
            .await?;

        Ok(response
            .instance_types
            .unwrap_or_default()
            .into_iter()
            .filter_map(|iti| iti.instance_type)
            .collect())
    }
    // snippet-end:[ec2.rust.list_instance_types.impl]

    // snippet-start:[ec2.rust.create_instance.impl]
    // Modified
    pub async fn create_instance<'a>(
        &self,
        image_id: &'a str,
        instance_type: InstanceType,
        key_pair: &'a KeyPairInfo,
        security_groups: Vec<&'a SecurityGroup>,
        name: &str,
    ) -> Result<String, EC2Error> {
        let ebs_volumes = vec![
            BlockDeviceMapping::builder()
                .device_name("/dev/xvda")
                .ebs(
                    EbsBlockDevice::builder()
                        .volume_size(8)
                        .volume_type(aws_sdk_ec2::types::VolumeType::Standard)
                        .build(),
                )
                .build(),
            BlockDeviceMapping::builder()
                .device_name("/dev/xvdb")
                .ebs(
                    EbsBlockDevice::builder()
                        .volume_size(125)
                        .volume_type(aws_sdk_ec2::types::VolumeType::Sc1)
                        .build(),
                )
                .build(),
            BlockDeviceMapping::builder()
                .device_name("/dev/xvdc")
                .ebs(
                    EbsBlockDevice::builder()
                        .volume_size(1)
                        .volume_type(aws_sdk_ec2::types::VolumeType::Standard)
                        .build(),
                )
                .build(),
        ];

        let run_instances = self
            .client
            .run_instances()
            .image_id(image_id)
            .instance_type(instance_type)
            .key_name(
                key_pair
                    .key_name()
                    .ok_or_else(|| EC2Error::new("Missing key name when launching instance"))?,
            )
            .set_security_group_ids(Some(
                security_groups
                    .iter()
                    .filter_map(|sg| sg.group_id.clone())
                    .collect(),
            ))
            .set_block_device_mappings(Some(ebs_volumes))
            .min_count(1)
            .max_count(1)
            .send()
            .await?;

        if run_instances.instances().is_empty() {
            return Err(EC2Error::new("Failed to create instance"));
        }

        let instance_id = run_instances.instances()[0].instance_id().unwrap();
        let response = self
            .client
            .create_tags()
            .resources(instance_id)
            .tags(Tag::builder().key("Name").value(name).build())
            .send()
            .await;

        match response {
            Ok(_) => tracing::info!("Created {instance_id} and applied tags."),
            Err(err) => {
                tracing::info!("Error applying tags to {instance_id}: {err:?}");
                return Err(err.into());
            }
        }

        tracing::info!("Instance is created.");

        Ok(instance_id.to_string())
    }
    // snippet-end:[ec2.rust.create_instance.impl]

    // snippet-start:[ec2.rust.wait_for_instance_ready.impl]
    /// Wait for an instance to be ready and status ok (default wait 60 seconds)
    pub async fn wait_for_instance_ready(
        &self,
        instance_id: &str,
        duration: Option<Duration>,
    ) -> Result<(), EC2Error> {
        self.client
            .wait_until_instance_status_ok()
            .instance_ids(instance_id)
            .wait(duration.unwrap_or(Duration::from_secs(60)))
            .await
            .map_err(|err| match err {
                WaiterError::ExceededMaxWait(exceeded) => EC2Error(format!(
                    "Exceeded max time ({}s) waiting for instance to start.",
                    exceeded.max_wait().as_secs()
                )),
                _ => EC2Error::from(err),
            })?;
        Ok(())
    }
    // snippet-end:[ec2.rust.wait_for_instance_ready.impl]

    // snippet-start:[ec2.rust.describe_instance.impl]
    pub async fn describe_instance(&self, instance_id: &str) -> Result<Instance, EC2Error> {
        let response = self
            .client
            .describe_instances()
            .instance_ids(instance_id)
            .send()
            .await?;

        let instance = response
            .reservations()
            .first()
            .ok_or_else(|| EC2Error::new(format!("No instance reservations for {instance_id}")))?
            .instances()
            .first()
            .ok_or_else(|| {
                EC2Error::new(format!("No instances in reservation for {instance_id}"))
            })?;

        Ok(instance.clone())
    }
    // snippet-end:[ec2.rust.describe_instance.impl]

    // snippet-start:[ec2.rust.start_instance.impl]
    pub async fn start_instance(&self, instance_id: &str) -> Result<(), EC2Error> {
        tracing::info!("Starting instance {instance_id}");

        self.client
            .start_instances()
            .instance_ids(instance_id)
            .send()
            .await?;

        tracing::info!("Started instance.");

        Ok(())
    }
    // snippet-end:[ec2.rust.start_instance.impl]

    // snippet-start:[ec2.rust.stop_instance.impl]
    pub async fn stop_instance(&self, instance_id: &str) -> Result<(), EC2Error> {
        tracing::info!("Stopping instance {instance_id}");

        self.client
            .stop_instances()
            .instance_ids(instance_id)
            .send()
            .await?;

        self.wait_for_instance_stopped(instance_id, None).await?;

        tracing::info!("Stopped instance.");

        Ok(())
    }
    // snippet-end:[ec2.rust.stop_instance.impl]

    // snippet-start:[ec2.rust.reboot_instance.impl]
    pub async fn reboot_instance(&self, instance_id: &str) -> Result<(), EC2Error> {
        tracing::info!("Rebooting instance {instance_id}");

        self.client
            .reboot_instances()
            .instance_ids(instance_id)
            .send()
            .await?;

        Ok(())
    }
    // snippet-end:[ec2.rust.reboot_instance.impl]

    // snippet-start:[ec2.rust.wait_for_instance_stopped.impl]
    pub async fn wait_for_instance_stopped(
        &self,
        instance_id: &str,
        duration: Option<Duration>,
    ) -> Result<(), EC2Error> {
        self.client
            .wait_until_instance_stopped()
            .instance_ids(instance_id)
            .wait(duration.unwrap_or(Duration::from_secs(60)))
            .await
            .map_err(|err| match err {
                WaiterError::ExceededMaxWait(exceeded) => EC2Error(format!(
                    "Exceeded max time ({}s) waiting for instance to stop.",
                    exceeded.max_wait().as_secs(),
                )),
                _ => EC2Error::from(err),
            })?;
        Ok(())
    }
    // snippet-end:[ec2.rust.wait_for_instance_stopped.impl]

    // snippet-start:[ec2.rust.delete_instance.impl]
    pub async fn delete_instance(&self, instance_id: &str) -> Result<(), EC2Error> {
        tracing::info!("Deleting instance with id {instance_id}");
        self.stop_instance(instance_id).await?;
        self.client
            .terminate_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        self.wait_for_instance_terminated(instance_id).await?;
        tracing::info!("Terminated instance with id {instance_id}");
        Ok(())
    }
    // snippet-end:[ec2.rust.delete_instance.impl]

    // snippet-start:[ec2.rust.wait_for_instance_terminated.impl]
    async fn wait_for_instance_terminated(&self, instance_id: &str) -> Result<(), EC2Error> {
        self.client
            .wait_until_instance_terminated()
            .instance_ids(instance_id)
            .wait(Duration::from_secs(60))
            .await
            .map_err(|err| match err {
                WaiterError::ExceededMaxWait(exceeded) => EC2Error(format!(
                    "Exceeded max time ({}s) waiting for instance to terminate.",
                    exceeded.max_wait().as_secs(),
                )),
                _ => EC2Error::from(err),
            })?;
        Ok(())
    }
    // snippet-end:[ec2.rust.wait_for_instance_terminated.impl]

    // snippet-start:[ec2.rust.allocate_address.impl]
    pub async fn allocate_ip_address(&self) -> Result<AllocateAddressOutput, EC2Error> {
        self.client
            .allocate_address()
            .domain(DomainType::Vpc)
            .send()
            .await
            .map_err(EC2Error::from)
    }
    // snippet-end:[ec2.rust.allocate_address.impl]

    // snippet-start:[ec2.rust.deallocate_address.impl]
    pub async fn deallocate_ip_address(&self, allocation_id: &str) -> Result<(), EC2Error> {
        self.client
            .release_address()
            .allocation_id(allocation_id)
            .send()
            .await?;
        Ok(())
    }
    // snippet-end:[ec2.rust.deallocate_address.impl]

    // snippet-start:[ec2.rust.associate_address.impl]
    pub async fn associate_ip_address(
        &self,
        allocation_id: &str,
        instance_id: &str,
    ) -> Result<AssociateAddressOutput, EC2Error> {
        let response = self
            .client
            .associate_address()
            .allocation_id(allocation_id)
            .instance_id(instance_id)
            .send()
            .await?;
        Ok(response)
    }
    // snippet-end:[ec2.rust.associate_address.impl]

    // snippet-start:[ec2.rust.disassociate_address.impl]
    pub async fn disassociate_ip_address(&self, association_id: &str) -> Result<(), EC2Error> {
        self.client
            .disassociate_address()
            .association_id(association_id)
            .send()
            .await?;
        Ok(())
    }
    // snippet-end:[ec2.rust.disassociate_address.impl]
}

// snippet-start:[ec2.rust.ec2error.impl]
#[derive(Debug)]
pub struct EC2Error(String);
impl EC2Error {
    pub fn new(value: impl Into<String>) -> Self {
        EC2Error(value.into())
    }

    pub fn add_message(self, message: impl Into<String>) -> Self {
        EC2Error(format!("{}: {}", message.into(), self.0))
    }
}

impl<T: ProvideErrorMetadata> From<T> for EC2Error {
    fn from(value: T) -> Self {
        EC2Error(format!(
            "{}: {}",
            value
                .code()
                .map(String::from)
                .unwrap_or("unknown code".into()),
            value
                .message()
                .map(String::from)
                .unwrap_or("missing reason".into()),
        ))
    }
}

impl std::error::Error for EC2Error {}

impl std::fmt::Display for EC2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
// snippet-end:[ec2.rust.ec2error.impl]

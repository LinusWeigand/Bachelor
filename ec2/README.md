# Steps required to execute this program

## 1. Aws sdk needs access to your account:
- Create user in the aws console
- execute: `aws configure` in your console and type in your credentials

## 2. Add SSM Role 
-- Create Role: AWS-Service, EC2, AmazonSSMManagedInstanceCore, 
    with name: EC2SSMRole

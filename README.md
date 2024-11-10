# Project Structure

## 1. Data Analysis
````cd data_analysis``````
This project takes its data from three different sources, transforms and plots them.
There are three binaries:

1. Fio: Plots the measured metrics from fio
````cargo run --bin fio``````
2. Vantage: Plots the Data from Vantage and compares EC2 Instances and EBS Volumes
````cargo run --bin vantage``````
3. Snowset: Plots different Graphs from analysing the Snowset Dataset
````cargo run --bin snowset``````

## 2. SDK
````cd sdk``````
This project creates a new EC2 Instance with the help of Amazon's SDK
````cargo run ``````

## 3. MVP
````cd mvp``````
This project has two binaries: The client and the server.

1. The Server has to endpoints: GET & PUT to to upload and download parquet files from a folder on disk
````cargo run --bin server``````
1. The client send / receives parquet files from the server for a specified amount of time to load test the server.
````cargo run --bin client``````

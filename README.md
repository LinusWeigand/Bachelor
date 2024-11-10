# Project Structure

## 1. Data Analysis

This project takes its data from three different sources, transforms and plots them.
There are three binaries:

1. Fio: Plots the measured metrics from fio
2. Vantage: Plots the Data from Vantage and compares EC2 Instances and EBS Volumes
3. Snowset: Plots different Graphs from analysing the Snowset Dataset

## 2. SDK

This project creates a new EC2 Instance with the help of Amazon's SDK

## 3. MVP

This project has two binaries: The client and the server.

1. The Server has to endpoints: GET & PUT to to upload and download parquet files from a folder on disk
1. The client send / receives parquet files from the server for a specified amount of time to load test the server.

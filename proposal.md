# Ziele

##  Viability von einem nachgebauten S3 testen
- Network connected EBS vs. Internal EC2 Storage: Latency, Throughput, Cost
- HDD vs SDD: Latency, Throughput, Cost
- S3 vs our implementation: Latency, Throughput, Cost, Durability (Theoretical)
- script to compute and visuali

## Implementation
- Object Storage based on distributed EC2 Instances and possibly EBS Volumes
- Durability through RAID-like sharding and erasure coding
- In-Memory Metadata Service (Key->Value Store)
- HTTP REST API for GET, PUT & AWS Athena-like aggregating & filtering, ...
- Near storage computation (aggregation & filtering & other) while reading data
- Parquet file support
- distributed near storage computation
- Parallel PUTs & GETs (range)
- maybe direct streaming api to cut out the middleman (metadata-service)


## Evaluation
- per iteration: where is the bottleneck
- Near Storage computation (aggragation & filtering) benchmarke



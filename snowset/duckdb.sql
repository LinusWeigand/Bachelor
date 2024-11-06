-- Get Everything 
SELECT * FROM './snowset-main.parquet';

-- Get Number of Warehouses: 2051
SELECT COUNT(DISTINCT warehouseId) AS unique_warehouses
FROM './snowset-main.parquet/*.parquet';

-- Get Number of Queries: 69182074
SELECT COUNT(DISTINCT queryId) AS unique_queries
FROM './snowset-main.parquet/*.parquet';


-- Get Estimated WarehouseSize
SELECT 
    warehouseId,
    AVG((scanBytes / NULLIF(scanAssignedFiles, 0)) * scanOriginalFiles) / 1000000000 AS estimated_warehouse_size_gb
FROM 
    'snowset-main.parquet/*.parquet'
GROUP BY 
    warehouseId;

-- Mean, Avg, Max per warehouse_size
SELECT 
    AVG(estimated_warehouse_size_gb) AS overall_avg_size_gb,
    MEDIAN(estimated_warehouse_size_gb) AS overall_median_size_gb,
    MAX(estimated_warehouse_size_gb) AS overall_max_size_gb
FROM (
    SELECT 
        warehouseId,
        AVG((scanBytes / NULLIF(scanAssignedFiles, 0)) * scanOriginalFiles) / 1000000000 AS estimated_warehouse_size_gb
    FROM 
        'snowset-main.parquet/*.parquet'
    GROUP BY 
        warehouseId
) AS warehouse_sizes;

-- Get Throughput per stored GB
SELECT 
    MEDIAN(scanbytes_per_estimated_size) AS median_gb_read_per_warehouse_size,
    AVG(scanbytes_per_estimated_size) AS avg_gb_read_per_warehouse_size
FROM (
  SELECT 
    warehouseId,
    (SUM(scanBytes) / NULLIF(AVG((scanBytes / NULLIF(scanAssignedFiles, 0)) * scanOriginalFiles), 0)) AS scanbytes_per_estimated_size
  FROM 
    'snowset-main.parquet/*.parquet'
  GROUP BY 
    warehouseId
) AS gb_read_per_size;

-- Outliers
SELECT 
    warehouseId,
    (SUM(scanBytes) / NULLIF(AVG((scanBytes / NULLIF(scanAssignedFiles, 0)) * scanOriginalFiles), 0)) AS scanbytes_per_estimated_size
FROM 
    'snowset-main.parquet/*.parquet'
GROUP BY 
    warehouseId
ORDER BY 
    scanbytes_per_estimated_size DESC
LIMIT 40;

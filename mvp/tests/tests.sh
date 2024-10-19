# Health Check
curl -X GET http://localhost:8000/api/healthchecker

# PUT Parquet File
curl -X PUT -F "file=@your_file.parquet" http://localhost:8000/parquet/your_file.parquet

# GEt Parquet File
curl -v http://localhost:8000/parquet/your_file.parquet

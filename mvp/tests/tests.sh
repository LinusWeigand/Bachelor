# Health Check
curl -X GET http://localhost:8000/api/healthchecker

# PUT Parquet File
curl -X PUT http://localhost:8000/parquet/test_file.parquet \
  -F "file=@/Users/linusweigand/Universität/7.Semester/Bachelor/mvp/tests/parquet_files/output.parquet"

# GET Parquet File
curl -X GET http://localhost:8000/parquet/test_file.parquet --output download.parquet

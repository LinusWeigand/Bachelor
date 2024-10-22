# Health Check
export IP_ADDRESS=18.159.254.5

curl -X GET http://localhost:8000/api/healthchecker
curl -X GET http://$IP_ADDRESS:8000/api/healthchecker

# PUT Parquet File
curl -X PUT http://localhost:8000/parquet/test_file.parquet \
  -F "file=@/Users/linusweigand/Universität/7.Semester/Bachelor/mvp/tests/parquet_files/output.parquet"

curl -X PUT http://$IP_ADDRESS:8000/parquet/test_file.parquet \
  -F "file=@/Users/linusweigand/Universität/7.Semester/Bachelor/mvp/tests/parquet_files/output.parquet"

# GET Parquet File
URL="http://$IP_ADDRESS:8000/parquet/test_file.parquet"
OUTPUT_FILE="download.parquet"
status_code=$(curl -s -o "$OUTPUT_FILE" -w "%{http_code}" "$URL")
echo $status_code
curl -X GET http://localhost:8000/parquet/test_file.parquet --output download.parquet
curl -X GET http://$IP_ADDRESS:8000/parquet/test_file.parquet --output download.parquet

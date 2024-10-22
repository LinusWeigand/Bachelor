duckdb

CREATE TABLE people (name VARCHAR, age INTEGER);
INSERT INTO people VALUES ('Alice', 34), ('Bob', 45), ('Charlie', 23);
COPY people TO 'output.parquet' (FORMAT 'parquet');

SELECT * FROM 'output.parquet';
SELECT * FROM 'download.parquet';


SELECT * FROM 'test_file2.parquet';

// use duckdb::{Connection, Statement};
//
// pub fn get_avg_scan_bytes_per_estimated_size() -> Result<Vec<(u32, f64)>, Box<dyn std::error::Error>> {
//     let conn = Connection::open_in_memory()?;
//     let query = "
//         SELECT 
//             warehouseId, 
//             scanbytes_per_estimated_size
//         FROM (
//             SELECT 
//                 warehouseId,
//                 (SUM(scanBytes) / NULLIF(AVG((scanBytes / NULLIF(scanAssignedFiles, 0)) * scanOriginalFiles), 0)) AS scanbytes_per_estimated_size
//             FROM 'snowset-main.parquet/*.parquet'
//             GROUP BY warehouseId
//         )
//     ";
//     let mut stmt: Statement = conn.prepare(query)?;
//     let mut result = Vec::new();
//     let rows = stmt.query_map([], |row| {
//         let warehouse_id: u32 = row.get(0)?;
//         let scanbytes_per_estimated_size: f64 = row.get(1)?;
//         Ok((warehouse_id, scanbytes_per_estimated_size))
//     })?;
//
//     for row in rows {
//         result.push(row?);
//     }
//
//     Ok(result)
// }
//
//

use tokio_postgres::{NoTls, Error};
use crate::{ts_kv, DatabaseConfig};

pub(crate) async fn query_data(target_name: &str) -> Result<Vec<ts_kv::TsKV>, Error>  {
    // 连接到 PostgreSQL
    let (client, connection) =
        tokio_postgres::connect("host=120.77.180.48 user=tb_view dbname=thingsboard_prod password=DBTBUser.Viewer", NoTls).await?;

    // 处理连接
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("connection error: {}", e);
        }
    });

    // 执行查询
    let rows = client.query("select entity_id::varchar, key, ts,
               coalesce(dbl_v, cast(str_v as double precision)) as dbl_v
            from ts_kv_latest
            where entity_id in (select id from device where name = $1) and key in (180,181)",
                            &[&target_name]).await?;

    // 将查询结果转换为 JSON
    let mut results = Vec::new();
    for row in rows {
        let ts_kv = ts_kv::TsKV {
            entity_id: row.get::<_, String>(0),
            key: row.get::<_, i32>(1),
            ts: row.get::<_, i64>(2),
            dbl_v: row.get::<_, f64>(3),
        };
        results.push(ts_kv);
    }
    Ok(results)
}

pub(crate) async fn query_target_id(target_name: &str, db_config: &DatabaseConfig) -> Result<String, Error> {
    // 连接到 PostgreSQL
    let connection_str = format!(
        "host={} user={} dbname={} password={}",
        db_config.host, db_config.user, db_config.dbname, db_config.password
    );
    let (client, connection) = tokio_postgres::connect(&connection_str, NoTls).await?;
    // 处理连接
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("connection error: {}", e);
        }
    });

    // 执行查询
    let rows = client.query("select id::varchar from device where name = $1",
                            &[&target_name]).await?;
    Ok(rows[0].get::<_, String>(0))

}
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
mod data_fetch;
mod data_post;
mod ts_kv;
mod rand_dis;

use std::time::Duration;
use tokio::time;
use data_fetch::query_data;
use data_post::post_data;
use serde::Deserialize;
use config::{Config, File};


#[derive(Deserialize)]
struct SyncConfig {
    source_target: String,
    dest_target: String,
    interval_secs: u64,
}

#[derive(Deserialize)]
struct AppConfig {
    jwt_token: String,
    http_server: String,
    sync_tasks: Vec<SyncConfig>,
    database: DatabaseConfig,

}
#[derive(Deserialize,Clone)]
struct DatabaseConfig {
    host: String,
    user: String,
    dbname: String,
    password: String,
}
impl AppConfig {
    fn load() -> Result<Self, config::ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config/config"))
            .build()?;

        config.try_deserialize()
    }
}


async fn sync_data(config: &SyncConfig, db_config: &DatabaseConfig) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Syncing data: {} -> {}", config.source_target, config.dest_target);
    match query_data(&config.source_target).await {
        Ok(mut data) => {

            static PREV_VALUES: Lazy<Mutex<std::collections::HashMap<String, f64>>> =
                Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

            let mut processed_data = Vec::new();
            {
                for ts_kv in data {
                    let current_value = ts_kv.dbl_v;

                    processed_data.push(ts_kv::TsKV::new(
                        ts_kv.entity_id,
                        ts_kv.key,
                        ts_kv.ts,
                        current_value
                    ));
                }
            }

            if !processed_data.is_empty() {
                if let Err(e) = post_data(processed_data, &config.dest_target, &db_config).await {
                    log::error!("Error posting data: {}", e);
                }
            }
        }
        Err(e) => log::error!("Error querying data: {}", e),
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default())?;
    log::info!("Starting data forward service...");
    let app_config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load configuration: {}", e);
            return Ok(());
        }
    };

    // Set the JWT token for data_post module
    data_post::set_jwt_token(&app_config.jwt_token);
    data_post::set_base_http_api(&app_config.http_server);

    let mut handles = vec![];

    for sync_config in app_config.sync_tasks {
        log::info!("Starting sync task: {} -> {} every {} seconds",
               sync_config.source_target, sync_config.dest_target, sync_config.interval_secs);
        let db_config = app_config.database.clone();
        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(sync_config.interval_secs));
            loop {
                interval.tick().await;
                if let Err(e) = sync_data(&sync_config, &db_config).await {
                    log::error!("Sync error: {}", e);
                }
                log::debug!("Sync completed for {} -> {} at: {:?}",
                     sync_config.source_target, sync_config.dest_target, time::Instant::now());
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {

        if let Err(e) = handle.await {
            log::error!("Task error: {}", e);
        }
    }
    
    Ok(())
}
use std::collections::HashMap;
use std::sync::OnceLock;
use phf::phf_map;
use crate::data_fetch::query_target_id;
use crate::DatabaseConfig;
use crate::ts_kv::TsKV;
static JWT_TOKEN: OnceLock<String> = OnceLock::new();
pub fn set_jwt_token(token: &str) {
    JWT_TOKEN.get_or_init(|| token.to_string());
}

static BASE_HTTP_API: OnceLock<String> = OnceLock::new();
pub fn set_base_http_api(http_api: &str) {
    crate::data_post::BASE_HTTP_API.get_or_init(|| http_api.to_string());
}


// const REFERENCE_DICT: HashMap<(String, i32), f64> = HashMap::from([
//     (("T01".to_string(), 180), -1.73),
//     (("T01".to_string(), 181), -0.93),
//     (("T02".to_string(), 180), -1.93),
//     (("T02".to_string(), 181), -0.73),
//     (("T03".to_string(), 180), -2.87),
//     (("T03".to_string(), 181), -3.4),
//     (("T04".to_string(), 180), 0.68),
//     (("T04".to_string(), 181), -0.23),
//     (("T05".to_string(), 180), 3.59),
//     (("T05".to_string(), 181), 2.54),
//     (("T06".to_string(), 180), 1.88),
//     (("T06".to_string(), 181), 0.5664),
// ]);

static REFERENCE_DICT: phf::Map<&'static str, phf::Map<i32, f64>> = phf_map! {
    "T01" => phf_map! {
        180_i32 => -0.73,
        181_i32 => -0.89,
    },
    "T02" => phf_map! {
        180_i32 => -0.33,
        181_i32 => -0.73,
    },
    "T03" => phf_map! {
        180_i32 => -0.67,
        181_i32 => -0.42,
    },
    "T04" => phf_map! {
        180_i32 => 0.68,
        181_i32 => -0.23,
    },
    "T05" => phf_map! {
        180_i32 => 0.67,
        181_i32 => 0.41,
    },
    "T06" => phf_map! {
        180_i32 => 0.78,
        181_i32 => 0.5664,
    },
};

pub(crate) async fn post_data(data: Vec<TsKV>, dest_target_name: &str, db_config: &DatabaseConfig) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let id = query_target_id(dest_target_name, db_config).await;

    let id = match id {
        Ok(id) => id,
        Err(e) => {
            log::error!("Error querying target id: {}", e);
            return Err(Box::new(e));
        }
    };

    log::info!("dest Target ID: {}", id);
    log::info!("Data fetched: {:?}", &data);
    for mut ts_kv in data {
        // 根据dest_target_name 和 key 获取参考值
        if let Some(reference_map) = REFERENCE_DICT.get(dest_target_name) {
            if let Some(&reference_value) = reference_map.get(&ts_kv.key) {
                let current_value = ts_kv.dbl_v;
                let diff = (current_value - reference_value).abs();

                // 检查值是否在参考字典值±0.3范围内
                if diff > 0.3 {
                    let scale_factor = (0.3 / diff).min(1.0);
                    ts_kv.dbl_v = reference_value + (current_value - reference_value) * scale_factor;
                    log::info!("Value {} for key {} is outside acceptable range (±0.3) of reference value {}. Scaled to {}",
              current_value, ts_kv.key, reference_value, ts_kv.dbl_v);
                } else {
                    log::debug!("Value {} for key {} is within acceptable range of reference value {}",
               current_value, ts_kv.key, reference_value);
                }
            }
        }

        let val = ts_kv.to_post_data();

        log::debug!("Posting data: {:?}", val);

        let url = format!("{}/api/plugins/telemetry/DEVICE/{}/timeseries/ANY?scope=ANY",
                          BASE_HTTP_API.get().expect("Base HTTP API not set"), id);
        let res = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Authorization", format!("Bearer {}", JWT_TOKEN.get().expect("JWT token not set")))
            .body(val.to_string())
            .send()
            .await?;

        // 检查响应状态
        if res.status().is_success() {
            log::debug!("Data submitted successfully!");
        } else {
            log::warn!("Failed to submit data: {}", res.status());
        }
    }

    Ok(())
}
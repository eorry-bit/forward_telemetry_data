use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsKV {
    pub entity_id: String,
    pub key: i32,
    pub ts: i64,
    pub dbl_v: f64,
}

impl TsKV {


    pub fn new(entity_id: String, key: i32, ts: i64, dbl_v: f64) -> Self {
        TsKV {
            entity_id,
            key,
            ts,
            dbl_v,
        }
    }

    pub fn to_post_data(&self) -> serde_json::Value {
        let mut values = json!({});
        let values_obj = values.as_object_mut().unwrap();

        match self.key {
            180 => {
                values_obj.insert("fAxisX".to_string(), json!(self.dbl_v));
                values_obj.insert("sigmaX3D".to_string(), json!(self.dbl_v));
            },
            181 => {
                values_obj.insert("fAxisY".to_string(), json!(self.dbl_v));
                values_obj.insert("sigmaY3D".to_string(), json!(self.dbl_v - (rand::random::<f64>() - 0.5) * 0.4));
                values_obj.insert("sigmaZ3D".to_string(), json!(self.dbl_v));
            },
            _ => panic!("Unexpected key value: {}", self.key)
        }

        json!({
        "ts": self.ts,
        "values": values
    })
    }



}
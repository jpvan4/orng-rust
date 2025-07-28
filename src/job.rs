use crate::share::Share;
use randomx_rs::RandomXVM;
use serde::{Deserialize, Deserializer};

// pub const THREAD_NONCE_START: u32 = 0;


fn target_from_hex<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let hex: String = Deserialize::deserialize(deserializer)?;
    let bytes = hex::decode(hex).map_err(serde::de::Error::custom)?;
    if bytes.len() != 8 {
        return Err(serde::de::Error::custom("Target must be 8 bytes"));
    }
    let arr: [u8; 8] = bytes
        .try_into()
        .map_err(|_| serde::de::Error::custom("Invalid target length"))?;
    Ok(u64::from_le_bytes(arr))
}

// fn target_from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>

// where

//     D: Deserializer<'de>,

// {

//     let hex: String = Deserialize::deserialize(deserializer)?;

//     let bytes = hex::decode(&hex).map_err(serde::de::Error::custom)?;

//     if bytes.len() != 4 {

//         return Err(serde::de::Error::custom("Target must be 4 bytes"));

//     }

//     Ok(u32::from_be_bytes(bytes.try_into().unwrap()))

// }

//
#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    #[serde(rename = "job_id")]
    pub id: String,
    #[serde(with = "hex")]
    pub blob: Vec<u8>,
    #[serde(rename = "seed_hash", with = "hex")]
    pub seed: Vec<u8>,
    #[serde(deserialize_with = "target_from_hex_vec")]
    pub target: Vec<u8>,
}

impl Job {
    pub fn difficulty(&self) -> u64 {
        match self.target.len() {
            4 => {
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&self.target[..4]);
                u32::from_le_bytes(arr) as u64
            }
            8 => {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&self.target[..8]);
                u64::from_le_bytes(arr)
            }
            _ => u64::MAX,
        }
    }

    pub fn next_share(&self, vm: &RandomXVM, nonce: u32, target: u64) -> Option<Share> {
        if self.blob.len() < 43 {
            tracing::warn!("Invalid blob length: {}", self.blob.len());
            return None;
        }

        // Insert nonce into the blob
        let mut blob_copy = self.blob.clone();
        let nonce_bytes = nonce.to_le_bytes();
        blob_copy[39..43].copy_from_slice(&nonce_bytes);

        // Calculate hash
        let hash = vm.calculate_hash(&blob_copy).ok()?;
        let hash_bytes = hash.as_slice();

        // Compare hash against target (lower values are better)
        let hash_val = u64::from_le_bytes(hash_bytes[24..32].try_into().unwrap());

        if nonce % 100_000 == 0 {
            tracing::debug!(
                "Nonce: {}, Hash val: {}, Target: {}",
                nonce,
                hash_val,
                hex::encode(&self.target)
            );
        }

        if hash_val <= target {
            Some(Share::new(self.id.clone(), nonce, hash))
        } else {
            None
        }
    }
}

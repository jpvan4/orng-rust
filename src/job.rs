use serde::{Deserialize, Deserializer};
use crate::share::Share;




// pub const THREAD_NONCE_START: u32 = 0;

fn target_from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let hex: String = Deserialize::deserialize(deserializer)?;
    hex::decode(hex)
        .map_err(serde::de::Error::custom)?
        .try_into()
        .map(u32::from_le_bytes)
        .map_err(|_| serde::de::Error::custom("Invalid target length"))
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
    #[serde(deserialize_with = "target_from_hex")]
    pub target: u32,
}

impl Job {
    pub fn difficulty(&self) -> u64 {
        if self.target == 0 {
            return u64::MAX; // Handle zero target case
        }
        u64::MAX / (u32::MAX as u64 / self.target as u64)
    }

    pub fn next_share(
        &self,
        hasher: &mut rust_randomx::Hasher,
        nonce: u32,
        difficulty: u64,
    ) -> Option<Share> {
        if self.blob.len() < 43 {
            tracing::warn!("Invalid blob length: {}", self.blob.len());
            return None;
        }

        // Insert nonce into the blob
        let mut blob_copy = self.blob.clone();
        let nonce_bytes = nonce.to_le_bytes();
        blob_copy[39..43].copy_from_slice(&nonce_bytes);

        // Calculate hash
        let hash = hasher.hash(&blob_copy);
        let hash_bytes = hash.as_ref();
        
        // Compare hash against target (lower values are better)
        let hash_val = u64::from_le_bytes(hash_bytes[24..32].try_into().unwrap());

        if nonce % 100_000 == 0 {
            tracing::debug!("Nonce: {}, Hash val: {}, Target: {}", nonce, hash_val, self.target);
        }

        if hash_val < difficulty {
            Some(Share::new(self.id.clone(), nonce, hash_bytes.to_vec()))
        } else {
            None
        }
    }
}

// 
// fn target_from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let hex: String = Deserialize::deserialize(deserializer)?;
//     // Parse as big-endian since targets are usually represented this way
//     let bytes = hex::decode(&hex).map_err(serde::de::Error::custom)?;
//     if bytes.len() != 4 {
//         return Err(serde::de::Error::custom(format!("Expected 4 bytes, got {}", bytes.len())));
//     }
//     Ok(u32::from_be_bytes(bytes.try_into().unwrap()))
// }
// 
// #[derive(Debug, Clone, Deserialize)]
// pub struct Job {
//     #[serde(rename = "job_id")]
//     pub id: String,
//     #[serde(with = "hex")]
//     pub blob: Vec<u8>,
//     #[serde(rename = "seed_hash", with = "hex")]
//     pub seed: Vec<u8>,
//     #[serde(deserialize_with = "target_from_hex")]
//     pub target: u32,
// }
// 
// impl Job {
//     pub fn difficulty(&self) -> u64 {
//         // Convert target to difficulty
//         // In cryptocurrency mining, lower target means higher difficulty
//         // The target represents the maximum acceptable hash value
//         // So we use it directly as the threshold for comparison
//         let difficulty = self.target as u64;
//         tracing::debug!("Job target: {}, difficulty (threshold): {}", self.target, difficulty);
//         difficulty
//     }
// 
//     pub fn next_share(
//         &self,
//         hasher: &mut rust_randomx::Hasher,
//         nonce: u32,
//         difficulty: u64,
//     ) -> Option<Share> {
//         if self.blob.len() < 43 {
//             return None;
//         }
// 
//         let mut blob_copy = self.blob.clone();
//         let nonce_bytes = nonce.to_le_bytes();
//         blob_copy[39..43].copy_from_slice(&nonce_bytes);
// 
//         let hash = hasher.hash(&blob_copy);
//         let hash_bytes = hash.as_ref();
//         let hash_val = u64::from_le_bytes(hash_bytes[24..32].try_into().unwrap());
// 
//         // Add some debug logging for the first few attempts
//         if nonce % 100000 == 0 {
//             tracing::debug!("Nonce: {}, Hash val: {}, Difficulty: {}, Target: {}", nonce, hash_val, difficulty, self.target);
//         }
// 
//         if hash_val < difficulty {
//             Some(Share::new(self.id.clone(), nonce, hash_bytes.to_vec()))
//         } else {
//             None
//         }
//     }
// }
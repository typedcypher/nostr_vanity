use anyhow::Result;
use bech32::{self, Hrp, Bech32};
use secp256k1::{PublicKey, SecretKey, SECP256K1};
use secp256k1::rand::rng;

#[derive(Debug, Clone)]
pub struct NostrKeyPair {
    pub npub: String,
    pub nsec: String,
    pub hex_pubkey: String,
}

impl NostrKeyPair {
    pub fn generate() -> Result<Self> {
        let mut rng = rng();
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(SECP256K1, &secret_key);
        
        let npub = encode_bech32("npub", &public_key.serialize()[1..])?;
        let nsec = encode_bech32("nsec", &secret_key.secret_bytes())?;
        let hex_pubkey = hex::encode(&public_key.serialize()[1..]);
        
        Ok(NostrKeyPair {
            npub,
            nsec,
            hex_pubkey,
        })
    }
    
}

fn encode_bech32(hrp_str: &str, data: &[u8]) -> Result<String> {
    let hrp = Hrp::parse(hrp_str)?;
    let encoded = bech32::encode::<Bech32>(hrp, data)?;
    Ok(encoded)
}

pub fn validate_bech32_chars(pattern: &str) -> bool {
    const VALID_CHARS: &str = "023456789acdefghjklmnpqrstuvwxyz";
    pattern.chars().all(|c| VALID_CHARS.contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let keypair = NostrKeyPair::generate().unwrap();
        assert!(keypair.npub.starts_with("npub1"));
        assert!(keypair.nsec.starts_with("nsec1"));
        assert_eq!(keypair.hex_pubkey.len(), 64);
    }
    
    #[test]
    fn test_validate_bech32_chars() {
        assert!(validate_bech32_chars("test"));
        assert!(validate_bech32_chars("023"));
        assert!(!validate_bech32_chars("test1")); 
        assert!(!validate_bech32_chars("TEST"));
        assert!(!validate_bech32_chars("bio"));
    }
}
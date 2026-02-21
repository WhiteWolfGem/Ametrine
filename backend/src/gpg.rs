use anyhow::{anyhow, Result};
use hickory_resolver::config::*;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::proto::rr::{RData, RecordType};
use hickory_resolver::Resolver;
use sha2::{Digest, Sha256};
use std::io::BufReader;

use pgp::composed::{Deserializable, SignedPublicKey, DetachedSignature};

pub struct GpgVerifier {
    email: String,
}

impl GpgVerifier {
    pub fn new(email: String) -> Self {
        Self { email }
    }

    fn get_dns_path(&self) -> Result<String> {
        let parts: Vec<&str> = self.email.split('@').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid email format"));
        }
        let (local, domain) = (parts[0], parts[1]);
        let mut hasher = Sha256::new();
        hasher.update(local.as_bytes());
        let hash = hex::encode(&hasher.finalize()[..28]);
        Ok(format!("{}._openpgpkey.{}", hash, domain))
    }

    pub async fn fetch_public_key(&self) -> Result<Vec<u8>> {
        let resolver = Resolver::builder_with_config(
            ResolverConfig::default(),
            TokioConnectionProvider::default(),
        )
        .build();
        let dns_path = self.get_dns_path()?;
        let response = resolver.lookup(dns_path, RecordType::from(61)).await?;
        let rdata = response
            .iter()
            .next()
            .ok_or_else(|| anyhow!("No OPENPGPKEY record found"))?;
        match rdata {
            RData::OPENPGPKEY(key_record) => Ok(key_record.public_key().to_vec()),
            RData::Unknown { rdata, .. } => Ok(rdata.anything().to_vec()),
            _ => Err(anyhow!("Record found but not in a recognized format")),
        }
    }

    /// Verify content against a detached signature
    pub async fn verify(&self, content: &str, signature_armor: &str) -> Result<()> {
        let key_data = self.fetch_public_key().await?;
        
        // OPENPGPKEY records are raw binary bytes. 
        // We use from_bytes which expects a BufRead.
        let pubkey = SignedPublicKey::from_bytes(BufReader::new(&key_data[..]))
            .map_err(|e| anyhow!("Failed to parse public key: {}", e))?;

        // Parse the signature. It should be armored in the config/post.
        let (sig, _) = DetachedSignature::from_string(signature_armor)
            .map_err(|e| anyhow!("Failed to parse armored signature: {}", e))?;

        // Verify the signature against the content
        sig.verify(&pubkey, content.as_bytes())
            .map_err(|e| anyhow!("GPG Verification failed: {}", e))?;

        Ok(())
    }
}

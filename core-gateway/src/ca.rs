use anyhow::*;
use directories::ProjectDirs;
use rcgen::{Certificate, CertificateParams, BasicConstraints, IsCa, DnType};
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug)]
pub struct CaStore {
    dir: PathBuf,
    pub root_cert_pem: Vec<u8>,
    pub root_key_pem: Vec<u8>,
}

impl CaStore {
    pub fn ensure() -> Result<Self> {
        let dir = if let std::result::Result::Ok(d) = std::env::var("AIPROXY_DATA_DIR") {
            PathBuf::from(d)
        } else {
            let proj = ProjectDirs::from("local", "ai-proxy", "gateway").ok_or_else(|| anyhow!("no project dirs"))?;
            proj.data_dir().to_path_buf()
        };
        fs::create_dir_all(&dir)?;
        let cert_path = dir.join("root_ca.pem");
        let key_path = dir.join("root_ca.key");
        if cert_path.exists() && key_path.exists() {
            return Ok(Self { dir, root_cert_pem: fs::read(cert_path)?, root_key_pem: fs::read(key_path)? });
        }
        let mut params = CertificateParams::new(vec![]);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.distinguished_name.push(DnType::CommonName, "AI Proxy Root CA");
        params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
        let ca = Certificate::from_params(params)?;
        fs::write(&cert_path, ca.serialize_pem()?)?;
        fs::write(&key_path, ca.serialize_private_key_pem())?;
        Ok(Self { dir, root_cert_pem: fs::read(cert_path)?, root_key_pem: fs::read(key_path)? })
    }

    pub fn dir(&self) -> &PathBuf { &self.dir }

    pub fn export_paths(&self) -> (PathBuf, PathBuf) {
        (self.dir.join("root_ca.pem"), self.dir.join("root_ca.key"))
    }

    // 预留：后续 MITM 接入时如需以 rustls-pki-types 返回，再补充实现
}

use std::sync::Arc;
use anyhow::*;
use tokio_rustls::rustls::{self, ServerConfig};
use tokio_rustls::rustls::server::{ClientHello, ResolvesServerCert};
use tokio_rustls::rustls::sign;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::crypto::aws_lc_rs::sign::any_supported_type;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use crate::ca::CaStore;

#[derive(Clone, Debug)]
pub struct DynResolver {
    ca: CaStore,
}

impl DynResolver {
    pub fn new(ca: CaStore) -> Self { Self { ca } }

    fn cert_for_sni(&self, sni: &str) -> Result<sign::CertifiedKey> {
        let ca_pem = std::fs::read(self.ca.dir().join("root_ca.pem"))?;
        let ca_key_pem = std::fs::read(self.ca.dir().join("root_ca.key"))?;
        let ca_key = rcgen::KeyPair::from_pem(std::str::from_utf8(&ca_key_pem)?)?;
        let mut ca_params = rcgen::CertificateParams::new(vec![]);
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
        ca_params.key_pair = Some(ca_key);
        ca_params.distinguished_name.push(rcgen::DnType::CommonName, "AI Proxy Root CA");
        let ca_cert = rcgen::Certificate::from_params(ca_params)?;

        let mut leaf_params = rcgen::CertificateParams::new(vec![sni.to_string()]);
        leaf_params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
        leaf_params.distinguished_name.push(rcgen::DnType::CommonName, sni);
        let leaf = rcgen::Certificate::from_params(leaf_params)?;
        let leaf_der = CertificateDer::from(leaf.serialize_der_with_signer(&ca_cert)?);
        let leaf_key_pkcs8 = PrivatePkcs8KeyDer::from(leaf.get_key_pair().serialize_der());
        let leaf_key: PrivateKeyDer<'_> = PrivateKeyDer::from(leaf_key_pkcs8);
        let signer = any_supported_type(&leaf_key)
            .map_err(|_| anyhow!("unsupported key"))?;
        let ca_parsed = pem::parse(ca_pem)?;
        let chain: Vec<CertificateDer<'static>> = vec![leaf_der, CertificateDer::from(ca_parsed.into_contents())];
        Ok(sign::CertifiedKey::new(chain, signer))
    }
}

impl ResolvesServerCert for DynResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<sign::CertifiedKey>> {
        let dns: &str = client_hello.server_name()?;
        match self.cert_for_sni(dns) {
            std::result::Result::Ok(ck) => Some(Arc::new(ck)),
            std::result::Result::Err(_) => None,
        }
    }
}

pub fn build_acceptor(ca: &CaStore) -> Result<TlsAcceptor> {
    // tokio-rustls 0.26 随 ring/awslc 自动选择，无需手动 install_default
    let resolver = DynResolver::new(ca.clone());
    let cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(resolver));
    Ok(TlsAcceptor::from(Arc::new(cfg)))
}

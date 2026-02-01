use crate::models::{FileInfo, InputData};
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::net::IpAddr;
use std::path::PathBuf;
use uuid::Uuid;

pub fn collect_input() -> Result<InputData, Box<dyn Error>> {
    let mut password = String::new();
    let mut file_count = String::new();

    print!("Enter password: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut password)?;

    print!("Enter the number of files to be shared: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut file_count)?;
    let count: usize = file_count
        .trim()
        .parse()
        .expect("Please enter a valid number");

    let mut paths = Vec::new();
    for i in 0..count {
        let mut path_input = String::new();
        print!("Enter file path {}/{}: ", i + 1, count);
        io::stdout().flush()?;
        io::stdin().read_line(&mut path_input)?;
        paths.push(PathBuf::from(path_input.trim()));
    }

    Ok(InputData {
        password: password.trim().to_string(),
        paths,
    })
}

pub fn get_canonical_paths(paths: Vec<PathBuf>) -> io::Result<Vec<PathBuf>> {
    paths.into_iter().map(|p| fs::canonicalize(p)).collect()
}

pub fn generate_metadata(paths: &[PathBuf]) -> Result<Vec<FileInfo>, Box<dyn Error>> {
    let namespace = Uuid::from_bytes([
        0x2d, 0x8a, 0xef, 0x7b, 0x51, 0x3a, 0x4b, 0x92, 0x9c, 0x6e, 0x1f, 0x44, 0x8d, 0x22, 0x71,
        0x05,
    ]);

    let mut metadata = Vec::new();

    for path in paths {
        let path_bytes = path.as_os_str().as_encoded_bytes();
        let uuid = Uuid::new_v5(&namespace, path_bytes);

        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("could not convert file name to a string")?
            .to_string();

        let file_info = fs::metadata(path)?;
        let file_size = file_info.len();

        metadata.push(FileInfo {
            uuid,
            file_name,
            file_size,
            file_path: path.clone(),
        });
    }

    Ok(metadata)
}

pub fn generate_cert(local_ip: IpAddr) -> Result<(String, String), Box<dyn Error>> {
    let key_pair = KeyPair::generate()?;
    let mut params = CertificateParams::default();

    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, local_ip.to_string());
    params.subject_alt_names = vec![rcgen::SanType::IpAddress(local_ip)];

    let cert = params.self_signed(&key_pair)?;
    Ok((cert.pem(), key_pair.serialize_pem()))
}

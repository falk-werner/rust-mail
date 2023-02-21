extern crate dirs;

use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Certificate {
    pub name: String,
    pub cert: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub accounts: Vec<Account>,
    pub certificates: Vec<Certificate>,
    pub mail_dir: String,
}

fn get_config_file() -> Option<PathBuf> {
    let mut path = dirs::home_dir()?;
    path.push(".mail.json");
    Some(path)
}

fn get_config() -> Option<Config> {
    let path = get_config_file()?;
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader).ok()?;
    Some(config)
}

impl Config {
    pub fn load() -> Config {
        match get_config() {
            Some(config) => config,
            None => Config {
                accounts: vec!(),
                certificates: vec!(),
                mail_dir: String::from("${HOME}/mail"),
            }
        }    
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let path = get_config_file().ok_or("failed to create file")?;
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn add_account(&mut self, account: Account) {
        if ! self.accounts.iter().any(|item| item.username == account.username) {
            self.accounts.push(account)
        }
    }

    pub fn remove_account(&mut self, email: String) {
        self.accounts.retain(|item| item.username != email);
    }

    pub fn add_certificate(&mut self, name: String, cert: String) {
        if ! self.certificates.iter().any(|item| item.name == name) {
            self.certificates.push(Certificate { name: name, cert });
        }
    }

    pub fn remove_certificate(&mut self, name: String) {
        self.certificates.retain(|item| item.name != name);
    }

    pub fn get_mail_dir(&self) -> PathBuf {
        let home_dir = dirs::home_dir().unwrap_or(PathBuf::from("."));
        let dir = self.mail_dir.replace("${HOME}", &home_dir.to_str().unwrap_or("."));
        dir.into()
    }
}


use std::error::Error;
use std::path::PathBuf;
use crate::config::Account;
use rust_pop3_client::Pop3Connection;
use std::fs::{File, create_dir_all};
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use regex::Regex;

pub struct MailBox {
    path: PathBuf,
    known_ids: HashMap::<String, String>,
}

fn get_subject(header: &str) -> Option<String> {
    let re = Regex::new(r"[a-zA-Z0-9_\- ]").unwrap();

    for line in header.split('\n') {
        let line = line.trim();
        if line.starts_with("Subject:") {
            let mut line = line.trim()[8..].to_string();
            line.retain(|c| re.is_match(&c.to_string()));
            return Some(line);
        }
    }

    None
}

impl MailBox {

    pub fn new(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        create_dir_all(&path)?;

        let mut me = MailBox { 
            path: path,
            known_ids: HashMap::<String,String>::new()
        };

        me.load();
        Ok(me)
    }

    fn load(&mut self)-> HashMap<String, String> {
        let mut path = PathBuf::from(&self.path);
        path.push("uid.json");
    
        let file = File::open(path);
        if let Ok(file) = file {
            let reader = BufReader::new(file);
            let ids: Result<HashMap::<String,String>, serde_json::Error> = serde_json::from_reader(reader);
            if let Ok(ids) = ids {
                self.known_ids.extend(ids);
            }
        }
    
        HashMap::<String,String>::new()
    }
    

    fn save(&self) {
        let mut path = PathBuf::from(&self.path);
        path.push("uid.json");
    
        let file = File::create(path);
        if let Ok(file) = file {
            let writer = BufWriter::new(file);
            let _ = serde_json::to_writer_pretty(writer, &self.known_ids);
        }
    
    }
    

    pub fn fetch(&mut self, account: &Account) -> Result<(), Box<dyn Error>> {
        let mut connection = Pop3Connection::new(&account.host, account.port)?;
        connection.login(&account.username, &account.password)?;
    
        let infos = connection.list_unique_ids()?;
        for info in infos {
            if ! self.known_ids.contains_key(&info.unique_id) {
                let header = connection.top(info.message_id, 0)?;
                let subject = get_subject(&header).unwrap_or(String::from(&info.unique_id));

                println!("download {}...", subject);
                let mut filename = PathBuf::from(&self.path);
                filename.push(&format!("{}.msg", &subject));
                let file = File::create(filename)?;
                let mut writer = BufWriter::new(file);
                connection.retrieve(info.message_id, &mut writer)?;
                self.known_ids.entry(info.unique_id).or_insert(subject);

            } else {
                println!("skip {}: already known", info.unique_id);
            }
        }

        self.save();
        Ok(())
    }
    

}
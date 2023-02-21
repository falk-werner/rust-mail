mod config;
mod mailbox;

use mailbox::MailBox;

use std::error::Error;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::fs;
use rpassword;



#[derive(Debug, Parser)]
#[clap(name = "mail-client", version)]
struct App {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Account {
        #[clap(subcommand)]
        command: AccountCommand
    },
    Cert {
        #[clap(subcommand)]
        command: CertCommand
    },
    Fetch,
}

#[derive(Debug, Subcommand)]
enum AccountCommand {
    Add,
    Remove { email: String },
    List
}

#[derive(Debug, Subcommand)]
enum CertCommand {
    Add { name: String, filename: String },
    Remove { name: String },
    List
}

fn list_accounts() -> Result<(), Box<dyn Error>> {
    let config = config::Config::load();
    for account in config.accounts {
        println!("{}: {}:{}", account.username, account.host, account.port);
    }

    Ok(())
}

fn read_value(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(String::from(value.trim()))
}

fn read_password(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    Ok(rpassword::read_password()?)
}

fn add_account() -> Result<(), Box<dyn Error>> {
    let name = read_value(&"Mail Address")?;
    let host = read_value(&"Host")?;
    let port: u16 = read_value(&"Port")?.parse()?;
    let password = read_password(&"Password")?;

    let account = config::Account { host: host, port: port, username: name, password: password };

    let mut config = config::Config::load();
    config.add_account(account);
    config.save()?;

    Ok(())
}

fn remove_account(email: String) -> Result<(), Box<dyn Error>> {
    let mut config = config::Config::load();
    config.remove_account(email);
    config.save()?;

    Ok(())
}

fn add_cert(name: String, filename: String) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;

    let mut config = config::Config::load();
    config.add_certificate(name, contents);
    config.save()?;

    Ok(())
}

fn remove_cert(name: String) -> Result<(), Box<dyn Error>> {
    let mut config = config::Config::load();
    config.remove_certificate(name);
    config.save()?;

    Ok(())
}

fn list_certs() -> Result<(), Box<dyn Error>> {
    let config = config::Config::load();
    for cert in config.certificates {
        println!("{}", cert.name);
    }

    Ok(())
}


fn fetch() -> Result<(), Box<dyn Error>> {
    let config = config::Config::load();
    for account in &config.accounts {
        let mut path = config.get_mail_dir();
        path.push(account.username.clone());

        let mut mailbox = MailBox::new(path)?;
        mailbox.fetch(&account)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::parse();
    match app.command {
        Command::Account { command } => {
            match command {
                AccountCommand::List => Ok(list_accounts()?),
                AccountCommand::Add => Ok(add_account()?),
                AccountCommand::Remove { email } => Ok(remove_account(email)?),
            }
        },
        Command::Cert { command } => {
            match command {
                CertCommand::List => Ok(list_certs()?),
                CertCommand::Add { name, filename }=> Ok(add_cert(name, filename)?),
                CertCommand::Remove { name } => Ok(remove_cert(name)?),
            }
        },
        Command::Fetch => Ok(fetch()?),
    }
}

mod cli;
mod crypto;
mod db;
mod error;
mod steg;

use {
    anyhow::Result,
    clap::{Parser, Subcommand},
    cli::{
        get_password_from_prompt,
        init_logging,
    },
//    crypto::{decrypt_password, encrypt_password, EncryptedData},
    db::Database,
    std::path::PathBuf,
//    steg::hide_data,
    zeroize::Zeroizing
};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Put {
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        value: Option<String>,
        #[arg(short, long)]
        image: PathBuf,
    },
    Get {
        #[arg(short, long)]
        key: String,
    },
    List,
    Rm {
        #[arg(short, long)]
        key: String,
    },
}


fn main() -> Result<()> {
    init_logging();
    let cli = Cli::parse();
    let db = Database::new()?;

    match cli.cmd {
        Commands::Put { key, value, image } => {
            // Convert the optional String to Zeroizing<String>
            let password: Zeroizing<String> = match value {
                Some(v) => Zeroizing::new(v),
                None => get_password_from_prompt("Enter password: ")?,
            };

            let master_key = get_password_from_prompt("Enter master key: ")?;
            let encrypted = crypto::encrypt_password(&password, master_key.as_bytes())?;
            let data = serde_json::to_vec(&encrypted)?;

            steg::hide_data(&image, &data)?;
            db.put(&key, &data)?;

            cli::print_success("Password stored successfully");
        }

        Commands::Get { key } => {
            let data = db.get(&key)?;
            let encrypted: crypto::EncryptedData = serde_json::from_slice(&data)?;

            let master_key = get_password_from_prompt("Enter master key: ")?;
            let password = crypto::decrypt_password(&encrypted, master_key.as_bytes())?;

            println!("Password: {}", password);
        }

        Commands::List => {
            let keys = db.list_keys()?;
            println!("Stored passwords:");
            for key in keys {
                println!("- {}", key);
            }
        }

        Commands::Rm { key } => {
            db.delete(&key)?;
            cli::print_success("Password removed successfully");
        }
    }

    Ok(())
}



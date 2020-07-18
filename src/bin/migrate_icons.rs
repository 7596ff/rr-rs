use std::env;
use std::fs::{self, DirEntry};
use std::path::Path;

use postgres::{Client, NoTls};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let mut client = Client::configure()
        .user(&dotenv::var("POSTGRES_USER")?)
        .dbname(&dotenv::var("POSTGRES_DBNAME")?)
        .host(&dotenv::var("POSTGRES_HOST")?)
        .connect(NoTls)?;

    let mut args = env::args().skip(1);
    let path = args.next().expect("no path specified. USAGE: migrate_icons <path>");

    let path = Path::new(&path);
    let dirs = fs::read_dir(&path)?;

    for dir in dirs {
        let dir = dir?;
        let dir_path_buf = dir.path();

        let name = dir_path_buf.file_stem().unwrap();
        print!("reading directory {}...", name.to_string_lossy());

        let files: Vec<DirEntry> = fs::read_dir(&dir_path_buf)?.map(|f| f.unwrap()).collect();
        println!(" found {} files.", files.len());

        for (i, entry) in files.iter().enumerate() {
            print!("\rinserting file {} of {}...", i + 1, files.len());
            client.execute(
                "INSERT INTO images (guild_id, message_id, image, filetype)
                VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING;",
                &[
                    &name.to_str().unwrap(),
                    &entry.path().file_stem().unwrap().to_str().unwrap(),
                    &fs::read(entry.path())?,
                    &entry.path().extension().unwrap().to_str().unwrap(),
                ],
            )?;
        }

        println!();
    }

    Ok(())
}

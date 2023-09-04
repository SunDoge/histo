use std::path::PathBuf;

use clap::Parser;
use rusqlite::{Connection, OptionalExtension};

#[derive(Debug, clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    Start {
        #[arg(long)]
        host: String,
        #[arg(long)]
        pwd: String,

        command: Vec<String>,
    },
    End {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        exit_code: i64,
    },
    Init {},
    List {},
}

fn main() {
    let cli = Cli::parse();
    // dbg!(&cli);

    match cli.command {
        Commands::Init {} => init_db().unwrap(),
        Commands::Start { host, pwd, command } => {
            let cmd = command.join(" ");
            start(&host, &pwd, &cmd).unwrap();
        }
        Commands::End { id, exit_code } => {
            end(id, exit_code).unwrap();
        }
        Commands::List {} => {
            list_all().unwrap();
        }
    }
}

fn init_db() -> rusqlite::Result<()> {
    let path = PathBuf::from("history.db");
    let conn = Connection::open(&path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS commands (
        id INTEGER PRIMARY KEY,
        argv TEXT,
        UNIQUE(argv) ON CONFLICT IGNORE
    )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS places (
        id INTEGER PRIMARY KEY,
        host TEXT,
        dir TEXT,
        UNIQUE(host, dir) ON CONFLICT IGNORE
    )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
        id INTEGER PRIMARY KEY,
        command_id INT,
        place_id INT,
        exit_code INT,
        start_time INT,
        duration INT
    )",
        (),
    )?;

    Ok(())
}

fn start(host: &str, pwd: &str, command: &str) -> rusqlite::Result<()> {
    let conn = Connection::open("history.db")?;

    let mut stmt = conn.prepare("SELECT id FROM commands WHERE argv = ?1")?;
    let command_id = stmt
        .query_row((command,), |row| row.get(0))
        .optional()
        .transpose()
        .unwrap_or_else(|| {
            let mut stmt = conn.prepare("INSERT INTO commands (argv) VALUES (?1)")?;
            stmt.insert((command,))
        })?;

    let mut stmt = conn.prepare("SELECT id FROM places WHERE host = ?1 AND dir = ?2")?;
    let place_id = stmt
        .query_row((host, pwd), |row| row.get(0))
        .optional()
        .transpose()
        .unwrap_or_else(|| {
            let mut stmt = conn.prepare("INSERT INTO places (host, dir) VALUES (?1, ?2)")?;
            stmt.insert((host, pwd))
        })?;

    let mut stmt = conn.prepare(
        "INSERT INTO history (command_id, place_id, start_time) VALUES (?1, ?2, unixepoch())",
    )?;

    let history_id = stmt.insert((command_id, place_id))?;
    println!("{}", history_id);

    Ok(())
}

fn end(id: i64, exit_code: i64) -> anyhow::Result<()> {
    let conn = Connection::open("history.db")?;

    let mut stmt = conn.prepare(
        "UPDATE history SET duration = unixepoch() - start_time, exit_code = ?1 WHERE id = ?2",
    )?;
    stmt.execute((exit_code, id))?;

    Ok(())
}

fn list_all() -> anyhow::Result<()> {
    let conn = Connection::open("history.db")?;
    let mut stmt = conn.prepare(
        "
        SELECT history.id, history.exit_code, places.dir, commands.argv 
        FROM history 
        JOIN commands ON history.command_id = commands.id 
        JOIN places ON history.place_id = places.id
    ",
    )?;

    let mut rows = stmt.query(())?;
    while let Some(row) = rows.next()? {
        let row_id: i64 = row.get(0)?;
        let exit_code: i64 = row.get(1).unwrap_or(0);
        let dir: String = row.get(2)?;
        let argv: String = row.get(3)?;
        println!(
            "id: {}\texit: {}\tdir: {}\targv: {}",
            row_id, exit_code, dir, argv
        );
    }

    Ok(())
}

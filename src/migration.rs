use crate::env;
use chrono::Utc;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use scylla::{statement::query::Query, Session, SessionBuilder};
use std::vec;
use std::{
    fs::{DirEntry, File},
    io::Write,
    path::Path,
};

pub struct Migration {
    pub name: String,
    pub timestamp: String,
    pub cql: Option<String>,
}

impl Migration {
    /// Creates a new migration with the given name.
    pub fn new(name: &str) -> Self {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        Self {
            name: name
                .chars()
                .map(|c| match c {
                    c if c.is_alphanumeric() => c.to_ascii_lowercase(),
                    _ => '-',
                })
                .collect(),
            timestamp: timestamp.to_string(),
            cql: None,
        }
    }

    /// Saves a template migration to disk.
    pub fn save_template_to_disk(&self) {
        let filename = self.filename();

        let path = Path::new("migrations");
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }

        let path = Path::new(&filename);
        let mut file = File::create(path).unwrap();
        file.write_all(format!("-- {}\n\n-- Write your migration here", self.name).as_bytes())
            .unwrap();
    }

    /// Loads the migration from disk.
    pub fn load_from_disk(&mut self) {
        let filename = self.filename();
        let path = Path::new(&filename);
        self.cql = Some(std::fs::read_to_string(path).unwrap());
    }

    /// Returns the filename of the migration.
    pub fn filename(&self) -> String {
        format!("migrations/{}.cql", self)
    }
}

impl std::fmt::Display for Migration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.timestamp, self.name)
    }
}

impl From<&str> for Migration {
    fn from(id: &str) -> Self {
        // eg, 20210901123456-create-users
        Self {
            name: id[15..].to_string(),
            timestamp: id[..14].to_string(),
            cql: None,
        }
    }
}

impl From<DirEntry> for Migration {
    fn from(entry: DirEntry) -> Self {
        let path = entry.path();
        let path = path.to_str().unwrap();
        let path = path.split('/').last().unwrap();
        // remove the .scm extension
        let path = &path[..path.len() - 4];
        Self::from(path)
    }
}

impl Into<Vec<Query>> for &mut Migration {
    fn into(self) -> Vec<Query> {
        let mut queries = Vec::new();
        for query in self.cql.clone().unwrap().split(';') {
            let query = query.trim();
            if query.len() > 0 {
                queries.push(query.into());
            }
        }
        queries
    }
}

/// Create a new, empty migration file with the given name and saves it to disk.
pub fn create(name: String) {
    let migration = Migration::new(&name);
    migration.save_template_to_disk();
    println!("Created migration file {}", migration.filename());
}

/// Apply all migrations or a single migration if the `migration` argument is
/// provided. If the `env` argument is provided, it will use the environment
/// with the given id. Otherwise, it will use the `dev` environment.
pub async fn apply(migration: Option<String>, env: Option<String>) {
    let env_id = match env {
        Some(id) => id,
        None => "dev".to_string(),
    };

    let env = match env::get_environment(env_id.clone()) {
        Some(env) => env,
        None => {
            println!("Environment {} not found", env_id.bold());
            std::process::exit(1);
        }
    };

    println!(
        "Using environment {}",
        format!(" {}@{} ", env_id, env.connection.host)
            .bold()
            .on_green()
    );

    let session: Session = SessionBuilder::new()
        .known_node(env.connection.host)
        .build()
        .await
        .unwrap();

    let mut migrations: Vec<Migration> = if let Some(migration) = migration {
        vec![migration.as_str().into()]
    } else {
        std::fs::read_dir("migrations")
            .unwrap()
            .map(|r| r.unwrap().into())
            .collect()
    };

    migrations.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let bar = ProgressBar::new(migrations.len() as u64).with_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed}] {bar:40.cyan/blue} {human_pos:>4}/{human_len:4} ETA {eta:4}    {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
    );

    let len = migrations.len();

    for (i, migration) in migrations.iter_mut().enumerate() {
        migration.load_from_disk();
        let queries: Vec<Query> = migration.into();
        for q in queries {
            let contents = q.contents.clone();
            if let Err(e) = session.query(q, []).await {
                panic!(
                    "Failed to apply migration {} at {}/{}: {}\nQuery: {}",
                    migration, i, len, e, contents
                );
            }
        }

        bar.inc(1);
        bar.set_message(format!("Applied migration {}", migration));
    }

    bar.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} Done in {elapsed}")
            .unwrap()
            .progress_chars("##-"),
    );
    bar.finish();
}

pub fn list_migrations() {
    let mut migrations: Vec<Migration> = std::fs::read_dir("migrations")
        .unwrap()
        .map(|r| r.unwrap().into())
        .collect();

    migrations.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    for migration in migrations {
        println!("{}", migration);
    }
}

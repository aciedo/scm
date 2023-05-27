use serde::{Deserialize, Serialize};

/// Environments are small TOML files determining what environment should be
/// used to run the migrations on. The default env ID is 'dev'. They're in the
/// form `<env>.scm.toml` and look like this:
/// 
/// Path: dev.scm.toml
/// ```toml
/// [connection]
/// host = "localhost:3333"
/// ```
/// 
/// Path: prod.scm.toml
/// ```toml
/// [connection]
/// host = "10.0.2.1"
/// ```
#[derive(Deserialize, Serialize)]
pub struct EnvironmentFile {
    pub connection: Connection,
}

#[derive(Deserialize, Serialize)]
pub struct Connection {
    pub host: String,
}

// we provide a function to get the active environment
pub fn get_environment(id: String) -> Option<EnvironmentFile> {
    let path = format!("{}.scm.toml", id);

    match std::fs::read_to_string(path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(env) => Some(env),
            Err(err) => {
                println!("Failed to parse environment file: {}", err);
                None
            }
        },
        Err(_) => None,
    }
}

pub fn list() {
    let paths = std::fs::read_dir(".").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let path = path.to_str().unwrap();

        if path.ends_with(".scm.toml") {
            println!("{}", path);
        }
    }
}

pub fn delete(name: String) {
    let path = format!("{}.scm.toml", name);

    match std::fs::remove_file(path) {
        Ok(_) => println!("Deleted environment {}", name),
        Err(err) => println!("Failed to delete environment {}: {}", name, err),
    }
}

pub fn create(name: String, host: String) {
    let path = format!("{}.scm.toml", name);

    let env = EnvironmentFile {
        connection: Connection { host },
    };

    match toml::to_string(&env) {
        Ok(contents) => match std::fs::write(path, contents) {
            Ok(_) => println!("Created environment {}", name),
            Err(err) => println!("Failed to create environment {}: {}", name, err),
        },
        Err(err) => println!("Failed to serialize environment {}: {}", name, err),
    }
}

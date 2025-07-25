use crate::{error, utils::to_human_bytes};
use alpm::{Alpm, AlpmList, Dep, SigLevel, Ver, vercmp};
use chrono::{DateTime, Local, TimeZone};
use pacmanconf::Config;
use std::cmp::Ordering;

#[allow(dead_code)]
pub struct PackageData<'a> {
    pub name: &'a str,
    pub version: &'a Ver,
    pub new_version: Option<&'a Ver>,
    pub description: Option<&'a str>,
    pub architecture: Option<&'a str>,
    pub url: Option<&'a str>,
    pub licenses: Vec<String>,
    pub provides: AlpmList<'a, &'a Dep>,
    pub dependencies: AlpmList<'a, &'a Dep>,
    pub optional_dependencies: AlpmList<'a, &'a Dep>,
    pub conflicts: AlpmList<'a, &'a Dep>,
    pub replaces: AlpmList<'a, &'a Dep>,
    pub size: String,
    pub packager: Option<&'a str>,
    pub install_date: Option<DateTime<Local>>,
}

pub struct Pacman {
    alpm: Alpm,
}

impl Pacman {
    pub fn new() -> error::Result<Self> {
        let pacman_conf = Config::new()?;

        // Initialize alpm
        let mut alpm = Alpm::new(pacman_conf.root_dir, pacman_conf.db_path)?;
        for repos in &pacman_conf.repos {
            alpm.register_syncdb(repos.name.clone(), SigLevel::USE_DEFAULT)?;
        }

        // Add servers
        let sync_dbs = alpm.syncdbs_mut();
        for repo in pacman_conf.repos {
            for db in sync_dbs {
                if db.name() == repo.name {
                    for server in &repo.servers {
                        db.add_server(server.clone())?;
                    }
                }
            }
        }

        // Update packages
        println!("Syncing packages...");
        sync_dbs.update(false)?;

        Ok(Self { alpm })
    }

    pub fn packages(&self) -> impl Iterator<Item = PackageData> {
        self.alpm.localdb().pkgs().iter().map(|pkg| {
            let mut install_date: Option<DateTime<Local>> = None;
            if let Some(install_timestamp) = pkg.install_date() {
                if let Some(install_datetime_utc) = DateTime::from_timestamp(install_timestamp, 0) {
                    install_date = Some(Local.from_utc_datetime(&install_datetime_utc.naive_utc()));
                }
            }

            let new_version = self
                .alpm
                .syncdbs()
                .into_iter()
                .find_map(|db| db.pkg(pkg.name()).ok())
                .and_then(|sync_pkg| {
                    if vercmp(pkg.version().to_string(), sync_pkg.version().to_string())
                        == Ordering::Less
                    {
                        Some(pkg.version())
                    } else {
                        None
                    }
                });

            PackageData {
                name: pkg.name(),
                version: pkg.version(),
                new_version,
                description: pkg.desc(),
                architecture: pkg.arch(),
                url: pkg.url(),
                licenses: pkg.licenses().into_iter().map(String::from).collect(),
                provides: pkg.provides(),
                dependencies: pkg.depends(),
                optional_dependencies: pkg.optdepends(),
                conflicts: pkg.conflicts(),
                replaces: pkg.replaces(),
                size: to_human_bytes(pkg.size() as i32),
                packager: pkg.packager(),
                install_date,
            }
        })
    }
}

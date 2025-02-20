#![allow(dead_code)]

pub mod reloaded3ririext { 
    // use semver::Version;
    use serde::Deserialize;
    use std::{
        error::Error,
        fs,
        path::Path,
    };

    #[derive(Deserialize, Debug)]
    pub enum PackageType {
        Mod,
        Profile,
        Translation,
        Tool
    }

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct Credits {
        Name: String,
        Role: String,
        Url: Option<String>
    }

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct GamebananaUpdateInfo {
        pub ItemType: String,
        pub ItemId: usize
    }
    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct GithubUpdateInfo {
        pub UserName: String,
        pub RepositoryName: String
    }

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct NexusUpdateInfo {
        GameID: u32,
        Id: u32
    }

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct R2Dependency {
        pub Id: String,
        // SourceUrl: Option<String>,
        pub HasInterface: Option<bool>,
        pub UpdateSourceData: Option<UpdateSourceData>
    }

    impl R2Dependency {
        pub fn get_release_metadata_name(&self) -> String {
            if let Some(v) = &self.UpdateSourceData {
                if let Some(u) = &v.ReleaseMetadataName {
                    return u.clone()
                }
            }
            format!("{}.ReleaseMetadata.json", self.Id)
        }
    }

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct UpdateSourceData {
        pub ReleaseMetadataName: Option<String>, // NOT IN SPEC, 
                                                 // default to Id + ReleaseMetadata.json
        pub AssetFileName: Option<String>, // NOT IN SPEC, default to Mod.zip
        pub Gamebanana: Option<GamebananaUpdateInfo>,
        pub Github: Option<GithubUpdateInfo>,
        pub Nexus: Option<NexusUpdateInfo>
    }

    impl UpdateSourceData {
        pub fn get_asset_file_name(&self) -> String {
            match &self.AssetFileName {
                Some(v) => v.clone(),
                None => "Mod.zip".to_owned()
            }
        }
    }
    // Mostly follows the Reloaded3 package specification:
    // https://reloaded-project.github.io/Reloaded-III/Server/Packaging/Package-Metadata.html

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct Package {
        pub Id: String,
        pub Reloaded2Id: Option<String>, // NOT IN SPEC
        pub LoggerPrefix: Option<String>, // NOT IN SPEC
        pub Name: String,
        pub Author: String,
        pub Summary: Option<String>, // is required
        pub PackageType: PackageType,
        pub DocsFile: Option<String>,
        pub Version: Option<String>, // is required
        pub IsDependency: bool,
        pub Tags: Vec<String>,
        pub Credits: Vec<Credits>,
        pub SourceUrl: Option<String>,
        pub ProjectUrl: Option<String>,
        pub ClientSide: bool,
        pub Icon: String,
        pub Reloaded2Icon: Option<String>, // NOT IN SPEC
        pub SupportedGames: Vec<String>,
        pub CargoImport: Option<ExtCargoImport>, // NOT IN SPEC
        pub UpdateSourceData: Option<UpdateSourceData>,
        pub R2Dependencies: Vec<R2Dependency>, // NOT IN SPEC
        pub HookSettings: ExtHookSettings, // NOT IN SPEC
    }

    pub const PACKAGE_FILENAME: &'static str = "package.toml";

    #[derive(Debug)]
    pub struct InvalidPackageToml(String);
    impl Error for InvalidPackageToml { }
    impl std::fmt::Display for InvalidPackageToml {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Failure while parsing {}: {}", PACKAGE_FILENAME, self.0)
        }
    }

    impl Package {
        pub fn get_mod_id(&self) -> &str {
            if let Some(n) = &self.Reloaded2Id { n } 
            else { &self.Id }
        }
        pub fn get_mod_name(&self) -> &str {
            &self.Name
        }
        /*
        pub fn ffi_hook_namespace(&self) -> String {
            format!("{}.ReloadedFFI.Hooks", self.get_mod_id())
        }

        pub fn ffi_interface_namespace(&self, i: &str) -> String {
            format!("{}.ReloadedFFI.Interfaces.{}", self.get_mod_id(), i)
        }
        */

        pub fn get_release_metadata_name(&self) -> String {
            if let Some(v) = &self.UpdateSourceData {
                if let Some(u) = &v.ReleaseMetadataName {
                    return u.clone()
                }
            }
            format!("{}.ReleaseMetadata.json", self.get_mod_id())
        }

        fn can_import_field_from_cargo<T>(p: &Option<bool>, v: &Option<T>) -> bool {
            if v.is_none() {
                if let Some(b) = p { if *b { return true }}
            }
            false
        }

        // Fetch from Cargo.toml and populate the value.
        fn import_string_from_cargo(cargo: &super::CargoInfo, 
            p: &Option<bool>, v: &mut Option<String>, t: &str) 
            -> Result<(), Box<dyn Error>> {
            if Self::can_import_field_from_cargo::<String>(p, v) {
                match cargo.get_package_string(t)? {
                    Some(s) => *v = Some(s.to_owned()),
                    None => return Err(Box::new(InvalidPackageToml(format!("{} is missing a value!", t))))
                };
            }
            Ok(())
        }
        fn check_field_populated<T>(p: &Option<T>, t: &str) -> Result<(), Box<dyn Error>> {
            if p.is_none() { Err(Box::new(InvalidPackageToml(format!("{} is missing a value!", t)))) } 
            else { Ok(()) }
        }

        pub fn new<T: AsRef<Path>>(base: T, cargo: &super::CargoInfo) -> Result<Self, Box<dyn Error>> {
            let data = base.as_ref().join("data");
            let res = fs::read_to_string(data.join(PACKAGE_FILENAME))?;
            let mut res = toml::from_str::<Self>(&res)?;
            if let Some(c) = &res.CargoImport {
                Self::import_string_from_cargo(cargo, &c.Summary, &mut res.Summary, "description")?;
                Self::import_string_from_cargo(cargo, &c.Version, &mut res.Version, "version")?;
                Self::import_string_from_cargo(cargo, &c.SourceUrl, &mut res.SourceUrl, "repository")?;
                Self::import_string_from_cargo(cargo, &c.ProjectUrl, &mut res.ProjectUrl, "homepage")?;
            } else {
                Self::check_field_populated(&res.Summary, "Summary")?;
                Self::check_field_populated(&res.Version, "Version")?;
            };
            Ok(res)
        }
    }

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct ExtCargoImport {
        pub Summary: Option<bool>,
        pub Version: Option<bool>,
        pub SourceUrl: Option<bool>,
        pub ProjectUrl: Option<bool>
    }

    #[derive(Deserialize, Debug)]
    pub enum HookLibrary {
        Reloaded2CSharpHooks,
        Reloaded2RustHooks,
        Reloaded3
    }

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct ExtHookSettings {
        pub HookLibrary: HookLibrary,
        pub DefaultCallingConvention: String
    }
}
pub mod reloaded2 {
    // from https://github.com/Reloaded-Project/Reloaded-II/blob/master/source/Reloaded.Mod.Loader.IO/Config/ModConfig.cs
    // use semver::Version;
    use serde::Serialize;
    use std::{
        collections::HashMap,
        error::Error,
        path::Path
    };

    #[allow(non_snake_case)]
    #[derive(Debug, Serialize)]
    pub struct DependencyData {
        Config: DependencyConfig,
        ReleaseMetadataName: String
    }

    #[allow(non_snake_case)]
    #[derive(Debug, Serialize)]
    pub struct DepedencyDataHashMap {
        pub IdToConfigMap: HashMap<String, DependencyData>
    }
    impl DepedencyDataHashMap {
        pub fn new() -> Self {
            Self { IdToConfigMap: HashMap::new() }
        }
        pub fn is_empty(&self) -> bool {
            self.IdToConfigMap.is_empty()
        }
        pub fn insert(&mut self, k: String, v: DependencyData) -> Option<DependencyData> {
            self.IdToConfigMap.insert(k, v)
        }
    }

    #[derive(Debug, Serialize)]
    #[serde(untagged)]
    pub enum DependencyConfig {
        Gamebanana(GamebananaDependency),
        Github(GithubDependency)
    }

    #[allow(non_snake_case)]
    #[derive(Debug, Serialize)]
    pub struct GamebananaDependency {
        ItemType: String,
        ItemId: usize
    }

    #[allow(non_snake_case)]
    #[derive(Debug, Serialize)]
    pub struct GithubDependency {
        UserName: String,
        RepositoryName: String,
        UseReleaseTag: bool,
        AssetFileName: String
    }

    #[allow(non_snake_case)]
    #[derive(Debug, Serialize)]
    pub struct PluginData {
        #[serde(skip_serializing_if = "DepedencyDataHashMap::is_empty")]
        GithubDependencies: DepedencyDataHashMap,
        #[serde(skip_serializing_if = "DepedencyDataHashMap::is_empty")]
        GameBananaDependencies: DepedencyDataHashMap,
        #[serde(skip_serializing_if = "Option::is_none")]
        GithubRelease: Option<GithubDependency>
    }

    #[allow(non_snake_case)]
    #[derive(Debug, Serialize)]
    pub struct Package {
        ModId: String,
        ModName: String,
        ModAuthor: String,
        ModVersion: String,
        ModDescription: String,
        ModDll: String,
        ModIcon: String,
        ModR2RManagedDll32: String,
        ModR2RManagedDll64: String,
        ModNativeDll32: String,
        ModNativeDll64: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        Tags: Vec<String>,
        PluginData: PluginData,
        #[serde(skip_serializing_if = "Option::is_none")]
        CanUnload: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        HasExports: Option<bool>,
        IsLibrary: bool,
        ReleaseMetadataFileName: String,
        IsUniversalMod: bool,
        ModDependencies: Vec<String>,
        OptionalDependencies: Vec<String>,
        SupportedAppId: Vec<String>,
        ProjectUrl: String
    }

    #[derive(Debug)]
    pub struct InvalidModConfigJson(String);
    impl Error for InvalidModConfigJson { }
    impl std::fmt::Display for InvalidModConfigJson {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Failure while parsing {}: {}", PACKAGE_FILENAME, self.0)
        }
    }

    pub const PACKAGE_FILENAME: &'static str = "ModConfig.json";

    impl Package {
        pub fn save<T: AsRef<Path>>(&self, base: T) -> Result<(), Box<dyn Error>> {
            let data_dir = base.as_ref().join("middata");
            let output_path = data_dir.join(PACKAGE_FILENAME);
            let output = std::fs::File::create(output_path)?;
            simd_json::serde::to_writer_pretty(output, self)?;
            Ok(())
        }
        pub fn get_mod_id(&self) -> &str { &self.ModId }
    }

    impl Default for Package {
        fn default() -> Self {
            Package {
                ModId: "reloaded.template.modconfig".to_owned(),
                ModName: "Reloaded Mod Config Template".to_owned(),
                ModAuthor: "Someone".to_owned(),
                ModVersion: "1.0.0".to_owned(),
                ModDescription: "Template for a Reloaded Mod Configuration".to_owned(),
                ModDll: String::new(),
                ModIcon: String::new(),
                ModR2RManagedDll32: String::new(),
                ModR2RManagedDll64: String::new(),
                ModNativeDll32: String::new(),
                ModNativeDll64: String::new(),
                Tags: vec![],
                PluginData: PluginData { 
                    GithubDependencies: DepedencyDataHashMap::new(), 
                    GameBananaDependencies: DepedencyDataHashMap::new(), 
                    GithubRelease: None 
                },
                CanUnload: None,
                HasExports: None,
                IsLibrary: false,
                ReleaseMetadataFileName: "Sewer56.Update.ReleaseMetadata.json".to_owned(),
                IsUniversalMod: false,
                ModDependencies: vec![],
                OptionalDependencies: vec![],
                SupportedAppId: vec![],
                ProjectUrl: String::new()
            }
        }
    }
    impl TryFrom<super::reloaded3ririext::Package> for Package {
        type Error = Box<dyn std::error::Error>;
        fn try_from(value: super::reloaded3ririext::Package) -> Result<Self, Self::Error> {
            // TODO: Make this not suck, use a by-reference conversion method instead to store
            // string references/slices
            let mod_id = value.get_mod_id().to_owned();
            let version = match &value.Version {
                Some(v) => v.to_owned(),
                None => return Err(Box::new(InvalidModConfigJson("No version field was found".to_owned())))
            };
            let description = match &value.Summary {
                Some(v) => v.to_owned(),
                None => return Err(Box::new(InvalidModConfigJson("No description field was found".to_owned())))
            };
            let dll_base = format!("{}.dll", value.get_mod_id());
            let proj_url = match &value.ProjectUrl {
                Some(u) => u.to_owned(),
                None => String::new()
            };
            let release_metadata = value.get_release_metadata_name();
            let dep_ids = value.R2Dependencies.iter().map(|m| m.Id.clone()).collect();
            // get plugin data
            let mut plugins = PluginData { 
                GameBananaDependencies: DepedencyDataHashMap::new(), 
                GithubDependencies: DepedencyDataHashMap::new(),
                GithubRelease: None
            };
            for dep in &value.R2Dependencies {
                if dep.UpdateSourceData.is_none() { continue; }
                let update_data = dep.UpdateSourceData.as_ref().unwrap();
                if let Some(d) = &update_data.Gamebanana {
                    plugins.GameBananaDependencies.insert(
                        dep.Id.clone(),
                        DependencyData {
                            Config: DependencyConfig::Gamebanana(GamebananaDependency {
                                ItemType: d.ItemType.clone(),
                                ItemId: d.ItemId, 
                            }),
                            ReleaseMetadataName: dep.get_release_metadata_name()
                        }
                    );
                }
                if let Some(d) = &update_data.Github {
                    plugins.GithubDependencies.insert(
                        dep.Id.clone(),
                        DependencyData {
                            Config: DependencyConfig::Github(GithubDependency {
                                UserName: d.UserName.clone(),
                                RepositoryName: d.RepositoryName.clone(),
                                UseReleaseTag: false,
                                AssetFileName: update_data.get_asset_file_name()
                            }),
                            ReleaseMetadataName: dep.get_release_metadata_name()
                        }
                    );
                }
            }
            if let Some(d) = &value.UpdateSourceData {
                if let Some(g) = &d.Github {
                    plugins.GithubRelease = Some(GithubDependency {
                        UserName: g.UserName.clone(),
                        RepositoryName: g.RepositoryName.clone(),
                        UseReleaseTag: false,
                        AssetFileName: d.get_asset_file_name()
                    });
                }
            }
            Ok(Package {
                ModId: mod_id,
                ModName: value.Name,
                ModAuthor: value.Author,
                ModVersion: version,
                ModDescription: description,
                ModDll: dll_base.clone(),
                ModIcon: "Icon.png".to_owned(),
                ModR2RManagedDll32: format!("x86/{}", dll_base),
                ModR2RManagedDll64: format!("x64/{}", dll_base),
                ModNativeDll32: String::new(),
                ModNativeDll64: String::new(),
                Tags: vec![],
                PluginData: plugins,
                CanUnload: None,
                HasExports: None,
                IsLibrary: value.IsDependency,
                ReleaseMetadataFileName: release_metadata,
                IsUniversalMod: false,
                ModDependencies: dep_ids,
                OptionalDependencies: vec![],
                SupportedAppId: value.SupportedGames,
                ProjectUrl: proj_url,
            })
        }
    }
}
use handlebars::Handlebars;
use std::{
    error::Error,
    fs,
    io::Write,
    path::{ Path, PathBuf },
    ptr::NonNull
};

// TODO: More error reporting info
#[derive(Debug)]
struct TomlBadParse(String);
impl std::fmt::Display for TomlBadParse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Couldn't parse TOML: {}", &self.0)
    }
}
impl Error for TomlBadParse { }

fn get_boolean<'a>(t: &'a toml::Table, k: &str) -> Result<&'a bool, Box<dyn Error>> {
    if let Some(n) = t.get(k) {
        if let toml::Value::Boolean(v) = n { return Ok(v) }
    }
    Err(Box::new(TomlBadParse(format!("Could not get boolean value from {}", k))))
}

fn get_boolean_mut<'a>(t: &'a mut toml::Table, k: &str) -> Result<&'a mut bool, Box<dyn Error>> {
    if let Some(n) = t.get_mut(k) {
        if let toml::Value::Boolean(v) = n { return Ok(v) }
    }
    Err(Box::new(TomlBadParse(format!("Could not get boolean value from {}", k))))
}

fn get_string<'a>(t: &'a toml::Table, k: &str) -> Result<&'a String, Box<dyn Error>> {
    if let Some(n) = t.get(k) {
        if let toml::Value::String(v) = n { return Ok(v) }
    }
    Err(Box::new(TomlBadParse(format!("Could not get string value from {}", k)))) 
}

fn get_string_mut<'a>(t: &'a mut toml::Table, k: &str) -> Result<&'a mut String, Box<dyn Error>> {
    if let Some(n) = t.get_mut(k) {
        if let toml::Value::String(v) = n { return Ok(v) }
    }
    Err(Box::new(TomlBadParse(format!("Could not get string value from {}", k)))) 
}

fn get_table<'a>(t: &'a toml::Table, k: &str) -> Result<&'a toml::Table, Box<dyn Error>> {
    if let Some(n) = t.get(k) {
        if let toml::Value::Table(v) = n { return Ok(v) }
    }
    Err(Box::new(TomlBadParse(format!("Could not get table from {}", k)))) 
}

fn get_table_mut<'a>(t: &'a mut toml::Table, k: &str) -> Result<&'a mut toml::Table, Box<dyn Error>> {
    if let Some(n) = t.get_mut(k) {
        if let toml::Value::Table(v) = n { return Ok(v) }
    }
    Err(Box::new(TomlBadParse(format!("Could not get table from {}", k)))) 
}

fn get_optional_table<'a>(t: &'a toml::Table, k: &str) -> Result<Option<&'a toml::Table>, Box<dyn Error>> {
    match t.get(k) {
        Some(n) => {
            match n {
                toml::Value::Table(v) => Ok(Some(v)),
                _ => Err(Box::new(TomlBadParse(format!("Could not get table from {}", k))))
            }
        },
        None => Ok(None)
    }
}

fn get_optional_table_mut<'a>(t: &'a mut toml::Table, k: &str) -> Result<Option<&'a mut toml::Table>, Box<dyn Error>> {
    match t.get_mut(k) {
        Some(n) => {
            match n {
                toml::Value::Table(v) => Ok(Some(v)),
                _ => Err(Box::new(TomlBadParse(format!("Could not get table from {}", k))))
            }
        },
        None => Ok(None)
    }
}

#[derive(Debug)]
pub struct CargoInfo {
    table: toml::Table,
    table_package: NonNull<toml::Table>,
}

impl CargoInfo {
    // fn set_package_string<'a>(f: &str, proj_pkg: &'a mut toml::Table, work_pkg: &'a mut toml::Table) -> Result<(), Box<dyn Error>> {
    fn set_package_string<'a>(&mut self, f: &str, work_pkg: &'a mut toml::Table) -> Result<(), Box<dyn Error>> {
        // SAFETY: table.package is not moved
        let proj_pkg = unsafe { self.table_package.as_mut() };
        let v = proj_pkg.get_mut(f).unwrap();
        match v {
            toml::Value::String(_) => Ok(()),
            toml::Value::Table(t) => {
                if *get_boolean_mut(t, "workspace")? {
                    // remove from workspace so we can own the target string
                    let v_new = work_pkg.remove(f).unwrap();
                    match v_new {
                        toml::Value::String(_) => {
                            *v = v_new;
                            Ok(())
                        },
                        _ => Err(Box::new(TomlBadParse(format!("{} in workspace must be a string!", f))))
                    }
                } else { Err(Box::new(TomlBadParse(format!("{}.workspace must be true", f)))) }
            },
            _ => Err(Box::new(TomlBadParse(format!("{}: expected string or table", f))))
        } 
    }
    // fn set_package_array<'a>(&mut self, f: &str, proj_pkg: &'a mut toml::Table, work_pkg: &'a mut toml::Table) -> Result<(), Box<dyn Error>> {
    fn set_package_array<'a>(&mut self, f: &str, work_pkg: &'a mut toml::Table) -> Result<(), Box<dyn Error>> {
        // SAFETY: table.package is not moved
        let proj_pkg = unsafe { self.table_package.as_mut() };
        let v = proj_pkg.get_mut(f).unwrap();
        match v {
            toml::Value::Array(_) => Ok(()),
            toml::Value::Table(t) => {
                if *get_boolean_mut(t, "workspace")? {
                    let v_new = work_pkg.remove(f).unwrap();
                    match v_new {
                        toml::Value::Array(_) => {
                            *v = v_new;
                            Ok(())
                        },
                        _ => Err(Box::new(TomlBadParse(format!("{} in workspace must be an array!", f))))
                    }
                } else { Err(Box::new(TomlBadParse(format!("{}.workspace must be true", f)))) }
            },
            _ => Err(Box::new(TomlBadParse(format!("{}: expected array or table", f))))
        } 
    }
    pub fn get_package_value(&self, k: &str) -> Option<&toml::Value> {
        let pkg = unsafe { self.table_package.as_ref() };
        pkg.get(k)
    } 

    pub fn get_package_string(&self, k: &str) -> Result<Option<&String>, Box<dyn Error>> {
        match self.get_package_value(k) {
            Some(v) => match v {
                toml::Value::String(s) => Ok(Some(s)),
                _ => Err(Box::new(TomlBadParse(format!("Value of {} is not a string", k))))
            }, None => Ok(None)
        }
    }

    pub fn get_package_string_required(&self, k: &str) -> Result<&String, Box<dyn Error>> {
        match self.get_package_string(k)? {
            Some(v) => Ok(v),
            None => Err(Box::new(TomlBadParse(format!("Value is required for {}", k))))
        }
    }

    pub fn get_package_array(&self, k: &str) -> Result<Option<&Vec<toml::Value>>, Box<dyn Error>> {
        match self.get_package_value(k) {
            Some(v) => match v {
                toml::Value::Array(a) => Ok(Some(a)),
                _ => Err(Box::new(TomlBadParse(format!("Value of {} is not a array", k))))
            }, None => Ok(None)
        }
    }

    pub fn get_package_array_required(&self, k: &str) -> Result<&Vec<toml::Value>, Box<dyn Error>> {
        match self.get_package_array(k)? {
            Some(v) => Ok(v),
            None => Err(Box::new(TomlBadParse(format!("Value is required for {}", k))))
        }
    }

    pub fn get_package_table(&self, k: &str) -> Result<Option<&toml::Table>, Box<dyn Error>> {
        match self.get_package_value(k) {
            Some(v) => match v {
                toml::Value::Table(t) => Ok(Some(t)),
                _ => Err(Box::new(TomlBadParse(format!("Value of {} is not a map", k))))
            }, None => Ok(None)
        }
    }

    pub fn get_package_table_required(&self, k: &str) -> Result<&toml::Table, Box<dyn Error>> {
        match self.get_package_table(k)? {
            Some(v) => Ok(v),
            None => Err(Box::new(TomlBadParse(format!("Value is required for {}", k))))
        }
    }

    pub fn new<T: AsRef<Path>>(base: T) -> Result<Self, Box<dyn Error>> {
    // pub fn new<T: AsRef<Path>>(base: T) -> Result<Pin<Box<Self>>, Box<dyn Error>> {
        // get project Cargo.toml
        let proj_cargo = fs::read_to_string(base.as_ref().join("Cargo.toml"))?;
        let mut res = CargoInfo { table: proj_cargo.parse::<toml::Table>()?, table_package: NonNull::dangling() };
        // let mut res = Box::new(CargoInfo { table: proj_cargo.parse::<toml::Table>()?, table_package: NonNull::dangling(), _pinned: PhantomPinned });
        res.table_package = NonNull::from(get_table(&res.table, "package")?); 
        // get workspace Cargo.toml package table, if it exists
        let mut work_pkg = match fs::read_to_string(base.as_ref().parent().unwrap().join("Cargo.toml")) {
            Ok(v) => {
                let base = v.parse::<toml::Table>()?;
                let wk = get_optional_table(&base, "workspace")?;
                if let Some(tbl) = wk {
                    if let Some(pkg) = get_optional_table(tbl, "package")? { Some(pkg.clone()) }
                    else { None }
                } else { None }
            },
            Err(_) => None
        };
        if work_pkg != None {
            // read contents of package subtable, replace workspace = true with entry from 
            let proj_pkg = unsafe { res.table_package.as_mut() };
            let pks: Vec<*const String> = proj_pkg.keys().map(|f| f as *const String).collect();
            for pk in pks {
                // SAFETY: proj_cargo is alive for the scope of the function
                let k: &str = unsafe { pk.as_ref().unwrap().as_str() };
                // https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table
                match k {
                    "description" |
                    "documentation" |
                    "edition" |
                    "homepage" |
                    "license" |
                    "license-file" |
                    "readme" |
                    "repository" |
                    "rust-version" |
                    "version"  => res.set_package_string(k, work_pkg.as_mut().unwrap())?,
                    "authors" |
                    "categories" |
                    "exclude" |
                    "include" |
                    "keywords" => res.set_package_array(k, work_pkg.as_mut().unwrap())?,
                    _ => continue
                };
            }
        } 
        Ok(res)
        // let pinned = Box::into_pin(res);
        // Ok(pinned)
    }
}

pub const HASHES_FILENAME: &'static str = "hashes.toml";

#[derive(Debug)]
pub struct HashFile<'a> {
    table: toml::Table,
    middata: PathBuf,
    data: HashFileData<'a>
}

#[derive(Debug, serde::Serialize)]
pub struct HashFileData<'a> {
    mod_id: &'a str,
    mod_name: &'a str,
    executable_hash: Vec<HashFileEntry>
}
impl<'a> HashFileData<'a> {
    fn new(mod_id: &'a str, mod_name: &'a str) -> Self { 
        Self { 
            mod_id, mod_name, executable_hash: vec![]
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct HashFileEntry {
    name: String,
    hash: u64
}

impl<'a> HashFile<'a> {
    pub fn new<T: AsRef<Path>>(base: T, mod_id: &'a str, mod_name: &'a str) -> Result<Self, Box<dyn Error>> {
        let data = base.as_ref().join("data");
        let middata = base.as_ref().join("middata");
        let res = fs::read_to_string(data.join(HASHES_FILENAME))?;
        Ok(HashFile {
            table: res.parse::<toml::Table>()?,
            middata,
            data: HashFileData::new(mod_id, mod_name)
        })
    }
    pub fn generate_mod_hashes(&mut self) -> Result<(), Box<dyn Error>> {
        let mut hbs = Handlebars::new();
        hbs.register_template_string("hashes", crate::hbs::hashes::FILE)?;
        let mut file = fs::File::create(self.middata.join("Hashes.g.cs"))?;
        for (k, v) in &self.table {
            let v_int = u64::from_str_radix(&v.as_str().unwrap()[2..], 16).unwrap();
            self.data.executable_hash.push(HashFileEntry{ name: k.clone(), hash: v_int });
        }
        file.write(hbs.render("hashes", &self.data)?.as_bytes())?;
        Ok(())
    }
}
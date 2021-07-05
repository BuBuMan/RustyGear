use std::collections::HashMap;
use std::fs;

pub struct Resources {
    pub prefabs: HashMap<String, String>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            prefabs : Resources::load_all_prefabs(),
        }
    }

    fn load_all_prefabs() -> HashMap<String, String> {
        let mut prefab_dir = std::env::current_dir().unwrap();
        prefab_dir.push("src\\resources\\prefabs");
        let paths = fs::read_dir(&prefab_dir).unwrap();
        
        let mut prefabs = HashMap::new();

        for p in paths {
            let path = p.unwrap().path();
            let contents = fs::read(&path).unwrap();
            let json_value : serde_json::Value = serde_json::from_slice(&contents).unwrap();
            let key = path.strip_prefix(&prefab_dir.as_path()).unwrap().to_owned().into_os_string().into_string().unwrap();
            let value = serde_json::to_string(&json_value).unwrap();

            prefabs.insert(key, value);
        }

        prefabs
    }
}

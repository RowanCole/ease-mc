
use std::{io, path::Path};

pub fn collect_jars(dir: &Path, base: &Path, jars: &mut Vec<String>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("read_dir failed: {}", e))? {
        let entry = entry.map_err(|e| format!("entry failed: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            collect_jars(&path, base, jars)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("jar") {
            if let Ok(rel) = path.strip_prefix(base) {
                jars.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(())
}

























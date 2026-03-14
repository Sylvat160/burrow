use std::env;
use std::os::unix::fs::PermissionsExt;

pub fn find_in_path(program: &str) -> Option<String> {
    if let Ok(path_str) = env::var("PATH") {
        for dir in env::split_paths(&path_str) {
            let path = dir.join(program);
            if path.exists() && path.is_file() {
                if let Ok(metadata) = path.metadata() {
                    let permissions = metadata.permissions();
                    if permissions.mode() & 0o111 != 0 {
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    None
}

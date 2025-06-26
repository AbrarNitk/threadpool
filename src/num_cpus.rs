use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn get_physical_cores() -> io::Result<usize> {
    let file = File::open("/proc/cpuinfo")?;
    let reader = BufReader::new(file);

    let mut unique_physical_cores = HashSet::new();

    let mut current_physical_id: Option<u32> = None;
    let mut current_core_id: Option<u32> = None;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            if let (Some(pid), Some(cid)) = (current_physical_id, current_core_id) {
                unique_physical_cores.insert((pid, cid));
            }
            current_physical_id = None;
            current_core_id = None;
        } else if line.starts_with("physical id") {
            if let Some(id_str) = line.split(':').nth(1) {
                if let Ok(id) = id_str.trim().parse::<u32>() {
                    current_physical_id = Some(id);
                }
            }
        } else if line.starts_with("core id") {
            if let Some(id_str) = line.split(':').nth(1) {
                if let Ok(id) = id_str.trim().parse::<u32>() {
                    current_core_id = Some(id);
                }
            }
        }
    }
    if let (Some(pid), Some(cid)) = (current_physical_id, current_core_id) {
        unique_physical_cores.insert((pid, cid));
    }

    Ok(unique_physical_cores.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_physical_cores() {
        println!("total cores: {}", get_physical_cores().unwrap());
    }
}

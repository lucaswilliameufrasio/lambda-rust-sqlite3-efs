use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    process,
    sync::atomic::{AtomicU32, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn get_machine_id() -> [u8; 3] {
    let hostname = match hostname::get() {
        Ok(s) => s.to_string_lossy().into_owned(),
        Err(_) => "localhost".to_string(),
    };
    let mut hasher = DefaultHasher::new();
    hostname.hash(&mut hasher);
    let h = hasher.finish();
    [(h >> 16) as u8, (h >> 8) as u8, h as u8]
}

pub fn generate_xid_string() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let pid = (process::id() % 65536) as u16;
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let machine_id = get_machine_id();

    let mut xid = [0u8; 12];
    xid[0..4].copy_from_slice(&now.to_be_bytes());
    xid[4..7].copy_from_slice(&machine_id);
    xid[7..9].copy_from_slice(&pid.to_be_bytes());
    xid[9..12].copy_from_slice(&counter.to_be_bytes()[1..4]);

    hex::encode(xid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn xid_is_24_char_hex() {
        let id = generate_xid_string();
        assert_eq!(id.len(), 24);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn xid_is_lowercase() {
        let id = generate_xid_string();
        assert_eq!(id, id.to_lowercase());
    }

    #[test]
    fn xid_are_unique() {
        let mut seen = HashSet::new();
        for _ in 0..1000 {
            let id = generate_xid_string();
            assert!(seen.insert(id), "duplicate XID generated");
        }
    }
}

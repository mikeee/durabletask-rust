use gethostname::gethostname;
use uuid::Uuid;

pub(crate) fn get_default_worker_name() -> String {
    let hostname: String = gethostname().into_string().unwrap_or("unknown".to_string());
    let pid = std::process::id();
    let uuid = Uuid::new_v4();
    format!("{hostname},{pid},{uuid}")
}

#[cfg(test)]
mod tests {
    use crate::internal::get_default_worker_name;
    use uuid::Uuid;

    #[test]
    fn test_get_default_worker_name() {
        let result = get_default_worker_name();
        println!("{}", result.clone()); //debug
        let parsed: Vec<String> = result.split(',').map(|s| s.to_string()).collect();
        assert_ne!(parsed[0], "unknown");
        assert!(parsed[1].parse::<u64>().is_ok());
        let id = Uuid::parse_str(&parsed[2]).unwrap();
        assert_eq!(id.get_version(), Some(uuid::Version::Random));
    }
}

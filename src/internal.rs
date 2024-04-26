use gethostname::gethostname;
use uuid::Uuid;

/// Return the function name as a String
pub(crate) fn get_task_function_name<F>(_: F) -> String {
    let function_path = std::any::type_name::<F>();
    // Return the function name without the preceding path
    function_path.split("::").last().unwrap().to_string()
}

/// Return the default worker name consisting of:
///
/// - Hostname
/// - Process ID (PID)
/// - Universally Unique Identifier v4 (UUID)
///
/// Returned as a String in this format:
/// {Hostname},{PID},{UUID}
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

    use super::get_task_function_name;

    #[test]
    fn test_get_task_function_name() {
        fn test_task_function(output: String) {
            println!("finished something: {output}")
        }
        let result = get_task_function_name(test_task_function);
        assert_eq!(result, "test_task_function")
    }

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

//! Path comparison utilities (pure technical, cross-platform)

use std::path::Path;

/// Compares two paths to determine if they refer to the same file
pub fn is_same_process(process_path: &str, expected_path: &str) -> bool {
    let process_path = Path::new(process_path);
    let expected_path = Path::new(expected_path);

    if cfg!(debug_assertions) {
        return process_path.file_name() == expected_path.file_name();
    }

    if let (Ok(p1), Ok(p2)) = (process_path.canonicalize(), expected_path.canonicalize()) {
        #[cfg(windows)]
        {
            if let (Some(s1), Some(s2)) = (p1.to_str(), p2.to_str()) {
                return s1.eq_ignore_ascii_case(s2);
            }
        }

        #[cfg(unix)]
        {
            return p1 == p2;
        }
    }

    #[cfg(windows)]
    {
        process_path.to_str().is_some_and(|p| {
            expected_path
                .to_str()
                .is_some_and(|e| p.eq_ignore_ascii_case(e))
        })
    }

    #[cfg(unix)]
    {
        process_path == expected_path
    }
}

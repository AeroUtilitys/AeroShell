use reedline::Completer;

#[derive(Clone)]
pub struct AeroCompleter;

impl Completer for AeroCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<reedline::Suggestion> {
        // Logic for start pos: simply split by whitespace, find which word we are in
        let mut start = pos;
        while start > 0 {
            // Check previous character
            let prev_slice = &line[..start];
            if let Some(c) = prev_slice.chars().last() {
                if c.is_whitespace() {
                    break;
                }
                start -= c.len_utf8();
            } else {
                break;
            }
        }

        let prefix = &line[start..pos];
        let mut suggestions = Vec::new();

        // 1. Command Completion (if it's the first word)
        // Check if there's any whitespace before our current word
        let is_first_word = line[..start].trim().is_empty();

        if is_first_word {
            if let Ok(paths) = std::env::var("PATH") {
                for path in std::env::split_paths(&paths) {
                    if let Ok(entries) = std::fs::read_dir(path) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let name = entry.file_name().to_string_lossy().to_string();
                                if name.starts_with(prefix) {
                                    suggestions.push(reedline::Suggestion {
                                        value: name,
                                        description: None,
                                        style: None,
                                        extra: None,
                                        span: reedline::Span { start, end: pos },
                                        append_whitespace: true,
                                        match_indices: Some(Vec::new()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            // Add built-ins
            for builtin in ["cd", "exit", "clear", "help", "config", "theme"] {
                if builtin.starts_with(prefix) {
                    suggestions.push(reedline::Suggestion {
                        value: builtin.to_string(),
                        description: Some("Built-in".to_string()),
                        style: None,
                        extra: None,
                        span: reedline::Span { start, end: pos },
                        append_whitespace: true,
                         match_indices: Some(Vec::new()),
                    });
                }
            }
        } else {
            // 2. File Path Completion
            // Basic implementation:
            let path_prefix = prefix;
            let (dir, file_part) = if path_prefix.ends_with('/') {
                (path_prefix, "")
            } else {
                match path_prefix.rfind('/') {
                    Some(idx) => (&path_prefix[..=idx], &path_prefix[idx+1..]),
                    None => ("", path_prefix),
                }
            };

            let search_dir = if dir.is_empty() { "." } else { dir };

            if let Ok(entries) = std::fs::read_dir(search_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with(file_part) {
                            let mut value = format!("{}{}", dir, name);
                            if entry.path().is_dir() {
                                value.push('/');
                            }
                            suggestions.push(reedline::Suggestion {
                                value,
                                description: None,
                                style: None,
                                extra: None,
                                span: reedline::Span { start, end: pos },
                                append_whitespace: false,
                                match_indices: Some(Vec::new()),
                            });
                        }
                    }
                }
            }
        }

        // Deduplicate
        suggestions.sort_by(|a, b| a.value.cmp(&b.value));
        suggestions.dedup_by(|a, b| a.value == b.value);

        suggestions
    }
}

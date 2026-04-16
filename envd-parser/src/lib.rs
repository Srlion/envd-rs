use std::collections::HashMap;

const PATTERN: &str = r#"(?m)^\s*(?:export\s+)?([\w\.-]+)(?:\s*=\s*?|:\s+?)(\s*'(?:\\'|[^'])*'|\s*"(?:\\"|[^"])*"|\s*`(?:\\`|[^`])*`|[^#\r\n]*?)?\s*(?:#.*)?$"#;

pub fn parse(src: &str) -> HashMap<String, String> {
    let normalized = src.replace("\r\n", "\n").replace('\r', "\n");

    let mut map: HashMap<String, String> = HashMap::new();

    let mut file_entries: Vec<(String, bool)> = Vec::new();

    let re = regex_lite::Regex::new(PATTERN).expect("invalid regex");
    for caps in re.captures_iter(&normalized) {
        let key = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let raw_val = caps
            .get(2)
            .map(|m| m.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        let (value, first_char) = if raw_val.len() >= 2 {
            let bytes = raw_val.as_bytes();
            let first = bytes[0] as char;
            let last = bytes[raw_val.len() - 1] as char;
            if (first == '\'' || first == '"' || first == '`') && first == last {
                (raw_val[1..raw_val.len() - 1].to_string(), first)
            } else {
                (raw_val, '\0')
            }
        } else {
            (raw_val, '\0')
        };

        let final_value = if first_char == '"' {
            value.replace(r"\n", "\n").replace(r"\r", "\r")
        } else {
            value
        };

        let should_expand = first_char != '\'';
        file_entries.push((key.clone(), should_expand));
        map.insert(key, final_value);
    }

    for (key, should_expand) in &file_entries {
        if !should_expand {
            continue;
        }

        let value = match map.get(key) {
            Some(v) => v.clone(),
            None => continue,
        };

        let expanded = expand_value(&value, &map);
        if expanded != value {
            map.insert(key.clone(), expanded);
        }
    }

    map
}

fn expand_value(value: &str, map: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(value.len());
    let chars: Vec<(usize, char)> = value.char_indices().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let (_, ch) = chars[i];

        if ch == '\\' && i + 1 < len && chars[i + 1].1 == '$' {
            // Escaped $: emit literal $
            result.push('$');
            i += 2;
        } else if ch == '$' {
            i += 1;
            if i >= len {
                result.push('$');
                break;
            }

            if chars[i].1 == '{' {
                // ${VAR} or ${VAR:-default}
                i += 1;
                let key_start = i;

                while i < len && (chars[i].1.is_ascii_alphanumeric() || chars[i].1 == '_') {
                    i += 1;
                }

                let var_name: String = chars[key_start..i].iter().map(|(_, c)| c).collect();

                // Check for :-default
                let default_value = if i + 1 < len && chars[i].1 == ':' && chars[i + 1].1 == '-' {
                    i += 2;
                    let default_start = i;
                    while i < len && chars[i].1 != '}' {
                        i += 1;
                    }
                    let val: String = chars[default_start..i].iter().map(|(_, c)| c).collect();
                    Some(val)
                } else {
                    None
                };

                // Skip closing }
                if i < len && chars[i].1 == '}' {
                    i += 1;
                }

                let resolved = map
                    .get(var_name.as_str())
                    .filter(|v| !v.is_empty())
                    .map(|v| v.as_str());

                match resolved {
                    Some(v) => result.push_str(v),
                    None => {
                        if let Some(ref d) = default_value {
                            result.push_str(d);
                        }
                    }
                }
            } else if chars[i].1.is_ascii_alphabetic() || chars[i].1 == '_' {
                // $VAR (no default syntax)
                let key_start = i;
                while i < len && (chars[i].1.is_ascii_alphanumeric() || chars[i].1 == '_') {
                    i += 1;
                }
                let var_name: String = chars[key_start..i].iter().map(|(_, c)| c).collect();

                let resolved = map.get(var_name.as_str()).map(|v| v.as_str()).unwrap_or("");
                result.push_str(resolved);
            } else {
                // Bare $ followed by something unexpected
                result.push('$');
            }
        } else {
            result.push(ch);
            i += 1;
        }
    }

    result
}

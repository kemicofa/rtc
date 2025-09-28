use anyhow::{ bail, Result, Ok };
use logs_to_graph::service_node_graph::{ HttpMethod, HttpPath, ServiceName };
use url::Url;
use regex::Regex;

use crate::{ consts::PATH_NORMALIZE_PATTERNS };

/// Normalizes a path by replacing id's and uuid's
pub fn normalize_path(
    url_str: &str,
    path_normalize_regexes: Vec<(String, Vec<Regex>)>
) -> Result<String> {
    let url = Url::parse(url_str)?;
    let path_segments: Vec<&str> = match url.path_segments() {
        Some(segments) => segments.collect(),
        None => bail!("Cannot extract segments from URL path"),
    };

    let mut normalized_segments = Vec::new();
    let mut segment_iter = path_segments.iter().peekable();

    while let Some(segment) = segment_iter.next() {
        let mut matched = false;
        for (r#type, regexes) in path_normalize_regexes.clone() {
            for regex in regexes {
                if !regex.is_match(&segment) {
                    continue;
                }

                if let Some(prev_segment) = normalized_segments.last() {
                    normalized_segments.push(format!("{{{}_{}}}", prev_segment, r#type));
                } else {
                    normalized_segments.push(format!("{{{}}}", r#type).to_string());
                }
                matched = true;
                break;
            }

            if matched {
                break;
            }
        }

        if !matched {
            normalized_segments.push(segment.to_string());
        }
    }

    Ok(format!("/{}", normalized_segments.join("/")))
}

pub fn get_default_path_normalize_regexes() -> Vec<(String, Vec<Regex>)> {
    let path_normalize_regexes: Vec<(String, Vec<Regex>)> = PATH_NORMALIZE_PATTERNS.iter()
        .map(|(key, value)| { (key.to_string(), vec![Regex::new(value).unwrap()]) })
        .collect();

    path_normalize_regexes
}

#[cfg(test)]
mod test {
    use regex::Regex;

    use crate::{ normalize::{ get_default_path_normalize_regexes, normalize_path } };

    #[test]
    fn should_replace_ids() {
        let url = "https://test.com/users/12345/books/12345";
        let expect = "/users/{users_id}/books/{books_id}".to_string();
        let path_normalize_regexes = get_default_path_normalize_regexes();
        let res = normalize_path(url, path_normalize_regexes);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expect);
    }

    #[test]
    fn should_replace_consecutive_ids() {
        let url = "https://test.com/users/12345/12345/books/12345/12345";
        let expect =
            "/users/{users_id}/{{users_id}_id}/books/{books_id}/{{books_id}_id}".to_string();
        let path_normalize_regexes = get_default_path_normalize_regexes();
        let res = normalize_path(url, path_normalize_regexes);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expect);
    }

    #[test]
    fn should_replace_uuids() {
        let url = "https://test.com/users/91366bf0-4c97-4832-af68-452c51ca38eb/books/12345";
        let expect = "/users/{users_uuid}/books/{books_id}".to_string();
        let path_normalize_regexes = get_default_path_normalize_regexes();
        let res = normalize_path(url, path_normalize_regexes);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expect);
    }

    #[test]
    fn should_replace_using_custom_path_regex() {
        let url =
            "https://test.com/users/91366bf0-4c97-4832-af68-452c51ca38eb/books/12345/car/prefix-12345";
        let expect = "/users/{users_uuid}/books/{books_id}/car/{car_custom_id}".to_string();
        let mut path_normalize_regexes = get_default_path_normalize_regexes();
        path_normalize_regexes.push(("custom".into(), vec![Regex::new("prefix-\\d+").unwrap()]));
        let res = normalize_path(url, path_normalize_regexes);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expect);
    }
}

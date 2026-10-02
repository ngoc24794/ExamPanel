//! Normalization and parsing helpers for Excel import and text processing.

use unicode_normalization::UnicodeNormalization;

/// Normalizes Vietnamese text using Unicode NFC, trims leading/trailing spaces,
/// and collapses consecutive whitespace into a single ASCII space.
pub fn normalize_text(s: &str) -> String {
    let nfc: String = s.nfc().collect();
    nfc.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalizes identifier codes (e.g. campus code "cs1" -> "CS1", teacher code "gv01" -> "GV01").
pub fn normalize_code(s: &str) -> String {
    normalize_text(s).to_uppercase()
}

/// Parses grade qualifications string into a sorted, deduplicated list of grade codes (10, 11, 12).
///
/// Supported formats:
/// - "10, 11" or "10; 11" or "10 11"
/// - "K10, K11" or "Khối 10, Khối 12"
/// - "10,11,12"
pub fn parse_grade_codes(s: &str) -> Result<Vec<i32>, String> {
    let norm = normalize_text(s);
    if norm.is_empty() {
        return Err("Danh sách khối dạy không được để trống".to_string());
    }

    // Split by common delimiters: comma, semicolon, slash, pipe
    let parts: Vec<&str> = norm.split([',', ';', '/', '|']).collect();

    let mut grades = Vec::new();

    for raw_part in parts {
        let part = raw_part.trim();
        if part.is_empty() {
            continue;
        }

        // If part contains whitespace, e.g. "10 11", split by whitespace
        for token in part.split_whitespace() {
            // Strip leading "khối", "khoi", "k", "grade" (case-insensitive)
            let lower = token.to_lowercase();
            let stripped = lower
                .trim_start_matches("khối")
                .trim_start_matches("khoi")
                .trim_start_matches('k')
                .trim_start_matches("grade")
                .trim();

            if stripped.is_empty() {
                continue;
            }

            match stripped.parse::<i32>() {
                Ok(grade_num) => {
                    if grade_num == 10 || grade_num == 11 || grade_num == 12 {
                        grades.push(grade_num);
                    } else {
                        return Err(format!(
                            "Khối {grade_num} không hợp lệ (chỉ hỗ trợ khối 10, 11, 12)"
                        ));
                    }
                }
                Err(_) => {
                    return Err(format!("Không thể nhận dạng khối dạy từ '{token}'"));
                }
            }
        }
    }

    if grades.is_empty() {
        return Err("Không tìm thấy khối dạy hợp lệ (10, 11, 12)".to_string());
    }

    grades.sort_unstable();
    grades.dedup();
    Ok(grades)
}

/// Parses boolean values from Vietnamese and standard strings.
///
/// Returns `default` if the string is empty.
pub fn parse_boolean(s: &str, default: bool) -> Result<bool, String> {
    let norm = normalize_text(s).to_lowercase();
    if norm.is_empty() {
        return Ok(default);
    }

    match norm.as_str() {
        "có" | "co" | "x" | "1" | "true" | "yes" | "v" | "y" | "đang dạy" | "dang day" => {
            Ok(true)
        }
        "không" | "khong" | "0" | "false" | "no" | "n" | "tạm nghỉ" | "tam nghi" => Ok(false),
        _ => Err(format!(
            "Giá trị không hợp lệ: '{s}' (kỳ vọng 'Có' hoặc 'Không')"
        )),
    }
}

/// Parses load weight decimals (e.g. "1", "0.5", "0,75").
///
/// Returns `default` if empty. Must be within [0.0, 1.0].
pub fn parse_load_weight(s: &str, default: f64) -> Result<f64, String> {
    let norm = normalize_text(s);
    if norm.is_empty() {
        return Ok(default);
    }

    let standardized = norm.replace(',', ".");
    match standardized.parse::<f64>() {
        Ok(val) => {
            if !(0.0..=1.0).contains(&val) {
                return Err(format!(
                    "Hệ số tải {val} vượt quá phạm vi cho phép (0.0 đến 1.0)"
                ));
            }
            // Round to 2 decimal places
            Ok((val * 100.0).round() / 100.0)
        }
        Err(_) => Err(format!("Hệ số tải không hợp lệ: '{s}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text_and_code() {
        assert_eq!(normalize_text("  Nguyễn   Văn   An  "), "Nguyễn Văn An");
        assert_eq!(normalize_code("  cs1  "), "CS1");
        assert_eq!(normalize_code("gv-001"), "GV-001");
    }

    #[test]
    fn test_parse_grade_codes() {
        assert_eq!(parse_grade_codes("10, 11").unwrap(), vec![10, 11]);
        assert_eq!(parse_grade_codes("10; 11; 12").unwrap(), vec![10, 11, 12]);
        assert_eq!(parse_grade_codes("K10, K11").unwrap(), vec![10, 11]);
        assert_eq!(parse_grade_codes("Khối 10, Khối 12").unwrap(), vec![10, 12]);
        assert_eq!(parse_grade_codes("10 11 12").unwrap(), vec![10, 11, 12]);
        assert_eq!(parse_grade_codes("12, 10").unwrap(), vec![10, 12]); // sorted
        assert_eq!(parse_grade_codes("10, 10").unwrap(), vec![10]); // deduped

        // Errors
        assert!(parse_grade_codes("").is_err());
        assert!(parse_grade_codes("9, 10").is_err());
        assert!(parse_grade_codes("abc").is_err());
    }

    #[test]
    fn test_parse_boolean() {
        assert_eq!(parse_boolean("Có", false).unwrap(), true);
        assert_eq!(parse_boolean("co", false).unwrap(), true);
        assert_eq!(parse_boolean("x", false).unwrap(), true);
        assert_eq!(parse_boolean("1", false).unwrap(), true);
        assert_eq!(parse_boolean("Không", true).unwrap(), false);
        assert_eq!(parse_boolean("khong", true).unwrap(), false);
        assert_eq!(parse_boolean("0", true).unwrap(), false);
        assert_eq!(parse_boolean("", true).unwrap(), true);
        assert_eq!(parse_boolean("", false).unwrap(), false);
        assert!(parse_boolean("maybe", false).is_err());
    }

    #[test]
    fn test_parse_load_weight() {
        assert_eq!(parse_load_weight("1", 1.0).unwrap(), 1.0);
        assert_eq!(parse_load_weight("0.5", 1.0).unwrap(), 0.5);
        assert_eq!(parse_load_weight("0,75", 1.0).unwrap(), 0.75);
        assert_eq!(parse_load_weight("0,25", 1.0).unwrap(), 0.25);
        assert_eq!(parse_load_weight("0", 1.0).unwrap(), 0.0);
        assert_eq!(parse_load_weight("", 1.0).unwrap(), 1.0);

        // Errors
        assert!(parse_load_weight("1.5", 1.0).is_err());
        assert!(parse_load_weight("-0.1", 1.0).is_err());
        assert!(parse_load_weight("xyz", 1.0).is_err());
    }
}

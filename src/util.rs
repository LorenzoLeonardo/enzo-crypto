use std::{error::Error, fs, path::Path};

pub fn data_source(input: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let path = Path::new(input);

    let data: Vec<u8> = if path.exists() && path.is_file() {
        // Definitely a real file
        fs::read(path)?
    } else {
        // Not a real file → treat as literal string
        input.as_bytes().to_vec()
    };
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_real_file() {
        let mut tmp = NamedTempFile::new().expect("temp file failed");
        writeln!(tmp, "hello").unwrap(); // file contains "hello\n"

        let path = tmp.path().to_str().unwrap();
        let data = data_source(path).unwrap();

        assert_eq!(data, b"hello\n");
    }

    #[test]
    fn test_literal_string() {
        let result = data_source("NotARealFile.txt").unwrap();
        assert_eq!(result, b"NotARealFile.txt");
    }

    #[test]
    fn test_directory_is_treated_as_string() {
        // use current directory, always exists and is a directory
        let current_dir = ".";
        let result = data_source(current_dir).unwrap();

        assert_eq!(result, b".");
    }

    #[test]
    fn test_binary_file() {
        let mut tmp = NamedTempFile::new().unwrap();
        let bytes = vec![0x00, 0xFF, 0x42, 0x10];
        tmp.write_all(&bytes).unwrap();

        let path = tmp.path().to_str().unwrap();
        let data = data_source(path).unwrap();

        assert_eq!(data, bytes);
    }

    #[test]
    fn test_utf8_string() {
        let input = "Lorenzo😊";
        let data = data_source(input).unwrap();

        assert_eq!(data, input.as_bytes());
    }
}

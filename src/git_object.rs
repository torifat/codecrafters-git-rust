#[derive(Debug)]
pub enum ObjectType {
    Blob,
    Commit,
    Tag,
    Tree,
}

#[derive(Debug)]
pub struct GitObject {
    pub object_type: ObjectType,
    pub size: usize,
    pub content: Vec<u8>,
}

impl From<Vec<u8>> for GitObject {
    fn from(data: Vec<u8>) -> Self {
        let null_byte_position = data
            .iter()
            .position(|&byte| byte == b'\x00')
            .expect("Failed to find the null byte");
        let header = std::str::from_utf8(&data[..null_byte_position])
            .expect("Failed to convert header to UTF-8");
        let header_parts: Vec<&str> = header.split(' ').collect();

        assert_eq!(header_parts.len(), 2, "Header must have exactly two parts");

        let object_type = match header_parts[0] {
            "blob" => ObjectType::Blob,
            "commit" => ObjectType::Commit,
            "tag" => ObjectType::Tag,
            "tree" => ObjectType::Tree,
            _ => panic!("Invalid object type"),
        };

        let size = header_parts[1]
            .parse::<usize>()
            .expect("Failed to parse size");

        let content = data[null_byte_position + 1..].to_vec();

        GitObject {
            object_type,
            size,
            content,
        }
    }
}

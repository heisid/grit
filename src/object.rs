enum GitObjectType {
    Blob,
    Commit,
    Tag,
    Tree,
}

struct GitObject {
    object_type: GitObjectType,
    size: usize,
    data: Vec<u8>,
}
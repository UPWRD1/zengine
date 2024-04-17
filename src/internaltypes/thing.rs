#[derive(Debug, Clone)]
pub enum PropertyKind {
    INT(i32),
    FLT(f32),
}

#[derive(Debug, Clone)]
pub enum Attribute {
    PROPERTY(PropertyKind),
    SCRIPT,
}

#[derive(Debug, Clone)]
pub struct Thing {
    name: String,
    attributes: Vec<Attribute>
}

impl Thing {
    pub fn new(name: String) -> Self {
        Thing { name, attributes: vec![] }
    }

    pub fn get_name(self) -> String {
        self.name
    }

    pub fn get_attributes(self) -> Vec<Attribute> {
        self.attributes
    }
}

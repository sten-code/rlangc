use std::fmt;

#[derive(Clone)]
pub enum Operator {
    Add,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operator::Add => "+",
            }
        )
    }
}

#[derive(Clone)]
pub enum Node {
    Program {
        body: Vec<Node>,
    },
    Scope {
        body: Vec<Node>,
    },
    BinOp {
        left: Box<Node>,
        right: Box<Node>,
        op: Operator,
    },
    Integer(i32),
    Float(f32),
    VarDecl {
        datatype: String,
        name: String,
        value: Box<Node>,
    },
    StructDecl {
        name: String,
        properties: Vec<(String, String)>,
    },
    TypeDef {
        name: String,
        value: Box<Node>,
    },
    StructType {
        properties: Vec<(String, String)>,
    },
    Identifier {
        value: String,
    },
    StructData {
        data: Vec<Node>,
    },
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Program { body } => {
                for expr in body {
                    write!(f, "{};\n", expr)?;
                }
                Ok(())
            }
            Node::Scope { body } => {
                write!(f, "{{")?;
                for expr in body {
                    write!(f, "{};\n", expr)?;
                }
                write!(f, "}}")
            }
            Node::BinOp { left, right, op } => write!(f, "{} {} {}", *left, op, *right),
            Node::Integer(value) => write!(f, "{}", value),
            Node::Float(value) => write!(f, "{}", value),
            Node::VarDecl {
                datatype,
                name,
                value,
            } => write!(f, "{} {} = {}", datatype, name, value),
            Node::StructDecl { name, properties } => {
                write!(f, "struct {} {{\n", name)?;
                for prop in properties {
                    write!(f, "    {} {};\n", prop.0, prop.1)?;
                }
                write!(f, "}}")
            }
            Node::TypeDef { name, value } => write!(f, "typedef {} {}", *value, name),
            Node::StructType { properties } => {
                write!(f, "struct {{\n")?;
                for prop in properties {
                    write!(f, "    {} {};\n", prop.0, prop.1)?;
                }
                write!(f, "}}")
            }
            Node::Identifier { value } => write!(f, "{}", value),
            Node::StructData { data } => {
                write!(f, "{{ ")?;
                for element in data {
                    write!(f, "{}, ", element)?;
                }
                write!(f, "}}")
            }
        }
    }
}

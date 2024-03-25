use crate::ast;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Datatype {
    Single {
        size: usize,
    },
    Struct {
        size: usize,
        offsets: Vec<(String, usize)>,
    },
}

impl Datatype {
    pub fn size(&self) -> usize {
        match *self {
            Datatype::Single { size } => size,
            Datatype::Struct { size, offsets: _ } => size,
        }
    }
}

pub struct VariableData {
    pub datatype: Datatype,
    pub location: usize,
}

pub struct Environment<'a> {
    pub parent: Option<&'a Environment<'a>>,
    pub top_stack: usize,
    pub variables: HashMap<String, VariableData>,
    pub datatypes: HashMap<String, Datatype>,
}

impl<'a> Environment<'a> {
    pub fn declare_var(
        &mut self,
        name: &str,
        var_data: VariableData,
    ) -> Result<(), GeneratorError> {
        if self.variables.contains_key(name) {
            return Err(GeneratorError::VariableAlreadyExists);
        }

        self.variables.insert(name.to_string(), var_data);
        Ok(())
    }

    pub fn lookup_var(&self, name: &str) -> Result<&VariableData, GeneratorError> {
        let env = self.resolve_var(name)?;
        let var = &env.variables[name];
        Ok(var)
    }

    pub fn resolve_var(&self, name: &str) -> Result<&Environment, GeneratorError> {
        if self.variables.contains_key(name) {
            return Ok(self);
        }

        match self.parent {
            Some(parent) => parent.resolve_var(name),
            None => Err(GeneratorError::VariableDoesNotExist),
        }
    }

    pub fn declare_datatype(
        &mut self,
        name: &str,
        datatype: Datatype,
    ) -> Result<(), GeneratorError> {
        if self.datatypes.contains_key(name) {
            return Err(GeneratorError::DatatypeDoesNotExist);
        }

        self.datatypes.insert(name.to_string(), datatype);
        Ok(())
    }

    pub fn lookup_datatype(&self, name: &str) -> Result<Datatype, GeneratorError> {
        let env = self.resolve_datatype(name)?;
        let datatype = env.datatypes[name].clone();
        Ok(datatype)
    }

    pub fn resolve_datatype(&self, name: &str) -> Result<&Environment, GeneratorError> {
        if self.datatypes.contains_key(name) {
            return Ok(self);
        }

        match self.parent {
            Some(parent) => parent.resolve_datatype(name),
            None => Err(GeneratorError::DatatypeDoesNotExist),
        }
    }
}

#[derive(Debug)]
pub enum GeneratorError {
    VariableAlreadyExists,
    VariableDoesNotExist,
    DatatypeAlreadyExists,
    DatatypeDoesNotExist,
    CannotAssignSingleValuetoStruct,
}

impl ast::Node {
    pub fn generate(&self, env: &mut Environment) -> Result<String, GeneratorError> {
        match self {
            ast::Node::Program { body } => {
                let mut code = format!(
                    "section .text
    global _start
_start:
    push rbp
    mov rbp, rsp
    "
                );

                for expr in body {
                    code += &expr.generate(env)?;
                }

                code = format!(
                    "{}
    push rax
    mov rax, 60
    pop rdi
    syscall
    pop rbp
    ret",
                    code
                );

                Ok(code)
            }
            ast::Node::Scope { body } => {
                let mut size = 0;
                for var in env.variables.values() {
                    size += var.datatype.size();
                }

                let mut new_env = Environment {
                    parent: Some(env),
                    variables: HashMap::new(),
                    datatypes: HashMap::new(),
                    top_stack: env.top_stack + size,
                };

                let mut code = String::from("");
                for expr in body {
                    code += &expr.generate(&mut new_env)?;
                }

                Ok(code)
            }
            ast::Node::BinOp { left, right, op: _ } => {
                let code = format!(
                    "{}
    push rax
    {}
    pop rbx
    add rax, rbx
    ",
                    left.generate(env)?,
                    right.generate(env)?
                );
                Ok(code)
            }
            ast::Node::Integer(value) => Ok(format!("mov rax, {}\n\t", value)),
            ast::Node::Float(value) => Ok(format!("mov rax, {}\n\t", value)),
            ast::Node::VarDecl {
                datatype,
                name,
                value,
            } => {
                // Return an error if the variable already exists
                if let Ok(_) = env.resolve_var(&name) {
                    return Err(GeneratorError::VariableAlreadyExists);
                }

                let datatype = env.lookup_datatype(&datatype)?;

                env.declare_var(
                    &name,
                    VariableData {
                        datatype: datatype.clone(),
                        location: env.top_stack + datatype.size(),
                    },
                )?;

                let location = env.variables.get(name).unwrap().location;
                let mut code = String::from("");
                match *value.clone() {
                    ast::Node::StructData { data } => match datatype {
                        Datatype::Single { size: _ } => {
                            return Err(GeneratorError::CannotAssignSingleValuetoStruct)
                        }
                        Datatype::Struct { size, offsets } => {
                            for i in 0..data.len() {
                                let expr = &data[i];

                                code += &format!(
                                    "{}
    mov [rbp-{}], rax
    ",
                                    expr.generate(env)?,
                                    location - size + offsets[i].1
                                );
                            }
                        }
                    },
                    _ => {
                        code = format!(
                            "{}
    mov [rbp-{}], rax
    ",
                            value.generate(env)?,
                            location
                        )
                    }
                }

                Ok(code)
            }
            ast::Node::StructDecl { name, properties } => {
                if let Ok(_) = env.lookup_datatype(&name) {
                    return Err(GeneratorError::DatatypeAlreadyExists);
                }

                let mut offsets = vec![];
                let mut offset = 0;
                for prop in properties {
                    let datatype = env.lookup_datatype(&prop.0)?;
                    let size = datatype.size();
                    offsets.push((prop.1.clone(), offset + size));
                    offset += size;
                }

                env.declare_datatype(
                    &name,
                    Datatype::Struct {
                        size: size(env, &properties)?,
                        offsets,
                    },
                )?;

                Ok(String::from(""))
            }
            ast::Node::StructType { properties: _ } => Ok(String::from("")),
            ast::Node::TypeDef { name, value } => {
                if let Ok(_) = env.lookup_datatype(name) {
                    return Err(GeneratorError::DatatypeAlreadyExists);
                }

                value.generate(env)?;
                env.declare_datatype(
                    name,
                    match *value.clone() {
                        ast::Node::StructType { properties } => {
                            let mut offsets = vec![];
                            let mut offset = 0;
                            for prop in &properties {
                                let datatype = env.lookup_datatype(&prop.0)?;
                                let size = datatype.size();
                                offsets.push((prop.1.clone(), offset + size));
                                offset += size;
                            }
                            Datatype::Struct {
                                size: size(env, &properties)?,
                                offsets,
                            }
                        }
                        ast::Node::Identifier { value } => env.lookup_datatype(&value)?,
                        _ => Datatype::Single { size: 0 },
                    },
                )?;

                Ok(String::from(""))
            }
            ast::Node::Identifier { value } => {
                let var_data = env.lookup_var(value)?;
                Ok(format!("mov rax, [rbp-{}]", var_data.location))
            }
            ast::Node::StructData { data: _ } => Ok(String::from("")),
        }
    }
}

fn size(env: &Environment, properties: &Vec<(String, String)>) -> Result<usize, GeneratorError> {
    let mut size = 0;
    for prop in properties {
        size += env.lookup_datatype(&prop.0)?.size();
    }
    Ok(size)
}

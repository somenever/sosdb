use std::{collections::HashMap, fmt::Display, fs::File, io::Write};

#[derive(Debug)]
pub struct Object {
    name: String,
    values: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct Database {
    objects: HashMap<String, Object>,
    path: String,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Str(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(value) => write!(f, "s:{value}"),
            Value::Int(value) => write!(f, "i:{value}"),
            Value::Float(value) => write!(f, "f:{value}"),
            Value::Bool(value) => write!(f, "b:{value}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValueParseError {
    InvalidType,
    InvalidValue,
    ValueIsEmpty,
}

impl TryFrom<&str> for Value {
    type Error = ValueParseError;

    fn try_from(str: &str) -> Result<Self, Self::Error> {
        let mut chars = str.chars();
        let type_hint = chars.next().ok_or(ValueParseError::ValueIsEmpty)?;
        assert_eq!(':', chars.next().ok_or(ValueParseError::InvalidValue)?);
        let value_str: String = chars.collect();
        match type_hint {
            's' => Ok(Value::Str(value_str)),
            'i' => Ok(Value::Int(
                value_str
                    .parse()
                    .map_err(|_| ValueParseError::InvalidValue)?,
            )),
            'f' => Ok(Value::Float(
                value_str
                    .parse()
                    .map_err(|_| ValueParseError::InvalidValue)?,
            )),
            'b' => Ok(Value::Bool(
                value_str
                    .parse()
                    .map_err(|_| ValueParseError::InvalidValue)?,
            )),
            _ => Err(ValueParseError::InvalidType),
        }
    }
}

impl Object {
    pub fn new(name: String) -> Self {
        Self {
            name,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn add(&mut self, name: &str, value: Value) {
        self.values.insert(name.into(), value);
    }

    pub fn delete(&mut self, name: &str) -> Option<Value> {
        self.values.remove(name)
    }

    pub fn with_value(mut self, name: &str, value: Value) -> Object {
        self.add(name, value);
        self
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "object={}", self.name)?;
        for (name, value) in &self.values {
            writeln!(f, "  {name}={value}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum DatabaseLoadError {
    IO(std::io::Error),
    ValueError(ValueParseError),
}

impl Database {
    pub fn new(path: String) -> Self {
        Self {
            objects: HashMap::new(),
            path,
        }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.insert(object.name.clone(), object);
    }

    pub fn remove_object(&mut self, name: &str) -> Option<Object> {
        self.objects.remove(name.into())
    }

    pub fn get_object(&self, name: &str) -> Option<&Object> {
        self.objects.get(name.into())
    }

    pub fn save(&self) -> std::io::Result<()> {
        let mut file = File::create(&self.path)?;
        file.write_all(self.to_string().as_bytes())?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), DatabaseLoadError> {
        let contents = std::fs::read_to_string(&self.path).map_err(|err| DatabaseLoadError::IO(err))?;
        let mut lines = contents.lines();
        lines.next();

        while let Some(line) = lines.next() {
            if let Some((_, object_name)) = line.split_once(" ") {
                let mut object = Object::new(object_name.into());
                while let Some(object_line) = lines.next() {
                    if object_line == "end" {
                        break;
                    }
                    let object_line = &object_line[2..];

                    if let Some((value_name, value_value)) = object_line.split_once("=") {
                        object.add(
                            value_name,
                            value_value
                                .try_into()
                                .map_err(|err| DatabaseLoadError::ValueError(err))?,
                        );
                    }
                }
                self.add_object(object);
            }
        }

        Ok(())
    }
}

impl Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "sosdb")?;
        for (_, obj) in &self.objects {
            writeln!(f, "{obj}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_parsing() {
        let value_str = "f:5";
        let value: Value = value_str.try_into().unwrap();
        assert_eq!(value, Value::Float(5.0));

        let value: Value = Value::Str("hello world".into());
        let value_str = &value.to_string()[..];
        assert_eq!(value_str, "s:hello world");
        let value2: Value = value_str.try_into().unwrap();
        assert_eq!(value2, value);
    }
}

#![feature(core_intrinsics)]

use std::{
    collections::HashMap,
    fs,
    ops::Index
};
use regex::Regex;
use mlua::prelude::*;

//pub mod clib; // todo!

#[derive(Clone, Debug, PartialEq)]
pub enum BluefoxDataType<'a> {
    NULL,
    BOOL(bool),
    INT(i64),
    FLOAT(f64),
    STRING(String),
    FUNCTION(String, Option<LuaFunction<'a>>),
    ARRAY(Vec<BluefoxDataType<'a>>),
    DATA(BluefoxData<'a>)
}
impl <'a> BluefoxDataType<'a> {
    fn parse(item: String) -> Result<Self, String> {
        Ok(if item == "null" {
            BluefoxDataType::NULL
        }
        else if item == "false" {
            BluefoxDataType::BOOL(false)
        }
        else if item == "true" {
            BluefoxDataType::BOOL(true)
        }
        else if let Ok(x) = item.parse::<i64>() {
            BluefoxDataType::INT(x)
        }
        else if let Ok(x) = item.parse::<f64>() {
            BluefoxDataType::FLOAT(x)
        }
        else if item.starts_with("[") && item.ends_with("]") {
            let mut output = vec![];
            let re = Regex::new(r"[\n]").unwrap();
            let elements: Vec<&str> = re.split(&item[1..item.len() - 1]).collect();
            for elem in elements {
                if elem.trim().to_owned() != "" {
                    output.push(BluefoxDataType::parse(elem.trim().to_owned())?);
                }
            }
            BluefoxDataType::ARRAY(output)
        }
        else if item.starts_with("{") && item.ends_with("}") {
            BluefoxDataType::DATA(BluefoxData::try_from(item[1..item.len() - 1].to_owned())?)
        }
        else if item.starts_with("`") && item.ends_with("`") {
            BluefoxDataType::FUNCTION(item[1..item.len() - 1].to_owned(), None)
        }
        else if item.starts_with("\"") && item.ends_with("\"") {
            BluefoxDataType::STRING(item[1..item.len() - 1].to_owned())
        }
        else {
            BluefoxDataType::STRING(item)
        })
    }

    pub fn compile<'b>(&'b mut self, lua: &'a Lua) -> LuaResult<()> {
        if let BluefoxDataType::FUNCTION(x, c) = self {
            if let None = c {
                let func = lua.load(x.clone()).into_function()?;
                *self = BluefoxDataType::FUNCTION(x.clone(), Some(func));
            }
            return Ok(());
        }
        Err(LuaError::RuntimeError(format!("\"{:?}\" is not a function", self)))
    }
}
impl <'a> FromLua<'a> for BluefoxDataType<'a> {
    fn from_lua(value: LuaValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
        Ok(match value.clone() {
            LuaValue::Nil => BluefoxDataType::NULL,
            LuaValue::Boolean(x) => BluefoxDataType::BOOL(x),
            LuaValue::Integer(x) => BluefoxDataType::INT(x),
            LuaValue::Number(x) => {
                if x.trunc() == x {
                    BluefoxDataType::INT(x.trunc() as i64)
                }
                else {
                    BluefoxDataType::FLOAT(x)
                }
            },
            LuaValue::String(x) => BluefoxDataType::STRING(x.to_str()?.to_owned()),
            LuaValue::Function(f) => BluefoxDataType::FUNCTION("nil".to_owned(), Some(f)),
            LuaValue::Table(x) => {
                let mut is_array = true;
                for e in x.clone().pairs::<String, LuaValue>() {
                    if let Ok((k, _)) = e {
                        if let Err(_) = k.parse::<usize>() {
                            is_array = false;
                            break;
                        }
                    }
                }
                if is_array {
                    let mut output = vec![];
                    for _ in x.clone().pairs::<String, LuaValue>() {
                        output.push(BluefoxDataType::NULL);
                    }
                    for e in x.pairs::<String, LuaValue>() {
                        if let Ok((k, v)) = e {
                            output[k.parse::<usize>().unwrap() - 1] = BluefoxDataType::from_lua(v, lua)?;
                        }
                    }
                    BluefoxDataType::ARRAY(output)
                }
                else {
                    BluefoxDataType::DATA(BluefoxData::from_lua(value, lua)?)
                }
            }
            _ => { return Err(LuaError::runtime(format!("not implemented for {:?}", value))); }
        })
    }
}
impl <'lua> IntoLua<'lua> for BluefoxDataType<'lua> {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(match self {
            BluefoxDataType::NULL => LuaValue::Nil,
            BluefoxDataType::BOOL(x) => LuaValue::Boolean(x),
            BluefoxDataType::INT(x) => LuaValue::Integer(x),
            BluefoxDataType::FLOAT(x) => LuaValue::Number(x),
            BluefoxDataType::STRING(x) => x.to_string().into_lua(lua)?,
            BluefoxDataType::FUNCTION(x, c) => {
                if let Some(f) = c.clone() {
                    LuaValue::Function(f)
                }
                else {
                    let func = lua.load(x.clone()).into_function()?;
                    LuaValue::Function(func)
                }
            },
            BluefoxDataType::ARRAY(x) => {
                let table = lua.create_table()?;
                for i in x {
                    table.push(i.into_lua(lua)?)?;
                }
                LuaValue::Table(table)
            }
            BluefoxDataType::DATA(x) => {
                x.into_lua(lua)?
            }
        })
    }
}
impl ToString for BluefoxDataType<'_> {
    fn to_string(&self) -> String {
        match self {
            BluefoxDataType::NULL => "null".to_owned(),
            BluefoxDataType::BOOL(x) => if *x { "true".to_owned() } else { "false".to_owned() },
            BluefoxDataType::INT(x) => x.to_string(),
            BluefoxDataType::FLOAT(x) => x.to_string(),
            BluefoxDataType::STRING(x) => "\"".to_owned() + x + "\"",
            BluefoxDataType::FUNCTION(x, _) => "`".to_owned() + x + "`",
            BluefoxDataType::ARRAY(x) => {
                let mut output = "[\n".to_owned();
                for i in x {
                    output += &(i.to_string() + "\n")[..];
                }
                output += "]";
                output
            },
            BluefoxDataType::DATA(x) => "{\n".to_owned() + &BluefoxData::to_string(x.clone()) + "\n}"
        }
    }
}
impl <'a, T> From<Vec<T>> for BluefoxDataType<'a> where T: BluefoxSerialize<'a> + Clone {
    fn from(value: Vec<T>) -> Self {
        let mut vec = vec![];
        for val in value.clone() {
            vec.push(Self::DATA(val.to_data()))
        }
        Self::ARRAY(vec)
    }
}

pub trait BluefoxSerialize<'a> {
    fn to_data(self) -> BluefoxData<'a>;
}

pub trait BluefoxDeserialize<'a>: Sized {
    fn from_data(data: BluefoxData<'a>) -> Result<Self, String>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct BluefoxData<'a> {
    pub data: HashMap<String, BluefoxDataType<'a>>
}
impl <'a> BluefoxData<'a> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    pub fn from_file(file: String) -> Result<Self, String> {
        let data_string = fs::read_to_string(file).map_err(|e| e.to_string())?;
        BluefoxData::try_from(data_string)
    }

    pub fn to_string<'b, T>(obj: T) -> String where T: BluefoxSerialize<'b> {
        let data = obj.to_data();

        let mut output = "".to_owned();
        for (key, elem) in data.data {
            output += &(key + ": " + &elem.to_string() + "\n");
        }

        output
    }

    pub fn execute<'b, A>(&'b mut self, lua: &'a Lua, func: String, args: A) -> Result<BluefoxDataType, LuaError> where A: IntoLuaMulti<'a> {
        // no need to check if function, lua will error if it isn't
        let globals = lua.globals();
        let self_table = self.clone().into_lua(lua)?;
        globals.set("notation", self_table)?;

        let re = Regex::new(r"[.\[\]]").unwrap();
        let path: Vec<&str> = re.split(&func).collect();
        let path: Vec<&str> = path.into_iter().filter(|s| !s.trim().is_empty()).collect();

        let mut target_table: LuaTable = globals.get("notation")?;
        for i in 0..path.len() - 1 {
            target_table = target_table.get(path[i])?;
        }

        let some_function: LuaFunction = if let Ok(i) = path[path.len() - 1].parse::<i64>() {
            target_table.get(i)?
        }
        else {
            target_table.get(path[path.len() - 1])?
        };

        let result = some_function.call::<A, BluefoxDataType>(args)?;

        *self = Self::from_lua(globals.get("notation")?, &lua)?;
        
        Ok(result)
    }
}
impl <'a> FromLua<'a> for BluefoxData<'a> {
    fn from_lua(value: LuaValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
        let mut output = BluefoxData::new();
        if let LuaValue::Table(x) = value {
            for e in x.pairs::<String, LuaValue>() {
                if let Ok((k, v)) = e {
                    output.data.insert(k, BluefoxDataType::from_lua(v, lua)?);
                }
            }
        }
        else {
            return Err(LuaError::runtime("Can only convert table to BluefoxData"));
        }
        Ok(output)
    }
}
impl <'a> IntoLua<'a> for BluefoxData<'a> {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let table = lua.create_table()?;
        for (k, v) in self.data {
            table.set(k, v.into_lua(lua)?)?;
        }
        Ok(LuaValue::Table(table))
    }
}
impl <'a> BluefoxSerialize<'a> for BluefoxData<'a> {
    fn to_data(self) -> BluefoxData<'a> {
        self
    }
}
impl <'a> Index<String> for BluefoxData<'a> {
    type Output = BluefoxDataType<'a>;
    fn index(&self, index: String) -> &BluefoxDataType<'a> {
        &self.data[&index]
    }
}
impl ToString for BluefoxData<'_> {
    fn to_string(&self) -> String {
        BluefoxData::to_string(self.clone())
    }
}
impl TryFrom<&str> for BluefoxData<'_> {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, String> {
        let re = Regex::new(r#"[:'`\n{}\[\]\\]"#).unwrap();
        let mut keywords: Vec<String> = vec![];

        let mut last_end = 0;
        for cap in re.captures_iter(value) {
            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();

            keywords.push(value[last_end..start].to_owned());
            keywords.push(value[start..end].to_owned());

            last_end = end;
        }
        keywords.push(value[last_end..value.len()].to_owned());

        let mut strings: Vec<String> = vec![];
        let mut accumulator = "".to_owned();
        let mut encapsulator = "".to_owned();
        for (i, item) in keywords.clone().into_iter().enumerate() {
            if item == "" {
                continue;
            }
            if (item == encapsulator || item == "}" || item == "]") && item != "{" && item != "[" && keywords[i - 1] != "\\" {
                strings.push(accumulator.clone() + &item);
                accumulator = "".to_owned();
                encapsulator = "".to_owned();
                continue;
            }
            else if encapsulator == "" && (item == "\"" || item == "'" || item == "`" || item == "{" || item == "[") {
                encapsulator = item.to_owned();
            }
            if encapsulator == "" {
                strings.push(item.to_owned());
            }
            else {
                accumulator += &item;
            }
        }

        let mut trimmed = vec![];
        for s in strings {
            let trim = s.trim();
            if trim != "" {
                trimmed.push(trim.to_owned());
            }
        }

        let mut data = HashMap::new();
        let mut current_key = "".to_owned();
        for (i, item) in trimmed.clone().into_iter().enumerate() {
            if item == ":" { continue; }
            if current_key != "" {
                data.insert(current_key, BluefoxDataType::parse(item)?);
                current_key = "".to_owned();
            }
            else if i + 1 < trimmed.len() && trimmed[i + 1] == ":" {
                current_key = item.clone();
            }
            else {
                return Err("Expected \":\" after ".to_owned() + &item);
            }
        }

        Ok(Self {
            data
        })
    }
}
impl TryFrom<String> for BluefoxData<'_> {
    type Error = String;
    fn try_from(value: String) -> Result<Self, String> {
        BluefoxData::try_from(&value[..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpreter() {
        let test = "
        is_null: null
        some_bool: false
        some_int: 4
        some_float: 6.4
        some_string: this is the, first test string
        some_quote: \"this is the, second test string\"
        some_function: `
            function some_function()
                print(\"hello world\")
            end function
        `
        some_array: [
            5
            6
            7
        ]
        some_data: {
            more_bool: true
            more_int: 9
            more_float: 1.67
        }";

        let data = BluefoxData::try_from(test).unwrap();

        assert_eq!(data.data.get("is_null").unwrap().clone(), BluefoxDataType::NULL);
        assert_eq!(data.data.get("some_bool").unwrap().clone(), BluefoxDataType::BOOL(false));
        assert_eq!(data.data.get("some_int").unwrap().clone(), BluefoxDataType::INT(4));
        assert_eq!(data.data.get("some_float").unwrap().clone(), BluefoxDataType::FLOAT(6.4));
        assert_eq!(data.data.get("some_string").unwrap().clone(), BluefoxDataType::STRING("this is the, first test string".to_owned()));
        assert_eq!(data.data.get("some_quote").unwrap().clone(), BluefoxDataType::STRING("this is the, second test string".to_owned()));
        assert_eq!(data.data.get("some_function").unwrap().clone(), BluefoxDataType::FUNCTION("
            function some_function()
                print(\"hello world\")
            end function
        ".to_owned(), None));
        assert_eq!(data.data.get("some_array").unwrap().clone(), BluefoxDataType::ARRAY(vec![BluefoxDataType::INT(5), BluefoxDataType::INT(6), BluefoxDataType::INT(7)]));
        if let BluefoxDataType::DATA(more) = data.data.get("some_data").unwrap().clone() {   
            assert_eq!(more.data.get("more_bool").unwrap().clone(), BluefoxDataType::BOOL(true));
            assert_eq!(more.data.get("more_int").unwrap().clone(), BluefoxDataType::INT(9));
            assert_eq!(more.data.get("more_float").unwrap().clone(), BluefoxDataType::FLOAT(1.67));
        }
        else {
            assert_eq!("some_data", "is not recognized as data");
        }
    }

    #[test]
    fn serializer() {
        let test = "
        is_null: null
        some_bool: false
        some_int: 4
        some_float: 6.4
        some_string: this is the, first test string
        some_quote: \"this is the, second test string\"
        some_function: `
            function some_function()
                print(\"hello world\")
            end function
        `
        some_array: [
            5
            6
            7
        ]
        some_data: {
            more_bool: true
            more_int: 9
            more_float: 1.67
        }";
        let data_intermediate = BluefoxData::try_from(test).unwrap();
        let data = BluefoxData::try_from(data_intermediate.to_string()).unwrap();

        assert_eq!(data.data.get("is_null").unwrap().clone(), BluefoxDataType::NULL);
        assert_eq!(data.data.get("some_bool").unwrap().clone(), BluefoxDataType::BOOL(false));
        assert_eq!(data.data.get("some_int").unwrap().clone(), BluefoxDataType::INT(4));
        assert_eq!(data.data.get("some_float").unwrap().clone(), BluefoxDataType::FLOAT(6.4));
        assert_eq!(data.data.get("some_string").unwrap().clone(), BluefoxDataType::STRING("this is the, first test string".to_owned()));
        assert_eq!(data.data.get("some_quote").unwrap().clone(), BluefoxDataType::STRING("this is the, second test string".to_owned()));
        assert_eq!(data.data.get("some_function").unwrap().clone(), BluefoxDataType::FUNCTION("
            function some_function()
                print(\"hello world\")
            end function
        ".to_owned(), None));
        assert_eq!(data.data.get("some_array").unwrap().clone(), BluefoxDataType::ARRAY(vec![BluefoxDataType::INT(5), BluefoxDataType::INT(6), BluefoxDataType::INT(7)]));
        if let BluefoxDataType::DATA(more) = data.data.get("some_data").unwrap().clone() {   
            assert_eq!(more.data.get("more_bool").unwrap().clone(), BluefoxDataType::BOOL(true));
            assert_eq!(more.data.get("more_int").unwrap().clone(), BluefoxDataType::INT(9));
            assert_eq!(more.data.get("more_float").unwrap().clone(), BluefoxDataType::FLOAT(1.67));
        }
        else {
            assert_eq!("some_data", "is not recognized as data");
        }
    }

    #[test]
    fn lua_implementation() {
        let test = "
        is_null: null
        some_bool: false
        some_int: 4
        some_float: 6.4
        some_string: this is the, first test string
        some_quote: \"this is the, second test string\"
        some_function: `
            notation.some_int = 8
            return notation.some_data.more_float
        `
        some_array: [
            5
            6
            7
        ]
        some_data: {
            more_bool: true
            more_int: 9
            more_float: 1.67
        }";

        let lua = Lua::new();

        let mut data = BluefoxData::try_from(test).unwrap();

        assert_eq!(data.execute(&lua, "some_function".to_owned(), ()).unwrap(), BluefoxDataType::FLOAT(1.67));
        assert_eq!(data.data["some_int"], BluefoxDataType::INT(8));

        assert_eq!(data.data.get("is_null").clone(), None); // when converted back from lua, null values are ignored
        assert_eq!(data.data.get("some_bool").unwrap().clone(), BluefoxDataType::BOOL(false));
        assert_eq!(data.data.get("some_float").unwrap().clone(), BluefoxDataType::FLOAT(6.4));
        assert_eq!(data.data.get("some_string").unwrap().clone(), BluefoxDataType::STRING("this is the, first test string".to_owned()));
        assert_eq!(data.data.get("some_quote").unwrap().clone(), BluefoxDataType::STRING("this is the, second test string".to_owned()));
        assert_eq!(data.execute(&lua, "some_function".to_owned(), ()).unwrap(), BluefoxDataType::FLOAT(1.67)); // when the function is run the function gets compiled and the source code is lost, so to check if it still works we just run it again
        assert_eq!(data.data.get("some_array").unwrap().clone(), BluefoxDataType::ARRAY(vec![BluefoxDataType::INT(5), BluefoxDataType::INT(6), BluefoxDataType::INT(7)]));
        if let BluefoxDataType::DATA(more) = data.data.get("some_data").unwrap().clone() {   
            assert_eq!(more.data.get("more_bool").unwrap().clone(), BluefoxDataType::BOOL(true));
            assert_eq!(more.data.get("more_int").unwrap().clone(), BluefoxDataType::INT(9));
            assert_eq!(more.data.get("more_float").unwrap().clone(), BluefoxDataType::FLOAT(1.67));
        }
        else {
            assert_eq!("some_data", "is not recognized as data");
        }

        drop(data);
    }

    #[test]
    fn lua_nested_function() {
        let test = r#"
        outside_function: `
            return 1
        `
        some_array: [
            `return 2`
        ]
        some_data: {
            data_function: `return 3`
        }"#;

        let lua = Lua::new();

        let mut data = BluefoxData::try_from(test).unwrap();

        assert_eq!(data.execute(&lua, "outside_function".to_owned(), ()).unwrap(), BluefoxDataType::INT(1));
        assert_eq!(data.execute(&lua, "some_array.1".to_owned(), ()).unwrap(), BluefoxDataType::INT(2));
        assert_eq!(data.execute(&lua, "some_array[1]".to_owned(), ()).unwrap(), BluefoxDataType::INT(2)); // tests both the array indexing and reusing compiled functions
        assert_eq!(data.execute(&lua, "some_data.data_function".to_owned(), ()).unwrap(), BluefoxDataType::INT(3));
    }
}

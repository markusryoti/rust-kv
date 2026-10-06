use std::{fmt::Display, str::FromStr};

use tracing::info;

#[derive(Debug)]
pub struct Command {
    pub method: Method,
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Method {
    GET,
    SET,
}

impl FromStr for Method {
    type Err = CommandErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "SET" => Ok(Method::SET),
            _ => Err(CommandErr::MethodErr),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Method::GET => "GET",
            Method::SET => "SET",
        };
        f.write_str(value)
    }
}

#[derive(Debug)]
pub enum CommandErr {
    ParseError,
    MethodErr,
}

impl std::fmt::Display for CommandErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            CommandErr::ParseError => "failed to parse",
            CommandErr::MethodErr => "invalid method",
        };
        f.write_str(str)
    }
}

impl Command {
    pub fn from(s: &str) -> Result<Self, CommandErr> {
        let s = s.trim();
        let p = s.split_whitespace();
        let values: Vec<&str> = p.into_iter().collect();

        info!(num_values = values.len(), "num of parsed values");

        let r = match values.len() {
            2 => {
                let method = values
                    .first()
                    .ok_or(CommandErr::ParseError)?
                    .parse::<Method>()?;

                if method != Method::GET {
                    return Err(CommandErr::MethodErr);
                }

                Command {
                    method,
                    key: values.get(1).ok_or(CommandErr::ParseError)?.to_string(),
                    value: None,
                }
            }
            3 => {
                let method = values
                    .first()
                    .ok_or(CommandErr::ParseError)?
                    .parse::<Method>()
                    .map_err(|_| CommandErr::ParseError)?;

                if method != Method::SET {
                    return Err(CommandErr::MethodErr);
                }

                Command {
                    method,
                    key: values.get(1).ok_or(CommandErr::ParseError)?.to_string(),
                    value: Some(values.last().ok_or(CommandErr::ParseError)?.to_string()),
                }
            }
            _ => return Err(CommandErr::ParseError),
        };

        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put() {
        let res = Command::from("set x 3");
        let cmd = res.unwrap();
        assert_eq!(cmd.method, Method::SET);
        assert_eq!(cmd.key, "x");
        assert_eq!(cmd.value, Some("3".into()));
    }

    #[test]
    fn test_get() {
        let res = Command::from("get x");
        let cmd = res.unwrap();
        assert_eq!(cmd.method, Method::GET);
        assert_eq!(cmd.key, "x");
        assert_eq!(cmd.value, None);
    }
}

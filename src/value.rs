use std::cell::RefCell;
use std::rc::Rc;

use crate::class::Class;
use crate::environment::Environment;
use crate::stmt::Stmt;
use crate::token::LiteralValue;
use crate::token::Token;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    LoxFunction {
        name: String,
        args: Vec<Token>,
        closure: Rc<RefCell<Environment>>,
        body: Rc<Vec<Stmt>>,
    },
    NativeFunction {
        name: String,
        arity: usize,
        function: Rc<dyn Fn(Vec<Value>) -> Value>,
    },
    LoxClass(Rc<Class>),
    Nil,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Str(l0), Self::Str(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::LoxFunction { .. }, Self::LoxFunction { .. }) => false,
            (Self::NativeFunction { .. }, Self::NativeFunction { .. }) => false,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::Str(arg0) => f.debug_tuple("Str").field(arg0).finish(),
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::LoxFunction {
                name, args, body, ..
            } => f
                .debug_struct("LoxFunction")
                .field("name", name)
                .field("args", args)
                .field("body", body)
                .finish(),
            Self::NativeFunction { name, arity, .. } => f
                .debug_struct("NativeFunction")
                .field("name", name)
                .field("arity", arity)
                .field("function", &"<native fn>")
                .finish(),
            Self::LoxClass(arg0) => f.debug_tuple("LoxClass").field(arg0).finish(),
            Self::Nil => write!(f, "Nil"),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(num) => {
                let s = format!("{}", num);
                let s = if s.contains('.') {
                    s.trim_end_matches('0').trim_end_matches('.')
                } else {
                    &s
                };
                write!(f, "{}", s)
            }
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::LoxFunction { name, .. } => {
                write!(f, "LoxFn {}", name)
            }
            Value::NativeFunction { name, .. } => {
                write!(f, "NativeFn {}", name)
            }
            Value::LoxClass(c) => {
                write!(f, "LoxClass {}", c.name)
            }
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl From<LiteralValue> for Value {
    fn from(value: LiteralValue) -> Self {
        match value {
            LiteralValue::Number(num) => Self::Number(num),
            LiteralValue::Str(s) => Self::Str(s),
            LiteralValue::Bool(b) => Self::Bool(b),
            LiteralValue::Nil => Self::Nil,
        }
    }
}

use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::Add;
use std::rc::Rc;

use crate::expressions::visitor::{ExpressionInterpreter, Visitor};
use crate::program::runtime::{Class, Instance, Method};
use crate::token::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum Expression {
    Expr {
        value: String,
        equality: Option<Box<Expression>>,
    },
    Equality {
        token: Token,
        value: String,

    },
    Comparison {
        token_type: TokenType,
        value: String,
    },
    GroupingExpr {
        value: Box<Expression>,
    },
    BinaryExpr {
        token: Token,
        rhs: Box<Expression>,
        lhs: Box<Expression>,
    },
    UnaryExpr {
        token: Token,
        rhs: Box<Expression>,
    },
    LiteralExpr {
        token_type: TokenType,
        value: String,
    },
    VariableExpr {
        token_type: TokenType,
        value: String,
    },
    Assignment {
        identifier: Box<Expression>,
        value: Box<Expression>,
    },
    Logical {
        token: Token,
        rhs: Box<Expression>,
        lhs: Box<Expression>,
    },
    Call {
       identifier: Box<Expression>,
       args: Vec<Box<Expression>>,
    },
    Get {
        expr: Box<Expression>,
        name: String,
    }

}

#[derive(Debug, Clone)]
pub struct ExpressionRes {
    pub type_: ExprResType,
    pub str: String,
    pub number: isize,
    pub boolean: bool,
    pub method: Option<Rc<Method>>,
    pub class: Option<Rc<Class>>,
    pub instance: Option<Rc<RefCell<Instance>>>,
}

impl ExpressionRes {
    pub fn copy(p: &ExpressionRes) -> ExpressionRes {
        ExpressionRes {
            type_: p.type_.clone(),
            str: p.str.clone(),
            number: p.number.clone(),
            boolean: p.boolean.clone(),
            method: None,
            class: None,
            instance: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ExprResType {
    String,
    Number,
    Boolean,
    Identifier,
    Function,
    Class,
    Instance,
    Nil,
}

impl ExpressionRes {
    pub fn from_str(str: String) -> ExpressionRes {
        ExpressionRes {
            type_: ExprResType::String,
            str,
            number: 0,
            boolean: false,
            method: None, class: None, instance: None,
        }
    }

    pub fn from_number(number: isize) -> ExpressionRes {
        ExpressionRes {
            type_: ExprResType::Number,
            str: String::new(),
            number,
            boolean: false,
            method: None, class: None,instance: None,
        }
    }

    pub fn from_bool(boolean: bool) -> ExpressionRes {
        ExpressionRes {
            type_: ExprResType::Boolean,
            str: String::new(),
            number: 0,
            boolean,
            method: None, class: None,instance: None,
        }
    }

    pub fn from_variable(str: String) -> ExpressionRes {
        ExpressionRes {
            type_: ExprResType::Identifier,
            str,
            number: 0,
            boolean: false,
            method: None, class: None,instance: None,
        }
    }

    pub fn from_method(method: Method) -> ExpressionRes {
        ExpressionRes {
            type_: ExprResType::Function,
            str: method.name.clone(),
            number: 0,
            boolean: false,
            method: Some(Rc::new(method)),
            class: None,
            instance: None,
        }
    }

    pub fn from_class(class: Class) -> ExpressionRes {
        ExpressionRes {
            type_: ExprResType::Class,
            str: "class ".to_string().add(&class.name.clone()),
            number: 0,
            boolean: false,
            method:  None,
            class: Some(Rc::new(class)),instance: None,
        }
    }

    pub fn from_instance(instance: Instance) -> ExpressionRes {
        ExpressionRes {
            type_: ExprResType::Instance,
            str: "instance of object".to_string(),
            number: 0,
            boolean: false,
            method:  None,
            class: None,
            instance: Some(Rc::new(RefCell::new(instance))),
        }
    }

    pub fn from_none() -> ExpressionRes {
        ExpressionRes {
            type_: ExprResType::Nil,
            str: "nil".to_string(),
            number: 0,
            boolean: false,
            method: None,
            class: None,
            instance: None,
        }
    }

    pub fn get_params_method(&self) -> Vec<String> {
        let mut args = vec![];
        for arg in &self.method.as_ref().unwrap().args {
            args.push(arg.str.clone())
        }
        args
    }

    pub fn get_method(&self) -> &Rc<Method> {
        self.method.as_ref().clone().unwrap()
    }

    pub fn eq_type(&self, other: &ExpressionRes) -> bool {
        self.type_ == other.type_
    }

    pub fn print(&self) -> String {
        match self.type_ {
            ExprResType::String => self.str.clone(),
            ExprResType::Number => self.number.to_string(),
            ExprResType::Boolean => if self.boolean { String::from("true") } else { String::from("false") },
            ExprResType::Nil => String::from("nil"),
            ExprResType::Identifier => self.str.clone(),
            ExprResType::Function => { "function :".to_string().add(&*self.str) }
            ExprResType::Class => {"class :" .to_string().add(&*self.str)}
            ExprResType::Instance => {format!("instance : {:#?}",& self.instance)}
        }
    }
}
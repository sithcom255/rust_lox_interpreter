use std::collections::LinkedList;
use log::trace;

use crate::expressions::expression::{Expression, ExpressionRes};
use crate::expressions::expression::Expression::{Assignment, BinaryExpr, Call, Get, GroupingExpr, LiteralExpr, Logical, UnaryExpr, VariableExpr};
use crate::statements::statement::Statement;
use crate::statements::statement::Statement::{BlockStatement, ClassDeclaration, ForStatement, FunStatement, IfStatement, ReturnStatement, Stmt, WhileStatement};
use crate::token::{Scanner, Token, TokenType};
use crate::token::TokenType::{And, Comma, Dot, Else, Equal, Identifier, LeftBrace, LeftParen, Or, RightBrace, RightParen, Semicolon};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    size: usize,

}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        let i = tokens.len();
        Parser {
            tokens,
            current: 0,
            size: i,
        }
    }

    pub fn program(&mut self) -> Vec<Box<Statement>> {
        let mut declarations = Vec::new();
        while self.current < self.size && self.get_current().token_type != TokenType::EOF {
            match self.declaration() {
                Some(value) => declarations.push(value),
                None => continue,
            };
        };
        declarations
    }

    pub fn declaration(&mut self) -> Option<Box<Statement>> {
        match self.get_current().token_type {
            TokenType::Var => {
                self.variable_declaration()
            }
            TokenType::Class => {
                self.class_declaration()
            }
            _ => {
                self.statement_get()
            }
        }
    }

    fn variable_declaration(&mut self) -> Option<Box<Statement>> {
        self.advance();
        let option = self.primary();
        if self.get_current().token_type == TokenType::Equal {
            self.advance();
            let expression = self.expression();
            self.consume(Semicolon, "Variable initialization without semicolon".to_string());
            Some(Box::new(Statement::VarDeclaration {
                expr: expression,
                identifier: option.unwrap(),
            }))
        } else {
            self.consume(Semicolon, "Declaration without semicolon".to_string());
            Some(Box::new(Statement::VarDeclaration {
                expr: Some(Box::new(VariableExpr { token_type: TokenType::Nil, value: "".to_string() })),
                identifier: option.unwrap(),
            }))
        }
    }

    fn class_declaration(&mut self) -> Option<Box<Statement>> {
        self.advance();
        let identifier = match self.get_current().token_type {
            TokenType::Identifier => { self.get_current().clone() }
            _ => { panic!("no identifier after fn declaration found: {:#?}", self.get_current()) }
        };
        self.advance();
        if self.peek_next(LeftBrace) {
            self.advance()
        } else {
            panic!("Did not find lef brace after identifier")
        }
        let mut functions = vec![];
        while (!self.peek_next(RightBrace)) {
            match self.function() {
                None => { break; }
                Some(value) => {
                    functions.push(value);
                }
            }
        }

        if self.peek_next(RightBrace) {
            self.advance()
        } else {
            panic!("Did not find lef brace after identifier")
        }
        Some(Box::new(ClassDeclaration { identifier, functions }))
    }

    pub fn statement_get(&mut self) -> Option<Box<Statement>> {
        match self.get_current().token_type {
            TokenType::Print => self.print_statement(),
            TokenType::If => self.if_statement(),
            TokenType::Fun => {
                self.advance();
                self.function()
            }
            TokenType::While => self.while_block(),
            TokenType::For => self.for_loop(),
            TokenType::LeftBrace => self.block(),
            TokenType::Return => self.return_stmt(),
            _ => self.expression_statement(),
        }
    }

    pub fn if_statement(&mut self) -> Option<Box<Statement>> {
        self.advance();
        self.consume(LeftParen, "Expected a brace before condition".to_string());
        let expr = *self.expression().unwrap();
        self.consume(RightParen, "Expected a brace after condition".to_string());
        let body = self.block().unwrap();
        if self.peek_next(Else) {
            self.advance();
            if self.peek_next(LeftBrace) {
                match self.block() {
                    None => { panic!(r" blcok after {{ in if statement"); }
                    Some(value) => {
                        return Some(Box::new(IfStatement { expr, body, else_body: Some(value) }));
                    }
                }
            } else {
                panic!(r"missing {{ after else ");
            }
        }
        Some(Box::new((IfStatement { expr, body, else_body: None })))
    }

    fn function(&mut self) -> Option<Box<Statement>> {
        let identifier = match self.get_current().token_type {
            TokenType::Identifier => { self.get_current().clone() }
            _ => { panic!("no identifier after fn declaration found: {:#?}", self.get_current()) }
        };
        self.advance();

        self.consume(LeftParen, "please define function (".to_string());
        let mut args = Vec::<Expression>::new();
        while !self.peek_next(RightParen) {
            args.push(*self.expression().unwrap());
            if self.peek_next(TokenType::Comma) {
                self.advance();
                if self.peek_next(RightParen) {
                    panic!("found rightparen after comman in fun declaration");
                }
            }
        }
        self.consume(RightParen, "please define function with )".to_string());
        match self.block() {
            None => { panic!("there should be block after function )") }
            Some(value) => {
                Some(Box::new(FunStatement {
                    identifier,
                    args,
                    block: Some(value),
                }))
            }
        }
    }

    fn while_block(&mut self) -> Option<Box<Statement>> {
        self.advance();
        self.consume(LeftParen, "Expected a brace before condition".to_string());
        let expr = self.expression().unwrap();
        self.consume(RightParen, "Expected a brace after condition".to_string());
        let statements = self.declaration().unwrap();
        Some(Box::new(WhileStatement { expr, body: statements }))
    }

    fn for_loop(&mut self) -> Option<Box<Statement>> {
        self.advance();
        self.consume(LeftParen, "Expected a brace before condition".to_string());
        let initiation = match self.get_current().token_type {
            TokenType::Var => {
                self.declaration()
            }
            TokenType::Semicolon => {
                None
            }
            _ => {
                self.expression_statement()
            }
        };

        let mut condition = None;
        if !self.peek_next(Semicolon) {
            condition = self.expression_statement();
        } else {
            self.consume(Semicolon, "ok".to_string());
        }

        let mut increment = None;
        if !self.peek_next(Semicolon) {
            increment = self.expression_statement();
        } else {
            self.consume(Semicolon, "ok".to_string());
        }

        self.consume(RightParen, "missing parenthesis after for loop".to_string());

        let body = self.declaration().unwrap();
        let mut else_body = None;
        if self.peek_next(Else) {
            self.advance();
            else_body = self.declaration();
        }
        Some(Box::new(ForStatement {
            initiation,
            condition,
            increment,
            body,
        }))
    }

    pub fn print_statement(&mut self) -> Option<Box<Statement>> {
        self.advance();
        let expression = self.expression();
        self.consume(Semicolon, "Print statement".to_string());

        Some(Box::new(Statement::PrintStatement { expr: expression.unwrap() }))
    }

    pub fn block(&mut self) -> Option<Box<Statement>> {
        self.advance();
        let mut list: LinkedList<Box<Statement>> = Default::default();

        while self.current < self.size && self.get_current().token_type != TokenType::RightBrace {
            match self.declaration() {
                Some(value) => list.push_back(value),
                None => continue,
            };
        };

        if self.get_current().token_type == TokenType::RightBrace {
            self.advance()
        } else {
            println!("found in block statement")
        }

        Some(Box::new(BlockStatement { statements: list }))
    }

    pub fn return_stmt(&mut self) -> Option<Box<Statement>> {
        self.advance();

        if self.get_current().token_type == TokenType::Semicolon {
            self.advance();
            return Some(Box::new(ReturnStatement { expr: None }));
        };
        let option = self.expression().unwrap();
        if self.peek_next(Semicolon) {
            self.advance()
        } else {
            panic!("there should be a semicolon at {:#?}", self.get_current())
        }
        Some(Box::new(ReturnStatement { expr: Some(option) }))
    }

    pub fn expression_statement(&mut self) -> Option<Box<Statement>> {
        match self.expression() {
            None => { None }
            Some(value) => {
                self.consume(Semicolon, "Ending of expression".to_string());
                Some(Box::new(
                    Stmt {
                        expr: value
                    }))
            }
        }
    }

    pub fn expression(&mut self) -> Option<Box<Expression>> {
        match self.assignment() {
            Some(value) => {
                Some(value)
            }
            None => { None }
        }
    }

    pub fn assignment(&mut self) -> Option<Box<Expression>> {
        let lhs = match self.logic_or() {
            None => { return None; }
            Some(value) => {
                value
            }
        };
        if self.current < self.size {
            return match self.get_current().token_type {
                TokenType::Equal => {
                    self.advance();
                    let value = self.assignment().unwrap();
                    Some(Box::new(Assignment { identifier: lhs, value }))
                }
                TokenType::Or => {
                    let token = self.get_current().clone();
                    self.advance();
                    let rhs = self.logic_or();
                    Some(Box::new(Logical {
                        token,
                        lhs,
                        rhs: rhs.unwrap_or_else(|| panic!("there should be second part of bool expr after or")),
                    }))
                }
                _ => { Some(lhs) }
            };
        }
        None
    }

    pub fn logic_or(&mut self) -> Option<Box<Expression>> {
        let mut lhs = self.logic_and();
        if self.peek_next(Or) && lhs.is_some() {
            let token = self.get_current().clone();
            self.advance();
            match self.logic_and() {
                None => { panic!("invalid logic pattern, missing second expression") }
                Some(rhs) => {
                    lhs = Some(Box::new(Logical {
                        token,
                        lhs: lhs.unwrap(),
                        rhs,
                    }))
                }
            }
        }
        lhs
    }

    pub fn logic_and(&mut self) -> Option<Box<Expression>> {
        let mut lhs = self.equality();
        if self.peek_next(And) && lhs.is_some() {
            let token = self.get_current().clone();
            self.advance();
            match self.equality() {
                None => { panic!("invalid logic pattern, missing second expression") }
                Some(rhs) => {
                    lhs = Some(Box::new(Logical {
                        token,
                        lhs: lhs.unwrap(),
                        rhs,
                    }))
                }
            }
        }
        lhs
    }

    pub fn equality(&mut self) -> Option<Box<Expression>> {
        let mut lhs = match self.comparison() {
            None => { return None; }
            Some(value) => { value }
        };
        ;

        while self.current < self.size && match self.get_current().token_type {
            TokenType::BangEqual |
            TokenType::EqualEqual => true,
            _ => false,
        } {
            let token = self.get_current().clone();
            self.advance();
            let rhs = self.comparison().unwrap();
            lhs = Box::new(BinaryExpr { token, rhs, lhs });
        }
        Some(lhs)
    }

    fn comparison(&mut self) -> Option<Box<Expression>> {
        let mut lhs = match self.term() {
            None => { return None; }
            Some(value) => { value }
        };

        while self.current < self.size && match self.get_current().token_type {
            TokenType::Greater |
            TokenType::GreaterEqual |
            TokenType::Less |
            TokenType::LessEqual => true,
            _ => false,
        } {
            let token = self.tokens[self.current].clone();
            self.advance();
            let rhs = self.term().unwrap();
            lhs = Box::new(BinaryExpr { token, rhs, lhs });
        };
        Some(lhs)
    }

    fn term(&mut self) -> Option<Box<Expression>> {
        let mut lhs = match self.factor() {
            None => { return None; }
            Some(value) => { value }
        };

        while self.current < self.size && match self.get_current().token_type {
            TokenType::Minus |
            TokenType::Plus => true,
            _ => false,
        } {
            let token = self.get_current().clone();
            self.advance();
            let rhs = self.factor().unwrap();
            lhs = Box::new(BinaryExpr { token, rhs, lhs });
        };
        Some(lhs)
    }

    fn factor(&mut self) -> Option<Box<Expression>> {
        let mut lhs = match self.unary() {
            None => { return None; }
            Some(value) => { value }
        };

        while self.current < self.size && match self.tokens[self.current].token_type {
            TokenType::Slash |
            TokenType::Star |
            TokenType::Percent => true,
            _ => false,
        } {
            let token = self.tokens[self.current].clone();
            self.advance();
            let rhs = self.unary().unwrap();
            lhs = Box::new(BinaryExpr { token, rhs, lhs });
        };
        Some(lhs)
    }

    fn unary(&mut self) -> Option<Box<Expression>> {
        while self.current < self.size && match self.tokens[self.current].token_type {
            TokenType::Bang |
            TokenType::Minus => true,
            _ => false,
        } {
            let token = self.tokens[self.current].clone();
            self.advance();
            let rhs = self.unary().unwrap();
            return Some(Box::new(UnaryExpr { token, rhs }));
        };
        return self.call();
    }


    fn call(&mut self) -> Option<Box<Expression>> {
        let mut res = self.primary();
        loop {
            if self.peek_next(LeftParen) {
                let mut args: Vec<Box<Expression>> = vec![];
                self.advance();
                match self.expression() {
                    None => {}
                    Some(value) => {
                        args.push(value);
                        for i in 0..255 {
                            if !self.peek_next(Comma) {
                                break;
                            }
                            match self.expression() {
                                None => {
                                    panic!("found comma but not the argument");
                                }
                                Some(value) => {
                                    args.push(value);
                                }
                            }
                        }
                    }
                }
                if self.peek_next(RightParen) {
                    self.advance();
                } else {
                    panic!("did not find the brace after arguments, found {:?}", self.get_current());
                }
                res = Some(Box::new(Call { identifier: res.unwrap(), args }));
            } else if self.peek_next(Dot){
                self.advance();
                if self.peek_next(Identifier) {
                    let x = self.get_current().value.clone();
                    self.advance();
                    res = Some(Box::new(Get { expr: res.unwrap(), name: x }))
                } else {
                    trace!("there should be indentifier after .")
                }

            }
            else {
                return res
            }
        }
        panic!("")
    }

    fn primary(&mut self) -> Option<Box<Expression>> {
        let primary: Expression = match self.tokens[self.current].token_type {
            TokenType::False |
            TokenType::True |
            TokenType::Nil => {
                let token = self.tokens[self.current].clone();
                self.advance();
                LiteralExpr { token_type: token.token_type, value: token.value }
            }
            TokenType::String |
            TokenType::Number => {
                let token = self.tokens[self.current].clone();
                self.advance();
                LiteralExpr { token_type: token.token_type, value: token.value }
            }
            TokenType::LeftParen => {
                self.advance();
                let expression = self.expression().unwrap();
                if self.tokens[self.current].token_type != TokenType::RightParen {
                    println!("Error, missing right brace {:?}", self.tokens[self.current])
                }
                self.advance();
                GroupingExpr { value: expression }
            }
            TokenType::Identifier => {
                let token = self.get_current().clone();
                self.advance();
                VariableExpr { token_type: token.token_type, value: token.value }
            }
            _ => {
                return None;
            }
        };
        Some(Box::new(primary))
    }

    fn get_current(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_next(&self, token: TokenType) -> bool {
        return self.current < self.size && &self.tokens[self.current].token_type == &token;
    }

    fn advance(&mut self) {
        self.current += 1;
    }
    fn consume(&mut self, token: TokenType, error: String) {
        let x = self.get_current();
        if self.current < self.size && x.token_type == token {
            self.advance();
        } else {
            println!("Token {:?} found with : {}", x, error)
        }
    }
}
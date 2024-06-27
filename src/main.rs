use std::fs::read_to_string;
use std::{fmt, io, usize};
use regex::Regex;
use std::collections::HashMap;

static binaryOps: [&str; 18] = ["+", "-", "*", "/", "%", "^", ":", "|->", "=", ":=", "==", "===", "->", "&&", "||", "..", ",", "."];
static lUnaryOps: [&str; 3] = ["~", "...", "#"];
static rUnaryOps: [&str; 4] = ["!", "!!", "++", "--"];
static types: [&str; 13] = ["i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "str", "char", "vector", "tuple", "void"];
static keywords: [&str; 3] = ["print", "input", "return"];
static LITERALS: [tokenType; 4] = [tokenType::INT, tokenType::FLOAT, tokenType::CHAR, tokenType::STRING];
static mut errorFlag: bool = false;

fn main() -> io::Result<()> {
    let s = read_to_string("/home/lisk77/coding/rust/viete/src/test.vt")?;
    //let s = "1+1".to_string();
    let mut tokenizer = Tokenizer{src: s.trim_end().to_string(), tokens: vec![], current: '\0', index: 0};//, errorFlag: false};
    tokenizer.tokenize();
    tokenizer.tokens = preParse(&mut blockify(&mut tokenizer.tokens));
    tokenizer.print();
    //let t: Token = Token{tokenType: tokenType::INT, content: "2".to_string(), start: 0, end: 0};
    //t.debug();
    
    Ok(())
}

fn tokenInVector(expression: &Vec<Token>, tokenContent: String) -> bool {
    for token in expression {
        if token.content == tokenContent {
            return true;
        }
    }
    return false;
}

#[derive(PartialEq, Clone, Debug, Copy)]
enum tokenType {
    ERROR,
    EOF,
    NEWLINE,
    BLOCK,
    INT,
    FLOAT,
    VECTOR,
    TUPLE,
    SET,
    STRING,
    CHAR,
    IDENTIFIER,
    BINARYOP,
    LUNARYOP,
    RUNARYOP,
    TYPE,
    KEYWORD,
    LPAREN,
    RPAREN,
    LBRACKET,
    RBRACKET,
    LCURLY,
    RCURLY,
}

impl fmt::Display for tokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match self {
            tokenType::ERROR => write!(f,"ERROR"),
            tokenType::EOF => write!(f, "EOF"),
            tokenType::NEWLINE => write!(f, "NEWLINE"),
            tokenType::BLOCK => write!(f, "BLOCK"),
            tokenType::INT => write!(f, "INT"),
            tokenType::FLOAT => write!(f, "FLOAT"),
            tokenType::VECTOR => write!(f, "VECTOR"),
            tokenType::TUPLE => write!(f, "TUPLE"),
            tokenType::SET => write!(f, "SET"),
            tokenType::STRING => write!(f, "STRING"),
            tokenType::CHAR => write!(f, "CHAR"),
            tokenType::IDENTIFIER => write!(f, "IDENTIFIER"),
            tokenType::BINARYOP => write!(f, "BINARYOP"),
            tokenType::LUNARYOP => write!(f, "LUNARYOP"),
            tokenType::RUNARYOP => write!(f, "RUNARYOP"),
            tokenType::TYPE => write!(f, "TYPE"),
            tokenType::KEYWORD => write!(f, "KEYWORD"),
            tokenType::LPAREN => write!(f, "LPAREN"),
            tokenType::RPAREN => write!(f, "RPAREN"),
            tokenType::LBRACKET => write!(f, "LBRACKET"),
            tokenType::RBRACKET => write!(f, "RBRACKET"),
            tokenType::LCURLY => write!(f, "LCURLY"),
            tokenType::RCURLY => write!(f, "RCURLY"),
        }
    }
}

#[derive(Clone, Debug)]
struct Token {
    tokenType: tokenType,
    content: String,
    block: Vec<Token>,
    start: u32,
    end: u32
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("[{}] {} {:?} {}..{} ", self.tokenType, self.content, self.block, self.start, self.end))
    }
}

fn checkSemantics(text: &str, pattern: &str) -> bool {
    let patterns: HashMap<&str, &str> = HashMap::from(
        [("integer", r"^(0|[1-9][0-9]*)$"),
         ("float", r"^(0|[1-9][0-9]*)(\.[0-9]+)?$"),
        ]
    );
    let pattern = Regex::new(patterns.get(pattern).unwrap()).unwrap();
    pattern.is_match(text)
}

fn checkSyntax(tokens: &Vec<Token>) -> (bool, u32) { 
    let mut i: u32 = 0;
    while i < tokens.len().try_into().unwrap() {
        let token = &tokens[i as usize].tokenType;
        
        let next = if i + 1 < tokens.len().try_into().unwrap() {
            &tokens[(i + 1) as usize].tokenType
        } else {
            &tokenType::EOF
        };

        match *token {
            tokenType::EOF => break,
            _ if LITERALS.contains(&*token) || *token == tokenType::IDENTIFIER => {
                if *next == tokenType::BINARYOP || *next == tokenType::RUNARYOP || *next == tokenType::RPAREN || *next == tokenType::LBRACKET || *next == tokenType::NEWLINE || *next == tokenType::EOF {
                    i += 1;
                    continue;
                }
                return (false, i);
            }
            tokenType::TYPE => {
                if *next == tokenType::BINARYOP || *next == tokenType::RPAREN || *next == tokenType::LBRACKET || *next == tokenType::LCURLY {
                    i += 1;
                    continue;
                }
                return (false, i);
            }
            tokenType::BINARYOP => {
                if LITERALS.contains(&*next) || *next == tokenType::IDENTIFIER || *next == tokenType::TYPE || *next == tokenType::LUNARYOP || *next == tokenType::LPAREN || *next == tokenType::LPAREN || *next == tokenType::LBRACKET || *next == tokenType::LCURLY {
                    i += 1;
                    continue;
                }
                return (false, i);
            }
            tokenType::LPAREN => {
                if LITERALS.contains(&*next) || *next == tokenType::IDENTIFIER || *next == tokenType::LUNARYOP || *next == tokenType::LPAREN || *next == tokenType::RPAREN || *next == tokenType::NEWLINE {
                    i += 1;
                    continue;
                }
                return (false, i);
            }
            tokenType::RPAREN => {
                if *next == tokenType::BINARYOP || *next == tokenType::RPAREN || *next == tokenType::LBRACKET || *next == tokenType::RBRACKET || *next == tokenType::NEWLINE || *next == tokenType::EOF {
                    i += 1;
                    continue;
                }
                return (false, i);
            }
            tokenType::LUNARYOP => {
                if LITERALS.contains(&*next) || *next == tokenType::IDENTIFIER || *next == tokenType::LPAREN {
                    i += 1;
                    continue; 
                }
                return (false, i);
            }
            tokenType::RUNARYOP => {
                if *next == tokenType::BINARYOP || *next == tokenType::RPAREN || *next == tokenType::NEWLINE || *next == tokenType::EOF {
                    i += 1;
                    continue;
                }
                return (false, i);
            }
            _ => {i += 1;}
        }
    }
    return (true, i);
}

struct Tokenizer {
    src: String,
    tokens: Vec<Token>,
    current: char,
    index: u32,
    //errorFlag: bool
}

impl Tokenizer {
    
    fn print(&self) {
        for token in &self.tokens {
            print!("{}", token);
        }
    }

    fn forward(&mut self) {
        self.index += 1;
        if self.index < self.src.chars().count() as u32 {
            self.current = self.src.chars().nth(self.index as usize).unwrap();
        }
        else {
            self.current = '\0';
        }
    }

    fn peek(&mut self) -> char {
        if self.index+1 < self.src.chars().count() as u32 {
            return self.src.chars().nth((self.index+1) as usize).unwrap();
        }
        unsafe{errorFlag = true};
        return '\0'
    }

    fn makeNumber(&mut self) -> Token {
        let mut number = String::new();
        number.push(self.current);
        let start: u32 = self.index;

        self.forward();
        while "0123456789._".contains(self.current) {
            if self.current == '.' && self.peek() == '.' {
                break;
            }
            if self.current == '_' {
                self.forward();
                continue;
            }

            number.push(self.current);
            self.forward();
        }
        
        if checkSemantics(&number, "integer") {
            return Token{tokenType: tokenType::INT, content: number, block: vec![], start: start, end: self.index-1};
        }
        else if checkSemantics(&number, "float") {
            return Token{tokenType: tokenType::FLOAT, content: number, block: vec![], start: start, end: self.index-1};
        }
    
        unsafe{errorFlag = true};
        return Token{tokenType: tokenType::ERROR, content: number, block: vec![], start: start, end: self.index-1};

    }

    fn makeIdentifier(&mut self) -> Token {
        let mut identifier = String::new();
        identifier.push(self.current);
        let start: u32 = self.index;

        self.forward();
        while self.current.is_alphanumeric() || self.current == '_' {
            identifier.push(self.current);
            self.forward();
        }

        if types.contains(&identifier.as_str()) {
            return Token{tokenType: tokenType::TYPE, content: identifier, block: vec![], start: start, end: self.index-1};
        }

        if keywords.contains(&identifier.as_str()) {
            return Token{tokenType: tokenType::KEYWORD, content: identifier, block: vec![], start: start, end: self.index-1};
        }

        return Token{tokenType: tokenType::IDENTIFIER, content: identifier, block: vec![], start: start, end: self.index-1};
    }

    fn makeOperator(&mut self) -> Token {
        let mut operator = String::new();
        operator.push(self.current);
        let start: u32 = self.index;

        self.forward();
        while "+-*/%&|.,>=<!:;~?$#".contains(self.current) {
            operator.push(self.current);
            self.forward();
        }

        if binaryOps.contains(&operator.as_str()) {
            return Token{tokenType: tokenType::BINARYOP, content: operator, block: vec![], start: start, end: self.index-1};
        }
        else if lUnaryOps.contains(&operator.as_str()) {
            return Token{tokenType: tokenType::LUNARYOP, content: operator, block: vec![], start: start, end: self.index-1};
        }
        else if rUnaryOps.contains(&operator.as_str()) {
            return Token{tokenType: tokenType::RUNARYOP, content: operator, block: vec![], start: start, end: self.index-1};
        }

        unsafe{errorFlag = true};
        Token{tokenType: tokenType::ERROR, content: operator, block: vec![], start: start, end: self.index-1}
    }

    fn makeString(&mut self) -> Token {
        let mut string = String::new();
        string.push(self.current);
        let start = self.index;
        let mut counter = 1;

        self.forward();
        while counter != 0 {
            if self.current == '\"' {
                counter -= 1
            }
            string.push(self.current);
            self.forward();
        }

        return Token{tokenType: tokenType::STRING, content: string, block: vec![], start: start, end: self.index-1};
    }

    fn makeChar(&mut self) -> Token {
        let mut chr = String::new();
        chr.push(self.current);
        let start = self.index;
        
        self.forward();
        chr.push(self.current);
        if self.peek() == '\'' {
            self.forward();
            return Token{tokenType: tokenType::CHAR, content: chr, block: vec![], start: start, end: self.index};
        }
        unsafe{errorFlag = true};
        return Token{tokenType: tokenType::ERROR, content: chr, block: vec![], start: start, end: self.index};
    }

    fn tokenize(&mut self) {
        if self.src.len() == 0 {
            return
        }
        self.current = self.src.chars().nth(self.index as usize).unwrap();
        while !(self.current == '\0') || unsafe { errorFlag == true } {
            match self.current {
                _ if self.current.is_numeric() => {
                    let number = self.makeNumber();
                    self.tokens.push(number);
                }
                _ if self.current.is_alphabetic() || self.current == '_' => {
                    let identifier = self.makeIdentifier();
                    self.tokens.push(identifier);
                }
                _ if "+-*/%&|.,>=<!:;~?$#".contains(self.current) => {
                    let operator = self.makeOperator();
                    self.tokens.push(operator);
                }
                '\"' => {
                    let string = self.makeString();
                    self.tokens.push(string);
                }
                '\'' => {
                    let chr = self.makeChar();
                    self.tokens.push(chr);
                    self.forward();
                }
                '(' => {
                    self.tokens.push(Token{tokenType: tokenType::LPAREN, content: "(".to_string(), block: vec![], start: self.index, end: self.index});
                    self.forward();
                }
                ')' => {
                    self.tokens.push(Token{tokenType: tokenType::RPAREN, content: ")".to_string(), block: vec![], start: self.index, end: self.index});
                    self.forward();
                }
                '[' => {
                    self.tokens.push(Token{tokenType: tokenType::LBRACKET, content: "[".to_string(), block: vec![], start: self.index, end: self.index});
                    self.forward();
                }
                ']' => {
                    self.tokens.push(Token{tokenType: tokenType::RBRACKET, content: "]".to_string(), block: vec![], start: self.index, end: self.index});
                    self.forward();
                }
                '{' => {
                    self.tokens.push(Token{tokenType: tokenType::LCURLY, content: "{".to_string(), block: vec![], start: self.index, end: self.index});
                    self.forward();
                }
                '}' => {
                    self.tokens.push(Token{tokenType: tokenType::RCURLY, content: "}".to_string(), block: vec![], start: self.index, end: self.index});
                    self.forward();
                }
                '\n' => {
                    self.tokens.push(Token{tokenType: tokenType::NEWLINE, content: "".to_string(), block: vec![], start: self.index, end: self.index});
                    self.forward();
                }
                _ => self.forward()
            }
        }
        self.tokens.push(Token{tokenType: tokenType::EOF, content: "\0".to_string(), block: vec![], start: self.index, end: self.index});
        let (correctSyntax, errorIndex) = checkSyntax(&self.tokens);
        
        if !correctSyntax {
            unsafe {
                errorFlag = true;
            }    
            println!("SYNTAX ERROR: Fault @ token number {}", errorIndex);
        }
    }
}

fn getExpression(entryPoint: u32, expression: Vec<Token>) -> (bool, (Token, u32)) {
    let mut exprStack: Vec<Token> = vec![];
    let mut i: u32 = entryPoint+1;
    let exprStart: u32 = expression[entryPoint as usize].start;
    let mut exprEnd: u32 = 0;
    let mut open: u32 = 1;

    while i < expression.len().try_into().unwrap() {
        let curr = &expression[i as usize];
        if open > 0 && curr.tokenType == tokenType::LPAREN {
            open += 1;
            exprStack.push(curr.clone());
            i += 1;
            continue;
        }
        else if open > 0 && curr.tokenType == tokenType::RPAREN {
            open -= 1;
            if open != 0 {
                exprStack.push(curr.clone());
                i += 1;
                continue;
            }
            else {
                exprEnd = curr.end;
                return (true, (Token{tokenType: tokenType::BLOCK, content: "".to_string(), block: blockify(&mut exprStack), start: exprStart, end: exprEnd}, i)); 
            }
        }
        else {
            exprStack.push(curr.clone());
            i += 1;
        }
    }
    return (false, (Token{tokenType: tokenType::BLOCK, content: "".to_string(), block: vec![], start: 0, end: 0},0));
} 

fn blockify(expression: &mut Vec<Token>) -> Vec<Token> { 
    let mut i: u32 = 0;

    if unsafe{errorFlag} == false {
        while i < expression.len().try_into().unwrap() {
            let curr = &expression[i as usize];
            if curr.tokenType == tokenType::LPAREN {
                let result = getExpression(i, expression.to_vec());
                if result.0 {
                    expression[i as usize] = (result.1).0;
                    for _ in 0..(result.1).1-i {
                        expression.remove((i+1) as usize);
                    }
                }
            }
            i += 1;
        }
        return expression.clone();
    }
    vec![]
}

fn getIterator(expression: Vec<Token>) -> (Token, u32) {
    let mut i: u32 = 0;
    let start = expression[i as usize].start;
    let mut iterator = vec![];

    while expression[i as usize].tokenType != tokenType::RBRACKET {
        iterator.push(expression[i as usize].clone());
        i += 1;
    }

    let end = expression[i as usize].end;
    iterator.push(expression[i as usize].clone());

    (Token{tokenType: tokenType::VECTOR, content: "".to_string(), block: iterator, start: start, end: end},i)
}

fn preParse(expression: &mut Vec<Token>) -> Vec<Token> {
     let mut i: u32 = 0;

     if unsafe {errorFlag} == false {
        while i < expression.len().try_into().unwrap() {
            let mut curr = expression[i as usize].clone();
            match curr.tokenType {
                tokenType::BLOCK => {
                    if tokenInVector(&curr.block, ",".to_string()) { 
                        curr.tokenType = tokenType::TUPLE;
                    }
                }
                tokenType::LBRACKET => {
                    let (iterator, endPoint) = getIterator(expression[i as usize..].to_vec());
                    curr = iterator;
                    for k in i+1..endPoint {
                        expression.remove(k as usize);
                    };
                }
                _ => {}
            } 
            expression[i as usize] = curr;
            i += 1;
        }
     }
     expression.to_vec()
}

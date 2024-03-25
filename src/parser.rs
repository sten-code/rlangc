use crate::ast;
use crate::lexer;

#[derive(Debug)]
pub enum ParseError {
    InvalidToken,
    ExpectedToken(lexer::TokenType),
}

fn expect(
    tokens: &mut Vec<lexer::Token>,
    token_type: lexer::TokenType,
) -> Result<lexer::Token, ParseError> {
    let token = tokens.pop().unwrap();
    if token.token_type != token_type {
        Err(ParseError::ExpectedToken(token_type))
    } else {
        Ok(token)
    }
}

pub fn parse(mut tokens: Vec<lexer::Token>) -> Result<ast::Node, ParseError> {
    // Reversing so we can pop from the end instead of the beginning which is faster
    tokens.reverse();

    let mut body = vec![];
    loop {
        let ast = parse_stmt(&mut tokens)?;
        body.push(ast);
        if tokens.len() == 0 {
            break;
        }
    }

    Ok(ast::Node::Program { body })
}

fn parse_stmt(tokens: &mut Vec<lexer::Token>) -> Result<ast::Node, ParseError> {
    let ast: ast::Node = match tokens.last().unwrap().token_type {
        lexer::TokenType::Identifier => parse_var_decl(tokens)?,
        lexer::TokenType::OpenBrace => parse_scope(tokens)?,
        lexer::TokenType::TypeDef => parse_typedef(tokens)?,
        lexer::TokenType::Struct => parse_type(tokens)?,
        _ => parse_expr(tokens)?,
    };

    expect(tokens, lexer::TokenType::Semicolon)?;

    Ok(ast)
}

fn parse_expr(tokens: &mut Vec<lexer::Token>) -> Result<ast::Node, ParseError> {
    let mut left = parse_primary(tokens)?;
    while tokens.len() > 0 && tokens.last().unwrap().token_type == lexer::TokenType::Add {
        tokens.pop().unwrap();
        let right = parse_primary(tokens)?;
        left = ast::Node::BinOp {
            left: Box::new(left),
            right: Box::new(right),
            op: ast::Operator::Add,
        };
    }

    Ok(left)
}

fn parse_var_decl(tokens: &mut Vec<lexer::Token>) -> Result<ast::Node, ParseError> {
    let var_type = &tokens.pop().unwrap().value;

    let var_name = expect(tokens, lexer::TokenType::Identifier)?.value;

    expect(tokens, lexer::TokenType::Equals)?;

    let ast = parse_expr(tokens)?;

    Ok(ast::Node::VarDecl {
        datatype: var_type.to_string(),
        name: var_name,
        value: Box::new(ast),
    })
}

fn parse_scope(tokens: &mut Vec<lexer::Token>) -> Result<ast::Node, ParseError> {
    if tokens.last().unwrap().token_type != lexer::TokenType::OpenBrace {
        return Err(ParseError::InvalidToken);
    }
    tokens.pop().unwrap();

    let mut body = vec![];
    loop {
        let ast = parse_stmt(tokens).unwrap();
        body.push(ast);
        if tokens.last().unwrap().token_type == lexer::TokenType::CloseBrace {
            tokens.pop().unwrap();
            break;
        }
    }

    Ok(ast::Node::Scope { body })
}

fn parse_typedef(tokens: &mut Vec<lexer::Token>) -> Result<ast::Node, ParseError> {
    // example: typedef struct { int x; int y; } vec2_t
    expect(tokens, lexer::TokenType::TypeDef)?;

    let ast = parse_type(tokens)?;
    let name = expect(tokens, lexer::TokenType::Identifier)?.value;

    Ok(ast::Node::TypeDef {
        name,
        value: Box::new(ast),
    })
}

fn parse_type(tokens: &mut Vec<lexer::Token>) -> Result<ast::Node, ParseError> {
    let ast = match tokens.pop().unwrap().token_type {
        lexer::TokenType::Struct => {
            let ast: ast::Node = match tokens.last().unwrap().token_type {
                lexer::TokenType::OpenBrace => {
                    // example: struct { int x; int y; }
                    expect(tokens, lexer::TokenType::OpenBrace)?;

                    let mut properties = vec![];
                    loop {
                        let datatype = expect(tokens, lexer::TokenType::Identifier)?.value;
                        let name = expect(tokens, lexer::TokenType::Identifier)?.value;
                        expect(tokens, lexer::TokenType::Semicolon)?;
                        properties.push((datatype, name));
                        if tokens.last().unwrap().token_type == lexer::TokenType::CloseBrace {
                            break;
                        }
                    }

                    ast::Node::StructType { properties }
                }
                lexer::TokenType::Identifier => {
                    // example: struct vec2 { int x; int y; }
                    let name = expect(tokens, lexer::TokenType::Identifier)?.value;
                    expect(tokens, lexer::TokenType::OpenBrace)?;

                    let mut properties = vec![];
                    loop {
                        let datatype = expect(tokens, lexer::TokenType::Identifier)?.value;
                        let name = expect(tokens, lexer::TokenType::Identifier)?.value;
                        expect(tokens, lexer::TokenType::Semicolon)?;
                        properties.push((datatype, name));
                        if tokens.last().unwrap().token_type == lexer::TokenType::CloseBrace {
                            break;
                        }
                    }

                    ast::Node::StructDecl { name, properties }
                }
                _ => return Err(ParseError::InvalidToken),
            };

            expect(tokens, lexer::TokenType::CloseBrace)?;
            ast
        }
        _ => return Err(ParseError::InvalidToken),
    };
    Ok(ast)
}

fn parse_primary(tokens: &mut Vec<lexer::Token>) -> Result<ast::Node, ParseError> {
    let token = tokens.pop().unwrap();
    let ast = match token.token_type {
        lexer::TokenType::Integer => ast::Node::Integer(token.value.parse().unwrap()),
        lexer::TokenType::Float => ast::Node::Float(token.value.parse().unwrap()),
        lexer::TokenType::Identifier => ast::Node::Identifier { value: token.value },
        lexer::TokenType::OpenBrace => {
            let mut data = vec![];
            loop {
                let node = parse_expr(tokens)?;
                data.push(node);
                let token_type = &tokens.last().unwrap().token_type;
                if *token_type == lexer::TokenType::CloseBrace {
                    tokens.pop().unwrap();
                    break;
                }
                if *token_type == lexer::TokenType::Comma {
                    tokens.pop().unwrap();
                }
            }

            ast::Node::StructData { data }
        }
        _ => return Err(ParseError::InvalidToken),
    };
    Ok(ast)
}

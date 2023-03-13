use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt::Write;

static SYMBOLS: Lazy<Vec<char>> = Lazy::new(|| vec!['=', '!', '<', '>', ':']);
static OPERATORS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut operators = HashMap::new();
    // operators.insert("or", "OR");
    // operators.insert("and", "AND");
    operators.insert("=", "=");
    operators.insert("eq", "=");
    operators.insert("!=", "<>");
    operators.insert("<>", "<>");
    operators.insert("neq", "<>");
    operators.insert("is", "IS");
    operators.insert("nis", "IS NOT");
    operators.insert("in", "IN");
    operators.insert("nin", "NOT IN");
    operators.insert("like", "LIKE");
    operators.insert("nlike", "NOT LIKE");
    operators.insert("ilike", "ILIKE");
    operators.insert("nilike", "NOT ILIKE");
    operators.insert("<", "<");
    operators.insert("lt", "<");
    operators.insert(">", ">");
    operators.insert("gt", "gt");
    operators.insert("<=", "<=");
    operators.insert("lte", "<=");
    operators.insert(">=", ">=");
    operators.insert("gte", ">=");
    operators
});
pub struct Parser {
    pub offset: usize,
    pub raw: String,
    pub chars: Vec<char>,
    pub idents: Vec<String>,
    pub joined_options: Vec<JoinedOption>,
}
#[derive(Clone, Debug)]
pub struct JoinedOption {
    pub outer_table: String,
    pub outer_key: String,
    pub inner_key: String,
    pub url_name_map: HashMap<String, String>,
}
impl Parser {
    pub fn new(raw: String, idents: Vec<String>, options: Vec<JoinedOption>) -> Parser {
        Parser {
            offset: 0,
            chars: raw.chars().collect(),
            raw,
            idents,
            joined_options: options,
        }
    }
    pub fn parse(&mut self) -> Result<String, String> {
        if self.raw.is_empty() {
            return Ok("".into());
        }
        Ok(self.scan_expr()?.trim().to_string())
    }
    fn next(&mut self, skip_blank: bool) -> Option<char> {
        self.offset += 1;
        if self.offset < self.chars.len() {
            if skip_blank {
                self.skip_blank();
            }
            Some(self.chars[self.offset])
        } else {
            None
        }
    }
    fn try_next(&mut self, skip_blank: bool) -> Result<char, String> {
        self.next(skip_blank)
            .ok_or(format!("unexcepted end 1, offset: {}, raw: {}", self.offset, &self.raw))
    }
    fn skip_blank(&mut self) {
        let ch = self.curr();
        if ch.is_none() {
            return;
        }
        let mut ch = ch.unwrap();
        while ch == ' ' || ch == '\t' {
            if self.offset < self.chars.len() - 1 {
                self.offset += 1;
                ch = self.chars[self.offset];
            } else {
                break;
            }
        }
    }
    fn peek(&self) -> Option<&char> {
        if self.offset < self.chars.len() - 1 {
            self.chars.get(self.offset + 1)
        } else {
            None
        }
    }
    fn peek_token(&self) -> Option<String> {
        let mut tok = "".to_owned();
        let mut offset = self.offset;
        while offset < self.chars.len() {
            let ch = self.chars[offset];
            if ch != ' ' && ch != '\t' {
                tok.push(ch);
            } else {
                break;
            }
            offset += 1;
        }
        if tok.is_empty() {
            None
        } else {
            // println!("=============toke: {:?}", &tok);
            Some(tok)
        }
    }
    fn curr(&self) -> Option<char> {
        if self.offset < self.chars.len() {
            Some(self.chars[self.offset])
        } else {
            None
        }
    }

    pub fn scan_expr(&mut self) -> Result<String, String> {
        let mut expr: String = "".into();
        self.skip_blank();
        if let Some(ch) = self.curr() {
            if ch == '(' {
                expr.push('(');
                self.offset += 1;
                expr.push_str(&self.scan_expr()?);
                self.skip_blank();
                if self.curr().unwrap_or_default() == ')' {
                    expr.push(')');
                    self.offset += 1;
                } else {
                    return Err(format!("excepted ')', offset: {}, raw: {}", self.offset, &self.raw));
                }
            } else {
                let left = self.scan_ident()?;
                // println!("==============left: {:#?}", &left);
                let operator = self.scan_operator()?;
                let right = self.scan_value()?;
                // println!("==============right: {:#?}", &right);
                if left.contains('.') {
                    if let Some(option) = self.find_option(&left) {
                        write!(
                            expr,
                            "{} in (select {} from {} where {} {} {})",
                            option.inner_key,
                            option.outer_key,
                            option.outer_table,
                            option.url_name_map[&left],
                            &operator,
                            right
                        )
                        .unwrap();
                    } else {
                        return Err(format!("ident is not correct, raw: {}", &self.raw));
                    }
                } else {
                    expr.push_str(&left);
                    expr.push(' ');
                    expr.push_str(&operator);
                    expr.push(' ');
                    expr.push_str(&right);
                }
            }
            self.skip_blank();
            if let Some(tok) = self.peek_token() {
                if tok == "and" || tok == "or" {
                    expr.push(' ');
                    expr.push_str(&tok);
                    expr.push(' ');
                    if tok == "and" {
                        self.offset += 4;
                    } else {
                        self.offset += 3;
                    }
                    expr.push_str(&self.scan_expr()?);
                }
            }
        }
        Ok(expr)
    }

    fn find_option(&self, url_name: &str) -> Option<&JoinedOption> {
        for option in &self.joined_options {
            if option.url_name_map.get(url_name).is_some() {
                return Some(option);
            }
        }
        None
    }

    fn scan_ident(&mut self) -> Result<String, String> {
        self.skip_blank();
        let mut ident = "".to_owned();
        let mut ch = self.curr().unwrap();
        while !ch.is_whitespace() && !SYMBOLS.contains(&ch) {
            ident.push(ch);
            if let Some(c) = self.next(false) {
                ch = c;
            } else {
                break;
            }
        }
        if ident.contains('.') {
            match self.find_option(&ident) {
                Some(_) => Ok(ident),
                None => Err(format!(
                    "ident is not allowed 0, offset:{}, raw: {}",
                    self.offset, &self.raw
                )),
            }
        } else if self.idents.contains(&ident) {
            if let Some(':') = self.curr() {
                if let Some(':') = self.peek() {
                    self.next(false);
                    ident.push_str("::");
                    let mut ch = self.try_next(false)?;
                    while !ch.is_whitespace() && !SYMBOLS.contains(&ch) {
                        ident.push(ch);
                        if let Some(c) = self.next(false) {
                            ch = c;
                        } else {
                            break;
                        }
                    }
                } else {
                    return Err(format!(
                        "':' is not allowed here, offset:{}, raw: {}, idents: {:#?}, joined_options: {:#?}, ident: {}",
                        self.offset, &self.raw, &self.idents, &self.joined_options, &ident
                    ));
                }
            }
            Ok(ident)
        } else {
            Err(format!(
                "ident is not allowed 1, offset:{}, raw: {}, idents: {:#?}, joined_options: {:#?}, ident: {}",
                self.offset, &self.raw, &self.idents, &self.joined_options, &ident
            ))
        }
    }
    fn scan_value(&mut self) -> Result<String, String> {
        self.skip_blank();
        let mut value = "".to_owned();
        let mut ch = self.curr().unwrap();
        if ch == '\'' || ch == 'E' {
            if ch == 'E' {
                value.push(ch);
                ch = self.try_next(false)?;
                if ch != '\'' {
                    return Err("except ' after E".to_owned());
                }
            }
            value.push(ch);
            ch = self.try_next(false)?;
            loop {
                value.push(ch);
                if ch == '\\' {
                    value.push(self.try_next(false)?);
                    ch = self.try_next(false)?;
                } else if ch == '\'' {
                    if let Some('\'') = self.peek() {
                        value.push(self.try_next(false)?);
                        ch = self.try_next(false)?;
                    } else {
                        self.next(false);
                        break;
                    }
                } else {
                    ch = self.try_next(false)?;
                }
            }
        } else {
            while !ch.is_whitespace() && ch != ')' && ch != '(' {
                value.push(ch);
                if let Some(c) = self.next(false) {
                    ch = c;
                } else {
                    break;
                }
            }
        }
        Ok(value)
    }

    fn scan_operator(&mut self) -> Result<String, String> {
        self.skip_blank();
        let mut url_opt = "".to_owned();
        let mut ch = self.curr().unwrap();
        let is_symbol = SYMBOLS.contains(&ch);
        if is_symbol {
            while SYMBOLS.contains(&ch) {
                url_opt.push(ch);
                if let Some(c) = self.next(false) {
                    ch = c;
                } else {
                    break;
                }
            }
        } else {
            while !ch.is_whitespace() {
                url_opt.push(ch);
                if let Some(c) = self.next(false) {
                    ch = c;
                } else {
                    break;
                }
            }
        }
        OPERATORS.get(&&*url_opt).cloned().map(|s| s.to_owned()).ok_or(format!(
            "operator is not correct, raw: {}, url_opt:{}, operators: {:#?}",
            &self.raw, &url_opt, &*OPERATORS
        ))
    }
}

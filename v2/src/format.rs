use crate::document::*;
use std::collections::BTreeMap;
use std::fmt;

pub const MAX_INPUT: usize = 65_536;
pub const MAX_OUTPUT: usize = 524_288;
const MAX_TOKENS: usize = 16_384;
const MAX_DEPTH: usize = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Indentation {
    Tabs,
    Spaces2,
    Spaces4,
    Spaces8,
}
impl Indentation {
    pub fn from_value(value: &str) -> Result<Self> {
        match value {
            "tab" => Ok(Self::Tabs),
            "2" => Ok(Self::Spaces2),
            "4" => Ok(Self::Spaces4),
            "8" => Ok(Self::Spaces8),
            _ => Err(FormatError("Invalid indentation")),
        }
    }
    pub fn value(self) -> &'static str {
        match self {
            Self::Tabs => "tab",
            Self::Spaces2 => "2",
            Self::Spaces4 => "4",
            Self::Spaces8 => "8",
        }
    }
    fn width(self) -> usize {
        match self {
            Self::Spaces4 => 4,
            Self::Spaces8 => 8,
            _ => 2,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError(pub &'static str);
impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for FormatError {}
type Result<T> = std::result::Result<T, FormatError>;
#[derive(Clone, Debug)]
struct Token {
    value: String,
    start: usize,
    end: usize,
    string: bool,
    block: bool,
}
#[derive(Clone, Debug)]
struct Comment {
    value: String,
    start: usize,
    end: usize,
}
#[derive(Clone, Debug)]
struct Node {
    kind: &'static str,
    value: String,
    fields: BTreeMap<&'static str, Vec<Node>>,
    start: usize,
    end: usize,
    block: bool,
    leading: Vec<Comment>,
    trailing: Vec<Comment>,
    dangling: Vec<Comment>,
}
impl Node {
    fn new(kind: &'static str, start: usize) -> Self {
        Self {
            kind,
            value: String::new(),
            fields: BTreeMap::new(),
            start,
            end: start,
            block: false,
            leading: vec![],
            trailing: vec![],
            dangling: vec![],
        }
    }
    fn set(&mut self, key: &'static str, child: Node) {
        self.fields.insert(key, vec![child]);
    }
    fn get(&self, key: &str) -> Option<&Node> {
        self.fields.get(key)?.first()
    }
    fn list(&self, key: &str) -> &[Node] {
        self.fields.get(key).map(Vec::as_slice).unwrap_or(&[])
    }
}
fn lex(input: &str) -> Result<(Vec<Token>, Vec<Comment>)> {
    let mut tokens = Vec::new();
    let mut comments = Vec::new();
    let mut i = 0;
    let b = input.as_bytes();
    let mut depth = 0;
    while i < b.len() {
        let start = i;
        if b[i].is_ascii_whitespace() || b[i] == b',' {
            i += 1;
            continue;
        }
        if input[i..].starts_with('\u{feff}') {
            i += 3;
            continue;
        }
        if b[i] == b'#' {
            while i < b.len() && !b"\r\n".contains(&b[i]) {
                i += 1;
            }
            if comments.len() >= 512 {
                return Err(FormatError("GraphQL comment limit exceeded"));
            }
            comments.push(Comment {
                value: input[start..i].trim_end().into(),
                start,
                end: i,
            });
            continue;
        }
        let mut string = false;
        let mut block = false;
        if input[i..].starts_with("\"\"\"") {
            string = true;
            block = true;
            i += 3;
            let content = i;
            loop {
                if i >= b.len() {
                    return Err(FormatError("Invalid GraphQL"));
                }
                if input[i..].starts_with("\\\"\"\"") {
                    i += 4;
                } else if input[i..].starts_with("\"\"\"") {
                    break;
                } else {
                    i += input[i..].chars().next().unwrap().len_utf8();
                }
            }
            if input[content..i]
                .bytes()
                .any(|b| b < 0x20 && !b"\t\n\r".contains(&b))
            {
                return Err(FormatError("Invalid GraphQL string"));
            }
            let value = block_value(&input[content..i]);
            i += 3;
            tokens.push(Token {
                value,
                start,
                end: i,
                string,
                block,
            });
        } else if b[i] == b'"' {
            string = true;
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 2;
                } else if b[i] == b'"' {
                    break;
                } else {
                    i += 1;
                }
            }
            if i >= b.len() {
                return Err(FormatError("Invalid GraphQL"));
            }
            i += 1;
            let value = serde_json::from_str(&input[start..i].replace('\t', "\\t"))
                .map_err(|_| FormatError("Invalid GraphQL string"))?;
            tokens.push(Token {
                value,
                start,
                end: i,
                string,
                block,
            });
        } else {
            if b[i].is_ascii_alphabetic() || b[i] == b'_' {
                i += 1;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
            } else if b[i].is_ascii_digit() || b[i] == b'-' {
                i += 1;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b".+-".contains(&b[i])) {
                    i += 1;
                }
            } else if input[i..].starts_with("...") {
                i += 3;
            } else {
                if b"([{ ".contains(&b[i]) {
                    depth += 1;
                    if depth > MAX_DEPTH {
                        return Err(FormatError("GraphQL nesting limit exceeded"));
                    }
                } else if b")] }".contains(&b[i]) {
                    depth = depth.saturating_sub(1);
                }
                i += input[i..].chars().next().unwrap().len_utf8();
            }
            tokens.push(Token {
                value: input[start..i].into(),
                start,
                end: i,
                string,
                block,
            });
        }
        if tokens.len() + comments.len() > MAX_TOKENS {
            return Err(FormatError("GraphQL token limit exceeded"));
        }
    }
    Ok((tokens, comments))
}
fn block_value(s: &str) -> String {
    let s = s
        .replace("\\\"\"\"", "\"\"\"")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let mut lines: Vec<String> = s.split('\n').map(str::to_owned).collect();
    let common = lines
        .iter()
        .skip(1)
        .filter(|s| !s.trim_matches([' ', '\t']).is_empty())
        .map(|s| s.bytes().take_while(|b| b" \t".contains(b)).count())
        .min()
        .unwrap_or(0);
    for s in lines.iter_mut().skip(1) {
        let n = s.len().min(common);
        s.drain(..n);
    }
    let first = lines
        .iter()
        .take_while(|s| s.trim_matches([' ', '\t']).is_empty())
        .count();
    lines.drain(..first);
    while lines
        .last()
        .is_some_and(|s| s.trim_matches([' ', '\t']).is_empty())
    {
        lines.pop();
    }
    lines.join("\n")
}
struct Parse {
    tokens: Vec<Token>,
    pos: usize,
}
impl Parse {
    fn is(&self, s: &str) -> bool {
        self.tokens
            .get(self.pos)
            .is_some_and(|t| !t.string && t.value == s)
    }
    fn take(&mut self, s: &str) -> bool {
        if self.is(s) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, s: &str) -> Result<()> {
        if self.take(s) {
            Ok(())
        } else {
            Err(FormatError("Invalid GraphQL"))
        }
    }
    fn start(&self) -> usize {
        self.tokens.get(self.pos).map_or(0, |t| t.start)
    }
    fn end(&self) -> usize {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map_or(0, |t| t.end)
    }
    fn leaf(&mut self, kind: &'static str) -> Result<Node> {
        let t = self
            .tokens
            .get(self.pos)
            .ok_or(FormatError("Invalid GraphQL"))?;
        let mut n = Node::new(kind, t.start);
        n.end = t.end;
        n.value = t.value.clone();
        n.block = t.block;
        self.pos += 1;
        Ok(n)
    }
    fn name(&mut self) -> Result<Node> {
        self.leaf("Name")
    }
    fn typ(&mut self) -> Result<Node> {
        let start = self.start();
        let mut n = if self.take("[") {
            let mut n = Node::new("ListType", start);
            n.set("type", self.typ()?);
            self.expect("]")?;
            n.end = self.end();
            n
        } else {
            let mut n = Node::new("NamedType", start);
            n.set("name", self.name()?);
            n.end = self.end();
            n
        };
        if self.take("!") {
            let mut outer = Node::new("NonNullType", start);
            outer.set("type", n);
            outer.end = self.end();
            n = outer;
        }
        Ok(n)
    }
    fn value(&mut self) -> Result<Node> {
        let start = self.start();
        let mut n;
        if self.tokens.get(self.pos).is_some_and(|t| t.string) {
            return self.leaf("StringValue");
        }
        if self.take("$") {
            n = Node::new("Variable", start);
            n.set("name", self.name()?);
        } else if self.take("[") {
            n = Node::new("ListValue", start);
            let mut values = vec![];
            while !self.is("]") {
                values.push(self.value()?);
            }
            self.expect("]")?;
            n.fields.insert("values", values);
        } else if self.take("{") {
            n = Node::new("ObjectValue", start);
            let mut fields = vec![];
            while !self.is("}") {
                fields.push(self.pair("ObjectField", false)?);
            }
            self.expect("}")?;
            n.fields.insert("fields", fields);
        } else {
            n = self.leaf("Literal")?;
        }
        n.end = self.end();
        Ok(n)
    }
    fn pair(&mut self, kind: &'static str, typed: bool) -> Result<Node> {
        let mut n = Node::new(kind, self.start());
        if typed && self.tokens.get(self.pos).is_some_and(|t| t.string) {
            n.set("description", self.leaf("StringValue")?);
        }
        n.set("name", self.name()?);
        self.expect(":")?;
        n.set(
            if typed { "type" } else { "value" },
            if typed { self.typ()? } else { self.value()? },
        );
        if typed {
            if self.take("=") {
                n.set("defaultValue", self.value()?);
            }
            n.fields.insert("directives", self.directives()?);
        }
        n.end = self.end();
        Ok(n)
    }
    fn arguments(&mut self, typed: bool) -> Result<Vec<Node>> {
        let mut v = vec![];
        if self.take("(") {
            while !self.is(")") {
                v.push(self.pair(
                    if typed {
                        "InputValueDefinition"
                    } else {
                        "Argument"
                    },
                    typed,
                )?);
            }
            self.expect(")")?;
        }
        Ok(v)
    }
    fn variables(&mut self) -> Result<Vec<Node>> {
        let mut v = vec![];
        if self.take("(") {
            while !self.is(")") {
                let mut n = Node::new("VariableDefinition", self.start());
                self.expect("$")?;
                let mut var = Node::new("Variable", n.start);
                var.set("name", self.name()?);
                var.end = self.end();
                n.set("variable", var);
                self.expect(":")?;
                n.set("type", self.typ()?);
                if self.take("=") {
                    n.set("defaultValue", self.value()?);
                }
                n.fields.insert("directives", self.directives()?);
                n.end = self.end();
                v.push(n);
            }
            self.expect(")")?;
        }
        Ok(v)
    }
    fn directives(&mut self) -> Result<Vec<Node>> {
        let mut v = vec![];
        while self.is("@") {
            let mut n = Node::new("Directive", self.start());
            self.pos += 1;
            n.set("name", self.name()?);
            n.fields.insert("arguments", self.arguments(false)?);
            n.end = self.end();
            v.push(n);
        }
        Ok(v)
    }
    fn selection(&mut self) -> Result<Node> {
        let mut n = Node::new("SelectionSet", self.start());
        self.expect("{")?;
        let mut v = vec![];
        while !self.is("}") {
            let mut f = Node::new("Field", self.start());
            if self.take("...") {
                if self.take("on") {
                    f.kind = "InlineFragment";
                    f.set("typeCondition", self.name()?);
                } else if self.is("@") || self.is("{") {
                    f.kind = "InlineFragment";
                } else {
                    f.kind = "FragmentSpread";
                    f.set("name", self.name()?);
                }
                f.fields.insert("directives", self.directives()?);
                if f.kind == "InlineFragment" {
                    f.set("selectionSet", self.selection()?);
                }
            } else {
                let name = self.name()?;
                if self.take(":") {
                    f.set("alias", name);
                    f.set("name", self.name()?);
                } else {
                    f.set("name", name);
                }
                f.fields.insert("arguments", self.arguments(false)?);
                f.fields.insert("directives", self.directives()?);
                if self.is("{") {
                    f.set("selectionSet", self.selection()?);
                }
            }
            f.end = self.end();
            v.push(f);
        }
        self.expect("}")?;
        n.fields.insert("selections", v);
        n.end = self.end();
        Ok(n)
    }
    fn definition(&mut self) -> Result<Node> {
        let mut n = Node::new("OperationDefinition", self.start());
        if self.tokens.get(self.pos).is_some_and(|t| t.string) {
            n.set("description", self.leaf("StringValue")?);
        }
        let extend = self.take("extend");
        let word = if self.is("{") {
            String::new()
        } else {
            self.leaf("Name")?.value
        };
        n.value = word.clone();
        match word.as_str() {
            "" | "query" | "mutation" | "subscription" => {
                if !self.is("{") && !self.is("(") && !self.is("@") {
                    n.set("name", self.name()?);
                }
                n.fields.insert("variableDefinitions", self.variables()?);
                n.fields.insert("directives", self.directives()?);
                n.set("selectionSet", self.selection()?);
            }
            "fragment" => {
                n.kind = "FragmentDefinition";
                n.set("name", self.name()?);
                n.fields.insert("variableDefinitions", self.variables()?);
                self.expect("on")?;
                n.set("typeCondition", self.name()?);
                n.fields.insert("directives", self.directives()?);
                n.set("selectionSet", self.selection()?);
            }
            "schema" => {
                if extend {
                    return Err(FormatError(
                        "Schema extensions are unsupported by Prettier 2.5.1",
                    ));
                }
                n.kind = "SchemaDefinition";
                n.fields.insert("directives", self.directives()?);
                self.expect("{")?;
                let mut v = vec![];
                while !self.is("}") {
                    let mut f = Node::new("OperationTypeDefinition", self.start());
                    f.set("operation", self.name()?);
                    self.expect(":")?;
                    f.set("type", self.name()?);
                    f.end = self.end();
                    v.push(f);
                }
                self.expect("}")?;
                n.fields.insert("operationTypes", v);
            }
            "directive" => {
                n.kind = "DirectiveDefinition";
                self.expect("@")?;
                n.set("name", self.name()?);
                n.fields.insert("arguments", self.arguments(true)?);
                if self.take("repeatable") {
                    n.value = "repeatable".into();
                } else {
                    n.value.clear();
                }
                self.expect("on")?;
                self.take("|");
                let mut v = vec![self.name()?];
                while self.take("|") {
                    v.push(self.name()?);
                }
                n.fields.insert("locations", v);
            }
            "type" | "interface" | "input" | "enum" | "scalar" | "union" => {
                n.kind = match word.as_str() {
                    "type" => "ObjectTypeDefinition",
                    "interface" => "InterfaceTypeDefinition",
                    "input" => "InputObjectTypeDefinition",
                    "enum" => "EnumTypeDefinition",
                    "scalar" => "ScalarTypeDefinition",
                    _ => "UnionTypeDefinition",
                };
                n.value = if extend {
                    format!("extend {word}")
                } else {
                    word.clone()
                };
                n.set("name", self.name()?);
                if self.take("implements") {
                    self.take("&");
                    let mut v = vec![self.name()?];
                    while self.take("&")
                        || self.tokens.get(self.pos).is_some_and(|t| {
                            !t.string
                                && t.value
                                    .as_bytes()
                                    .first()
                                    .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
                        })
                    {
                        v.push(self.name()?);
                    }
                    n.fields.insert("interfaces", v);
                }
                n.fields.insert("directives", self.directives()?);
                if word == "union" && self.take("=") {
                    self.take("|");
                    let mut v = vec![self.name()?];
                    while self.take("|") {
                        v.push(self.name()?);
                    }
                    n.fields.insert("types", v);
                }
                if self.take("{") {
                    let mut v = vec![];
                    while !self.is("}") {
                        let mut f = Node::new(
                            if word == "enum" {
                                "EnumValueDefinition"
                            } else {
                                "FieldDefinition"
                            },
                            self.start(),
                        );
                        if word == "input" {
                            v.push(self.pair("InputValueDefinition", true)?);
                            continue;
                        }
                        if self.tokens.get(self.pos).is_some_and(|t| t.string) {
                            f.set("description", self.leaf("StringValue")?);
                        }
                        f.set("name", self.name()?);
                        if word != "enum" {
                            f.fields.insert("arguments", self.arguments(true)?);
                            self.expect(":")?;
                            f.set("type", self.typ()?);
                        }
                        f.fields.insert("directives", self.directives()?);
                        f.end = self.end();
                        v.push(f);
                    }
                    self.expect("}")?;
                    n.fields.insert("fields", v);
                }
            }
            _ => return Err(FormatError("Invalid GraphQL")),
        }
        n.end = self.end();
        Ok(n)
    }
}
fn empty_line(input: &str, end: usize) -> bool {
    let s = &input[end..];
    let Some(n) = s.find('\n') else { return false };
    if !s[..n].trim_matches([' ', '\t', ',', '\r']).is_empty() {
        return false;
    }
    s[n + 1..]
        .split('\n')
        .next()
        .is_some_and(|s| s.trim().is_empty())
        && s[n + 1..].contains('\n')
}
fn same_line(input: &str, a: usize, b: usize) -> bool {
    !input[a..b].contains(['\r', '\n'])
}
fn attach(n: &mut Node, c: Comment, input: &str) {
    let mut keys: Vec<_> = n
        .fields
        .iter()
        .flat_map(|(k, v)| {
            v.iter()
                .enumerate()
                .map(move |(i, n)| (*k, i, n.start, n.end))
        })
        .collect();
    keys.sort_by_key(|x| (x.2, x.3));
    if let Some((k, i, _, _)) = keys
        .iter()
        .find(|x| x.2 <= c.start && c.end <= x.3)
        .copied()
    {
        attach(&mut n.fields.get_mut(k).unwrap()[i], c, input);
        return;
    }
    let prev = keys.iter().rev().find(|x| x.3 <= c.start).copied();
    let next = keys.iter().find(|x| x.2 >= c.end).copied();
    if let Some((k, i, _, end)) = prev {
        if same_line(input, end, c.start) {
            n.fields.get_mut(k).unwrap()[i].trailing.push(c);
            return;
        }
    }
    if let Some((k, i, _, _)) = next {
        n.fields.get_mut(k).unwrap()[i].leading.push(c);
    } else if let Some((k, i, _, _)) = prev {
        n.fields.get_mut(k).unwrap()[i].trailing.push(c);
    } else {
        n.dangling.push(c);
    }
}
fn seq(n: &Node, key: &str, input: &str) -> Vec<Doc> {
    let v = n.list(key);
    v.iter()
        .enumerate()
        .map(|(i, n)| {
            let d = render(n, input);
            if i + 1 < v.len() && empty_line(input, n.end) {
                cat(vec![d, hard()])
            } else {
                d
            }
        })
        .collect()
}
fn args(n: &Node, key: &str, input: &str, open: &str, close: &str, spacing: bool) -> Doc {
    let v = seq(n, key, input);
    if v.is_empty() {
        return text(format!("{open}{close}"));
    }
    group(cat(vec![
        text(open),
        text(if spacing { " " } else { "" }),
        indent(cat(vec![
            soft(),
            join(cat(vec![if_break(text(""), text(", ")), soft()]), v),
        ])),
        soft(),
        if_break(text(""), text(if spacing { " " } else { "" })),
        text(close),
    ]))
}
fn directives(n: &Node, input: &str) -> Doc {
    if n.list("directives").is_empty() {
        return text("");
    }
    let d = join(
        line(),
        n.list("directives")
            .iter()
            .map(|x| render(x, input))
            .collect(),
    );
    if matches!(n.kind, "OperationDefinition" | "FragmentDefinition") {
        group(cat(vec![line(), d]))
    } else {
        cat(vec![text(" "), group(indent(cat(vec![soft(), d])))])
    }
}
fn braces(n: &Node, key: &str, input: &str) -> Doc {
    cat(vec![
        text("{"),
        indent(cat(vec![hard(), join(hard(), seq(n, key, input))])),
        hard(),
        text("}"),
    ])
}
fn join_interfaces(n: &Node, input: &str) -> Doc {
    let list = n.list("interfaces");
    let mut docs = vec![];
    for (i, x) in list.iter().enumerate() {
        docs.push(render(x, input));
        if let Some(next) = list.get(i + 1) {
            let between = &input[x.end..next.start];
            let separator = between
                .lines()
                .map(|l| l.split('#').next().unwrap_or(""))
                .collect::<Vec<_>>()
                .join(" ");
            docs.push(text(if separator.trim() == "," { "," } else { " &" }));
            docs.push(if between.contains('#') {
                line()
            } else {
                text(" ")
            });
        }
    }
    cat(docs)
}
fn render(n: &Node, input: &str) -> Doc {
    let p = |key: &str| n.get(key).map_or_else(|| text(""), |x| render(x, input));
    let has = |key: &str| n.get(key).is_some();
    let a = |key: &str| {
        if n.list(key).is_empty() {
            text("")
        } else {
            args(n, key, input, "(", ")", false)
        }
    };
    let description = || {
        cat(vec![
            p("description"),
            if has("description") { hard() } else { text("") },
        ])
    };
    let body = if n
        .leading
        .iter()
        .any(|c| c.value.trim() == "# prettier-ignore")
    {
        text(&input[n.start..n.end])
    } else {
        match n.kind {
            "Document" => {
                let mut v = seq(n, "definitions", input);
                for c in &n.dangling {
                    v.push(text(&c.value));
                }
                cat(vec![join(hard(), v), hard()])
            }
            "Name" | "Literal" => text(&n.value),
            "StringValue" => {
                if n.block {
                    cat(vec![
                        text("\"\"\""),
                        hard(),
                        join(
                            hard(),
                            n.value
                                .replace("\"\"\"", "\\\"\"\"")
                                .split('\n')
                                .map(text)
                                .collect(),
                        ),
                        hard(),
                        text("\"\"\""),
                    ])
                } else {
                    text(format!(
                        "\"{}\"",
                        n.value
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('\n', "\\n")
                    ))
                }
            }
            "Variable" => cat(vec![text("$"), p("name")]),
            "NamedType" => p("name"),
            "ListType" => cat(vec![text("["), p("type"), text("]")]),
            "NonNullType" => cat(vec![p("type"), text("!")]),
            "Argument" | "ObjectField" => cat(vec![p("name"), text(": "), p("value")]),
            "OperationTypeDefinition" => cat(vec![p("operation"), text(": "), p("type")]),
            "ListValue" => args(n, "values", input, "[", "]", false),
            "ObjectValue" => args(n, "fields", input, "{", "}", true),
            "Directive" => cat(vec![text("@"), p("name"), a("arguments")]),
            "VariableDefinition" | "InputValueDefinition" => cat(vec![
                p("description"),
                if has("description") {
                    if n.get("description").unwrap().block {
                        hard()
                    } else {
                        line()
                    }
                } else {
                    text("")
                },
                p(if n.kind == "VariableDefinition" {
                    "variable"
                } else {
                    "name"
                }),
                text(": "),
                p("type"),
                if has("defaultValue") {
                    cat(vec![text(" = "), p("defaultValue")])
                } else {
                    text("")
                },
                directives(n, input),
            ]),
            "SelectionSet" => braces(n, "selections", input),
            "Field" => group(cat(vec![
                if has("alias") {
                    cat(vec![p("alias"), text(": ")])
                } else {
                    text("")
                },
                p("name"),
                a("arguments"),
                directives(n, input),
                if has("selectionSet") {
                    text(" ")
                } else {
                    text("")
                },
                p("selectionSet"),
            ])),
            "FragmentSpread" => cat(vec![text("..."), p("name"), directives(n, input)]),
            "InlineFragment" => cat(vec![
                text("..."),
                if has("typeCondition") {
                    cat(vec![text(" on "), p("typeCondition")])
                } else {
                    text("")
                },
                directives(n, input),
                text(" "),
                p("selectionSet"),
            ]),
            "OperationDefinition" => cat(vec![
                text(&n.value),
                if has("name") {
                    cat(vec![text(" "), p("name")])
                } else if !n.value.is_empty() && !n.list("variableDefinitions").is_empty() {
                    text(" ")
                } else {
                    text("")
                },
                a("variableDefinitions"),
                directives(n, input),
                if !n.value.is_empty() {
                    text(" ")
                } else {
                    text("")
                },
                p("selectionSet"),
            ]),
            "FragmentDefinition" => cat(vec![
                text("fragment "),
                p("name"),
                a("variableDefinitions"),
                text(" on "),
                p("typeCondition"),
                directives(n, input),
                text(" "),
                p("selectionSet"),
            ]),
            "FieldDefinition" => cat(vec![
                description(),
                p("name"),
                a("arguments"),
                text(": "),
                p("type"),
                directives(n, input),
            ]),
            "EnumValueDefinition" => cat(vec![description(), p("name"), directives(n, input)]),
            "DirectiveDefinition" => cat(vec![
                description(),
                text("directive @"),
                p("name"),
                a("arguments"),
                text(if n.value == "repeatable" {
                    " repeatable"
                } else {
                    ""
                }),
                text(" on "),
                join(
                    text(" | "),
                    n.list("locations")
                        .iter()
                        .map(|x| render(x, input))
                        .collect(),
                ),
            ]),
            "SchemaDefinition" => cat(vec![
                text("schema"),
                directives(n, input),
                text(" "),
                braces(n, "operationTypes", input),
            ]),
            "UnionTypeDefinition" => group(cat(vec![
                description(),
                group(cat(vec![
                    text(format!("{} ", n.value)),
                    p("name"),
                    directives(n, input),
                    if n.list("types").is_empty() {
                        text("")
                    } else {
                        cat(vec![
                            text(" ="),
                            if_break(text(""), text(" ")),
                            indent(cat(vec![
                                if_break(cat(vec![line(), text("  ")]), text("")),
                                join(
                                    cat(vec![line(), text("| ")]),
                                    n.list("types").iter().map(|x| render(x, input)).collect(),
                                ),
                            ])),
                        ])
                    },
                ])),
            ])),
            _ => cat(vec![
                description(),
                text(format!("{} ", n.value)),
                p("name"),
                if n.list("interfaces").is_empty() {
                    text("")
                } else {
                    cat(vec![text(" implements "), join_interfaces(n, input)])
                },
                directives(n, input),
                if n.list("fields").is_empty() {
                    text("")
                } else {
                    cat(vec![text(" "), braces(n, "fields", input)])
                },
            ]),
        }
    };
    let mut v = vec![];
    for c in &n.leading {
        v.push(text(&c.value));
        v.push(hard());
        if empty_line(input, c.end) {
            v.push(hard());
        }
    }
    v.push(body);
    for c in &n.trailing {
        if same_line(input, n.end, c.start) {
            v.push(Doc::Suffix(Box::new(cat(vec![text(" "), text(&c.value)]))));
        } else {
            v.push(hard());
            v.push(text(&c.value));
        }
    }
    cat(v)
}
/// Format with the original command's parser, printWidth=80 and LF output.
/// All errors are static and never echo a private document or parser diagnostic.
pub fn format(input: &str, indentation: Indentation) -> Result<String> {
    if input.len() > MAX_INPUT {
        return Err(FormatError("GraphQL input limit exceeded"));
    }
    let (tokens, comments) = lex(input)?;
    if tokens.is_empty() {
        if comments.is_empty() {
            return Ok(String::new());
        }
        return Err(FormatError("Invalid GraphQL"));
    }
    let mut parser = Parse {
        tokens: tokens.clone(),
        pos: 0,
    };
    let mut root = Node::new("Document", 0);
    root.end = input.len();
    let mut defs = vec![];
    while parser.pos < parser.tokens.len() {
        let pos = parser.pos;
        defs.push(parser.definition()?);
        if parser.pos <= pos {
            return Err(FormatError("Invalid GraphQL"));
        }
    }
    // Apollo lacks graphql-js's legacy interface and fragment-variable options.
    // Normalize those syntactic forms only for validation, never for printing.
    let mut replacements: Vec<(usize, usize, String)> = vec![];
    for t in &tokens {
        if t.string {
            replacements.push((t.start, t.end, "\"x\"".into()));
        }
    }
    for n in &defs {
        let interfaces = n.list("interfaces");
        for pair in interfaces.windows(2) {
            if !tokens
                .iter()
                .any(|t| t.start >= pair[0].end && t.end <= pair[1].start && t.value == "&")
            {
                replacements.push((pair[1].start, pair[1].start, " & ".into()));
            }
        }
        if n.kind == "FragmentDefinition" && !n.list("variableDefinitions").is_empty() {
            replacements.push((n.start, n.start + 8, "query".into()));
            let condition = n.get("typeCondition").unwrap();
            let on = tokens
                .iter()
                .rev()
                .find(|t| t.end <= condition.start && t.value == "on")
                .unwrap();
            replacements.push((on.start, condition.end, String::new()));
        }
    }
    replacements.sort_by_key(|x| x.0);
    let mut validation = String::new();
    let mut cursor = 0;
    for (start, end, replacement) in replacements {
        validation.push_str(&input[cursor..start]);
        validation.push_str(&replacement);
        cursor = end;
    }
    validation.push_str(&input[cursor..]);
    let tree = apollo_parser::Parser::new(&validation)
        .recursion_limit(256)
        .token_limit(MAX_TOKENS * 2)
        .parse();
    if tree.errors().next().is_some() {
        return Err(FormatError("Invalid GraphQL"));
    }
    root.fields.insert("definitions", defs);
    for c in comments {
        attach(&mut root, c, input);
    }
    let out = print(
        &render(&root, input),
        indentation == Indentation::Tabs,
        indentation.width(),
        MAX_OUTPUT,
    )
    .map_err(FormatError)?;
    Ok(if input.starts_with('\u{feff}') {
        format!("\u{feff}{out}")
    } else {
        out
    })
}

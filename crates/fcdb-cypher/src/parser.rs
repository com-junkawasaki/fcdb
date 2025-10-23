use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "grammar/cypher.pest"]
pub struct CypherParser;

pub fn parse_query(input: &str) -> Result<Query, String> {
    let pairs = CypherParser::parse(Rule::cypher_query, input)
        .map_err(|e| format!("Parse error: {}", e))?;

    let mut statements = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::match_clause => {
                let pattern = parse_pattern(pair)?;
                statements.push(Statement::Match(MatchClause { pattern }));
            }
            Rule::where_clause => {
                let condition = parse_expression(pair.into_inner().next().unwrap())?;
                statements.push(Statement::Where(WhereClause { condition }));
            }
            Rule::return_clause => {
                let return_clause = parse_return_clause(pair)?;
                statements.push(Statement::Return(return_clause));
            }
            _ => {} // Skip other rules
        }
    }

    Ok(Query { statements })
}

fn parse_pattern(pair: pest::iterators::Pair<Rule>) -> Result<Pattern, String> {
    let mut elements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::pattern_element => {
                let element = parse_pattern_element(inner_pair)?;
                elements.push(element);
            }
            _ => {}
        }
    }

    Ok(Pattern { elements })
}

fn parse_pattern_element(pair: pest::iterators::Pair<Rule>) -> Result<PatternElement, String> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::node_pattern => {
            let node = parse_node_pattern(inner)?;
            Ok(PatternElement::Node(node))
        }
        Rule::relationship_pattern => {
            let rel = parse_relationship_pattern(inner)?;
            Ok(PatternElement::Relationship(rel))
        }
        _ => Err("Unknown pattern element".to_string()),
    }
}

fn parse_node_pattern(pair: pest::iterators::Pair<Rule>) -> Result<NodePattern, String> {
    let mut variable = None;
    let mut labels = Vec::new();
    let mut properties = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::variable => {
                variable = Some(inner.as_str().to_string());
            }
            Rule::label => {
                labels.push(inner.as_str().trim_start_matches(':').to_string());
            }
            Rule::property_map => {
                properties = parse_property_map(inner)?;
            }
            _ => {}
        }
    }

    Ok(NodePattern {
        variable,
        labels,
        properties,
    })
}

fn parse_relationship_pattern(pair: pest::iterators::Pair<Rule>) -> Result<RelationshipPattern, String> {
    let mut variable = None;
    let mut types = Vec::new();
    let direction = Direction::Outgoing; // Default
    let mut length = None;
    let mut properties = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::variable => {
                variable = Some(inner.as_str().to_string());
            }
            Rule::label => {
                types.push(inner.as_str().trim_start_matches(':').to_string());
            }
            Rule::path_length => {
                length = Some(parse_path_length(inner)?);
            }
            Rule::property_map => {
                properties = parse_property_map(inner)?;
            }
            _ => {}
        }
    }

    Ok(RelationshipPattern {
        variable,
        types,
        direction,
        length,
        properties,
    })
}

fn parse_path_length(pair: pest::iterators::Pair<Rule>) -> Result<PathLength, String> {
    let inner = pair.into_inner().next();

    match inner {
        Some(p) => {
            match p.as_str() {
                "*" => Ok(PathLength::Any),
                s if s.starts_with('*') => {
                    // Parse range like *1..3
                    let range_str = &s[1..];
                    let parts: Vec<&str> = range_str.split("..").collect();
                    match parts.len() {
                        1 => {
                            let min = parts[0].parse().map_err(|_| "Invalid range")?;
                            Ok(PathLength::Range(min, None))
                        }
                        2 => {
                            let min = parts[0].parse().map_err(|_| "Invalid range")?;
                            let max = parts[1].parse().map_err(|_| "Invalid range")?;
                            Ok(PathLength::Range(min, Some(max)))
                        }
                        _ => Err("Invalid path length".to_string()),
                    }
                }
                _ => Err("Invalid path length".to_string()),
            }
        }
        None => Ok(PathLength::Any),
    }
}

fn parse_property_map(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Property>, String> {
    let mut properties = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::property_pair {
            let prop = parse_property_pair(inner)?;
            properties.push(prop);
        }
    }

    Ok(properties)
}

fn parse_property_pair(pair: pest::iterators::Pair<Rule>) -> Result<Property, String> {
    let mut key = String::new();
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::property_key => {
                key = inner.as_str().to_string();
            }
            Rule::expression => {
                value = Some(parse_expression(inner)?);
            }
            _ => {}
        }
    }

    Ok(Property {
        key,
        value: value.ok_or("Missing property value")?,
    })
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::literal => parse_literal(inner),
        Rule::variable => Ok(Expression::Variable(inner.as_str().to_string())),
        Rule::property_access => parse_property_access(inner),
        Rule::comparison_expression => parse_comparison_expression(inner),
        _ => Err("Unsupported expression type".to_string()),
    }
}

fn parse_literal(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::string => {
            let s = inner.as_str();
            let cleaned = s.trim_matches('"').replace("\\\"", "\"");
            Ok(Expression::Literal(Literal::String(cleaned)))
        }
        Rule::integer => {
            let i: i64 = inner.as_str().parse().map_err(|_| "Invalid integer")?;
            Ok(Expression::Literal(Literal::Integer(i)))
        }
        Rule::float => {
            let f: f64 = inner.as_str().parse().map_err(|_| "Invalid float")?;
            Ok(Expression::Literal(Literal::Float(f)))
        }
        Rule::boolean => {
            let b: bool = inner.as_str().parse().map_err(|_| "Invalid boolean")?;
            Ok(Expression::Literal(Literal::Boolean(b)))
        }
        Rule::null => Ok(Expression::Literal(Literal::Null)),
        _ => Err("Unknown literal type".to_string()),
    }
}

fn parse_property_access(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut variable = String::new();
    let mut property = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::variable => variable = inner.as_str().to_string(),
            Rule::property_key => property = inner.as_str().to_string(),
            _ => {}
        }
    }

    Ok(Expression::PropertyAccess { variable, property })
}

fn parse_comparison_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut parts = pair.into_inner();

    let left = parse_expression(parts.next().unwrap())?;
    let op_pair = parts.next().unwrap();
    let right = parse_expression(parts.next().unwrap())?;

    let op = match op_pair.as_str() {
        "=" => BinaryOperator::Equal,
        "<>" => BinaryOperator::NotEqual,
        "<" => BinaryOperator::LessThan,
        ">" => BinaryOperator::GreaterThan,
        "<=" => BinaryOperator::LessEqual,
        ">=" => BinaryOperator::GreaterEqual,
        _ => return Err("Unknown operator".to_string()),
    };

    Ok(Expression::BinaryOp {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

fn parse_return_clause(pair: pest::iterators::Pair<Rule>) -> Result<ReturnClause, String> {
    let mut items = Vec::new();
    let mut distinct = false;
    let mut limit = None;
    let mut skip = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::DISTINCT => distinct = true,
            Rule::return_item => {
                let item = parse_return_item(inner)?;
                items.push(item);
            }
            Rule::LIMIT => {
                limit = Some(inner.into_inner().next().unwrap().as_str().parse().unwrap());
            }
            Rule::SKIP => {
                skip = Some(inner.into_inner().next().unwrap().as_str().parse().unwrap());
            }
            _ => {}
        }
    }

    Ok(ReturnClause {
        items,
        distinct,
        limit,
        skip,
    })
}

fn parse_return_item(pair: pest::iterators::Pair<Rule>) -> Result<ReturnItem, String> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::variable => Ok(ReturnItem::Variable(inner.as_str().to_string())),
        Rule::property_access => {
            let mut variable = String::new();
            let mut property = String::new();

            for part in inner.into_inner() {
                match part.as_rule() {
                    Rule::variable => variable = part.as_str().to_string(),
                    Rule::property_key => property = part.as_str().to_string(),
                    _ => {}
                }
            }

            Ok(ReturnItem::Property { variable, property })
        }
        _ => Ok(ReturnItem::Count), // Simplified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_match() {
        let query = "MATCH (n:Person) RETURN n";
        let result = parse_query(query);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_match_with_relationship() {
        let query = "MATCH (n:Person)-[:KNOWS]->(m:Person) RETURN n, m";
        let result = parse_query(query);
        assert!(result.is_ok());
    }
}

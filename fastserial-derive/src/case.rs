pub enum RenameRule {
    None,
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameRule {
    pub fn from_str(rule: &str) -> Option<Self> {
        match rule {
            "lowercase" => Some(RenameRule::LowerCase),
            "UPPERCASE" => Some(RenameRule::UpperCase),
            "PascalCase" => Some(RenameRule::PascalCase),
            "camelCase" => Some(RenameRule::CamelCase),
            "snake_case" => Some(RenameRule::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(RenameRule::ScreamingSnakeCase),
            "kebab-case" => Some(RenameRule::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(RenameRule::ScreamingKebabCase),
            _ => None,
        }
    }

    pub fn apply_to_field(&self, field: &str) -> String {
        match self {
            RenameRule::None => field.to_owned(),
            RenameRule::LowerCase => field.to_ascii_lowercase(),
            RenameRule::UpperCase => field.to_ascii_uppercase(),
            RenameRule::PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for c in field.chars() {
                    if c == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(c.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(c);
                    }
                }
                pascal
            }
            RenameRule::CamelCase => {
                let pascal = RenameRule::PascalCase.apply_to_field(field);
                if pascal.is_empty() {
                    pascal
                } else {
                    let mut camel = String::new();
                    let mut chars = pascal.chars();
                    camel.push(chars.next().unwrap().to_ascii_lowercase());
                    camel.extend(chars);
                    camel
                }
            }
            RenameRule::SnakeCase => field.to_owned(),
            RenameRule::ScreamingSnakeCase => field.to_ascii_uppercase(),
            RenameRule::KebabCase => field.replace('_', "-"),
            RenameRule::ScreamingKebabCase => field.replace('_', "-").to_ascii_uppercase(),
        }
    }
}

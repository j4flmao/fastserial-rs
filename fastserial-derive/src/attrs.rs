use syn::{Attribute, Field, meta::ParseNestedMeta};

pub struct ContainerAttrs {
    pub rename_all: crate::case::RenameRule,
    pub default: bool,
    pub default_path: Option<String>,
}

pub fn parse_container_attrs(attrs: &[Attribute]) -> ContainerAttrs {
    let mut rename_all = crate::case::RenameRule::None;
    let mut default = false;
    let mut default_path = None;

    for attr in attrs {
        if attr.meta.path().is_ident("fastserial") {
            let _ = attr.parse_nested_meta(|meta: ParseNestedMeta| {
                if meta.path.is_ident("rename_all") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    if let Some(rule) = crate::case::RenameRule::from_str(&lit.value()) {
                        rename_all = rule;
                    }
                } else if meta.path.is_ident("default") {
                    if let Ok(value) = meta.value() {
                        let lit: syn::LitStr = value.parse()?;
                        default_path = Some(lit.value());
                    } else {
                        default = true;
                    }
                }
                Ok(())
            });
        }
    }

    ContainerAttrs {
        rename_all,
        default,
        default_path,
    }
}

pub struct FieldAttrs {
    pub rename: Option<String>,
    pub skip: bool,
    pub skip_serializing_if: Option<String>,
    pub default: bool,
    pub default_path: Option<String>,
}

pub fn parse_field_attrs_raw(field: &Field) -> FieldAttrs {
    let mut rename = None;
    let mut skip = false;
    let mut skip_serializing_if = None;
    let mut default = false;
    let mut default_path = None;

    for attr in &field.attrs {
        if attr.meta.path().is_ident("fastserial") {
            let _ = attr.parse_nested_meta(|meta: ParseNestedMeta| {
                if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("rename") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    rename = Some(lit.value());
                } else if meta.path.is_ident("skip_serializing_if") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    skip_serializing_if = Some(lit.value());
                } else if meta.path.is_ident("default") {
                    if let Ok(value) = meta.value() {
                        let lit: syn::LitStr = value.parse()?;
                        default_path = Some(lit.value());
                    } else {
                        default = true;
                    }
                }
                Ok(())
            });
        }
    }

    FieldAttrs {
        rename,
        skip,
        skip_serializing_if,
        default,
        default_path,
    }
}

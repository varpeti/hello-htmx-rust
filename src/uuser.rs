use edb_orm::{includes, table_struct};

includes!();

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
    Company,
    Customer,
}

table_struct! {r#"
name = "Uuser"
    params = [
        { name = "email", ty = "String" },
        { name = "role", ty = "Role", pty = "Enum" },
    ]
"#}

table_struct! {r#"
name = "Auth"
    params = [
        { name = "uuser", ty = "Uuser", pty = "Key" },
        { name = "phc_string", ty = "String" },
    ]
"#}

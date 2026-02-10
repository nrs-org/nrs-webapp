use sea_query::{Expr, InsertStatement, Nullable, SimpleExpr, UpdateStatement};
use thiserror::Error;

use crate as sqlbindable;
pub use sqlbindable_macros::FieldNames;
pub use sqlbindable_macros::Fields;

pub trait HasFieldNames {
    fn field_names() -> FieldNameVec;
}

pub trait HasFields: HasFieldNames {
    fn not_none_fields(self) -> Result<FieldVec, TryIntoExprError>;
    fn all_fields(self) -> Result<FieldVec, TryIntoExprError>;
}

pub struct Field {
    pub name: &'static str,
    pub value: SimpleExpr,
}

pub struct DerivedField {
    pub name: String,
    pub value: SimpleExpr,
}

pub struct FieldVec(pub Vec<Field>);
pub struct DerivedFieldVec(pub Vec<DerivedField>);

impl FieldVec {
    pub fn add_prefix(&self, prefix: &str) -> DerivedFieldVec {
        let derived_fields = self
            .0
            .iter()
            .map(|field| DerivedField {
                name: format!("{}.{}", prefix, field.name),
                value: field.value.clone(),
            })
            .collect();
        DerivedFieldVec(derived_fields)
    }
}

impl DerivedFieldVec {
    pub fn add_prefix(&self, prefix: &str) -> DerivedFieldVec {
        let derived_fields = self
            .0
            .iter()
            .map(|field| DerivedField {
                name: format!("{}.{}", prefix, field.name),
                value: field.value.clone(),
            })
            .collect();
        DerivedFieldVec(derived_fields)
    }
}

pub struct FieldNameVec(pub &'static [&'static str]);
pub struct DerivedFieldNameVec(pub Vec<String>);

impl FieldNameVec {
    pub fn add_prefix(&self, prefix: &str) -> DerivedFieldNameVec {
        let derived_names = self
            .0
            .iter()
            .map(|name| format!("{}.{}", prefix, name))
            .collect();
        DerivedFieldNameVec(derived_names)
    }

    pub fn iter(&self) -> impl Iterator<Item = &'static &'static str> {
        self.0.iter()
    }

    pub fn iter_copied(&self) -> impl Iterator<Item = &'static str> {
        self.0.iter().copied()
    }
}

impl DerivedFieldNameVec {
    pub fn add_prefix(&self, prefix: &str) -> DerivedFieldNameVec {
        let derived_names = self
            .0
            .iter()
            .map(|name| format!("{}.{}", prefix, name))
            .collect();
        DerivedFieldNameVec(derived_names)
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }

    pub fn iter_cloned(&self) -> impl Iterator<Item = String> {
        self.0.iter().cloned()
    }
}

impl IntoIterator for DerivedFieldNameVec {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", serde_with::serde_as)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TryIntoExprError {
    #[cfg(feature = "sqlx-json")]
    #[error("JSON serialization error: {0}")]
    JsonSerialize(
        #[from]
        #[serde_as(as = "serde_with::DisplayFromStr")]
        serde_json::Error,
    ),
}

pub trait TryIntoExpr {
    fn into_expr(self) -> Result<Expr, TryIntoExprError>;
}

impl TryIntoExpr for Expr {
    /// Enables using an `Expr` instance wherever `TryIntoExpr` is expected by returning the same `Expr`.
    fn into_expr(self) -> Result<Expr, TryIntoExprError> {
        Ok(self)
    }
}

#[macro_export]
macro_rules! impl_into_expr_through_value_nocopy {
    ($ty:ty) => {
        impl sqlbindable::TryIntoExpr for $ty {
            fn into_expr(self) -> Result<sea_query::Expr, sqlbindable::TryIntoExprError> {
                Ok(sea_query::Value::from(self).into())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_into_expr_through_value {
    ($ty:ty) => {
        impl_into_expr_through_value_nocopy!($ty);

        impl<'a> sqlbindable::TryIntoExpr for &'a $ty {
            fn into_expr(self) -> Result<sea_query::Expr, sqlbindable::TryIntoExprError> {
                Ok(sea_query::Value::from(*self).into())
            }
        }
    };
}

impl_into_expr_through_value!(i8);
impl_into_expr_through_value!(i16);
impl_into_expr_through_value!(i32);
impl_into_expr_through_value!(i64);
impl_into_expr_through_value!(u8);
impl_into_expr_through_value!(u16);
impl_into_expr_through_value!(u32);
impl_into_expr_through_value!(u64);
impl_into_expr_through_value!(f32);
impl_into_expr_through_value!(f64);
impl_into_expr_through_value_nocopy!(Vec<u8>);
impl_into_expr_through_value_nocopy!(String);

#[cfg(feature = "with-time")]
impl_into_expr_through_value!(time::OffsetDateTime);
#[cfg(feature = "with-chrono")]
impl_into_expr_through_value!(chrono::DateTime<chrono::Utc>);
#[cfg(feature = "with-uuid")]
impl_into_expr_through_value!(uuid::Uuid);

impl<T: TryIntoExpr + Nullable> TryIntoExpr for Option<T> {
    fn into_expr(self) -> Result<Expr, TryIntoExprError> {
        match self {
            Some(value) => value.into_expr(),
            None => Ok(T::null().into()),
        }
    }
}

impl From<Vec<Field>> for FieldVec {
    fn from(value: Vec<Field>) -> Self {
        Self(value)
    }
}

#[cfg(feature = "sqlx-json")]
pub mod json {
    use sqlx::types::Json;

    use crate::TryIntoExpr;

    impl<T: serde::Serialize> TryIntoExpr for Json<T> {
        fn into_expr(self) -> Result<sea_query::Expr, crate::TryIntoExprError> {
            Ok(sea_query::Value::Json(Some(Box::new(serde_json::to_value(self.0)?))).into())
        }
    }
}

impl IntoIterator for FieldVec {
    type Item = SimpleExpr;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .into_iter()
            .map(|field| field.value)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl IntoIterator for DerivedFieldVec {
    type Item = SimpleExpr;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .into_iter()
            .map(|field| field.value)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Field {
    pub fn new<T: TryIntoExpr>(name: &'static str, value: T) -> Result<Self, TryIntoExprError> {
        Ok(Self {
            name,
            value: value.into_expr()?,
        })
    }
}

pub trait BindContext {
    fn bind(&mut self, fields: FieldVec) -> &mut Self;
}

impl BindContext for InsertStatement {
    fn bind(&mut self, fields: FieldVec) -> &mut Self {
        let (names, values): (Vec<_>, Vec<SimpleExpr>) = fields
            .0
            .into_iter()
            .map(|field| (field.name, field.value))
            .unzip();
        self.columns(names)
            .values(values)
            .expect("names.len() == values.len(), this should not panic")
    }
}

impl BindContext for UpdateStatement {
    fn bind(&mut self, fields: FieldVec) -> &mut Self {
        let fields: Vec<_> = fields
            .0
            .into_iter()
            .map(|field| (field.name, field.value))
            .collect();
        self.values(fields)
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use crate::{self as sqlbindable, BindContext, HasFieldNames, HasFields};
    use sea_query::PostgresQueryBuilder;
    use sqlbindable_macros::{FieldNames, Fields};

    #[derive(Clone, FieldNames, Fields)]
    pub struct Test {
        x: i32,
        y: Option<i32>,
        z: Option<f32>,
    }

    // generic (lifetimes, type params) fields are currently unsupported
    #[derive(Clone, FieldNames)]
    #[allow(dead_code)]
    pub struct TestNameOnly {
        x: i32,
        y: PhantomData<*const u8>,
        z: Vec<[std::sync::Arc<f32>; 1024]>,
    }

    #[test]
    fn test_bindable() {
        let test = Test {
            x: 1,
            y: Some(2),
            z: None,
        };
        let query = sea_query::Query::insert()
            .into_table("test")
            .bind(test.clone().all_fields().unwrap())
            .to_owned();
        assert_eq!(
            query.to_string(PostgresQueryBuilder),
            r#"INSERT INTO "test" ("x", "y", "z") VALUES (1, 2, NULL)"#
        );

        let query = sea_query::Query::update()
            .table("test")
            .bind(test.not_none_fields().unwrap())
            .to_owned();
        assert_eq!(
            query.to_string(PostgresQueryBuilder),
            r#"UPDATE "test" SET "x" = 1, "y" = 2"#
        );
    }

    #[test]
    fn test_field_names() {
        let field_names = TestNameOnly::field_names();
        let names: Vec<&'static str> = field_names.iter_copied().collect();
        assert_eq!(names, vec!["x", "y", "z"]);

        let prefixed_field_names = field_names.add_prefix("prefix");
        let prefixed_names: Vec<String> = prefixed_field_names.iter_cloned().collect();
        assert_eq!(prefixed_names, vec!["prefix.x", "prefix.y", "prefix.z"]);
    }
}

use sea_query::{Expr, InsertStatement, Nullable, SimpleExpr, UpdateStatement};
use thiserror::Error;

use crate as sqlbindable;
pub use sqlbindable_macros::Fields;

pub trait HasFields {
    fn not_none_fields(self) -> Result<FieldVec, TryIntoExprError>;
    fn all_fields(self) -> Result<FieldVec, TryIntoExprError>;
    fn field_names() -> &'static [&'static str];

    fn all_fields_except(self, field_name: &str) -> Result<FieldVec, TryIntoExprError>
    where
        Self: Sized,
    {
        let mut all_fields = self.all_fields()?;
        all_fields.0.retain(|field| field.name != field_name);
        Ok(all_fields)
    }
}

pub struct Field {
    pub name: &'static str,
    pub value: SimpleExpr,
}

pub struct FieldVec(pub Vec<Field>);

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
    fn into_expr(self) -> Result<Expr, TryIntoExprError> {
        Ok(self)
    }
}

#[macro_export]
macro_rules! impl_into_expr_through_value {
    ($ty:ty) => {
        impl sqlbindable::TryIntoExpr for $ty {
            fn into_expr(self) -> Result<sea_query::Expr, sqlbindable::TryIntoExprError> {
                Ok(sea_query::Value::from(self).into())
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
impl_into_expr_through_value!(Vec<u8>);
impl_into_expr_through_value!(String);

#[cfg(feature = "with-time")]
impl_into_expr_through_value!(time::OffsetDateTime);
#[cfg(feature = "with-chrono")]
impl_into_expr_through_value!(chrono::DateTime<chrono::Utc>);

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
            Ok(sea_query::Value::Json(Some(serde_json::to_value(self.0)?)).into())
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
    use crate::{self as sqlbindable, BindContext, HasFields};
    use sea_query::PostgresQueryBuilder;
    use sqlbindable_macros::Fields;

    #[derive(Clone, Fields)]
    pub struct Test {
        x: i32,
        y: Option<i32>,
        z: Option<f32>,
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
}

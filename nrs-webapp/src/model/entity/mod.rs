use sea_query::{Expr, ExprTrait, Order, Query, ReturningClause, SimpleExpr, Value};
use sqlbindable::{BindContext, HasFields};

use crate::model::{
    Error, Result, SqlxDatabase, SqlxRow, entity::id::EntityId, store::primary_store::PrimaryStore,
};

pub mod id;

#[derive(Default, Clone, Debug)]
pub struct ListPayload {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub order_by: Option<(&'static str, Order)>,
}

#[allow(async_fn_in_trait)]
pub trait DbBmc: Send {
    const TABLE_NAME: &'static str;

    /// Constructs an `Error::EntityNotFound` for this trait's `TABLE_NAME` using the given id.
    ///
    /// The provided `id` is converted into an `EntityId` and embedded in the generated error.
    ///
    /// # Examples
    ///
    /// ```
    /// // Given a type `T` implementing `DbBmc` with TABLE_NAME = "users",
    /// // calling `T::not_found_error(42)` produces an EntityNotFound error for id 42.
    /// ```
    fn not_found_error<Id: Into<EntityId>>(id: Id) -> Error {
        Error::EntityNotFound {
            name: Self::TABLE_NAME,
            id: id.into(),
        }
    }

    /// Inserts a new row into the implementing table using only the fields present in `create_req`.
    ///
    /// Returns `Ok(())` on success; any underlying primary store errors are propagated.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(ps: &mut impl PrimaryStore, req: impl HasFields + Send) -> Result<()> {
    /// MyEntity::create(ps, req).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn create(ps: &mut impl PrimaryStore, create_req: impl HasFields + Send) -> Result<()> {
        ps.query_with(
            Query::insert()
                .into_table(Self::TABLE_NAME)
                .bind(create_req.not_none_fields()?),
        )
        .execute()
        .await?;
        Ok(())
    }

    async fn get_optional_by_expr<E>(ps: &mut impl PrimaryStore, cond: Expr) -> Result<Option<E>>
    where
        E: for<'r> sqlx::FromRow<'r, SqlxRow> + Send + Unpin + HasFields,
    {
        let maybe_entity = ps
            .query_as_with::<E>(
                Query::select()
                    .from(Self::TABLE_NAME)
                    .columns(E::field_names().iter().copied())
                    .and_where(cond),
            )
            .fetch_optional()
            .await?;
        Ok(maybe_entity)
    }

    async fn get_all_by_expr<E>(ps: &mut impl PrimaryStore, cond: Expr) -> Result<Vec<E>>
    where
        E: for<'r> sqlx::FromRow<'r, SqlxRow> + Send + Unpin + HasFields,
    {
        let entities = ps
            .query_as_with::<E>(
                Query::select()
                    .from(Self::TABLE_NAME)
                    .columns(E::field_names().iter().copied())
                    .and_where(cond),
            )
            .fetch_all()
            .await?;
        Ok(entities)
    }

    async fn list<E>(ps: &mut impl PrimaryStore, payload: ListPayload) -> Result<Vec<E>>
    where
        E: for<'r> sqlx::FromRow<'r, SqlxRow> + Send + Unpin + HasFields,
    {
        let mut query = Query::select();
        let mut query = query
            .from(Self::TABLE_NAME)
            .columns(E::field_names().iter().copied());

        if let Some((col, order)) = payload.order_by {
            query = query.order_by(col, order);
        }

        if let Some(offset) = payload.offset {
            query = query.offset(offset as u64);
        }

        if let Some(limit) = payload.limit {
            query = query.limit(limit as u64);
        }

        let entities = ps.query_as_with::<E>(query).fetch_all().await?;
        Ok(entities)
    }

    async fn update_cond(
        ps: &mut impl PrimaryStore,
        update_req: impl HasFields,
        cond: Expr,
    ) -> Result<u64> {
        let rows_affected = ps
            .query_with(
                Query::update()
                    .table(Self::TABLE_NAME)
                    .bind(update_req.not_none_fields()?)
                    .and_where(cond),
            )
            .execute()
            .await?;
        Ok(rows_affected)
    }

    async fn delete_cond(ps: &mut impl PrimaryStore, cond: Expr) -> Result<u64> {
        let rows_affected = ps
            .query_with(Query::delete().from_table(Self::TABLE_NAME).and_where(cond))
            .execute()
            .await?;
        Ok(rows_affected)
    }
}

#[allow(async_fn_in_trait)]
pub trait DbBmcWithPkey: DbBmc {
    const PRIMARY_KEY: &'static str;
    type PkeyType: Into<EntityId>;

    fn cond_pkey(pkey: &Self::PkeyType) -> SimpleExpr
    where
        Value: for<'a> From<&'a Self::PkeyType>,
    {
        Expr::col(Self::PRIMARY_KEY).eq(pkey)
    }

    fn returning_pkey() -> ReturningClause {
        Query::returning().columns([Self::PRIMARY_KEY])
    }

    async fn create_returning_pkey(
        mm: &mut impl PrimaryStore,
        create_req: impl HasFields,
    ) -> Result<Self::PkeyType>
    where
        Self::PkeyType:
            for<'r> sqlx::Decode<'r, SqlxDatabase> + sqlx::Type<SqlxDatabase> + Send + Unpin,
    {
        let (pkey,) = mm
            .query_as_with::<(Self::PkeyType,)>(
                Query::insert()
                    .into_table(Self::TABLE_NAME)
                    .bind(create_req.not_none_fields()?)
                    .returning(Self::returning_pkey()),
            )
            .fetch_one()
            .await?;
        Ok(pkey)
    }

    async fn get<E>(mm: &mut impl PrimaryStore, id: Self::PkeyType) -> Result<E>
    where
        E: for<'r> sqlx::FromRow<'r, SqlxRow> + Send + Unpin + HasFields,
        Value: for<'e> From<&'e Self::PkeyType>,
    {
        Self::get_optional_by_expr::<E>(mm, Self::cond_pkey(&id))
            .await?
            .ok_or_else(|| Self::not_found_error(id))
    }

    async fn update(
        mm: &mut impl PrimaryStore,
        update_req: impl HasFields,
        id: Self::PkeyType,
    ) -> Result<()>
    where
        Value: for<'e> From<&'e Self::PkeyType>,
    {
        let rows_affected = Self::update_cond(mm, update_req, Self::cond_pkey(&id)).await?;
        if rows_affected == 0 {
            return Err(Self::not_found_error(id));
        }
        Ok(())
    }

    async fn delete(mm: &mut impl PrimaryStore, id: Self::PkeyType) -> Result<()>
    where
        Value: for<'e> From<&'e Self::PkeyType>,
    {
        let rows_affected = Self::delete_cond(mm, Self::cond_pkey(&id)).await?;
        if rows_affected == 0 {
            return Err(Self::not_found_error(id));
        }
        Ok(())
    }
}

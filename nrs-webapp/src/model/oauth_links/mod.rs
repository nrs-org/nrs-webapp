use sea_query::{Expr, ExprTrait, IntoColumnRef, Query, ReturningClause};
use sqlbindable::{BindContext, Fields, HasFields};
use time::OffsetDateTime;
use uuid::Uuid;

use super::Result;
use crate::model::{entity::DbBmc, store::primary_store::PrimaryStore};

pub struct OAuthLinkBmc;

impl DbBmc for OAuthLinkBmc {
    const TABLE_NAME: &'static str = "app_user_oauth_link";
}

#[derive(Fields)]
pub struct OAuthLinkForCreate {
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: Option<String>,
    pub access_token: Vec<u8>,
    pub refresh_token: Option<Vec<u8>>,
    pub access_token_expires_at: Option<OffsetDateTime>,
}

#[derive(Fields)]
pub struct OAuthLinkForUpdate {
    pub access_token: Vec<u8>,
    pub refresh_token: Option<Vec<u8>>,
    pub access_token_expires_at: Option<OffsetDateTime>,
}

impl OAuthLinkBmc {
    pub async fn update_link(
        ps: &mut impl PrimaryStore,
        provider_name: &str,
        provider_user_id: &str,
        update_req: OAuthLinkForUpdate,
    ) -> Result<Option<Uuid>> {
        let ret: Option<(Uuid,)> = ps
            .query_as_with(
                Query::update()
                    .table(Self::TABLE_NAME)
                    .bind(update_req.not_none_fields()?)
                    .and_where(Expr::col("provider_user_id").eq(provider_user_id))
                    .and_where(Expr::col("provider").eq(provider_name))
                    .and_where(Expr::col("revoked_at").is_null())
                    .returning(ReturningClause::Columns(vec!["user_id".into_column_ref()])),
            )
            .fetch_optional()
            .await?;

        Ok(ret.map(|(user_id,)| user_id))
    }

    pub async fn revoke(
        ps: &mut impl PrimaryStore,
        user_id: Uuid,
        provider_name: &str,
    ) -> Result<()> {
        #[derive(Fields)]
        struct OAuthLinkRevokePayload {
            pub revoked_at: Expr,
        }

        let payload = OAuthLinkRevokePayload {
            revoked_at: Expr::current_timestamp(),
        };

        let num_affected = Self::update_cond(
            ps,
            payload,
            Expr::col("user_id")
                .eq(user_id)
                .and(Expr::col("provider").eq(provider_name))
                .and(Expr::col("revoked_at").is_null()),
        )
        .await?;

        if num_affected == 0 {
            // TODO: maybe include provider_name in the error message?
            return Err(Self::not_found_error(user_id));
        }

        Ok(())
    }
}

use super::google_auth::verify_google;
use serde::{Deserialize, Serialize};
use worker::*;

// ---- helpers ---------------------------------------------------------------

/// D1 rejects bigint JsValues, so bind every integer column as f64 (all our ids
/// and cent amounts are well within f64's safe-integer range).
fn n(v: i64) -> wasm_bindgen::JsValue {
    (v as f64).into()
}

/// Resolve the signed-in bookkeeper from the `Authorization: Bearer <id_token>`
/// header, upserting them into `game_users`. The outer `Result` carries infra
/// errors (use `?`); the inner `Result` carries an auth failure `Response` the
/// caller should return as-is.
async fn resolve_owner(
    req: &Request,
    ctx: &RouteContext<()>,
) -> Result<std::result::Result<i64, Response>> {
    let auth = req.headers().get("Authorization")?.unwrap_or_default();
    let token = auth.strip_prefix("Bearer ").unwrap_or(auth.as_str()).trim();
    if token.is_empty() {
        return Ok(Err(Response::error("unauthorized", 401)?));
    }
    let client_id = ctx.env.var("GOOGLE_CLIENT_ID")?.to_string();
    let info = match verify_google(token, &client_id).await {
        Ok(i) => i,
        Err(resp) => return Ok(Err(resp)),
    };
    let name = info.name.unwrap_or_else(|| "anonymous".to_string());
    let email = info.email.unwrap_or_default();
    let picture = info.picture.unwrap_or_default();

    let d1 = ctx.d1("IFKASH_D1")?;
    d1.prepare(
        "INSERT INTO game_users (google_sub, email, name, picture) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(google_sub) DO UPDATE SET email = ?2, name = ?3, picture = ?4",
    )
    .bind(&[
        info.sub.clone().into(),
        email.into(),
        name.into(),
        picture.into(),
    ])?
    .run()
    .await?;

    #[derive(Deserialize)]
    struct IdRow {
        id: i64,
    }
    let row: Option<IdRow> = d1
        .prepare("SELECT id FROM game_users WHERE google_sub = ?1")
        .bind(&[info.sub.into()])?
        .first(None)
        .await?;
    match row {
        Some(r) => Ok(Ok(r.id)),
        None => Ok(Err(Response::error("could not resolve user", 500)?)),
    }
}

/// Confirm `group_id` exists and is owned by `owner_id`. Returns Ok(true) when
/// the group belongs to the owner.
async fn owns_group(d1: &D1Database, owner_id: i64, group_id: i64) -> Result<bool> {
    #[derive(Deserialize)]
    struct IdRow {
        id: i64,
    }
    let row: Option<IdRow> = d1
        .prepare("SELECT id FROM split_groups WHERE id = ?1 AND owner_id = ?2")
        .bind(&[n(group_id), n(owner_id)])?
        .first(None)
        .await?;
    Ok(row.is_some())
}

macro_rules! owner {
    ($req:expr, $ctx:expr) => {
        match resolve_owner(&$req, &$ctx).await? {
            Ok(id) => id,
            Err(resp) => return Ok(resp),
        }
    };
}

// ---- request / response shapes ---------------------------------------------

#[derive(Deserialize)]
struct CreateGroupRequest {
    name: String,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default)]
    members: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct GroupRow {
    id: i64,
    name: String,
    currency: String,
    created: String,
}

#[derive(Serialize, Deserialize)]
struct MemberRow {
    id: i64,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct ExpenseRow {
    id: i64,
    payer_id: i64,
    description: String,
    amount_cents: i64,
    split_type: String,
    created: String,
}

#[derive(Serialize, Deserialize)]
struct ShareRow {
    expense_id: i64,
    member_id: i64,
    owed_cents: i64,
}

#[derive(Serialize)]
struct GroupDetail {
    group: GroupRow,
    members: Vec<MemberRow>,
    expenses: Vec<ExpenseRow>,
    shares: Vec<ShareRow>,
}

#[derive(Deserialize)]
struct AddMemberRequest {
    name: String,
}

#[derive(Deserialize)]
struct ShareInput {
    member_id: i64,
    owed_cents: i64,
}

#[derive(Deserialize)]
struct AddExpenseRequest {
    payer_id: i64,
    amount_cents: i64,
    #[serde(default)]
    description: String,
    #[serde(default = "default_split_type")]
    split_type: String,
    shares: Vec<ShareInput>,
}

fn default_split_type() -> String {
    "equal".to_string()
}

#[derive(Serialize)]
struct IdResponse {
    id: i64,
}

// ---- handlers --------------------------------------------------------------

/// GET /api/splitter/groups — list the bookkeeper's groups.
pub async fn list_groups(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let d1 = ctx.d1("IFKASH_D1")?;
    let rows = d1
        .prepare(
            "SELECT id, name, currency, created FROM split_groups \
             WHERE owner_id = ?1 ORDER BY datetime(updated) DESC, id DESC",
        )
        .bind(&[n(owner_id)])?
        .all()
        .await?;
    let groups: Vec<GroupRow> = rows.results()?;
    Response::from_json(&groups)
}

/// POST /api/splitter/groups — create a group, optionally seeding member names.
pub async fn create_group(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let body: CreateGroupRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return Response::error("bad request", 400),
    };
    let name = body.name.trim();
    if name.is_empty() {
        return Response::error("group name required", 400);
    }
    let currency = body
        .currency
        .map(|c| c.trim().to_uppercase())
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| "INR".to_string());

    let d1 = ctx.d1("IFKASH_D1")?;
    #[derive(Deserialize)]
    struct InsertedRow {
        id: i64,
    }
    let inserted: Option<InsertedRow> = d1
        .prepare(
            "INSERT INTO split_groups (owner_id, name, currency) VALUES (?1, ?2, ?3) RETURNING id",
        )
        .bind(&[n(owner_id), name.into(), currency.into()])?
        .first(None)
        .await?;
    let group_id = inserted
        .ok_or_else(|| Error::RustError("group insert returned no id".into()))?
        .id;

    for member in body.members {
        let member = member.trim();
        if member.is_empty() {
            continue;
        }
        d1.prepare("INSERT INTO split_members (group_id, name) VALUES (?1, ?2)")
            .bind(&[n(group_id), member.into()])?
            .run()
            .await?;
    }

    Response::from_json(&IdResponse { id: group_id })
}

/// GET /api/splitter/groups/:id — full group detail (members + expenses + shares).
pub async fn get_group(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let group_id = match parse_id(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("invalid id", 400),
    };
    let d1 = ctx.d1("IFKASH_D1")?;

    let group: Option<GroupRow> = d1
        .prepare(
            "SELECT id, name, currency, created FROM split_groups WHERE id = ?1 AND owner_id = ?2",
        )
        .bind(&[n(group_id), n(owner_id)])?
        .first(None)
        .await?;
    let group = match group {
        Some(g) => g,
        None => return Response::error("not found", 404),
    };

    let members: Vec<MemberRow> = d1
        .prepare("SELECT id, name FROM split_members WHERE group_id = ?1 ORDER BY id ASC")
        .bind(&[n(group_id)])?
        .all()
        .await?
        .results()?;

    let expenses: Vec<ExpenseRow> = d1
        .prepare(
            "SELECT id, payer_id, description, amount_cents, split_type, created \
             FROM split_expenses WHERE group_id = ?1 ORDER BY datetime(created) DESC, id DESC",
        )
        .bind(&[n(group_id)])?
        .all()
        .await?
        .results()?;

    let shares: Vec<ShareRow> = d1
        .prepare(
            "SELECT s.expense_id AS expense_id, s.member_id AS member_id, s.owed_cents AS owed_cents \
             FROM split_shares s JOIN split_expenses e ON e.id = s.expense_id \
             WHERE e.group_id = ?1",
        )
        .bind(&[n(group_id)])?
        .all()
        .await?
        .results()?;

    Response::from_json(&GroupDetail {
        group,
        members,
        expenses,
        shares,
    })
}

/// DELETE /api/splitter/groups/:id — delete a group and all its data.
pub async fn delete_group(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let group_id = match parse_id(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("invalid id", 400),
    };
    let d1 = ctx.d1("IFKASH_D1")?;
    if !owns_group(&d1, owner_id, group_id).await? {
        return Response::error("not found", 404);
    }
    d1.prepare(
        "DELETE FROM split_shares WHERE expense_id IN \
         (SELECT id FROM split_expenses WHERE group_id = ?1)",
    )
    .bind(&[n(group_id)])?
    .run()
    .await?;
    d1.prepare("DELETE FROM split_expenses WHERE group_id = ?1")
        .bind(&[n(group_id)])?
        .run()
        .await?;
    d1.prepare("DELETE FROM split_members WHERE group_id = ?1")
        .bind(&[n(group_id)])?
        .run()
        .await?;
    d1.prepare("DELETE FROM split_groups WHERE id = ?1")
        .bind(&[n(group_id)])?
        .run()
        .await?;
    Response::from_json(&IdResponse { id: group_id })
}

/// POST /api/splitter/groups/:id/members — add a named member.
pub async fn add_member(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let group_id = match parse_id(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("invalid id", 400),
    };
    let body: AddMemberRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return Response::error("bad request", 400),
    };
    let name = body.name.trim();
    if name.is_empty() {
        return Response::error("member name required", 400);
    }
    let d1 = ctx.d1("IFKASH_D1")?;
    if !owns_group(&d1, owner_id, group_id).await? {
        return Response::error("not found", 404);
    }
    #[derive(Deserialize)]
    struct InsertedRow {
        id: i64,
    }
    let inserted: Option<InsertedRow> = d1
        .prepare("INSERT INTO split_members (group_id, name) VALUES (?1, ?2) RETURNING id")
        .bind(&[n(group_id), name.into()])?
        .first(None)
        .await?;
    let id = inserted
        .ok_or_else(|| Error::RustError("member insert returned no id".into()))?
        .id;
    Response::from_json(&IdResponse { id })
}

/// DELETE /api/splitter/members/:id — remove a member (only if unreferenced).
pub async fn delete_member(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let member_id = match parse_id(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("invalid id", 400),
    };
    let d1 = ctx.d1("IFKASH_D1")?;

    // The member must belong to a group owned by this bookkeeper.
    #[derive(Deserialize)]
    struct IdRow {
        id: i64,
    }
    let owned: Option<IdRow> = d1
        .prepare(
            "SELECT m.id AS id FROM split_members m \
             JOIN split_groups g ON g.id = m.group_id \
             WHERE m.id = ?1 AND g.owner_id = ?2",
        )
        .bind(&[n(member_id), n(owner_id)])?
        .first(None)
        .await?;
    if owned.is_none() {
        return Response::error("not found", 404);
    }

    #[derive(Deserialize)]
    struct CountRow {
        c: i64,
    }
    let refs: Option<CountRow> = d1
        .prepare(
            "SELECT (SELECT COUNT(*) FROM split_expenses WHERE payer_id = ?1) \
                  + (SELECT COUNT(*) FROM split_shares WHERE member_id = ?1) AS c",
        )
        .bind(&[n(member_id)])?
        .first(None)
        .await?;
    if refs.map(|r| r.c).unwrap_or(0) > 0 {
        return Response::error("member is referenced by expenses", 409);
    }

    d1.prepare("DELETE FROM split_members WHERE id = ?1")
        .bind(&[n(member_id)])?
        .run()
        .await?;
    Response::from_json(&IdResponse { id: member_id })
}

/// POST /api/splitter/groups/:id/expenses — add an expense with resolved shares.
pub async fn add_expense(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let group_id = match parse_id(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("invalid id", 400),
    };
    let body: AddExpenseRequest = match req.json().await {
        Ok(b) => b,
        Err(_) => return Response::error("bad request", 400),
    };
    if body.amount_cents <= 0 {
        return Response::error("amount must be positive", 400);
    }
    if body.shares.is_empty() {
        return Response::error("at least one share required", 400);
    }
    let share_sum: i64 = body.shares.iter().map(|s| s.owed_cents).sum();
    if share_sum != body.amount_cents {
        return Response::error("shares must sum to the amount", 400);
    }

    let d1 = ctx.d1("IFKASH_D1")?;
    if !owns_group(&d1, owner_id, group_id).await? {
        return Response::error("not found", 404);
    }

    // Validate the payer and every share member belong to this group.
    let member_ids: Vec<i64> = d1
        .prepare("SELECT id FROM split_members WHERE group_id = ?1")
        .bind(&[n(group_id)])?
        .all()
        .await?
        .results::<MemberRow>()
        .map(|rows| rows.into_iter().map(|m| m.id).collect())
        .unwrap_or_default();
    let in_group = |id: i64| member_ids.contains(&id);
    if !in_group(body.payer_id) || !body.shares.iter().all(|s| in_group(s.member_id)) {
        return Response::error("payer or share member not in group", 400);
    }

    #[derive(Deserialize)]
    struct InsertedRow {
        id: i64,
    }
    let inserted: Option<InsertedRow> = d1
        .prepare(
            "INSERT INTO split_expenses (group_id, payer_id, description, amount_cents, split_type) \
             VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id",
        )
        .bind(&[
            n(group_id),
            n(body.payer_id),
            body.description.trim().into(),
            n(body.amount_cents),
            body.split_type.into(),
        ])?
        .first(None)
        .await?;
    let expense_id = inserted
        .ok_or_else(|| Error::RustError("expense insert returned no id".into()))?
        .id;

    for share in &body.shares {
        d1.prepare(
            "INSERT INTO split_shares (expense_id, member_id, owed_cents) VALUES (?1, ?2, ?3)",
        )
        .bind(&[n(expense_id), n(share.member_id), n(share.owed_cents)])?
        .run()
        .await?;
    }

    // Bump the group's updated timestamp so it sorts to the top of the list.
    d1.prepare("UPDATE split_groups SET updated = datetime('now') WHERE id = ?1")
        .bind(&[n(group_id)])?
        .run()
        .await?;

    Response::from_json(&IdResponse { id: expense_id })
}

/// DELETE /api/splitter/expenses/:id — delete an expense and its shares.
pub async fn delete_expense(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let owner_id = owner!(req, ctx);
    let expense_id = match parse_id(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("invalid id", 400),
    };
    let d1 = ctx.d1("IFKASH_D1")?;

    #[derive(Deserialize)]
    struct IdRow {
        id: i64,
    }
    let owned: Option<IdRow> = d1
        .prepare(
            "SELECT e.id AS id FROM split_expenses e \
             JOIN split_groups g ON g.id = e.group_id \
             WHERE e.id = ?1 AND g.owner_id = ?2",
        )
        .bind(&[n(expense_id), n(owner_id)])?
        .first(None)
        .await?;
    if owned.is_none() {
        return Response::error("not found", 404);
    }

    d1.prepare("DELETE FROM split_shares WHERE expense_id = ?1")
        .bind(&[n(expense_id)])?
        .run()
        .await?;
    d1.prepare("DELETE FROM split_expenses WHERE id = ?1")
        .bind(&[n(expense_id)])?
        .run()
        .await?;
    Response::from_json(&IdResponse { id: expense_id })
}

fn parse_id(ctx: &RouteContext<()>, name: &str) -> Option<i64> {
    ctx.param(name).and_then(|s| s.parse::<i64>().ok())
}

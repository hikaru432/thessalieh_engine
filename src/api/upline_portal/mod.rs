//! Generic self-service portal API for any admin-configured upline role (Lead Broker,
//! Titling Officer, or a custom role added via /upline-role-types). Replaces what used to
//! be two near-identical modules (`api::leadbroker`, `api::titlingofficer`) with one
//! implementation parameterized by `role_slug`, so a new upline role never needs new
//! backend code — just a row in `upline_role_types`.

mod agents;
mod get_project_context;
mod guards;
mod list_commission_status;
mod list_my_projects;
mod list_project_contracts;
mod list_project_lots;
mod list_project_payments;
mod patch_lot_reserve;
mod rows;

pub use get_project_context::{UplineContextResponse, get_project_context};
pub use list_commission_status::list_commission_status;
pub use list_my_projects::list_my_projects;
pub use list_project_contracts::list_project_contracts;
pub use list_project_lots::list_project_lots;
pub use list_project_payments::{UplineCashFlowPaymentResponse, list_project_payments};
pub use patch_lot_reserve::{UplineReserveLotInput, patch_lot_reserve};

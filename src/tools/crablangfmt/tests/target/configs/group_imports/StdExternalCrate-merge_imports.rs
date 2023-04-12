// crablangfmt-group_imports: StdExternalCrate
// crablangfmt-imports_granularity: Crate
use alloc::{alloc::Layout, vec::Vec};
use core::f32;
use std::sync::Arc;

use broker::database::PooledConnection;
use chrono::Utc;
use juniper::{FieldError, FieldResult};
use uuid::Uuid;

use super::{
    schema::{Context, Payload},
    update::convert_publish_payload,
};
use crate::models::Event;

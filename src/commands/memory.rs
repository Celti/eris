use db::{MemEntry, NewMemEntry};
use db::schema::memory::dsl::*;
use diesel::prelude::*;
use key::DatabaseConnection;
use std::collections::BTreeMap;
use key::MemoryIndex;

// .recall <keyword>
command!(mem_recall(ctx, msg, args) {
    let mut data = ctx.data.lock();
    let db = data.get::<DatabaseConnection>()
        .expect("Lost connection to database!");
});

// .next [keyword]
// .prev [keyword]
// .details

// .remember <keyword> <definition>
// .forget <keyword> <definition>
// .memory <keyword> <ro|rw|or|uo>

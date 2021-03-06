# database-table

Utility crate for generic access to schemas defined using [rql](https://github.com/kaikalii/rql).

Example:
```rust
// define Schema
schema! {
    pub Schema {
        user: User,
        subscription: Subscription,
    }
}
lazy_static! {
    pub static ref DB: Schema = Schema::new("example_database", rql::BinaryStable).unwrap();
}
// define access to table for User type
impl<'db> Database<'db, User> for Schema {
    fn table() -> TableGuard<'db, User> {
        DB.user()
    }
    fn table_mut() -> TableGuardMut<'db, User> {
        DB.user_mut()
    }
}
// define access to table for Subscription type
impl<'db> Database<'db, Subscription> for Schema {
    fn table() -> TableGuard<'db, User> {
        DB.subscription()
    }
    fn table_mut() -> TableGuardMut<'db, User> {
        DB.subscription_mut()
    }
}
```
Now you can write generic functions working with any Schema definition:
```rust
// login into any Database with a table for User
pub async fn login<'db, D: Database<'db, User>>(credentials: Credentials) -> Result<UserSession, Error> {
    DatabaseTable::<'db, D>::find(|user| *user.name() == credentials.username)
        .ok_or(ErrorNotFound("User not found"))
        .and_then(|entry| {
            let user = entry.data();
            ...
        })
        ...
}
```

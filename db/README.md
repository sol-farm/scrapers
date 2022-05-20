generated crud helpers:

```
fn save(self, conn: &PgConnection) -> QueryResult<Self>;
fn find_all(conn: &PgConnection) -> QueryResult<Vec<Self>>;
fn find_one(
    conn: &PgConnection,
    id: <&'a Self as Identifiable>::Id,
) -> QueryResult<Option<Self>>;
fn exists(conn: &PgConnection, id: <&'a Self as Identifiable>::Id) -> QueryResult<bool>;
fn count_all(conn: &PgConnection) -> QueryResult<i64>;
fn destroy(self, conn: &PgConnection) -> QueryResult<()>;
```


https://gist.github.com/bonedaddy/7c14292c8cda194197cf0a690c506e7d
# v1.5.5 PRD Part 1: SQL Support Research

**Version**: 1.5.5
**Date**: 2026-02-07
**Author**: Parseltongue Research Team
**Status**: Research Phase

---

## Executive Summary

This document presents comprehensive research findings for adding SQL language support to Parseltongue v1.5.5. The key innovation is **cross-language edge detection** - connecting SQL entities (tables, views, functions) to backend code (C#, TypeScript, Python, Java, Rust) through ORM mappings, raw SQL queries, and schema definitions.

### Key Findings

1. **Tree-sitter SQL Ecosystem**: Multiple crates available, with `tree-sitter-sql` (PostgreSQL-focused) being the most suitable for Rust integration
2. **SQL Entity Types**: 10+ entity types to track (tables, views, functions, procedures, triggers, indexes, schemas, materialized views, CTEs, cursors)
3. **Cross-Language Innovation**: Novel approach to link SQL entities with ORM models, raw SQL strings, and schema files across 5+ backend languages
4. **ISGL1 v2 Extension**: Proposed key format `sql:table:users:____migrations_001:T1234567890`

---

## 1. Tree-Sitter SQL Ecosystem

### 1.1 Available Crates

| Crate | Focus | Downloads | Maintenance | tree-sitter Version |
|-------|-------|-----------|-------------|---------------------|
| [tree-sitter-sql](https://crates.io/crates/tree-sitter-sql) | PostgreSQL | 4,851 | Active | 0.20+ compatible |
| [tree-sitter-sql-bigquery](https://crates.io/crates/tree-sitter-sql-bigquery) | BigQuery/GoogleSQL | 40,051 | Active | 0.20+ compatible |
| [tree-sitter-sequel](https://crates.io/crates/tree-sitter-sequel) | General/Permissive | Low | Moderate | 0.20+ compatible |

**GitHub Repositories**:
- PostgreSQL: [m-novikov/tree-sitter-sql](https://github.com/m-novikov/tree-sitter-sql)
- MySQL/General: [DerekStride/tree-sitter-sql](https://github.com/DerekStride/tree-sitter-sql)
- SQLite: [dhcmrlchtdj/tree-sitter-sqlite](https://github.com/dhcmrlchtdj/tree-sitter-sqlite)

### 1.2 Comparison Matrix

| Feature | tree-sitter-sql (PostgreSQL) | DerekStride/tree-sitter-sql | tree-sitter-sql-bigquery |
|---------|------------------------------|------------------------------|--------------------------|
| **CREATE TABLE** | Full support | Full support | Full support |
| **CREATE VIEW** | Full support | Full support | Full support |
| **CREATE FUNCTION** | PostgreSQL syntax | General syntax | GoogleSQL syntax |
| **CREATE PROCEDURE** | PostgreSQL syntax | General syntax | Limited |
| **CREATE TRIGGER** | PostgreSQL syntax | General syntax | N/A |
| **CTEs (WITH clause)** | Full support | Full support | Full support |
| **Window Functions** | Full support | Full support | Full support |
| **Lax Parsing** | Editor-friendly | Very permissive | Strict |
| **Rust Crate** | Yes (0.0.2) | Via tree-sitter-sequel | Yes (0.2.x) |

### 1.3 Recommended Crate

**Primary Recommendation**: `tree-sitter-sql` (m-novikov/tree-sitter-sql)

**Rationale**:
1. PostgreSQL is the most feature-rich SQL dialect and serves as a good baseline
2. "Very lax in what it considers valid SQL parse" - ideal for editor/analysis use cases
3. Active maintenance with 359 stars
4. Provides convenient selection anchors for code navigation
5. MIT licensed, compatible with Parseltongue

**Secondary/Fallback**: `tree-sitter-sequel` (DerekStride) for MySQL-heavy codebases

**Installation**:
```toml
# Cargo.toml
[dependencies]
tree-sitter-sql = "0.0.2"
```

**Usage Pattern** (following Parseltongue conventions):
```rust
use tree_sitter::Parser;
use tree_sitter_sql::language;

fn create_sql_parser_instance() -> Parser {
    let mut parser = Parser::new();
    parser.set_language(language()).expect("SQL grammar load failed");
    parser
}
```

---

## 2. SQL Entity Types

### 2.1 DDL Entities (Data Definition Language)

| Entity Type | SQL Syntax | Priority | Notes |
|-------------|------------|----------|-------|
| **Table** | `CREATE TABLE` | P0 | Core entity, most referenced |
| **View** | `CREATE VIEW` | P0 | Virtual tables, dependency sources |
| **Materialized View** | `CREATE MATERIALIZED VIEW` | P1 | Cached views, PostgreSQL/Oracle |
| **Function** | `CREATE FUNCTION` | P1 | User-defined functions |
| **Procedure** | `CREATE PROCEDURE` | P1 | Stored procedures |
| **Trigger** | `CREATE TRIGGER` | P2 | Event-driven code |
| **Index** | `CREATE INDEX` | P2 | Performance structures |
| **Schema** | `CREATE SCHEMA` | P2 | Namespace organization |
| **Sequence** | `CREATE SEQUENCE` | P3 | Auto-increment sources |
| **Type** | `CREATE TYPE` | P3 | Custom types (PostgreSQL) |

### 2.2 DML Patterns (Data Manipulation Language)

These are not entities themselves but create **edges** to entities:

| Pattern | Creates Edge To | Edge Type |
|---------|-----------------|-----------|
| `SELECT ... FROM table` | Table/View | `Reads` |
| `INSERT INTO table` | Table | `Writes` |
| `UPDATE table SET` | Table | `Writes` |
| `DELETE FROM table` | Table | `Writes` |
| `JOIN table ON` | Table/View | `Reads` |
| `CALL procedure()` | Procedure | `Calls` |
| `function_name()` | Function | `Calls` |

### 2.3 Entity Hierarchy

```
sql:schema:public
  |
  +-- sql:table:users
  |     +-- sql:index:users_email_idx
  |     +-- sql:trigger:users_audit_trigger
  |
  +-- sql:view:active_users
  |
  +-- sql:function:get_user_by_email
  |
  +-- sql:procedure:sync_user_data
  |
  +-- sql:materialized_view:user_statistics
```

### 2.4 Proposed EntityType Extension

```rust
// Extension to crates/parseltongue-core/src/entities.rs
pub enum EntityType {
    // ... existing types ...

    // SQL DDL Entities
    SqlTable,
    SqlView,
    SqlMaterializedView,
    SqlFunction,
    SqlProcedure,
    SqlTrigger,
    SqlIndex,
    SqlSchema,
    SqlSequence,
    SqlType,

    // SQL Query Entities (anonymous, for edge tracking)
    SqlQuery {
        query_type: SqlQueryType,
    },
}

pub enum SqlQueryType {
    Select,
    Insert,
    Update,
    Delete,
    Merge,
}
```

---

## 3. SQL Dependency Graph

### 3.1 Edge Types

| Edge Type | From | To | Example |
|-----------|------|----|---------|
| `References` | Table (FK) | Table (PK) | `orders.user_id REFERENCES users.id` |
| `DependsOn` | View | Table | `CREATE VIEW active_users AS SELECT * FROM users` |
| `DependsOn` | View | View | Nested view dependencies |
| `DependsOn` | Materialized View | Table/View | Source data dependency |
| `Triggers` | Trigger | Table | `CREATE TRIGGER ... ON users` |
| `Calls` | Procedure | Function | `CALL get_user()` |
| `Calls` | Function | Function | Nested function calls |
| `Calls` | Trigger | Procedure | `EXECUTE PROCEDURE sync_audit()` |
| `Uses` | Index | Table | `CREATE INDEX ... ON users` |

### 3.2 Cross-Object Dependencies

```sql
-- Example: View depends on multiple tables
CREATE VIEW order_summary AS
SELECT
    o.id,
    u.name,
    p.product_name
FROM orders o
JOIN users u ON o.user_id = u.id
JOIN products p ON o.product_id = p.id;
```

**Generated Edges**:
- `sql:view:order_summary` --DependsOn--> `sql:table:orders`
- `sql:view:order_summary` --DependsOn--> `sql:table:users`
- `sql:view:order_summary` --DependsOn--> `sql:table:products`

### 3.3 Proposed EdgeType Extension

```rust
// Extension to crates/parseltongue-core/src/entities.rs
pub enum EdgeType {
    // ... existing types ...

    // SQL-specific edges
    References,      // FK -> PK relationship
    DependsOn,       // View -> Table, MView -> Source
    Triggers,        // Trigger -> Table
    Indexes,         // Index -> Table

    // Cross-language edges (SQL <-> Backend)
    MapsTo,          // ORM Model -> SQL Table
    Queries,         // Backend code -> SQL entity
    Mutates,         // Backend code -> SQL table (INSERT/UPDATE/DELETE)
}
```

---

## 4. Backend <-> SQL Integration (Key Feature)

This section documents the **critical innovation** of v1.5.5: detecting connections between SQL entities and backend code across multiple languages.

### 4.1 C# / .NET Patterns

#### Entity Framework Core

```csharp
// Detection: DbSet<T> property indicates table mapping
public class ApplicationDbContext : DbContext
{
    public DbSet<User> Users { get; set; }  // -> sql:table:users
    public DbSet<Order> Orders { get; set; } // -> sql:table:orders
}

// Detection: [Table] attribute for explicit mapping
[Table("app_users")]
public class User
{
    public int Id { get; set; }
    public string Email { get; set; }
}
```

**Detection Strategy**:
1. Parse `*.cs` files with tree-sitter-c-sharp
2. Find classes inheriting from `DbContext`
3. Extract `DbSet<T>` properties
4. Map entity name to table name (convention: pluralized)
5. Check for `[Table("name")]` attribute overrides

**Generated Edges**:
- `csharp:class:ApplicationDbContext` --MapsTo--> `sql:table:users`
- `csharp:class:User` --MapsTo--> `sql:table:app_users`

#### Dapper / ADO.NET

```csharp
// Detection: SQL strings in SqlCommand or Dapper Query
var users = await connection.QueryAsync<User>(
    "SELECT * FROM users WHERE status = @Status",  // -> sql:table:users
    new { Status = "active" }
);

using var cmd = new SqlCommand("INSERT INTO orders (user_id) VALUES (@UserId)", conn);
```

**Detection Strategy**:
1. Find string literals containing SQL keywords (SELECT, INSERT, UPDATE, DELETE)
2. Parse embedded SQL with tree-sitter-sql
3. Extract table references from parsed AST
4. Link calling function to SQL tables

### 4.2 JavaScript / TypeScript Patterns

#### TypeORM

```typescript
// Detection: @Entity decorator with table name
@Entity('users')  // -> sql:table:users
export class User {
    @PrimaryGeneratedColumn()
    id: number;

    @Column()
    email: string;
}
```

**Detection Strategy**:
1. Parse `*.ts` files with tree-sitter-typescript
2. Find classes with `@Entity()` decorator
3. Extract table name from decorator argument or class name

#### Prisma

```prisma
// schema.prisma - Detection: model blocks
model User {          // -> sql:table:User (or @@map name)
    id    Int     @id @default(autoincrement())
    email String  @unique
    posts Post[]

    @@map("users")   // Explicit table name mapping
}

model Post {
    id       Int    @id @default(autoincrement())
    authorId Int
    author   User   @relation(fields: [authorId], references: [id])
}
```

**Detection Strategy**:
1. Parse `schema.prisma` with dedicated Prisma parser ([prisma-ast](https://github.com/MrLeebo/prisma-ast) or custom grammar)
2. Extract model names and `@@map` attributes
3. Extract `@relation` for FK edges

**Third-Party Parser**: [prisma-schema-parser](https://github.com/loancrate/prisma-schema-parser) - TypeScript library based on PEG grammar

#### Sequelize

```javascript
// Detection: Model.init or define() calls
const User = sequelize.define('User', {
    email: DataTypes.STRING
}, {
    tableName: 'users'  // -> sql:table:users
});
```

#### Raw SQL in Template Literals

```typescript
// Detection: Tagged template or SQL keywords in template
const users = await db.query(sql`
    SELECT * FROM users
    WHERE created_at > ${startDate}
`);

// Or Knex-style
const result = await knex('orders')  // -> sql:table:orders
    .where('status', 'pending')
    .join('users', 'orders.user_id', 'users.id');  // -> sql:table:users
```

### 4.3 Python Patterns

#### SQLAlchemy

```python
# Detection: Table() definition or declarative base
from sqlalchemy import Table, Column, Integer, String

users = Table(
    'users',  # -> sql:table:users
    metadata,
    Column('id', Integer, primary_key=True),
    Column('email', String(255))
)

# Or declarative style
class User(Base):
    __tablename__ = 'users'  # -> sql:table:users
    id = Column(Integer, primary_key=True)
```

**Detection Strategy**:
1. Parse `*.py` files with tree-sitter-python
2. Find `Table()` calls and extract first argument
3. Find classes inheriting from declarative base with `__tablename__`

#### Django ORM

```python
# Detection: Model class with class name -> table name convention
class User(models.Model):  # -> sql:table:app_user (app_modelname)
    email = models.CharField(max_length=255)

    class Meta:
        db_table = 'users'  # Explicit override -> sql:table:users
```

**Detection Strategy**:
1. Find classes inheriting from `models.Model`
2. Check for `Meta.db_table` attribute
3. Apply Django naming convention: `{app_name}_{model_name_lower}`

#### Raw SQL (psycopg2, mysql-connector)

```python
# Detection: execute() with SQL string
cursor.execute("SELECT * FROM users WHERE id = %s", (user_id,))

# Or f-string (security risk, but exists in codebases)
cursor.execute(f"DELETE FROM orders WHERE id = {order_id}")
```

### 4.4 Java Patterns

#### JPA / Hibernate

```java
// Detection: @Entity and @Table annotations
@Entity
@Table(name = "users")  // -> sql:table:users
public class User {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "email_address")
    private String email;

    @ManyToOne
    @JoinColumn(name = "department_id", referencedColumnName = "id")
    private Department department;  // FK edge
}
```

**Detection Strategy**:
1. Parse `*.java` files with tree-sitter-java
2. Find classes with `@Entity` annotation
3. Extract `@Table(name = "...")` for table mapping
4. Extract `@JoinColumn` and `@ManyToOne`/`@OneToMany` for FK edges

**Reference**: [JPA Hibernate Annotations](https://www.digitalocean.com/community/tutorials/jpa-hibernate-annotations)

#### JDBC PreparedStatement

```java
// Detection: SQL strings in PreparedStatement
String sql = "SELECT * FROM users WHERE id = ?";
PreparedStatement stmt = conn.prepareStatement(sql);
```

#### MyBatis

```xml
<!-- Detection: SQL in mapper XML -->
<mapper namespace="com.example.UserMapper">
    <select id="getUser" resultType="User">
        SELECT * FROM users WHERE id = #{id}
    </select>
</mapper>
```

### 4.5 Rust Patterns

#### Diesel

```rust
// Detection: table! macro in schema.rs
table! {
    users (id) {  // -> sql:table:users
        id -> Integer,
        email -> Varchar,
        created_at -> Timestamp,
    }
}

// Detection: #[derive(Queryable)] structs
#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
}
```

**Detection Strategy**:
1. Parse `schema.rs` (generated by Diesel)
2. Extract `table!` macro invocations
3. First identifier = table name

#### SQLx

```rust
// Detection: query! and query_as! macros
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE id = $1",  // -> sql:table:users
    user_id
).fetch_one(&pool).await?;

// Or compile-time checked queries
let users = sqlx::query!("SELECT id, email FROM users")
    .fetch_all(&pool)
    .await?;
```

#### SeaORM

```rust
// Detection: #[sea_orm(table_name = "...")] attribute
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]  // -> sql:table:users
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub email: String,
}
```

### 4.6 Detection Strategies Summary

| Language | ORM/Library | Detection Method | Confidence |
|----------|-------------|------------------|------------|
| C# | Entity Framework | `DbSet<T>`, `[Table]` | High |
| C# | Dapper/ADO.NET | SQL string parsing | Medium |
| TypeScript | TypeORM | `@Entity()` decorator | High |
| TypeScript | Prisma | `schema.prisma` parsing | High |
| TypeScript | Sequelize | `define()` call | High |
| JavaScript | Knex | Query builder chain | Medium |
| Python | SQLAlchemy | `Table()`, `__tablename__` | High |
| Python | Django | `Meta.db_table` | High |
| Java | JPA/Hibernate | `@Table`, `@Entity` | High |
| Java | MyBatis | XML mapper parsing | High |
| Rust | Diesel | `table!` macro | High |
| Rust | SQLx | `query!` macro | Medium |
| Rust | SeaORM | `#[sea_orm]` attribute | High |

---

## 5. ISGL1 Key Format for SQL

### 5.1 Proposed Format

Following ISGL1 v2 conventions from the existing codebase:

```
{language}:{entity_type}:{name}:{semantic_path}:T{birth_timestamp}
```

**SQL-specific examples**:
```
sql:table:users:____migrations_001_create_users:T1706284800
sql:view:active_users:____views_user_views:T1706284801
sql:function:get_user_by_id:____functions_user_functions:T1706284802
sql:procedure:sync_user_data:____procedures_sync:T1706284803
sql:trigger:users_audit_trigger:____triggers_audit:T1706284804
sql:index:users_email_idx:____migrations_002_add_indexes:T1706284805
sql:schema:public:____migrations_000_init:T1706284806
sql:materialized_view:user_stats:____views_materialized:T1706284807
```

### 5.2 Entity Type Abbreviations

| Full Type | Abbreviation | Example |
|-----------|--------------|---------|
| Table | `table` | `sql:table:users` |
| View | `view` | `sql:view:active_users` |
| Materialized View | `mview` | `sql:mview:user_stats` |
| Function | `fn` | `sql:fn:get_user` |
| Procedure | `proc` | `sql:proc:sync_data` |
| Trigger | `trigger` | `sql:trigger:audit_log` |
| Index | `index` | `sql:index:users_email_idx` |
| Schema | `schema` | `sql:schema:public` |
| Sequence | `seq` | `sql:seq:users_id_seq` |
| Type | `type` | `sql:type:status_enum` |

### 5.3 Cross-Language Edge Keys

When linking backend code to SQL:

```
// Edge from C# class to SQL table
from: csharp:class:User:____models_user:T1706300000
to:   sql:table:users:____migrations_001:T1706284800
edge_type: MapsTo

// Edge from TypeScript function to SQL table
from: typescript:fn:getActiveUsers:____services_user_service:T1706400000
to:   sql:table:users:____migrations_001:T1706284800
edge_type: Queries
```

---

## 6. File Extensions & Dialect Detection

### 6.1 Supported File Extensions

| Extension | SQL Dialect | Priority | Notes |
|-----------|-------------|----------|-------|
| `.sql` | Auto-detect | P0 | Most common, requires dialect hints |
| `.psql` | PostgreSQL | P1 | PostgreSQL-specific scripts |
| `.pgsql` | PostgreSQL | P1 | Alternative PostgreSQL extension |
| `.mysql` | MySQL | P1 | MySQL-specific scripts |
| `.sqlite` | SQLite | P1 | SQLite-specific scripts |
| `.ddl` | Auto-detect | P2 | Data Definition Language files |
| `.dml` | Auto-detect | P2 | Data Manipulation Language files |
| `.prisma` | Prisma Schema | P0 | Prisma ORM schema files |

### 6.2 Dialect Detection Heuristics

When dialect cannot be determined from extension:

| Heuristic | Detected Dialect | Confidence |
|-----------|------------------|------------|
| `SERIAL` type | PostgreSQL | High |
| `AUTO_INCREMENT` | MySQL | High |
| `AUTOINCREMENT` | SQLite | High |
| `RETURNING` clause | PostgreSQL | Medium |
| `ON DUPLICATE KEY` | MySQL | High |
| `INSERT OR REPLACE` | SQLite | High |
| `::` cast operator | PostgreSQL | High |
| `CONVERT()` function | MySQL/SQL Server | Medium |
| `ILIKE` operator | PostgreSQL | High |
| `REGEXP` operator | MySQL | Medium |
| `GLOB` operator | SQLite | High |

### 6.3 Configuration File Detection

Also scan for ORM/migration configuration files:

| File Pattern | Purpose | Language |
|--------------|---------|----------|
| `schema.prisma` | Prisma schema | TypeScript/JavaScript |
| `*.entity.ts` | TypeORM entities | TypeScript |
| `models.py` | Django models | Python |
| `**/migrations/*.py` | Django migrations | Python |
| `**/migrations/*.sql` | Raw SQL migrations | Any |
| `diesel.toml` | Diesel config | Rust |
| `schema.rs` | Diesel schema | Rust |
| `persistence.xml` | JPA config | Java |

---

## 7. Implementation Roadmap

### Phase 1: Core SQL Parsing (v1.5.5-alpha)
- [ ] Add `tree-sitter-sql` dependency
- [ ] Extend `Language` enum with `Sql` variant
- [ ] Add SQL file extensions to detection
- [ ] Implement basic DDL entity extraction (tables, views)
- [ ] Add SQL entity types to `EntityType` enum

### Phase 2: SQL Dependency Graph (v1.5.5-beta)
- [ ] Parse DML statements for table references
- [ ] Detect view-to-table dependencies
- [ ] Detect FK relationships
- [ ] Add SQL-specific edge types
- [ ] Implement CTE dependency tracking

### Phase 3: Cross-Language Detection (v1.5.5-rc)
- [ ] Implement C# Entity Framework detection
- [ ] Implement TypeScript ORM detection (TypeORM, Prisma, Sequelize)
- [ ] Implement Python ORM detection (SQLAlchemy, Django)
- [ ] Implement Java JPA/Hibernate detection
- [ ] Implement Rust ORM detection (Diesel, SQLx, SeaORM)

### Phase 4: Embedded SQL Parsing (v1.5.5)
- [ ] Detect SQL strings in C# code
- [ ] Detect SQL in TypeScript template literals
- [ ] Detect SQL in Python f-strings and raw strings
- [ ] Detect SQL in Java String literals
- [ ] Detect SQL in Rust macros

### Phase 5: Polish & Optimization (v1.5.6)
- [ ] Optimize cross-language edge resolution
- [ ] Add SQL dialect auto-detection
- [ ] Implement migration file ordering
- [ ] Add schema evolution tracking

---

## 8. Open Questions

### 8.1 Technical Questions

1. **Multi-dialect Support**: Should we use multiple tree-sitter grammars or a single permissive one?
   - Recommendation: Start with `tree-sitter-sql` (PostgreSQL), add dialect-specific grammars as needed

2. **Embedded SQL Confidence**: How to handle false positives when detecting SQL in strings?
   - Recommendation: Require SQL keywords (SELECT, INSERT, CREATE) and table-like identifiers

3. **Dynamic SQL**: How to handle SQL built dynamically at runtime?
   - Recommendation: Mark as "partial" dependency with lower confidence score

4. **Schema Evolution**: How to track table renames across migrations?
   - Recommendation: Use content hashing + name matching similar to ISGL1 v2 entity matching

### 8.2 Design Questions

1. **Entity Class for SQL**: Should SQL entities have `CodeImplementation` or a new `SqlSchema` class?
   - Recommendation: Add `EntityClass::SchemaDefinition` for DDL entities

2. **Cross-Language Edge Direction**: Should edges go from backend -> SQL or SQL -> backend?
   - Recommendation: Backend code `MapsTo` SQL table (backend is the "caller/user")

3. **ORM Model vs SQL Table**: When both exist, which is the "primary" entity?
   - Recommendation: SQL table is primary (source of truth), ORM model is a reference

### 8.3 Scope Questions

1. **Database Introspection**: Should we support reading live database schemas?
   - Recommendation: Out of scope for v1.5.5; focus on file-based analysis

2. **Query Performance Analysis**: Should we analyze query complexity?
   - Recommendation: Future enhancement (v1.6.x)

3. **Migration Ordering**: Should we track migration execution order?
   - Recommendation: Use file naming conventions (timestamps, sequence numbers)

---

## Sources

### Tree-sitter SQL Ecosystem
- [tree-sitter-sql on crates.io](https://crates.io/crates/tree-sitter-sql)
- [tree-sitter-sql-bigquery on crates.io](https://crates.io/crates/tree-sitter-sql-bigquery)
- [tree-sitter-sequel on crates.io](https://crates.io/crates/tree-sitter-sequel)
- [m-novikov/tree-sitter-sql (PostgreSQL)](https://github.com/m-novikov/tree-sitter-sql)
- [DerekStride/tree-sitter-sql (General)](https://github.com/DerekStride/tree-sitter-sql)

### SQL Dependency Analysis
- [Microsoft SQL Server sys.sql_expression_dependencies](https://learn.microsoft.com/en-us/sql/relational-databases/tables/view-the-dependencies-of-a-table)
- [ApexSQL Dependency Viewer](https://solutioncenter.apexsql.com/sql-dependency-viewer/)
- [RedGate SQL Dependency Tracker](https://www.red-gate.com/hub/product-learning/sql-toolbelt-essentials/finding-dependencies-in-sql-server-databases-using-sql-dependency-tracker)
- [AWS Database Dependency Analyzer](https://github.com/aws-samples/database-dependency-analyzer)
- [Using GraphDbs for SQL Dependencies](https://dev.to/dealeron/using-graphdbs-to-visualize-code-sql-dependencies-3370)

### ORM Documentation
- [Entity Framework Core Entity Types](https://learn.microsoft.com/en-us/ef/core/modeling/entity-types)
- [JetBrains ReSharper EF Analysis](https://blog.jetbrains.com/dotnet/2023/11/20/visualize-entity-framework-relationships-and-additional-query-analysis-in-resharper-2023-3/)
- [Prisma Schema Documentation](https://www.prisma.io/docs/orm/prisma-schema)
- [prisma-ast Parser](https://github.com/MrLeebo/prisma-ast)
- [prisma-schema-parser](https://github.com/loancrate/prisma-schema-parser)
- [SQLAlchemy ORM Mapping Styles](https://docs.sqlalchemy.org/en/20/orm/mapping_styles.html)
- [JPA/Hibernate Annotations](https://docs.hibernate.org/stable/annotations/reference/en/html/entity.html)
- [Baeldung: JPA Entity Table Names](https://www.baeldung.com/jpa-entity-table-names)

### Rust ORMs
- [Diesel ORM](https://diesel.rs/)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [SQLx vs Diesel Comparison](https://blog.logrocket.com/interacting-databases-rust-diesel-vs-sqlx/)
- [Shuttle Rust ORM Guide 2025](https://www.shuttle.dev/blog/2024/01/16/best-orm-rust)

### SQL Dialects
- [Comparing SQL Dialects](https://codefinity.com/blog/Comparing-Various-SQL-Dialects)
- [Writing SQL for PostgreSQL, MySQL, SQLite](https://evertpot.com/writing-sql-for-postgres-mysql-sqlite/)
- [LearnSQL: What SQL Dialect to Learn](https://learnsql.com/blog/what-sql-dialect-to-learn/)

### Language Injection
- [tree-sitter-language-injection.nvim](https://github.com/DariusCorvus/tree-sitter-language-injection.nvim)
- [Neovim Treesitter Docs](https://neovim.io/doc/user/treesitter.html)
- [Tree-sitter SQL Injection Discussion](https://github.com/tree-sitter/tree-sitter/discussions/1577)

### SQL Parsing Tools
- [Microsoft ScriptDom T-SQL Parser](https://devblogs.microsoft.com/azure-sql/programmatically-parsing-transact-sql-t-sql-with-the-scriptdom-parser/)
- [Dapper Testing Tool](https://github.com/hryz/Dapper.Testing)
- [JetBrains Rider SQL in C# Strings](https://blog.jetbrains.com/dotnet/2018/10/29/sql-inside-c-strings-fragment-editor-run-query-console-language-injection-updates-rider-2018-3/)

### CTEs and Dependency Tracking
- [Atlassian: Using Common Table Expressions](https://www.atlassian.com/data/sql/using-common-table-expressions)
- [dbt: Getting Started with CTEs](https://www.getdbt.com/blog/getting-started-with-cte)
- [Microsoft WITH CTE Documentation](https://learn.microsoft.com/en-us/sql/t-sql/queries/with-common-table-expression-transact-sql)

---

**Document Version**: 1.0
**Last Updated**: 2026-02-07
**Next Steps**: Review with team, begin Phase 1 implementation

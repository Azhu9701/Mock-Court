# Code Summary — B2: Soul Registry

## Generated Files

| File | Lines | Purpose |
|------|-------|---------|
| `rust/foundation/src/models.rs` | +49 | Added SoulListEntry, SoulMatch, IsmismStats, IsmismSearch, IsmismCode::distance(), extended IsmismFilter + SoulGrade derives |
| `rust/registry/Cargo.toml` | 11 | Crate deps (foundation + workspace) |
| `Cargo.toml` | modified | Uncommented registry workspace member |
| `rust/registry/src/lib.rs` | 165 | SoulRegistry struct with async lifecycle + sync query + async CRUD |
| `rust/registry/src/search.rs` | 180 | Tokenizer (CJK bigram), fulltext_search, nearest_search, build_inverted_index |
| `rust/registry/src/ismism.rs` | 45 | parse_ismism, distance, distribution computation |

## API Surface

```rust
impl SoulRegistry {
    // Lifecycle (async — calls FileStore via Storage trait)
    pub async fn new(store: Arc<dyn Storage>) -> Result<Self>
    pub async fn reload(&self) -> Result<()>

    // Query (sync — reads in-memory indexes)
    pub fn list_souls(&self, filter: &IsmismFilter) -> Result<Vec<SoulListEntry>>
    pub fn get_soul(&self, name: &str) -> Result<SoulProfile>
    pub fn search_souls(&self, query: &str) -> Result<Vec<SoulMatch>>
    pub fn get_ismism_distribution(&self) -> Result<IsmismStats>

    // CRUD (async — writes through FileStore)
    pub async fn create_soul(&self, profile: SoulProfile) -> Result<()>
    pub async fn update_soul(&self, profile: SoulProfile) -> Result<()>
    pub async fn delete_soul(&self, name: &str) -> Result<()>
}
```

## Design Decisions

- **Async lifecycle + sync queries**: Storage calls (I/O) are async, but queries read from RwLock-protected HashMap (zero I/O)
- **Full index rebuild on CRUD**: After each create/update/delete, the inverted index is rebuilt from the updated HashMap
- **Nearest-neighbor via `list_souls`**: When `IsmismFilter.nearest` is set, `list_souls` uses 4D Euclidean distance instead of simple filtering
- **Graceful load failures**: `reload()` skips unparseable soul files with `tracing::warn!`

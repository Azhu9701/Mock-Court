# Domain Entities — B2: Soul Registry

> extends types defined in `foundation::models`

## New Types (add to foundation::models)

### SoulListEntry — 魂列表项

用于 `list_souls()` 返回的列表视图，比 `SoulProfile` 更轻量（不包含 summon_prompt 全文）。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulListEntry {
    pub name: String,
    pub ismism_code: String,
    pub grade: SoulGrade,
    pub field: String,
    pub tags: Vec<String>,
    pub summon_count: u32,
}
```

### SoulMatch — 搜索结果项

扩展 `SoulListEntry`，增加搜索相关度信息。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulMatch {
    pub entry: SoulListEntry,
    /// 相关度得分 (全文搜索: 关键词命中次数/位置; ismism: 归一化欧氏距离)
    pub relevance: f64,
    /// 命中的搜索字段 (如 "name", "tags", "prompt")
    pub matched_fields: Vec<String>,
}
```

### IsmismStats — ismism 分布统计

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsmismStats {
    /// 按 field 维度的魂数量分布
    pub field_distribution: HashMap<u8, usize>,
    /// 按 ontology 维度的魂数量分布
    pub ontology_distribution: HashMap<u8, usize>,
    /// 按 epistemology 维度的魂数量分布
    pub epistemology_distribution: HashMap<u8, usize>,
    /// 按 teleology 维度的魂数量分布
    pub teleology_distribution: HashMap<u8, usize>,
    /// 按品级的魂数量分布
    pub grade_distribution: HashMap<SoulGrade, usize>,
    /// 魂总数
    pub total_souls: usize,
}
```

### IsmismSearch — 最近邻搜索参数

用于 Q1 选择的最近邻搜索模式。给定目标坐标，按 4D 欧氏距离排序。

```rust
#[derive(Debug, Clone)]
pub struct IsmismSearch {
    /// 目标坐标 (field, ontology, epistemology, teleology)
    pub target: IsmismCode,
    /// 可选：每个维度的权重 (默认等权 1.0)
    pub weights: Option<(f64, f64, f64, f64)>,
    /// 可选：最大返回数量
    pub limit: Option<usize>,
}
```

## Extended Existing Types

### IsmismFilter 扩展

原有 `IsmismFilter` 支持维度字符串过滤。为支持最近邻搜索，增加可选字段：

```rust
#[derive(Debug, Clone, Default)]
pub struct IsmismFilter {
    pub field: Option<String>,
    pub ontology: Option<String>,
    pub epistemology: Option<String>,
    pub teleology: Option<String>,
    pub grade: Option<SoulGrade>,
    /// 最近邻搜索参数。如果设置，忽略 field/ontology/epistemology/teleology 过滤
    pub nearest: Option<IsmismSearch>,
}
```

### IsmismCode 扩展

增加欧氏距离计算方法：

```rust
impl IsmismCode {
    /// 计算与另一个 IsmismCode 的加权欧氏距离
    /// 各维度值域 [1,4]，最大距离 = sqrt(4 * 3²) ≈ 6.0
    pub fn distance(&self, other: &IsmismCode, weights: Option<(f64, f64, f64, f64)>) -> f64 {
        let (wf, wo, we, wt) = weights.unwrap_or((1.0, 1.0, 1.0, 1.0));
        let df = (self.field as f64 - other.field as f64) * wf;
        let od = (self.ontology as f64 - other.ontology as f64) * wo;
        let ed = (self.epistemology as f64 - other.epistemology as f64) * we;
        let td = (self.teleology as f64 - other.teleology as f64) * wt;
        (df * df + od * od + ed * ed + td * td).sqrt()
    }
}
```

## Entity Relationships

```
Registry ──1:N──> RegistryEntry (YAML, 已定义)
   │
   └── 运行时加载为 ──> SoulRegistry
                           │
                           ├── list_souls() ──> Vec<SoulListEntry>
                           ├── get_soul()  ──> SoulProfile
                           ├── search_souls() ──> Vec<SoulMatch>
                           └── get_ismism_distribution() ──> IsmismStats
```

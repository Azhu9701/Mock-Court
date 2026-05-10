# NFR Design Patterns — B5: Archive & Analytics

## Pattern 1: Stats Cache（统计缓存）

**问题**: 重复查询统计时避免频繁 SQL 聚合。

**方案**: TTL 内存缓存 `HashMap<(CacheKey, Option<Instant>)>`。

```
StatsCache<T> {
    data: RwLock<Option<(T, Instant)>>,
    ttl: Duration,
}

impl<T: Clone> StatsCache<T> {
    fn get_or_compute(&self, compute: impl FnOnce() -> Result<T>) -> Result<T> {
        if let Some((data, ts)) = self.data.read().unwrap().as_ref() {
            if ts.elapsed() < self.ttl { return Ok(data.clone()); }
        }
        let data = compute()?;
        *self.data.write().unwrap() = Some((data.clone(), Instant::now()));
        Ok(data)
    }
}
```

## Pattern 2: Async Export（异步导出）

**问题**: 大量数据导出时如何不阻塞 HTTP 响应。

**方案**: `tokio::spawn` 后台任务 + `task_id` 轮询。

```
export_archive():
  1. task_id = Uuid::new_v4()
  2. status_map.insert(task_id, ExportStatus::Pending)
  3. tokio::spawn(build_export(task_id))
  4. return task_id

build_export(task_id):
  1. status = Running
  2. 查询所有 sessions + messages + call_records
  3. serde_json::to_writer_pretty(tmp_path)
  4. std::fs::rename(tmp_path, final_path)
  5. status = Complete(final_path)

export_status(task_id):
  status_map.get(task_id) → Pending/Running/Complete(path)/Failed(e)
```

## Pattern 3: Dual-Write CallRecord（继承 B1）

**问题**: CallRecord 双写 SQLite + YAML。

**方案**: 委托 `Storage::record_call()`，B1 已实现 dual-write。

## Pattern 4: Verify by Expected Files（预期文件校验）

**问题**: 如何确定一次会话应该有哪些存档文件。

**方案**: 按 `PossessionMode` 计算预期文件列表。

```
expected_files(session: &Session) -> Vec<String>:
  match session.mode:
    Single → ["{soul}.md", "{soul}_record.md"]
    Conference → souls.map("{soul}.md") + ["synthesis.md"]
    Debate → ["{a}.md", "{b}.md", "verdict.md"]
    Relay → souls.map("{soul}.md")
    Learn → ["learning_output.md"]
    PracticeOpening → ["P1_field.md", "P2_digestion.md", "P3_revision.md", "P4_action.md"]

verify_archive(session_id):
  1. session = store.get_session(session_id)
  2. expected = expected_files(&session)
  3. for file in expected: check exists
  4. return ArchiveVerification { ok, expected_files, found_files, missing_files }
```

## Pattern 5: Cached Mode Distribution（缓存模式分布）

**问题**: Dashboard 需要频繁查询模式分布。

**方案**: 与 SummonStats 共享 TTL 缓存机制。

granitedb — Year Plan & Monthly TODOs

A from-scratch plan to build a single‑node LSM-based embedded DB in Rust, sized for ~6–10 hrs/weekend. Milestones: v0.1 in ~4 months, v0.3 in ~8 months, v1.0 by ~12 months.
High‑Level Roadmap (What We’ll Ship)
	•	v0.1 (Months 1–4): WAL + memtable → SSTables, reads across tables, tombstones, size‑tiered compaction, crash recovery, basic benches, docs.
	•	v0.2 (Months 5–6): table cache, block compression, leveled compaction, range scans, snapshots, write batches, metrics.
	•	v0.3 (Months 7–8): concurrency (background compaction), iterators with bounds, options tuning (bloom bits/key, block size), CLI tooling.
	•	v1.0 (Months 9–12): stability runs, fuzz/property tests, durable fsync strategy, format versioning, API cleanup, docs site, perf report.

Core Design (Locked to Avoid Bikeshedding)
	•	LSM shape
	•	WAL (append‑only)
	•	memtable = BTreeMap<Vec<u8>, Vec<u8>>, immutable memtable queue
	•	SSTables with:
	•	Data blocks (16–32KB) w/ restart points + prefix compression
	•	Per‑block Bloom filters (10–14 bits/key) in a filter block
	•	Index block (sparse: first key of each data block → file offset)
	•	Metaindex block → filter location
	•	Footer {index_off, meta_off, magic, version}
	•	CRC32 checksums per block; file magic/version
	•	Compaction: start size‑tiered, later add leveled (L0→L1…)
	•	Deletes via tombstones; purge during compaction
	•	API: put, get, delete, scan(range), write_batch
	•	Recovery: replay WAL → memtable; discover live SSTables by manifest
	•	Manifest: append‑only metadata of table set + seqno

⸻

Month‑by‑Month Plan

Month 1 — Skeleton + Single Table I/O (v0.0.1)

Goals
	•	Repo scaffolding, crate layout, CI, basic style rules
	•	WAL + memtable; flush a single SSTable; simple reads

Weekends
	1.	Project setup
	•	Add workspace scaffolding & modules: engine/, table/, wal/, format/, iter/, options/
	•	Add deps: anyhow, thiserror, crc32fast, rand, criterion, proptest, tracing (add zstd later)
	•	CI: fmt --check, clippy -D warnings, tests
	2.	WAL v1
	•	Record layout: seqno | klen | vlen | key | val | crc32
	•	Write/append, fsync on commit (configurable)
	•	Basic recovery test
	3.	Memtable + Flush
	•	Memtable = BTreeMap with flush threshold (e.g., 32MB)
	•	Naive SSTable writer: sorted entries → 1 data block + 1 index + footer
	4.	Read Path v1
	•	File reader + index lookup
	•	get from memtable → SSTable fallback
	•	Tests: happy path + basic corruption

Deliverables
	•	put/get for single SSTable
	•	WAL replay on startup
	•	README with “hello world”
	•	Tag v0.0.1

⸻

Month 2 — Real SSTables + Multi‑Table Reads (v0.0.2)

Goals
	•	Block format with restart points, sparse index, Bloom filters
	•	Multiple SSTables + merge iterator; tombstones

Weekends
	1.	Block Format
	•	Data block w/ restart array; prefix‑compressed keys
	•	Microbench encode/decode throughput; avg block fan‑out
	2.	Filters + Index
	•	Per‑block Bloom; pluggable filter policy
	•	Sparse index → binary search → block read
	3.	Multi‑SSTable Merge Iterator
	•	K‑way heap merge across memtable + SSTables
	•	Last‑write‑wins by seqno
	4.	Deletes / Tombstones
	•	Write tombstones on delete
	•	Read path honors tombstones

Deliverables
	•	put/get/delete/scan across many SSTables
	•	Criterion benches: point read, short range scan, write throughput
	•	Tag v0.0.2

⸻

Month 3 — Compaction + Recovery + Manifest (v0.0.3)

Goals
	•	Size‑tiered compaction (manual trigger; single thread)
	•	Solid crash‑recovery: WAL replay + manifest truth
	•	Durability toggles (every‑write fsync vs periodic)

Weekends
	1.	Manifest
	•	Track live tables, (future) levels, current seqno, next file id
	•	Atomic replace‑on‑rename for manifest snapshots
	2.	Compaction v1 (Size‑tiered)
	•	Pick K same‑size tables → merge → 1 bigger
	•	Drop tombstoned keys; verify ordering
	3.	Crash Testing
	•	Fault injection: crash between table emit & manifest swap
	•	Recovery invariants; proptest seeds
	4.	Bench + Docs
	•	Measure write amp, compaction time; document tradeoffs

Deliverables
	•	Stable recovery across crashes
	•	Compaction reduces read amplification
	•	Tag v0.0.3

⸻

Month 4 — Polish to v0.1.0

Goals
	•	Range scans with bounds; basic options (block size, bloom bits/key)
	•	CLI: kvd put/get/del/scan, kvd compact, kvd stats
	•	README: architecture diagram, file format sketch, quickstart

Weekends
	1.	Iterators & Ranges
	•	Forward iterator w/ seek, seek_prefix, upper_bound
	2.	Options & Metrics
	•	Counters for bytes read/written, (stub) block cache misses, compaction stats
	3.	CLI + Docs
	•	Friendly errors; sample datasets; animated GIF of CLI usage
	4.	Hardening Pass
	•	Fuzz small tables; property tests for iterator ordering; perf sanity

Deliverables
	•	v0.1.0 release
	•	CHANGELOG
	•	docs.rs
	•	Basic benches in README

⸻

Month 5 — Performance & Cache (v0.1.1 → v0.2.0)

Goals
	•	Table cache (LRU of file handles + block cache)
	•	Compression (zstd) for data blocks (+ checksum after compression)
	•	Microbench comparison vs sled/rocksdb (1–2 scenarios, caveats noted)

Weekends
	•	Table + block cache; pin hot index/filter
	•	Pluggable compression trait; zstd on/off; size & speed measurements
	•	Pick sane defaults (block size, bloom bits, memtable size); document effects
	•	Bench report; regressions CI job

Deliverables
	•	v0.2.0 with cache+compression
	•	Performance doc

⸻

Month 6 — Leveled Compaction + Snapshots + Write Batches

Goals
	•	Leveled compaction (L0→L1, target sizes) to reduce read amp
	•	Snapshots (seqno pinning) & write_batch with WAL group commit

Weekends
	•	Level size targets; compaction picking; overlap checks
	•	Snapshots: hold read view by seqno; iterator respects upper bound
	•	Write batches: single WAL record; apply with one seq range
	•	Tests for snapshot + compaction interaction

Deliverables
	•	v0.2.x feature set

⸻

Month 7 — Background Workers + Observability

Goals
	•	Background compaction worker; graceful shutdown; progress metrics
	•	tracing spans for read/write/compaction; Prometheus exporter

Weekends
	•	BG compaction via channels; priority; avoid stop‑the‑world
	•	Tracing spans; counters/gauges; /metrics
	•	Operational CLI: kvd stats, kvd diag, kvd dump-sstable

Deliverables
	•	Responsive foreground ops under compaction; ops visibility

⸻

Month 8 — Iterators++ & Corner Cases (v0.3.0)

Goals
	•	Backward iterator; prefix/range optimizing; large‑key handling
	•	Cold start tuning (open many tables without thrash)

Weekends
	•	Backward scan mechanics; prefix seeks
	•	Large key/value spill tests
	•	Startup: lazy open vs prefetch index/filter

Deliverables
	•	v0.3.0: complete iterator suite; smoother cold start

⸻

Month 9 — Correctness at Scale

Goals
	•	Long‑running soaks (48–72h)
	•	Fuzzing (cargo‑fuzz) on block decoder & footer
	•	Disk fault simulations (short writes, torn pages) + recovery proofs

Weekends
	•	Fuzz harnesses; coverage tracking
	•	Chaos scripts: kill at random points during compaction & manifest rotate
	•	Document invariants & recovery guarantees

⸻

Month 10 — Format Versioning & API Cleanup

Goals
	•	Format version field; guarded feature flips (e.g., new compression)
	•	Ergonomic API (builder pattern, DBOptions, typed keys option)
	•	Minor index format upgrade if needed behind version

Weekends
	•	Version negotiation; migration note
	•	Public API review; docs pass

⸻

Month 11 — Docs Site & Examples

Goals
	•	mdBook/MkDocs: how it works, file format, tuning guide
	•	End‑to‑end examples (tiny kv, simple queue, config store)
	•	Operating guide (backups, compaction advice, limits)

Weekends
	•	Write & publish docs; diagrams
	•	Examples folder + README cross‑links

⸻

Month 12 — 1.0 Release & Perf Report

Goals
	•	Final perf runs; compare versions; lock defaults
	•	Triage/post‑1.0 roadmap
	•	Short demo recording; publish v1.0.0

Weekends
	•	Reproducible benchmarks & plots
	•	Finalize CHANGELOG; “year in review” post
	•	Publish & announce

⸻

Definition of Done (Per Milestone)
	•	Feature complete for that month
	•	Tests: unit + property where meaningful; recovery tests after Month 3
	•	Docs: README always current; versioned CHANGELOG after v0.1
	•	Benchmarks: Criterion runs for critical paths, checked into repo
	•	Tagged release at v0.1, v0.2, v0.3, v1.0

⸻

Risks & How We Handle Them
	•	Time crunch: each month has a thin slice; skip nice‑to‑haves if needed
	•	Perf rabbit holes: benchmark after correctness; log wins, don’t chase micro‑wins early
	•	Design churn: format versioning in Month 10 allows safe evolution

⸻

Next Up (Immediate)
	•	Create GitHub issues for Month 1 checklist
	•	Wire CI (fmt, clippy, test) on PRs
	•	Commit this TODO.md and pin it to the repo homepage

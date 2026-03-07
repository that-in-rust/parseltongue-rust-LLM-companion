# v152 Async Pipeline Architecture Diagrams

**Version**: 1.5.2
**Date**: 2026-02-07

---

## Table of Contents

1. [Current Architecture (v1.4.7)](#current-architecture-v147)
2. [Target Architecture (v1.5.2)](#target-architecture-v152)
3. [Write Queue Pattern](#write-queue-pattern)
4. [Streaming Pipeline](#streaming-pipeline)
5. [Database Migration Path](#database-migration-path)

---

## Current Architecture (v1.4.7)

### Sequential File Processing

```mermaid
flowchart TB
    subgraph CURRENT["v1.4.7 Sequential Architecture"]
        direction TB

        subgraph WALK["Directory Walk (WalkDir)"]
            FILE1[File 1]
            FILE2[File 2]
            FILE3[File 3]
        end

        subgraph PROCESS["Sequential Processing"]
            READ1[Read File 1]
            PARSE1[Parse File 1]
            LSP1[LSP Enrich 1 - Sequential]
            BATCH1[Accumulate Batch 1]
            WRITE1[Write Batch 1 - BLOCKS]

            READ2[Read File 2]
            PARSE2[Parse File 2]
            LSP2[LSP Enrich 2 - Sequential]
            BATCH2[Accumulate Batch 2]
            WRITE2[Write Batch 2 - BLOCKS]
        end

        subgraph DB["RocksDB (Single Writer)"]
            LOCK[LOCK File]
            DATA[Data Files]
        end
    end

    FILE1 --> READ1 --> PARSE1 --> LSP1 --> BATCH1 --> WRITE1
    WRITE1 --> LOCK
    WRITE1 --> READ2
    READ2 --> PARSE2 --> LSP2 --> BATCH2 --> WRITE2
    WRITE2 --> LOCK
```

### Problems with Current Architecture

```mermaid
flowchart LR
    subgraph BLOCKERS["Blocking Issues"]
        direction TB

        B1[Sequential File Processing<br/>Only 1 file at a time]
        B2[Sequential LSP Enrichment<br/>Await each hover request]
        B3[Batch Accumulation<br/>Memory grows with file size]
        B4[Blocking Database Writes<br/>Blocks Tokio worker threads]
        B5[Single Writer Lock<br/>Cannot parallelize writes]
    end

    B1 -.->|Causes| SLOW1[Slow Ingestion]
    B2 -.->|Causes| SLOW2[LSP Bottleneck]
    B3 -.->|Causes| MEM[Memory Spikes]
    B4 -.->|Causes| THREAD[Thread Pool Exhaustion]
    B5 -.->|Causes| SERIAL[Forced Serialization]
```

---

## Target Architecture (v1.5.2)

### Async Pipeline with Write Queue

```mermaid
flowchart TB
    subgraph TARGET["v1.5.2 Async Pipeline Architecture"]
        direction TB

        subgraph CONCURRENT["Concurrent File Processing"]
            direction LR

            subgraph TASK1["Task 1"]
                READ1[Read File 1<br/>Tokio async I/O]
                PARSE1[Parse File 1<br/>spawn_blocking]
                LSP1[LSP Enrich 1<br/>join_all parallel]
            end

            subgraph TASK2["Task 2"]
                READ2[Read File 2<br/>Tokio async I/O]
                PARSE2[Parse File 2<br/>spawn_blocking]
                LSP2[LSP Enrich 2<br/>join_all parallel]
            end

            subgraph TASKN["Task N"]
                READN[Read File N<br/>Tokio async I/O]
                PARSEN[Parse File N<br/>spawn_blocking]
                LSPN[LSP Enrich N<br/>join_all parallel]
            end
        end

        subgraph QUEUE["Write Queue"]
            CHANNEL[mpsc::unbounded_channel]
            CMDS[WriteCommand Queue]
        end

        subgraph WRITER["Single Writer Task"]
            RECV[Receive Command]
            EXEC[Execute Write]
            RESP[Send Response]
        end

        subgraph DB["Database (Any Backend)"]
            ROCKS[RocksDB]
            SQLITE[SQLite WAL]
            POSTGRES[PostgreSQL]
        end
    end

    READ1 --> PARSE1 --> LSP1 --> CHANNEL
    READ2 --> PARSE2 --> LSP2 --> CHANNEL
    READN --> PARSEN --> LSPN --> CHANNEL

    CHANNEL --> CMDS --> RECV --> EXEC --> RESP
    EXEC --> ROCKS
    EXEC --> SQLITE
    EXEC --> POSTGRES
```

### Benefits of New Architecture

```mermaid
flowchart LR
    subgraph BENEFITS["Architecture Benefits"]
        direction TB

        B1[Concurrent File Processing<br/>N files in parallel]
        B2[Parallel LSP Enrichment<br/>Nx faster]
        B3[Streaming Pipeline<br/>Constant memory]
        B4[Non-Blocking Writes<br/>Async API]
        B5[Multi-Backend Support<br/>RocksDB/SQLite/PostgreSQL]
    end

    B1 -.->|Achieves| FAST1[2x+ Throughput]
    B2 -.->|Achieves| FAST2[Nx LSP Speed]
    B3 -.->|Achieves| MEM[Constant Memory]
    B4 -.->|Achieves| ASYNC[Clean Async API]
    B5 -.->|Achieves| FLEX[Deployment Flexibility]
```

---

## Write Queue Pattern

### Request-Response Flow

```mermaid
sequenceDiagram
    participant Task1 as Task 1
    participant Task2 as Task 2
    participant Queue as Write Queue
    participant Writer as Writer Task
    participant DB as Database

    Task1->>Queue: InsertEntity(entity1)
    Note over Queue: Create oneshot<br/>response channel

    Task2->>Queue: InsertEntity(entity2)
    Note over Queue: Create oneshot<br/>response channel

    Queue->>Writer: WriteCommand::InsertEntity<br/>{entity1, response_tx}

    Writer->>DB: db.run_script(...)
    DB-->>Writer: Ok(())

    Writer->>Task1: response_tx.send(Ok(()))
    Task1->>Task1: Continue processing

    Queue->>Writer: WriteCommand::InsertEntity<br/>{entity2, response_tx}

    Writer->>DB: db.run_script(...)
    DB-->>Writer: Ok(())

    Writer->>Task2: response_tx.send(Ok(()))
    Task2->>Task2: Continue processing
```

### Write Queue Implementation

```mermaid
flowchart TB
    subgraph IMPL["Write Queue Implementation"]
        direction TB

        subgraph STORAGE["CozoDbStorage"]
            DB_HANDLE[db: Arc DbInstance]
            WRITE_TX[write_tx: mpsc::Sender]
        end

        subgraph COMMAND["WriteCommand Enum"]
            INSERT_ONE[InsertEntity<br/>entity + oneshot]
            INSERT_BATCH[InsertBatch<br/>entities + oneshot]
            INSERT_EDGE[InsertEdge<br/>edge + oneshot]
            SHUTDOWN[Shutdown]
        end

        subgraph WRITER_TASK["Writer Task Loop"]
            RECV[recv.await]
            MATCH[match cmd]
            EXEC_INSERT[Execute Insert]
            EXEC_BATCH[Execute Batch]
            SEND_RESP[Send Response]
        end
    end

    WRITE_TX --> COMMAND
    COMMAND --> RECV
    RECV --> MATCH
    MATCH --> EXEC_INSERT
    MATCH --> EXEC_BATCH
    EXEC_INSERT --> SEND_RESP
    EXEC_BATCH --> SEND_RESP
    SEND_RESP --> DB_HANDLE
```

---

## Streaming Pipeline

### Entity Processing Flow

```mermaid
flowchart TB
    subgraph STREAMING["Streaming Entity Pipeline"]
        direction TB

        subgraph PARSE["Parse Stage"]
            READ[Read File Content]
            TREE_SITTER[Tree-Sitter Parse]
            ENTITIES[Extract Entities]
        end

        subgraph ENRICH["Enrich Stage (Parallel)"]
            LSP1[LSP Hover 1]
            LSP2[LSP Hover 2]
            LSPN[LSP Hover N]
            JOIN[join_all results]
        end

        subgraph TRANSFORM["Transform Stage"]
            CONVERT[ParsedEntity → CodeEntity]
            ISGL1[Generate ISGL1 Key]
            METADATA[Add Metadata]
        end

        subgraph CHANNEL_FLOW["Channel Flow"]
            ENTITY_TX[entity_tx.send]
            ENTITY_RX[entity_rx.recv]
            BATCH[Batch Accumulator]
        end

        subgraph PERSIST["Persist Stage"]
            WRITE_QUEUE[Write Queue]
            DB_WRITE[Database Write]
        end
    end

    READ --> TREE_SITTER --> ENTITIES
    ENTITIES --> LSP1
    ENTITIES --> LSP2
    ENTITIES --> LSPN
    LSP1 --> JOIN
    LSP2 --> JOIN
    LSPN --> JOIN
    JOIN --> CONVERT --> ISGL1 --> METADATA
    METADATA --> ENTITY_TX --> ENTITY_RX --> BATCH
    BATCH --> WRITE_QUEUE --> DB_WRITE
```

### Memory Usage Pattern

```mermaid
flowchart LR
    subgraph MEMORY["Memory Usage Over Time"]
        direction TB

        subgraph OLD["v1.4.7 Batch Accumulation"]
            M1[Start: 100MB]
            M2[File 1: 150MB]
            M3[File 2: 200MB]
            M4[File 3: 250MB]
            M5[Batch Write: 100MB - SPIKE]
        end

        subgraph NEW["v1.5.2 Streaming"]
            N1[Start: 100MB]
            N2[Steady: 110MB]
            N3[Steady: 110MB]
            N4[Steady: 110MB]
            N5[Steady: 110MB - CONSTANT]
        end
    end

    M1 --> M2 --> M3 --> M4 --> M5
    N1 --> N2 --> N3 --> N4 --> N5

    M5 -.->|Problem| SPIKE[Memory Spikes]
    N5 -.->|Solution| CONST[Constant Memory]
```

---

## Database Migration Path

### Incremental Backend Migration

```mermaid
flowchart LR
    subgraph MIGRATION["Database Migration Timeline"]
        direction TB

        subgraph V147["v1.4.7"]
            ROCKS_ONLY[RocksDB Only<br/>Hard-coded]
        end

        subgraph V152["v1.5.2"]
            MULTI_BACKEND[Multi-Backend Support<br/>RocksDB default]
            FEATURE_FLAGS[Feature Flags<br/>rocksdb, sqlite, postgres]
        end

        subgraph V153["v1.5.3 (Optional)"]
            SQLITE_DEFAULT[SQLite WAL Default<br/>Embedded deployment]
        end

        subgraph V160["v1.6.0"]
            POSTGRES_DEFAULT[PostgreSQL Default<br/>Multi-writer production]
        end
    end

    ROCKS_ONLY --> MULTI_BACKEND
    MULTI_BACKEND --> SQLITE_DEFAULT
    SQLITE_DEFAULT --> POSTGRES_DEFAULT
```

### Backend Comparison Matrix

```mermaid
flowchart TB
    subgraph COMPARISON["Database Backend Comparison"]
        direction LR

        subgraph ROCKSDB["RocksDB"]
            R1[Single Writer<br/>LOCK file]
            R2[Fast Writes<br/>~1ms]
            R3[Embedded<br/>No server]
            R4[Current Default]
        end

        subgraph SQLITE["SQLite WAL"]
            S1[Single Writer<br/>Concurrent Reads]
            S2[Fast Writes<br/>~1ms]
            S3[Embedded<br/>No server]
            S4[Quick Win]
        end

        subgraph POSTGRES["PostgreSQL"]
            P1[Multi-Writer<br/>MVCC]
            P2[Moderate Writes<br/>~3ms]
            P3[Server Required<br/>Port 5432]
            P4[Production Target]
        end
    end

    ROCKSDB -.->|Upgrade| SQLITE
    SQLITE -.->|Upgrade| POSTGRES

    R1 -.->|Limitation| BLOCK1[Cannot Parallelize]
    S1 -.->|Better| BLOCK2[Readers Don't Block]
    P1 -.->|Best| BLOCK3[True Concurrency]
```

### Connection String Routing

```mermaid
flowchart TB
    subgraph ROUTING["Connection String Routing"]
        direction TB

        START[User Provides<br/>Connection String]

        PARSE{Parse Prefix}

        ROCKS["rocksdb:path/to/db"<br/>→ RocksDB Backend]
        SQLITE["sqlite:path/to/db"<br/>→ SQLite Backend]
        POSTGRES["postgres://host/db"<br/>→ PostgreSQL Backend]
        MEM["mem"<br/>→ In-Memory Backend]

        ROCKS_INIT[DbInstance::new<br/>engine=rocksdb]
        SQLITE_INIT[DbInstance::new<br/>engine=sqlite]
        POSTGRES_INIT[DbInstance::new<br/>engine=postgres]
        MEM_INIT[DbInstance::new<br/>engine=mem]

        STORAGE[CozoDbStorage<br/>Ready]
    end

    START --> PARSE
    PARSE -->|rocksdb:| ROCKS --> ROCKS_INIT
    PARSE -->|sqlite:| SQLITE --> SQLITE_INIT
    PARSE -->|postgres://| POSTGRES --> POSTGRES_INIT
    PARSE -->|mem| MEM --> MEM_INIT

    ROCKS_INIT --> STORAGE
    SQLITE_INIT --> STORAGE
    POSTGRES_INIT --> STORAGE
    MEM_INIT --> STORAGE
```

---

## Concurrency Patterns

### File Processing Concurrency

```mermaid
flowchart TB
    subgraph CONCURRENCY["Concurrent File Processing"]
        direction TB

        subgraph DISCOVERY["Directory Discovery"]
            WALK[WalkDir Iterator]
            FILES[File List]
        end

        subgraph EXECUTOR["FuturesUnordered Executor"]
            TASK1[Task 1: File A]
            TASK2[Task 2: File B]
            TASK3[Task 3: File C]
            TASKN[Task N: File Z]

            LIMIT[Concurrency Limit<br/>MAX_CONCURRENT_FILES]
        end

        subgraph BACKPRESSURE["Backpressure Control"]
            CHECK{Queue Full?}
            WAIT[Wait for Completion]
            SPAWN[Spawn New Task]
        end
    end

    WALK --> FILES
    FILES --> CHECK
    CHECK -->|Yes| WAIT --> CHECK
    CHECK -->|No| SPAWN --> TASK1
    SPAWN --> TASK2
    SPAWN --> TASK3
    SPAWN --> TASKN

    TASK1 --> LIMIT
    TASK2 --> LIMIT
    TASK3 --> LIMIT
    TASKN --> LIMIT
```

### LSP Enrichment Concurrency

```mermaid
flowchart TB
    subgraph LSP_PARALLEL["Parallel LSP Enrichment"]
        direction TB

        subgraph ENTITIES["Parsed Entities"]
            E1[Entity 1<br/>Function foo]
            E2[Entity 2<br/>Struct Bar]
            E3[Entity 3<br/>Impl Baz]
            EN[Entity N]
        end

        subgraph FUTURES["LSP Futures"]
            F1[hover entity1]
            F2[hover entity2]
            F3[hover entity3]
            FN[hover entityN]
        end

        subgraph JOIN["join_all"]
            AWAIT[await all futures]
            RESULTS[Vec Option LspMetadata]
        end

        subgraph ZIP["Zip Results"]
            COMBINE[entities.zip results]
            ENRICHED[Enriched CodeEntities]
        end
    end

    E1 --> F1
    E2 --> F2
    E3 --> F3
    EN --> FN

    F1 --> AWAIT
    F2 --> AWAIT
    F3 --> AWAIT
    FN --> AWAIT

    AWAIT --> RESULTS --> COMBINE --> ENRICHED
```

---

## Error Handling

### Graceful Degradation

```mermaid
flowchart TB
    subgraph ERROR_HANDLING["Error Handling Strategy"]
        direction TB

        subgraph LSP_ERROR["LSP Enrichment Errors"]
            LSP_REQ[LSP Hover Request]
            LSP_TIMEOUT{Timeout?}
            LSP_UNAVAIL{Server Down?}
            LSP_FALLBACK[Continue Without LSP<br/>metadata = None]
        end

        subgraph PARSE_ERROR["Parse Errors"]
            PARSE[Tree-Sitter Parse]
            PARSE_FAIL{Parse Failed?}
            SKIP_FILE[Skip File<br/>Log Error]
            CONTINUE[Continue Next File]
        end

        subgraph WRITE_ERROR["Write Errors"]
            WRITE[Write to Queue]
            WRITE_FAIL{Write Failed?}
            RETRY[Retry 3x]
            GIVE_UP[Log Error<br/>Continue]
        end
    end

    LSP_REQ --> LSP_TIMEOUT
    LSP_TIMEOUT -->|Yes| LSP_FALLBACK
    LSP_TIMEOUT -->|No| LSP_UNAVAIL
    LSP_UNAVAIL -->|Yes| LSP_FALLBACK

    PARSE --> PARSE_FAIL
    PARSE_FAIL -->|Yes| SKIP_FILE --> CONTINUE

    WRITE --> WRITE_FAIL
    WRITE_FAIL -->|Yes| RETRY
    RETRY --> WRITE_FAIL
    WRITE_FAIL -->|No Retry| GIVE_UP
```

---

## Performance Characteristics

### Throughput Comparison

```mermaid
flowchart LR
    subgraph THROUGHPUT["Ingestion Throughput"]
        direction TB

        subgraph V147_PERF["v1.4.7 Sequential"]
            SEQ1[1 file at a time]
            SEQ2[~500 files/sec]
            SEQ3[LSP: Sequential<br/>N × 10ms]
        end

        subgraph V152_PERF["v1.5.2 Async Pipeline"]
            ASYNC1[N files concurrent]
            ASYNC2[~1000 files/sec]
            ASYNC3[LSP: Parallel<br/>10ms total]
        end

        subgraph V160_PERF["v1.6.0 PostgreSQL"]
            MULTI1[N files + N writers]
            MULTI2[~2700 files/sec]
            MULTI3[LSP: Parallel<br/>+ Concurrent Writes]
        end
    end

    SEQ2 -.->|2x Improvement| ASYNC2
    ASYNC2 -.->|2.7x Improvement| MULTI2

    SEQ1 -.->|Bottleneck| SLOW1[Single Thread]
    ASYNC1 -.->|Improvement| FAST1[Parallel Tasks]
    MULTI1 -.->|Optimal| FAST2[True Concurrency]
```

### Latency Percentiles

```mermaid
flowchart TB
    subgraph LATENCY["Query Latency Percentiles"]
        direction LR

        subgraph ROCKS_LAT["RocksDB"]
            R_P50[p50: 0.1ms]
            R_P99[p99: 0.3ms]
            R_MAX[max: 1ms]
        end

        subgraph SQLITE_LAT["SQLite WAL"]
            S_P50[p50: 0.2ms]
            S_P99[p99: 0.5ms]
            S_MAX[max: 2ms]
        end

        subgraph POSTGRES_LAT["PostgreSQL"]
            P_P50[p50: 0.8ms]
            P_P99[p99: 2.1ms]
            P_MAX[max: 5ms]
        end
    end

    ROCKS_LAT -.->|Fastest| EMBEDDED[Embedded Speed]
    SQLITE_LAT -.->|Fast| EMBEDDED
    POSTGRES_LAT -.->|Acceptable| NETWORK[Network Overhead]
```

---

## Conclusion

### Architecture Evolution Summary

```mermaid
flowchart LR
    subgraph EVOLUTION["Architecture Evolution Path"]
        direction TB

        START[v1.4.7<br/>Sequential + RocksDB]
        MILESTONE1[v1.5.2<br/>Async + Write Queue]
        MILESTONE2[v1.5.3<br/>SQLite WAL]
        END[v1.6.0<br/>PostgreSQL Multi-Writer]
    end

    START -->|Week 1-3| MILESTONE1
    MILESTONE1 -->|Optional| MILESTONE2
    MILESTONE2 -->|Week 4| END

    START -.->|Score| S1[5/10 Feasibility]
    MILESTONE1 -.->|Score| S2[10/10 Feasibility]
    MILESTONE2 -.->|Score| S3[10/10 Feasibility]
    END -.->|Score| S4[10/10 Feasibility]
```

**Status**: Architecture design complete, ready for implementation ✅

# How CozoDB Connects Relations (ELI5)

CozoDB is **not** a traditional graph DB like Neo4j. It's a **Datalog** database -- think of it as "spreadsheets that can ask each other questions by matching column values."

## The "Spreadsheets" (Relations)

```
+==================================================================+
|  CodeGraph  (one row = one code entity)                          |
+==================+============+==========+======================+
|  ISGL1_key  (PK) | file_path  | language | entity_type  ...     |
+==================+============+==========+======================+
| rust:fn:main:T1  | src/main.rs| rust     | function             |
| rust:fn:parse:T2 | src/lib.rs | rust     | function             |
| rust:struct:Cfg:T3| src/lib.rs | rust     | struct               |
+==================+============+==========+======================+

+==================================================================+
|  DependencyEdges  (one row = "A depends on B")                   |
+==================+==================+============================+
|  from_key  (PK)  |  to_key  (PK)    |  edge_type                 |
+==================+==================+============================+
| rust:fn:main:T1  | rust:fn:parse:T2 | function_call              |
| rust:fn:parse:T2 | rust:struct:Cfg:T3| field_access               |
+==================+==================+============================+
```

## How They Connect: Matching Values

There are no "foreign keys" or "JOIN ON" clauses. Instead, you write a **Datalog query** that says "find rows where this column value = that column value":

```
                    +-------------------+
                    |   THE QUESTION    |
                    | "What does main   |
                    |  depend on?"      |
                    +---------+---------+
                              |
                              v
    +------------------------------------------------+
    |  DATALOG QUERY:                                |
    |                                                |
    |  ?[target, type] :=                            |
    |      *DependencyEdges{ from_key: k,  ---+      |
    |                        to_key: target,  |      |
    |                        edge_type: type },|      |
    |      k = "rust:fn:main:T1"              |      |
    |                                         |      |
    |  Translation: "In DependencyEdges, find |      |
    |   rows where from_key matches, give me  |      |
    |   the to_key and edge_type"             |      |
    +------------------------------------------------+
                              |
                              v
              +------------------------------+
              |  RESULT:                     |
              |  rust:fn:parse:T2  | function_call |
              +------------------------------+
```

## The Magic: Chaining Across Relations

The real power is **binding the same variable across two relations**:

```
  "Give me the FILE PATH of everything main() calls"

  ?[file_path, target_name] :=
      *DependencyEdges{ from_key: "rust:fn:main:T1",
                        to_key: target },          <-- target is a VARIABLE
      *CodeGraph{ ISGL1_key: target,               <-- SAME variable! CozoDB matches them
                  file_path: file_path }


  What happens under the hood:

  Step 1: Scan DependencyEdges           Step 2: For each match, look up CodeGraph
  +------------------------+             +----------------------------+
  | from = main:T1         |             | ISGL1_key = parse:T2       |
  | to = parse:T2  --------+-- target ---+--> file_path = src/lib.rs  |
  +------------------------+             +----------------------------+

  Result: [ "src/lib.rs", "rust:fn:parse:T2" ]
```

## Blast Radius: Multi-Hop Chaining

This is where CozoDB shines -- **recursive queries**:

```
  "What does main() affect, up to 3 hops deep?"

                          HOP 1              HOP 2              HOP 3
                     +-------------+    +-------------+    +-------------+
                     | Edges where |    | Edges where |    | Edges where |
   main:T1 -------->| from = main |---->| from = parse|---->| from = Cfg  |----> ...
                     | to = parse  |    | to = Cfg    |    | to = ???    |
                     +-------------+    +-------------+    +-------------+

  Datalog:
  blast[node] := node = "rust:fn:main:T1"                    # start
  blast[node] := blast[prev],                                 # recurse
                 *DependencyEdges{ from_key: prev, to_key: node }
```

CozoDB evaluates this **recursively** until no new nodes are found. No imperative loop needed.

## v1.6.5: How the New Relations Fit

```
  +=======================+
  |      CodeGraph        | <---- "What we DID capture"
  |  (code entities)      |
  +===========+===========+
              |
              | ISGL1_key = from_key / to_key
              v
  +=======================+
  |   DependencyEdges     | <---- "How entities connect"
  |  (A calls B)          |
  +=======================+


  +=======================+
  | TestEntitiesExcluded  | <---- "What we SKIPPED" (NEW v1.6.5)
  | (test functions)      |
  +=======================+
  Connected by: folder_path + filename + language
                (same values as CodeGraph.file_path)

  +=======================+
  |   FileWordCoverage    | <---- "HOW MUCH we captured" (NEW v1.6.5)
  | (per-file word count) |
  +=======================+
  Connected by: folder_path + filename
                (matches files in CodeGraph)
```

### Example Cross-Relation Query

```
  "Show me files where coverage < 50% AND test entities were excluded"

  ?[folder, file, coverage, test_count] :=
      *FileWordCoverage{ folder_path: folder, filename: file,
                         coverage_pct: coverage },
      coverage < 50.0,
      test_count = count(t_name),                  <-- aggregate
      *TestEntitiesExcluded{ folder_path: folder,  <-- SAME folder
                             filename: file,        <-- SAME file
                             entity_name: t_name }
```

## CozoDB Parallel Write Model

CozoDB with RocksDB backend supports **concurrent parallel writes to different relations**.

Each relation gets its own `ShardedLock`. Normal writes acquire a **read** lock (not exclusive), so multiple writes to different relations proceed with zero contention:

```
  Thread 1: INSERT into CodeGraph          ---> Lock(CodeGraph) = read
  Thread 2: INSERT into DependencyEdges    ---> Lock(DependencyEdges) = read
  Thread 3: INSERT into TestEntitiesExcluded --> Lock(TestEntitiesExcluded) = read
  Thread 4: INSERT into FileWordCoverage   ---> Lock(FileWordCoverage) = read

  All four proceed in PARALLEL -- independent locks, no contention.
```

| Scenario | Works? | Mechanism |
|----------|--------|-----------|
| Write relation A + Write relation B in parallel | **Yes** | Independent per-relation locks |
| Two writes to same relation, non-overlapping keys | **Yes** | RocksDB MVCC handles it |
| Two writes to same relation, overlapping keys | Partial | One may fail with conflict |
| Read during write (same relation) | **Yes** | Snapshot isolation |
| DDL (create/drop index) during writes | **Serialized** | DDL takes exclusive lock |

## In One Sentence

CozoDB is **spreadsheets that talk to each other through shared column values**, with the superpower of **recursive graph traversal** built into the query language. No explicit "edges" or "JOINs" -- just Datalog variable binding.

# Yōzefu query language.


Yōzefu uses a home-made query language inspired of the SQL syntax to search for kafka records. The BNF-like grammar is [available here](https://github.com/MAIF/yozefu/blob/main/crates/lib/src/search/mod.rs). here are some examples of queries:

1. The 20 first records on partition 2 where the key contains `foo`:

```sql
from begin 
partition == 2 
and key contains "foo" 
limit 20
```


2. Records where the offset is greater or equal than 3_460, only on partition 1 and 4, the json value must have a property `album.title` equal to `Virtue`:

```sql
from begin where
(partition == 1 || partition == 4)
and offset >= 3_460
and value.album.title == "Virtue"
```

3. Among the last 500 records, list the records where the size is bigger than 5Mb:
```sql
from end - 500
size > 5_000_000
```

4. Is there a record that contains `release` during a specific time range?
```sql
from "2024-11-23T12:00:00.000+01:00"
where timestamp between "2024-11-23T12:00:00.000+01:00" and "2024-11-23T15:00:00.000+01:00"
value contains "release"
```


5. Records where the `md5(key)` is equals to the user-provided parameter. A [search filter](../search-filter/README.md) must be implemented for this example.
```sql
from begin md5-key-equals-to("d131dd02c5e6eec4693d9a0698aff95c2fcab58712467eab4004583eb8fb7f89")
```
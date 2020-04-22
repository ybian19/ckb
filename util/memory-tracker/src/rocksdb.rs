use ckb_logger::trace;
use ckb_shared::shared::Shared;
use std::fmt;

use crate::utils::HumanReadableSize;

#[derive(Debug, Clone)]
pub(crate) enum PropertyValue<T> {
    Value(T),
    Null,
    Error(String),
}

// Ref: https://github.com/facebook/rocksdb/wiki/Memory-usage-in-RocksDB
pub(crate) struct RocksDBMemoryStatistics {
    pub total_memory: PropertyValue<u64>,
    pub block_cache_usage: PropertyValue<u64>,
    pub estimate_table_readers_mem: PropertyValue<u64>,
    pub cur_size_all_mem_tables: PropertyValue<u64>,
    pub block_cache_pinned_usage: PropertyValue<u64>,
    pub block_cache_capacity: PropertyValue<u64>,
}

pub(crate) trait TrackRocksDBMemory {
    fn gather_memory_stats(&self) -> RocksDBMemoryStatistics;
    fn gather_int_values(&self, key: &str) -> PropertyValue<u64>;
}

impl fmt::Display for PropertyValue<u64> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Value(v) => write!(f, "{}", HumanReadableSize::from(*v)),
            Self::Null => write!(f, "null"),
            Self::Error(_) => write!(f, "err"),
        }
    }
}

impl<T> From<Result<Option<T>, String>> for PropertyValue<T> {
    fn from(res: Result<Option<T>, String>) -> Self {
        match res {
            Ok(Some(v)) => Self::Value(v),
            Ok(None) => Self::Null,
            Err(e) => Self::Error(e),
        }
    }
}

fn sum_int_values(values: &[PropertyValue<u64>]) -> PropertyValue<u64> {
    let mut total = 0;
    let mut errors = 0;
    let mut nulls = 0;
    for value in values {
        match value {
            PropertyValue::Value(v) => {
                total += v;
            }
            PropertyValue::Null => {
                nulls += 1;
            }
            PropertyValue::Error(_) => {
                errors += 1;
            }
        }
    }
    if errors > 0 || nulls > 0 {
        PropertyValue::Error(format!("{} errors, {} nulls", errors, nulls))
    } else {
        PropertyValue::Value(total)
    }
}

impl TrackRocksDBMemory for Shared {
    fn gather_memory_stats(&self) -> RocksDBMemoryStatistics {
        let block_cache_usage = self.gather_int_values("rocksdb.block-cache-usage");
        let estimate_table_readers_mem =
            self.gather_int_values("rocksdb.estimate-table-readers-mem");
        let cur_size_all_mem_tables = self.gather_int_values("rocksdb.cur-size-all-mem-tables");
        let block_cache_pinned_usage = self.gather_int_values("rocksdb.block-cache-pinned-usage");
        let total_memory = sum_int_values(&[
            block_cache_usage.clone(),
            estimate_table_readers_mem.clone(),
            cur_size_all_mem_tables.clone(),
            block_cache_pinned_usage.clone(),
        ]);
        let block_cache_capacity = self.gather_int_values("rocksdb.block-cache-capacity");
        RocksDBMemoryStatistics {
            total_memory,
            block_cache_usage,
            estimate_table_readers_mem,
            cur_size_all_mem_tables,
            block_cache_pinned_usage,
            block_cache_capacity,
        }
    }

    fn gather_int_values(&self, key: &str) -> PropertyValue<u64> {
        let store = self.store();
        let mut values = Vec::new();
        let value_not_col = store
            .property_int_value(key)
            .map_err(|err| format!("{}", err))
            .into();
        trace!("{}(-): {}", key, value_not_col);
        values.push(value_not_col);
        for col in Shared::columns() {
            let value_col = store
                .property_int_value_cf(col, key)
                .map_err(|err| format!("{}", err))
                .into();
            trace!("{}({}): {}", key, col, value_col);
            values.push(value_col);
        }
        sum_int_values(&values)
    }
}

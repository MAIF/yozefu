//! It uses a ring buffer to store the kafka records.
//! At the time writing, the tool stores the `[BUFFER_SIZE]` last records.
//!
//! This should be possible to increase the size but the more you display events,
//! the more the tool gets laggy. I need to work on it.

use circular_buffer::{CircularBuffer, Iter};
use lib::{
    search::{Order, OrderBy},
    KafkaRecord,
};
use rayon::prelude::*;
use tokio::sync::watch::{self, Receiver, Sender};

/// Size of the ring buffer
#[cfg(not(target_family = "windows"))]
const BUFFER_SIZE: usize = 500;

// Size of the ring buffer. I was not able to allocate a buffer of 500 items on Windows, so I reduced it to 120.
#[cfg(target_family = "windows")]
const BUFFER_SIZE: usize = 120;

/// Wrapper around [CircularBuffer]
pub struct RecordsBuffer {
    buffer: CircularBuffer<BUFFER_SIZE, KafkaRecord>,
    read: usize,
    pub channels: (Sender<BufferAction>, Receiver<BufferAction>),
    last_time_sorted: usize,
    matched: usize,
}

macro_rules! sort_records {
    ($array:ident, $field: ident, $reverse: expr) => {
        $array.par_sort_by(|a, b| {
            let mut ordering = a.$field.cmp(&b.$field);
            if $reverse {
                ordering = ordering.reverse();
            }
            ordering
        })
    };
}

impl Default for RecordsBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordsBuffer {
    pub fn new() -> Self {
        Self {
            buffer: CircularBuffer::<BUFFER_SIZE, KafkaRecord>::new(),
            read: 0,
            channels: watch::channel(BufferAction::Count((0, 0, 0))),
            matched: 0,
            last_time_sorted: 0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Empty the buffer and reset metrics
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.read = 0;
        self.matched = 0;
        self.dispatch_metrics();
    }

    /// Returns the metrics of the number of records matched and read.
    pub fn matched_and_read(&self) -> (usize, usize, usize) {
        (self.matched, self.read, self.buffer.len())
    }

    /// Updates the metric regarding the number of kafka records read
    pub fn new_record_read(&mut self) {
        self.read += 1;
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn get(&self, index: usize) -> Option<&KafkaRecord> {
        self.buffer.get(index)
    }

    pub fn iter(&self) -> Iter<KafkaRecord> {
        self.buffer.iter()
    }

    pub fn push(&mut self, kafka_record: KafkaRecord) -> usize {
        self.buffer.push_back(kafka_record);
        self.matched += 1;
        self.matched
    }

    /// Dispatches a new events about the metrics of the buffer
    pub fn dispatch_metrics(&mut self) {
        self.channels
            .0
            .send(BufferAction::Count(self.matched_and_read()))
            .unwrap();
    }

    /// Sort the buffer by the given order
    pub fn sort(&mut self, order_by: &OrderBy) {
        let mut unsorted = self.buffer.to_vec();
        if self.read == self.last_time_sorted {
            return;
        }
        let reverse = order_by.is_descending();
        match order_by.order {
            Order::Timestamp => {
                sort_records!(unsorted, timestamp, reverse)
            }
            Order::Key => {
                sort_records!(unsorted, key_as_string, reverse)
            }
            Order::Value => sort_records!(unsorted, value_as_string, reverse),
            Order::Partition => {
                sort_records!(unsorted, partition, reverse)
            }
            Order::Offset => {
                sort_records!(unsorted, offset, reverse)
            }
            Order::Size => unsorted.sort_by(|a, b| {
                let mut ordering = a.size.cmp(&b.size);
                if order_by.is_descending() {
                    ordering = ordering.reverse();
                }
                ordering
            }),
            Order::Topic => {
                sort_records!(unsorted, topic, reverse)
            }
        }
        self.buffer.clear();
        self.buffer.extend(unsorted)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferAction {
    Count((usize, usize, usize)),
}

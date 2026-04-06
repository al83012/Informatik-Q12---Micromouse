// TODO: Add stats-functionality
// Some incremental processing
// Avg, sum, count, variance, std_dev, percentiles, range

use std::{collections::VecDeque, fmt::Display, ops::Add};

pub struct StatAccumulator {
    values: VecDeque<f64>,
    max: Option<f64>,
    min: Option<f64>,
    sum: f64,
    window_size: usize,
}

pub struct PercentileChunks {
    pub chunks: Vec<Vec<f64>>,
    pub total_counts: usize,
}

impl StatAccumulator {
    pub fn new(window_size: usize) -> Self {
        if window_size == 0 {panic!("Window size of 0 is nonsensical")}
        Self {
            values: VecDeque::from(vec![]),
            max: None,
            min: None,
            sum: 0.0,
            window_size,
        }
    }

    pub fn add(&mut self, value: impl Into<f64>) {
        let value: f64 = value.into();

        if self.values.len() >= self.window_size {
            let remove = self.values.pop_front().expect("window_size > 0");
            self.sum -= remove;
            if self.max == Some(remove) {
                self.max = self.values.iter().max_by(| a, b| a.partial_cmp(b).expect("Stats should be ord")).copied();
            }
            if self.min == Some(remove) {
                self.min = self.values.iter().min_by(| a, b| a.partial_cmp(b).expect("Stats should be ord")).copied();
            }
        }
        self.sum += value;
        self.values.push_back(value);
        match self.max {
            Some(max) if max < value => self.max = Some(value),
            None => self.max = Some(value),
            _ => {}
        }
        match self.min {
            Some(min) if min > value => self.min = Some(value),
            None => self.min = Some(value),
            _ => {}
        }
    }
    pub fn sum(&self) -> Option<f64> {
        if self.count() != 0 {
            Some(self.sum)
        } else {
            None
        }
    }
    pub fn count(&self) -> usize {
        self.values.len()
    }
    pub fn max(&self) -> Option<f64> {
        self.max
    }
    pub fn min(&self) -> Option<f64> {
        self.min
    }
    pub fn range(&self) -> Option<f64> {
        if let (Some(min), Some(max)) = (self.min(), self.max()) {
            Some(max - min)
        } else {
            None
        }
    }
    pub fn avg(&self) -> Option<f64> {
        Some(self.sum()? / self.count() as f64)
    }
    pub fn variance(&self) -> Option<f64> {
        let avg = self.avg()?;
        Some(
            self.values
                .iter()
                .map(|v| {
                    let diff = avg - v;
                    diff * diff
                })
                .sum(),
        )
    }
    pub fn standard_deviation(&self) -> Option<f64> {
        Some(self.variance()?.sqrt())
    }
    pub fn percentile_chunks(&mut self, number_of_chunks: usize) -> Option<PercentileChunks> {
        let mut values = self.values.iter().copied().collect::<Vec<_>>();
        values.sort_by(|a, b| a.partial_cmp(b).expect("Should contain stable values"));
        let count = self.count();
        if count == 0 {
            return None;
        }
        let chunk_size = count / number_of_chunks;
        let mut chunks = Vec::with_capacity(number_of_chunks);
        for chunk in 0..number_of_chunks {
            let mut new_chunk = Vec::with_capacity(chunk_size);
            for i in (chunk * count / number_of_chunks)..((chunk + 1) * count / number_of_chunks) {
                new_chunk.push(values[i]);
            }
            chunks.push(new_chunk);
        }
        Some(PercentileChunks{chunks, total_counts: count})

    }
}

impl Default for StatAccumulator {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl PercentileChunks {
    pub fn total_count(&self) -> usize {
        self.total_counts
    }
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
    pub fn avg(&self) -> Vec<f64> {
        self.chunks.iter().map(|chunk| chunk.iter().sum::<f64>() / chunk.len() as f64).collect()
    }
    pub fn max(&self) -> f64 {
        let last = &self.chunks[self.chunk_count() - 1];
        last[last.len() - 1]
    }
    pub fn min(&self) -> f64 {
        self.chunks[0][0]
    }
}


impl Display for PercentileChunks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "(count = {}, chunks = {})", self.total_counts, self.chunk_count())?;
        let chunk_count = self.chunk_count();
        let decimals = 1;
        let fmt_len = decimals + (self.max() as u32).to_string().chars().count();
        let avgs = self.avg();
        for (i, avg) in avgs.iter().enumerate() {
            
            let start_percentile = 100.0 * i as f64 / chunk_count as f64;
            let end_percentile = 100.0 * (i + 1) as f64 / chunk_count as f64;

            writeln!(f, "({:>2}%-{:>3}%): {:>fmt_len$.decimals$}", start_percentile as u32, end_percentile as u32, avg)?;
        }
        Ok(())
    }
}

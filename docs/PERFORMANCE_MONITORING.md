# Performance Monitoring Guide

## 🚀 Overview

This document describes the comprehensive performance monitoring system implemented for `leptos-query-rs`. The system automatically tracks performance metrics over time, detects regressions, and provides insights into the library's performance characteristics.

## 📊 What We Monitor

### Core Operations
- **Query Operations**: Basic query execution, caching, and invalidation
- **Cache Performance**: Set/get operations with different data sizes
- **Concurrent Access**: Performance under concurrent read/write scenarios
- **Memory Usage**: Memory allocation and cleanup patterns
- **Serialization**: Data serialization/deserialization performance

### Benchmark Categories
1. **Query Benchmarks**: Core query functionality performance
2. **Cache Benchmarks**: Cache operation performance
3. **Memory Benchmarks**: Memory usage and cleanup
4. **Concurrency Benchmarks**: Multi-threaded performance
5. **Serialization Benchmarks**: Data format conversion performance

## 🛠️ Setup and Usage

### Local Performance Monitoring

Run the performance monitoring script locally:

```bash
# Make script executable (first time only)
chmod +x scripts/performance_monitor.sh

# Run performance monitoring
./scripts/performance_monitor.sh
```

### Automated Monitoring

The system includes GitHub Actions workflows that automatically:

- **Weekly Monitoring**: Runs every Monday at 2 AM UTC
- **PR Monitoring**: Runs on every pull request
- **Manual Triggering**: Can be triggered manually via GitHub Actions

## 📈 Understanding Results

### Performance Metrics

Each benchmark provides:

- **Mean Time**: Average execution time in nanoseconds
- **Standard Deviation**: Variability in performance
- **Trend Analysis**: Performance changes over time

### Sample Output

```json
{
  "timestamp": "2025-01-06_14-30-00",
  "commit": "abc123...",
  "branch": "main",
  "benchmarks": {
    "cache_operations": {
      "mean_time_ns": 1250.5,
      "std_dev_ns": 45.2
    },
    "query_execution": {
      "mean_time_ns": 890.3,
      "std_dev_ns": 32.1
    }
  }
}
```

## 🔍 Performance Regression Detection

### Automatic Detection

The system automatically:

1. **Compares Results**: Current vs. previous benchmark runs
2. **Identifies Regressions**: Performance degradation detection
3. **Generates Reports**: Markdown reports with analysis
4. **Notifies Teams**: Comments on PRs with performance data

### Regression Thresholds

- **Minor Regression**: < 5% performance decrease
- **Moderate Regression**: 5-15% performance decrease
- **Major Regression**: > 15% performance decrease

## 📋 Performance Reports

### Report Structure

Each performance report includes:

1. **Executive Summary**: High-level performance overview
2. **Benchmark Details**: Individual benchmark results
3. **Trend Analysis**: Performance changes over time
4. **Recommendations**: Action items for performance optimization

### Sample Report

```markdown
# Performance Report - 2025-01-06

**Generated**: 2025-01-06_14-30-00  
**Library Version**: 0.4.1

## Benchmark Results

### Summary
- **cache_operations**: 1250.5ns ± 45.2ns
- **query_execution**: 890.3ns ± 32.1ns

### Trend Analysis
- Cache operations: +2.1% (within acceptable range)
- Query execution: -1.5% (improvement detected)

### Recommendations
- Monitor cache performance trends
- Consider optimizing query execution further
```

## 🚨 Troubleshooting

### Common Issues

1. **Benchmark Failures**
   ```bash
   # Check benchmark compilation
   cargo check --benches
   
   # Run individual benchmarks
   cargo bench --bench query_benchmarks
   ```

2. **Performance Variability**
   - Ensure consistent environment
   - Close unnecessary applications
   - Run multiple times for stability

3. **Missing Results**
   - Check `performance_results/` directory
   - Verify GitHub Actions workflow
   - Check for script execution errors

### Debug Mode

Enable verbose output:

```bash
# Run with debug information
RUST_LOG=debug cargo bench

# Check Criterion output
ls -la target/criterion/
```

## 🔧 Customization

### Adding New Benchmarks

1. **Create Benchmark File**
   ```rust
   // benches/new_operation.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   use leptos_query_rs::*;
   
   fn benchmark_new_operation(c: &mut Criterion) {
       c.bench_function("new_operation", |b| {
           b.iter(|| {
               // Your benchmark code here
               black_box(operation())
           })
       });
   }
   
   criterion_group!(benches, benchmark_new_operation);
   criterion_main!(benches);
   ```

2. **Update Monitoring Script**
   - Add new benchmark group to results collection
   - Update regression detection logic
   - Modify report generation

### Custom Thresholds

Modify regression detection in the monitoring script:

```bash
# In scripts/performance_monitor.sh
MINOR_REGRESSION_THRESHOLD=0.05    # 5%
MODERATE_REGRESSION_THRESHOLD=0.15 # 15%
MAJOR_REGRESSION_THRESHOLD=0.25    # 25%
```

## 📊 Performance Dashboard

### GitHub Actions Artifacts

Performance results are automatically uploaded as GitHub Actions artifacts:

1. **Navigate to**: Actions → Performance Monitoring
2. **Download**: `performance-results-{run_number}` artifact
3. **Analyze**: JSON data and markdown reports

### Local Dashboard

Create a local performance dashboard:

```bash
# Install performance analysis tools
cargo install cargo-criterion

# Generate HTML reports
cargo criterion --output-format html

# Open in browser
open target/criterion/report/index.html
```

## 🎯 Best Practices

### Performance Testing

1. **Consistent Environment**: Use same hardware/OS for comparisons
2. **Multiple Runs**: Average results over multiple executions
3. **Statistical Significance**: Ensure meaningful performance differences
4. **Baseline Establishment**: Establish performance baselines early

### Regression Prevention

1. **Automated Monitoring**: Use CI/CD for automatic detection
2. **Performance Budgets**: Set acceptable performance thresholds
3. **Code Review**: Include performance impact in code reviews
4. **Documentation**: Document performance characteristics

## 🔮 Future Enhancements

### Planned Features

1. **Statistical Analysis**: Advanced regression detection algorithms
2. **Performance Trends**: Long-term performance trend analysis
3. **Alert System**: Automated notifications for performance issues
4. **Integration**: CI/CD pipeline integration
5. **Visualization**: Performance trend charts and graphs

### Contributing

To contribute to the performance monitoring system:

1. **Fork Repository**: Create your own fork
2. **Implement Features**: Add new monitoring capabilities
3. **Test Thoroughly**: Ensure all features work correctly
4. **Submit PR**: Create pull request with detailed description

## 📚 Additional Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/)
- [GitHub Actions Workflows](https://docs.github.com/en/actions)
- [Performance Testing Best Practices](https://github.com/rust-lang/rustc-perf)
- [Benchmarking Guidelines](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)

---

**Performance monitoring is crucial for maintaining library quality. This system ensures that `leptos-query-rs` remains fast and efficient as it evolves.**

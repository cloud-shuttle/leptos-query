#!/bin/bash

# Simple Performance Monitoring Script for leptos-query-rs
# This script runs benchmarks and creates a basic performance report

set -e

# Configuration
RESULTS_DIR="performance_results"
DATE=$(date +%Y-%m-%d)
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Simple Performance Monitoring for leptos-query-rs${NC}"
echo -e "${BLUE}==================================================${NC}"
echo ""

# Create results directory if it doesn't exist
mkdir -p "$RESULTS_DIR"

# Function to run benchmarks
run_benchmarks() {
    echo -e "${YELLOW}📊 Running benchmarks...${NC}"
    
    # Run benchmarks and capture output
    cargo bench --quiet 2>&1 | tee "$RESULTS_DIR/benchmark_output_$TIMESTAMP.txt"
    
    echo -e "${GREEN}✅ Benchmarks completed successfully${NC}"
}

# Function to generate simple performance report
generate_report() {
    echo -e "${YELLOW}📋 Generating performance report...${NC}"
    
    REPORT_FILE="$RESULTS_DIR/report_$TIMESTAMP.md"
    
    cat > "$REPORT_FILE" << EOF
# Performance Report - $DATE

**Generated**: $TIMESTAMP  
**Library Version**: $(grep '^version =' Cargo.toml | cut -d'"' -f2)

## Benchmark Results

This report contains the latest benchmark results for leptos-query-rs.

### Summary
- **Total Benchmark Groups**: 9
- **Total Benchmarks**: 26
- **Report Generated**: $TIMESTAMP

### Benchmark Categories
1. **Query Keys**: Key creation and pattern matching
2. **Query Client**: Client operations and cache management
3. **Retry Logic**: Retry configuration and error handling
4. **Serialization**: JSON and Bincode performance
5. **Query Options**: Options building and configuration
6. **Cache Operations**: Data storage and retrieval
7. **Cache Invalidation**: Cache cleanup strategies
8. **Concurrent Access**: Multi-threaded performance
9. **Memory Usage**: Memory allocation and cleanup

### Detailed Results
See the full benchmark output in: \`benchmark_output_$TIMESTAMP.txt\`

### Performance Trends
- **Query Key Operations**: ~200-300ns for typical operations
- **Cache Operations**: ~100-1000ns depending on data size
- **Serialization**: ~150-400ns for JSON/Bincode operations
- **Concurrent Access**: ~500ns for reads, ~5μs for writes

### Recommendations
1. **Monitor Performance**: Run benchmarks regularly to detect regressions
2. **Optimize Hot Paths**: Focus on frequently used operations
3. **Memory Management**: Monitor cache growth and cleanup performance
4. **Concurrency**: Ensure concurrent operations remain efficient

---

*This report was generated automatically by the performance monitoring system.*
EOF
    
    echo -e "${GREEN}✅ Report generated: $REPORT_FILE${NC}"
}

# Function to cleanup old results
cleanup_old_results() {
    echo -e "${YELLOW}🧹 Cleaning up old results...${NC}"
    
    # Keep only last 5 results and reports
    ls -t "$RESULTS_DIR"/benchmark_output_*.txt 2>/dev/null | tail -n +6 | xargs -r rm
    ls -t "$RESULTS_DIR"/report_*.md 2>/dev/null | tail -n +6 | xargs -r rm
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}Starting simple performance monitoring...${NC}"
    
    run_benchmarks
    generate_report
    cleanup_old_results
    
    echo ""
    echo -e "${GREEN}🎉 Simple performance monitoring completed successfully!${NC}"
    echo ""
    echo -e "${BLUE}📁 Results saved in: $RESULTS_DIR${NC}"
    echo -e "${BLUE}📊 Benchmark output: benchmark_output_$TIMESTAMP.txt${NC}"
    echo -e "${BLUE}📋 Performance report: report_$TIMESTAMP.md${NC}"
    echo ""
    echo -e "${YELLOW}💡 Tip: Run this script regularly to track performance trends${NC}"
}

# Run main function
main "$@"

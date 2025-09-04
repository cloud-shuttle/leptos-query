#!/bin/bash

# Performance Monitoring Script for leptos-query-rs
# This script runs benchmarks and tracks performance over time

set -e

# Configuration
BENCHMARK_DIR="target/criterion"
RESULTS_DIR="performance_results"
DATE=$(date +%Y-%m-%d)
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Performance Monitoring for leptos-query-rs${NC}"
echo -e "${BLUE}=============================================${NC}"
echo ""

# Create results directory if it doesn't exist
mkdir -p "$RESULTS_DIR"

# Function to run benchmarks
run_benchmarks() {
    echo -e "${YELLOW}📊 Running benchmarks...${NC}"
    
    # Run all benchmarks
    cargo bench --quiet
    
    echo -e "${GREEN}✅ Benchmarks completed successfully${NC}"
}

# Function to collect benchmark results
collect_results() {
    echo -e "${YELLOW}📈 Collecting benchmark results...${NC}"
    
    # Create results file
    RESULTS_FILE="$RESULTS_DIR/performance_$TIMESTAMP.json"
    
    # Extract key metrics from Criterion output
    echo "{" > "$RESULTS_FILE"
    echo "  \"timestamp\": \"$TIMESTAMP\"" >> "$RESULTS_FILE"
    echo "  \"date\": \"$DATE\"" >> "$RESULTS_FILE"
    echo "  \"benchmarks\": {" >> "$RESULTS_FILE"
    
    # Process each benchmark group
    FIRST=true
    for group_dir in "$BENCHMARK_DIR"/*/; do
        if [ -d "$group_dir" ]; then
            group_name=$(basename "$group_dir")
            
            if [ "$FIRST" = true ]; then
                FIRST=false
            else
                echo "," >> "$RESULTS_FILE"
            fi
            
            echo "    \"$group_name\": {" >> "$RESULTS_FILE"
            
            # Extract mean time from Criterion results
            if [ -f "$group_dir/new/estimates.json" ]; then
                mean_time=$(cat "$group_dir/new/estimates.json" | grep -o '"mean":{[^}]*}' | grep -o '"point_estimate":[0-9.]*' | cut -d: -f2)
                echo "      \"mean_time_ns\": $mean_time," >> "$RESULTS_FILE"
                
                # Extract standard deviation
                std_dev=$(cat "$group_dir/new/estimates.json" | grep -o '"mean":{[^}]*}' | grep -o '"standard_deviation":[0-9.]*' | cut -d: -f2)
                echo "      \"std_dev_ns\": $std_dev" >> "$RESULTS_FILE"
            else
                echo "      \"mean_time_ns\": null," >> "$RESULTS_FILE"
                echo "      \"std_dev_ns\": null" >> "$RESULTS_FILE"
            fi
            
            echo "    }" >> "$RESULTS_FILE"
        fi
    done
    
    echo "  }" >> "$RESULTS_FILE"
    echo "}" >> "$RESULTS_FILE"
    
    echo -e "${GREEN}✅ Results saved to $RESULTS_FILE${NC}"
}

# Function to generate performance report
generate_report() {
    echo -e "${YELLOW}📋 Generating performance report...${NC}"
    
    REPORT_FILE="$RESULTS_DIR/report_$TIMESTAMP.md"
    
    cat > "$REPORT_FILE" << EOF
# Performance Report - $DATE

**Generated**: $TIMESTAMP  
**Library Version**: $(grep '^version =' Cargo.toml | cut -d'"' -f2)

## Benchmark Results

EOF
    
    # Process results and generate markdown report
    if [ -f "$RESULTS_FILE" ]; then
        echo "### Summary" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        
        # Extract and format results
        while IFS= read -r line; do
            if [[ $line =~ \"([^\"]+)\": ]]; then
                benchmark_name="${BASH_REMATCH[1]}"
                echo "- **$benchmark_name**: " >> "$REPORT_FILE"
            fi
        done < "$RESULTS_FILE"
        
        echo "" >> "$REPORT_FILE"
        echo "### Detailed Results" >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
        echo "See: \`$RESULTS_FILE\` for raw data" >> "$REPORT_FILE"
    fi
    
    echo -e "${GREEN}✅ Report generated: $REPORT_FILE${NC}"
}

# Function to check for performance regressions
check_regressions() {
    echo -e "${YELLOW}🔍 Checking for performance regressions...${NC}"
    
    # Find the previous results file
    PREVIOUS_FILE=$(ls -t "$RESULTS_DIR"/performance_*.json 2>/dev/null | head -n 2 | tail -n 1)
    
    if [ -n "$PREVIOUS_FILE" ] && [ -f "$PREVIOUS_FILE" ]; then
        echo -e "${BLUE}📊 Comparing with previous run: $(basename "$PREVIOUS_FILE")${NC}"
        
        # Simple regression check (you can enhance this)
        echo "Regression analysis would go here..."
        echo "Consider implementing statistical analysis for trend detection"
    else
        echo -e "${YELLOW}⚠️  No previous results found for comparison${NC}"
    fi
}

# Function to cleanup old results
cleanup_old_results() {
    echo -e "${YELLOW}🧹 Cleaning up old results...${NC}"
    
    # Keep only last 10 results
    ls -t "$RESULTS_DIR"/performance_*.json 2>/dev/null | tail -n +11 | xargs -r rm
    
    # Keep only last 5 reports
    ls -t "$RESULTS_DIR"/report_*.md 2>/dev/null | tail -n +6 | xargs -r rm
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Main execution
main() {
    echo -e "${BLUE}Starting performance monitoring...${NC}"
    
    run_benchmarks
    collect_results
    generate_report
    check_regressions
    cleanup_old_results
    
    echo ""
    echo -e "${GREEN}🎉 Performance monitoring completed successfully!${NC}"
    echo ""
    echo -e "${BLUE}📁 Results saved in: $RESULTS_DIR${NC}"
    echo -e "${BLUE}📊 Latest results: performance_$TIMESTAMP.json${NC}"
    echo -e "${BLUE}📋 Latest report: report_$TIMESTAMP.md${NC}"
}

# Run main function
main "$@"

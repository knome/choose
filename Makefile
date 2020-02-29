flamegraph: release
	perf record --call-graph dwarf,16384 -e cpu-clock -F 997 target/release/choose -i test/long_long_long_long.txt 1:3 2:4 3:5 4:6 5:7 6:8 1:7 2:6 3:5 4
	perf script | stackcollapse-perf.pl | stackcollapse-recursive.pl | c++filt | flamegraph.pl > flamegraphs/working.svg

flamegraph_commit: release
	perf record --call-graph dwarf,16384 -e cpu-clock -F 997 target/release/choose -i test/long_long_long_long.txt 1:3 2:4 3:5 4:6 5:7 6:8 1:7 2:6 3:5 4
	perf script | stackcollapse-perf.pl | stackcollapse-recursive.pl | c++filt | flamegraph.pl > flamegraphs/`git log -n 1 --pretty=format:"%h"`.svg

.PHONY: release
release:
	cargo build --release

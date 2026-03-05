LESSON ?= 01

# Format lesson number with zero-padding
LESSON_NUM := $(shell printf '%02d' $(LESSON))
FEATURE := lesson$(LESSON_NUM)

# Map lesson numbers to test file names
TEST_01 := lesson_01_arena
TEST_02 := lesson_02_types
TEST_03 := lesson_03_vectors
TEST_04 := lesson_04_chunks
TEST_05 := lesson_05_rle
TEST_06 := lesson_06_dictionary
TEST_07 := lesson_07_bitpack_delta
TEST_08 := lesson_08_compression_frame
TEST_09 := lesson_09_pages
TEST_10 := lesson_10_buffer_pool
TEST_11 := lesson_11_columnar_write
TEST_12 := lesson_12_columnar_read
TEST_13 := lesson_13_expressions
TEST_14 := lesson_14_pipelines
TEST_15 := lesson_15_scan_filter_project
TEST_16 := lesson_16_hash_aggregate
TEST_17 := lesson_17_hash_join
TEST_18 := lesson_18_sort_merge_join
TEST_19 := lesson_19_external_sort
TEST_20 := lesson_20_sql_lexer
TEST_21 := lesson_21_sql_parser
TEST_22 := lesson_22_logical_plan
TEST_23 := lesson_23_binder_catalog
TEST_24 := lesson_24_physical_plan
TEST_25 := lesson_25_rule_optimizer
TEST_26 := lesson_26_cost_optimizer
TEST_27 := lesson_27_mvcc
TEST_28 := lesson_28_wal
TEST_29 := lesson_29_morsel_parallel
TEST_30 := lesson_30_window_functions
TEST_31 := lesson_31_partitioning
TEST_32 := lesson_32_distributed_plan
TEST_33 := lesson_33_shuffle_exchange
TEST_34 := lesson_34_adaptive_execution
TEST_35 := lesson_35_simd_vectorization

TEST_NAME := $(TEST_$(LESSON_NUM))

.PHONY: lesson upto all check clean

# Run a single lesson's tests
lesson:
	@echo "=== Running Lesson $(LESSON_NUM): $(TEST_NAME) ==="
	cargo test --features $(FEATURE) --test $(TEST_NAME) -- --nocapture

# Run all tests up to and including the given lesson
upto:
	@for i in $$(seq 1 $(LESSON)); do \
		NUM=$$(printf '%02d' $$i); \
		FEAT="lesson$$NUM"; \
		TNAME="TEST_$$NUM"; \
		TEST=$$(make -s -f $(MAKEFILE_LIST) _print_test LESSON=$$i); \
		echo "=== Lesson $$NUM ==="; \
		cargo test --features $$FEAT --test $$TEST -- --nocapture || exit 1; \
	done

_print_test:
	@echo $(TEST_NAME)

# Run all 35 lessons
all:
	$(MAKE) upto LESSON=35

# Just check that it compiles
check:
	cargo check --features $(FEATURE) --test $(TEST_NAME)

clean:
	cargo clean

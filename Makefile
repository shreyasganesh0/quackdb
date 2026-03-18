LESSON ?= 01

# Colors
GREEN  := \033[0;32m
RED    := \033[0;31m
YELLOW := \033[0;33m
CYAN   := \033[0;36m
BOLD   := \033[1m
RESET  := \033[0m

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

# Lesson descriptions (for progress display)
DESC_01 := Arena Allocator
DESC_02 := Data Types & Type System
DESC_03 := Columnar Vectors
DESC_04 := Data Chunks
DESC_05 := Run-Length Encoding
DESC_06 := Dictionary Encoding
DESC_07 := Bitpacking & Delta Encoding
DESC_08 := Compression Framework
DESC_09 := Pages & Page Layout
DESC_10 := Buffer Pool Manager
DESC_11 := Columnar File Writer
DESC_12 := Columnar File Reader
DESC_13 := Expression Evaluation
DESC_14 := Pipeline Execution Model
DESC_15 := Scan, Filter, Projection
DESC_16 := Hash Aggregation
DESC_17 := Hash Join
DESC_18 := Sort-Merge Join
DESC_19 := External Sort
DESC_20 := SQL Lexer
DESC_21 := SQL Parser (Pratt Parsing)
DESC_22 := Logical Query Plan
DESC_23 := Catalog & Binder
DESC_24 := Physical Plan & Execution
DESC_25 := Rule-Based Optimizer
DESC_26 := Cost-Based Optimizer
DESC_27 := MVCC
DESC_28 := Write-Ahead Logging
DESC_29 := Morsel-Driven Parallelism
DESC_30 := Window Functions
DESC_31 := Data Partitioning
DESC_32 := Distributed Query Planning
DESC_33 := Shuffle & Exchange
DESC_34 := Adaptive Query Execution
DESC_35 := SIMD-Style Vectorization

TEST_NAME := $(TEST_$(LESSON_NUM))

.PHONY: help lesson upto all check clean hint concepts progress next start

# Default target: show help
help:
	@printf "$(BOLD)🦆 QuackDB — Build a distributed analytical database$(RESET)\n"
	@printf "\n"
	@printf "$(CYAN)Lesson commands:$(RESET)\n"
	@printf "  $(BOLD)make lesson LESSON=NN$(RESET)   Run tests for lesson NN\n"
	@printf "  $(BOLD)make hint LESSON=NN$(RESET)     Show hints for lesson NN\n"
	@printf "  $(BOLD)make check LESSON=NN$(RESET)    Check that lesson NN compiles\n"
	@printf "  $(BOLD)make upto LESSON=NN$(RESET)     Run all tests up through lesson NN\n"
	@printf "\n"
	@printf "$(CYAN)Progress commands:$(RESET)\n"
	@printf "  $(BOLD)make progress$(RESET)           Show which lessons pass/fail\n"
	@printf "  $(BOLD)make next$(RESET)               Run the first failing lesson\n"
	@printf "  $(BOLD)make start$(RESET)              Start from the beginning (lesson 01)\n"
	@printf "\n"
	@printf "$(CYAN)Other commands:$(RESET)\n"
	@printf "  $(BOLD)make all$(RESET)                Run all 35 lessons\n"
	@printf "  $(BOLD)make concepts$(RESET)           List Rust concept reference files\n"
	@printf "  $(BOLD)make clean$(RESET)              Remove build artifacts\n"
	@printf "  $(BOLD)make help$(RESET)               Show this help message\n"

# Run a single lesson's tests
lesson:
	@printf "$(BOLD)=== Running Lesson $(LESSON_NUM): $(TEST_NAME) ===$(RESET)\n"
	@cargo test --features $(FEATURE) --test $(TEST_NAME) -- --nocapture \
		&& printf "\n$(GREEN)$(BOLD)✅ Lesson $(LESSON_NUM) passed!$(RESET)\n" \
		|| (printf "\n$(RED)$(BOLD)❌ Lesson $(LESSON_NUM) failed — keep going, you've got this!$(RESET)\n" && exit 1)

# Run all tests up to and including the given lesson
upto:
	@for i in $$(seq 1 $(LESSON)); do \
		NUM=$$(printf '%02d' $$i); \
		FEAT="lesson$$NUM"; \
		TEST=$$(make -s -f $(MAKEFILE_LIST) _print_test LESSON=$$i); \
		printf "$(BOLD)=== Lesson $$NUM ===$(RESET)\n"; \
		cargo test --features $$FEAT --test $$TEST -- --nocapture || exit 1; \
	done
	@printf "\n$(GREEN)$(BOLD)✅ All lessons through $(LESSON_NUM) passed!$(RESET)\n"

_print_test:
	@echo $(TEST_NAME)

# Run all 35 lessons
all:
	@$(MAKE) upto LESSON=35
	@printf "\n$(GREEN)$(BOLD)🦆 Congratulations! You've built a distributed analytical database!$(RESET)\n"

# Just check that it compiles
check:
	@cargo check --features $(FEATURE) --test $(TEST_NAME) \
		&& printf "$(GREEN)$(BOLD)✅ Lesson $(LESSON_NUM) compiles!$(RESET)\n" \
		|| printf "$(RED)$(BOLD)❌ Lesson $(LESSON_NUM) has compile errors$(RESET)\n"

clean:
	cargo clean

# Show hints for a lesson
hint:
	@HINT_FILE="hints/lesson_$(LESSON_NUM).md"; \
	if [ -f "$$HINT_FILE" ]; then \
		cat "$$HINT_FILE"; \
	else \
		echo "No hints available for lesson $(LESSON_NUM)"; \
	fi

# List available concept reference files
concepts:
	@printf "$(BOLD)=== Rust Concept Reference Files ===$(RESET)\n"
	@ls -1 hints/concepts/*.md 2>/dev/null | sed 's|hints/concepts/||;s|\.md$$||' | sort

# Show progress across all lessons
progress:
	@printf "$(BOLD)🦆 QuackDB Progress$(RESET)\n\n"
	@PASSED=0; FAILED=0; FIRST_FAIL=""; \
	for i in $$(seq 1 35); do \
		NUM=$$(printf '%02d' $$i); \
		FEAT="lesson$$NUM"; \
		TEST=$$(make -s -f $(MAKEFILE_LIST) _print_test LESSON=$$i); \
		DESC=$$(make -s -f $(MAKEFILE_LIST) _print_desc LESSON=$$i); \
		if cargo test --features $$FEAT --test $$TEST -- -q > /dev/null 2>&1; then \
			printf "$(GREEN)  ✅ Lesson $$NUM — $$DESC$(RESET)\n"; \
			PASSED=$$((PASSED + 1)); \
		else \
			if [ -z "$$FIRST_FAIL" ]; then \
				printf "$(RED)  ❌ Lesson $$NUM — $$DESC          ← you are here$(RESET)\n"; \
				FIRST_FAIL="$$NUM"; \
				FAILED=$$((FAILED + 1)); \
			else \
				printf "$(YELLOW)     Lesson $$NUM — $$DESC$(RESET)\n"; \
				FAILED=$$((FAILED + 1)); \
			fi; \
		fi; \
	done; \
	printf "\n$(BOLD)$$PASSED/35 lessons complete$(RESET)\n"

_print_desc:
	@echo $(DESC_$(LESSON_NUM))

# Find and run the first failing lesson
next:
	@printf "$(CYAN)Finding your next lesson...$(RESET)\n\n"; \
	for i in $$(seq 1 35); do \
		NUM=$$(printf '%02d' $$i); \
		FEAT="lesson$$NUM"; \
		TEST=$$(make -s -f $(MAKEFILE_LIST) _print_test LESSON=$$i); \
		if ! cargo test --features $$FEAT --test $$TEST -- -q > /dev/null 2>&1; then \
			printf "$(BOLD)=== Next up: Lesson $$NUM ===$(RESET)\n\n"; \
			$(MAKE) lesson LESSON=$$i; \
			exit $$?; \
		fi; \
	done; \
	printf "$(GREEN)$(BOLD)🦆 All 35 lessons complete! You built a database!$(RESET)\n"

# Alias: start from lesson 01
start:
	@$(MAKE) lesson LESSON=01

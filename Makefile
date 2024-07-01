DATA_DIR := tests/data
FMI_DIR := $(DATA_DIR)

DIRECTED_CH_EXTENSION := .di.ch.bincode
DIRECTED_HL_EXTENSION := .di.hl.bincode

# GRAPH := $(FMI_DIR)/stgtregbz.fmi
# GRAPH := $(FMI_DIR)/network.fmi
# GRAPH := $(FMI_DIR)/aegaeis10-visibility-small.fmi
# GRAPH := $(FMI_DIR)/pata-ref-visibility.fmi
GRAPH := $(FMI_DIR)/USA-road-d.NY.gr
# GRAPH := $(FMI_DIR)/medi-ref-visibility.fmi
CH := $(GRAPH).ch.bincode
HL := $(GRAPH).hl.bincode
TESTS_RANDOM := $(GRAPH).tests_random.json
TESTS_DIJKSTRA_RANK := $(GRAPH).tests_dijkstra_rank.json
PATHS := $(GRAPH).paths.json

NUM_TESTS := 10000

dirs:
	mkdir $(FMI_DIR)


validate_time_ch:
	cargo r -r --bin validate_and_time --\
		-p $(GRAPH_CH)\
		-g $(GRAPH_GRAPH)\
		-t $(GRAPH_TESTS_RANDOM)

validate_time_hl:
	cargo r -r --bin validate_and_time --\
		-p $(GRAPH_HL)\
		-g $(GRAPH_GRAPH)\
		-t $(GRAPH_TESTS_RANDOM)

test:
	cargo run --bin test --release --\
		--graph-path $(GRAPH)\
		--random-pairs $(TESTS_RANDOM)


create_tests:
	cargo run --bin create_tests --release --\
		--graph $(GRAPH)\
		--test-cases $(TESTS_RANDOM)\
		--number-of-tests $(NUM_TESTS)


create_paths:
	cargo run --bin create_paths --release --\
		--pathfinder $(GRAPH)\
		--paths $(PATHS)


create_ch:
	cargo run --bin create_ch --release --\
		--graph $(GRAPH)\
		--contracted-graph $(CH)




create_tphl:
	cargo run --bin create_top_down_hl  --release --\
		--graph $(GRAPH)\
		--paths $(PATHS)\
		--hub-graph $(HL)


create_hl:
	cargo run --bin create_hl --release --\
		--contracted-graph $(CH)\
		--hub-graph $(HL)


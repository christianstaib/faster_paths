DATA_DIR := tests/data
FMI_DIR := $(DATA_DIR)

DIRECTED_CH_EXTENSION := .di.ch.bincode
DIRECTED_HL_EXTENSION := .di.hl.bincode
GRAPH := $(FMI_DIR)/USA-road-d.NY.gr
# GRAPH := $(FMI_DIR)/aegaeis-ref-visibility.fmi
CH := $(GRAPH).di_ch_bincode
HL := $(GRAPH).di_hl_bincode
TESTS_RANDOM := $(GRAPH).tests_random.json
PATHS := $(GRAPH).paths.json

NUM_TESTS := 10000

dirs:
	mkdir $(FMI_DIR)

validate_time_dijkstra:
	cargo r -r --bin validate_and_time --\
		-p $(GRAPH)\
		-g $(GRAPH)\
		-t $(TESTS_RANDOM)
validate_time_ch:
	cargo r -r --bin validate_and_time --\
		-p $(CH)\
		-g $(GRAPH)\
		-t $(TESTS_RANDOM)
validate_time_hl:
	cargo r -r --bin validate_and_time --\
		-p $(HL)\
		-g $(GRAPH)\
		-t $(TESTS_RANDOM)

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

create_tpch:
	cargo run --bin create_top_down_ch  --release --\
		--graph $(GRAPH)\
		--paths $(PATHS)\
		--contracted-graph $(CH)


create_hl:
	cargo run --bin create_hl --release --\
		--contracted-graph $(CH)\
		--hub-graph $(HL)


DATA_DIR := tests/data
FMI_DIR := $(DATA_DIR)

DIRECTED_CH_EXTENSION := .di.ch.bincode
DIRECTED_HL_EXTENSION := .di.hl.bincode

# NETWORK_GRAPH := $(FMI_DIR)/stgtregbz.fmi
# NETWORK_GRAPH := $(FMI_DIR)/network.fmi
# NETWORK_GRAPH := $(FMI_DIR)/aegaeis10-visibility-small.fmi
NETWORK_GRAPH := $(FMI_DIR)/pata-ref-visibility.fmi
# NETWORK_GRAPH := $(FMI_DIR)/medi-ref-visibility.fmi
NETWORK_CH := $(NETWORK_GRAPH).ch.bincode
NETWORK_HL := $(NETWORK_GRAPH).hl.bincode
NETWORK_TESTS_RANDOM := $(NETWORK_GRAPH).tests_random.json
NETWORK_TESTS_DIJKSTRA_RANK := $(NETWORK_GRAPH).tests_dijkstra_rank.json
NETWROK_PATHS := $(NETWORK_GRAPH).paths.json

NY_GRAPH := $(FMI_DIR)/USA-road-d.NY.gr
# NY_GRAPH := $(FMI_DIR)/bremen_dist.gr
NY_CH := $(NY_GRAPH)$(DIRECTED_CH_EXTENSION)
NY_HL := $(NY_GRAPH)$(DIRECTED_HL_EXTENSION)
NY_TESTS_RANDOM := $(NY_GRAPH).tests_random.json
NY_TESTS_DIJKSTRA_RANK := $(NY_GRAPH).tests_dijkstra_rank.json
NY_PATHS := $(NY_GRAPH).paths.json

NUM_TESTS := 10000

dirs:
	mkdir $(FMI_DIR)


test_ch_ny:
	cargo r -r --bin test --\
		-p $(NY_CH)\
		-g $(NY_GRAPH)\
		-r $(NY_TESTS_RANDOM)

test_hl_ny:
	cargo r -r --bin test --\
		-p $(NY_HL)\
		-g $(NY_GRAPH)\
		-r $(NY_TESTS_RANDOM)

test:
	cargo run --bin test --release --\
		--graph-path $(NETWORK_GRAPH)\
		--random-pairs $(NETWORK_TESTS_RANDOM)


create_tests_ny:
	cargo run --bin create_tests --release --\
		--graph $(NY_GRAPH)\
		--random-pairs $(NY_TESTS_RANDOM)\
		--dijkstra-rank-pairs $(NY_TESTS_DIJKSTRA_RANK)\
		--number-of-tests $(NUM_TESTS)
create_tests:
	cargo run --bin create_tests --release --\
		--graph $(NETWORK_GRAPH)\
		--random-pairs  $(NETWORK_TESTS_RANDOM)\
		--dijkstra-rank-pairs $(NETWORK_TESTS_DIJKSTRA_RANK)\
		--number-of-tests $(NUM_TESTS)

create_paths_ny:
	cargo run --bin create_paths --release --\
		--hub-graph $(NY_GRAPH)\
		--paths $(NY_PATHS)
create_paths:
	cargo run --bin create_paths --release --\
		--hub-graph $(NETWORK_GRAPH)\
		--paths $(NETWROK_PATHS)


create_ch_ny:
	cargo run --bin create_ch --release --\
		--graph $(NY_GRAPH)\
		--contracted-graph  $(NY_CH)

create_ch:
	cargo run --bin create_ch --release --\
		--graph $(NETWORK_GRAPH)\
		--contracted-graph $(NETWORK_CH)



create_tphl_ny:
	cargo run --bin create_top_down_hl --release --\
		--graph $(NY_GRAPH)\
		--hub-graph $(NY_HL)

create_tphl:
	cargo run --bin create_top_down_hl  --release --\
		--graph $(NETWORK_GRAPH)\
		--hub-graph $(NETWORK_HL)


create_hl_ny:
	cargo run --bin create_hl --release --\
		--contracted-graph $(NY_CH)\
		--hub-graph $(NY_HL)

create_hl:
	cargo run --bin create_hl --release --\
		--contracted-graph $(NETWORK_CH)\
		--hub-graph $(NETWORK_HL)

